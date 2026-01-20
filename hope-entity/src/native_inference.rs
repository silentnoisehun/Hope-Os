//! Natív Inference - A modell BEOLVAD az entitásba
//!
//! Nincs Ollama. Nincs HTTP. Nincs várakozás.
//! A GGUF modell közvetlenül fut a binárisban.
//!
//! ()=>[] - A tiszta potenciálból AZONNAL minden megszületik

#[cfg(feature = "native")]
use llama_cpp_2::context::params::LlamaContextParams;
#[cfg(feature = "native")]
use llama_cpp_2::llama_backend::LlamaBackend;
#[cfg(feature = "native")]
use llama_cpp_2::llama_batch::LlamaBatch;
#[cfg(feature = "native")]
use llama_cpp_2::model::params::LlamaModelParams;
#[cfg(feature = "native")]
use llama_cpp_2::model::LlamaModel;
#[cfg(feature = "native")]
use llama_cpp_2::token::data_array::LlamaTokenDataArray;

use std::path::PathBuf;

/// Natív modell konfiguráció
#[derive(Clone, Debug)]
pub struct NativeModelConfig {
    /// Modell fájl útvonal (GGUF)
    pub model_path: PathBuf,
    /// Kontextus méret (tokenekben)
    pub context_size: u32,
    /// GPU rétegek száma (0 = csak CPU)
    pub gpu_layers: u32,
    /// Szálak száma CPU inference-hez
    pub threads: u32,
    /// Batch méret
    pub batch_size: u32,
    /// Hőmérséklet (kreativitás)
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
    /// Maximum generált tokenek
    pub max_tokens: u32,
}

impl Default for NativeModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            context_size: 4096,
            gpu_layers: 0, // Alapból CPU
            threads: 4,
            batch_size: 512,
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2048,
        }
    }
}

impl NativeModelConfig {
    pub fn new(model_path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: model_path.into(),
            ..Default::default()
        }
    }

    pub fn with_gpu_layers(mut self, layers: u32) -> Self {
        self.gpu_layers = layers;
        self
    }

    pub fn with_context_size(mut self, size: u32) -> Self {
        self.context_size = size;
        self
    }

    pub fn with_threads(mut self, threads: u32) -> Self {
        self.threads = threads;
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }
}

/// Feloldott natív modell típus
#[derive(Clone, Debug, PartialEq)]
pub enum NativeModellTípus {
    Magyar,
    Kódoló,
    Többnyelvű,
    Általános,
}

/// Egy beolvasztott modell
pub struct BeolvasztottModell {
    pub név: String,
    pub típus: NativeModellTípus,
    pub config: NativeModelConfig,
    #[cfg(feature = "native")]
    model: Option<Arc<LlamaModel>>,
    #[cfg(feature = "native")]
    backend: Option<Arc<LlamaBackend>>,
}

impl BeolvasztottModell {
    pub fn new(név: &str, típus: NativeModellTípus, config: NativeModelConfig) -> Self {
        Self {
            név: név.to_string(),
            típus,
            config,
            #[cfg(feature = "native")]
            model: None,
            #[cfg(feature = "native")]
            backend: None,
        }
    }

    /// Modell betöltése memóriába
    #[cfg(feature = "native")]
    pub fn betölt(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Modell betöltése: {} ...", self.név);

        // Backend inicializálás
        let backend = LlamaBackend::init()?;

        // Model params
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(self.config.gpu_layers as i32);

        // Model betöltés
        let model = LlamaModel::load_from_file(&backend, &self.config.model_path, &model_params)?;

        self.backend = Some(Arc::new(backend));
        self.model = Some(Arc::new(model));

        println!("✅ Modell betöltve: {}", self.név);
        Ok(())
    }

    #[cfg(not(feature = "native"))]
    pub fn betölt(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Err("Native feature nincs engedélyezve! Használd: cargo build --features native".into())
    }

    /// Szöveg generálás
    #[cfg(feature = "native")]
    pub fn generál(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let model = self.model.as_ref().ok_or("Modell nincs betöltve!")?;
        let backend = self.backend.as_ref().ok_or("Backend nincs inicializálva!")?;

        // Context létrehozás
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(self.config.context_size).unwrap())
            .with_n_threads(self.config.threads)
            .with_n_threads_batch(self.config.threads);

        let mut ctx = model.new_context(backend, ctx_params)?;

        // Tokenizálás
        let tokens = model.str_to_token(prompt, llama_cpp_2::model::AddBos::Always)?;

        // Batch létrehozás
        let mut batch = LlamaBatch::new(self.config.batch_size as usize, 1);

        // Tokenek hozzáadása
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == tokens.len() - 1)?;
        }

        // Első decode
        ctx.decode(&mut batch)?;

        // Generálás
        let mut output = String::new();
        let mut n_cur = tokens.len();

        for _ in 0..self.config.max_tokens {
            // Logits lekérése
            let logits = ctx.get_logits_ith((n_cur - 1) as i32);

            // Token data array
            let candidates: Vec<_> = logits
                .iter()
                .enumerate()
                .map(|(id, &logit)| llama_cpp_2::token::data::LlamaTokenData::new(
                    llama_cpp_2::token::LlamaToken::new(id as i32),
                    logit,
                    0.0,
                ))
                .collect();

            let mut candidates = LlamaTokenDataArray::from_iter(candidates, false);

            // Sampling
            ctx.sample_temp(&mut candidates, self.config.temperature);
            ctx.sample_top_p(&mut candidates, self.config.top_p, 1);

            let new_token = ctx.sample_token(&mut candidates);

            // EOS check
            if model.is_eog_token(new_token) {
                break;
            }

            // Token -> String
            let piece = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize)?;
            output.push_str(&piece);

            // Következő batch
            batch.clear();
            batch.add(new_token, n_cur as i32, &[0], true)?;
            ctx.decode(&mut batch)?;

            n_cur += 1;
        }

        Ok(output)
    }

    #[cfg(not(feature = "native"))]
    pub fn generál(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        Err("Native feature nincs engedélyezve! Használd: cargo build --features native".into())
    }

    /// Modell betöltve?
    #[cfg(feature = "native")]
    pub fn betöltve(&self) -> bool {
        self.model.is_some()
    }

    #[cfg(not(feature = "native"))]
    pub fn betöltve(&self) -> bool {
        false
    }
}

/// Natív Engine - Több modell kezelése
pub struct NativeEngine {
    modellek: Vec<BeolvasztottModell>,
    aktív_index: Option<usize>,
}

impl NativeEngine {
    pub fn new() -> Self {
        Self {
            modellek: Vec::new(),
            aktív_index: None,
        }
    }

    /// Modell hozzáadása
    pub fn modell_hozzáad(mut self, modell: BeolvasztottModell) -> Self {
        self.modellek.push(modell);
        self
    }

    /// Gyors konfiguráció builder
    pub fn magyar_modell(self, path: impl Into<PathBuf>) -> Self {
        let config = NativeModelConfig::new(path);
        let modell = BeolvasztottModell::new("Magyar", NativeModellTípus::Magyar, config);
        self.modell_hozzáad(modell)
    }

    pub fn kódoló_modell(self, path: impl Into<PathBuf>) -> Self {
        let config = NativeModelConfig::new(path);
        let modell = BeolvasztottModell::new("Kódoló", NativeModellTípus::Kódoló, config);
        self.modell_hozzáad(modell)
    }

    pub fn többnyelvű_modell(self, path: impl Into<PathBuf>) -> Self {
        let config = NativeModelConfig::new(path);
        let modell = BeolvasztottModell::new("Többnyelvű", NativeModellTípus::Többnyelvű, config);
        self.modell_hozzáad(modell)
    }

    /// Összes modell betöltése
    pub fn betölt_mindent(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🚀 Natív modellek betöltése...\n");

        for modell in &mut self.modellek {
            modell.betölt()?;
        }

        if !self.modellek.is_empty() {
            self.aktív_index = Some(0);
        }

        println!("\n✅ Összes modell betöltve! Készen áll.\n");
        Ok(())
    }

    /// Legjobb modell választása szöveg alapján
    pub fn válaszd_modellt(&self, szöveg: &str) -> Option<&BeolvasztottModell> {
        let szöveg_lower = szöveg.to_lowercase();

        // Kód detektálás
        let kód_jelek = ["fn ", "let ", "impl ", "pub ", "def ", "class ", "```"];
        if kód_jelek.iter().any(|j| szöveg_lower.contains(j)) {
            if let Some(m) = self.modellek.iter().find(|m| m.típus == NativeModellTípus::Kódoló) {
                return Some(m);
            }
        }

        // Magyar detektálás
        let magyar_jelek = ["szia", "hogy", "köszön", "kérem", "á", "é", "ő", "ű"];
        if magyar_jelek.iter().any(|j| szöveg_lower.contains(j)) {
            if let Some(m) = self.modellek.iter().find(|m| m.típus == NativeModellTípus::Magyar) {
                return Some(m);
            }
        }

        // Alapértelmezett
        self.modellek.first()
    }

    /// Generálás automatikus modell választással
    pub fn generál(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let modell = self.válaszd_modellt(prompt)
            .ok_or("Nincs betöltött modell!")?;

        if !modell.betöltve() {
            return Err(format!("A '{}' modell nincs betöltve!", modell.név).into());
        }

        println!("🧠 Natív generálás: {}", modell.név);
        modell.generál(prompt)
    }

    /// Modellek listája
    pub fn modellek(&self) -> &[BeolvasztottModell] {
        &self.modellek
    }

    /// Van betöltött modell?
    pub fn kész(&self) -> bool {
        self.modellek.iter().any(|m| m.betöltve())
    }
}

impl Default for NativeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Gyors benchmark a natív inference-hez
pub fn native_benchmark(engine: &NativeEngine) {
    println!("\n⚡ NATÍV INFERENCE BENCHMARK\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let teszt_promptok = [
        "Szia! Ki vagy te?",
        "fn main() { println!(\"Hello\"); }",
        "Mi a gravitáció?",
    ];

    for prompt in teszt_promptok {
        println!("\n📝 Prompt: {}", prompt);

        let start = std::time::Instant::now();
        match engine.generál(prompt) {
            Ok(válasz) => {
                let elapsed = start.elapsed();
                let tokens_approx = válasz.split_whitespace().count();
                let tokens_per_sec = tokens_approx as f64 / elapsed.as_secs_f64();

                println!("💬 Válasz: {}...", &válasz[..válasz.len().min(100)]);
                println!("⏱️  Idő: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
                println!("🚀 ~{:.1} token/sec", tokens_per_sec);
            }
            Err(e) => {
                println!("❌ Hiba: {}", e);
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = NativeModelConfig::new("/path/to/model.gguf")
            .with_gpu_layers(35)
            .with_context_size(8192)
            .with_temperature(0.8);

        assert_eq!(config.gpu_layers, 35);
        assert_eq!(config.context_size, 8192);
        assert!((config.temperature - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_engine_modell_választás() {
        let engine = NativeEngine::new()
            .modell_hozzáad(BeolvasztottModell::new(
                "Magyar",
                NativeModellTípus::Magyar,
                NativeModelConfig::default(),
            ))
            .modell_hozzáad(BeolvasztottModell::new(
                "Kódoló",
                NativeModellTípus::Kódoló,
                NativeModelConfig::default(),
            ));

        let m = engine.válaszd_modellt("Szia, hogy vagy?");
        assert!(m.is_some());
        assert_eq!(m.unwrap().típus, NativeModellTípus::Magyar);

        let m = engine.válaszd_modellt("fn main() {}");
        assert!(m.is_some());
        assert_eq!(m.unwrap().típus, NativeModellTípus::Kódoló);
    }
}
