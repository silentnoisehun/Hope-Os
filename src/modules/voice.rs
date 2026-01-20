//! Hope Voice Module - Streaming TTS/STT
//!
//! Magyar és angol hangok, férfi és női,
//! 21 dimenziós érzelem-alapú prozódia.
//!
//! ()=>[] - A tiszta potenciálból a hang megszületik

use crate::core::HopeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// VOICE CONFIG
// ============================================================================

/// TTS/STT szerver konfiguráció
#[derive(Clone, Debug)]
pub struct VoiceConfig {
    /// TTS szerver URL
    pub tts_url: String,
    /// TTS port
    pub tts_port: u16,
    /// STT szerver URL
    pub stt_url: String,
    /// STT port
    pub stt_port: u16,
    /// Alapértelmezett hang
    pub default_voice: String,
    /// Alapértelmezett nyelv
    pub default_language: String,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            tts_url: "http://127.0.0.1".to_string(),
            tts_port: 8880,
            stt_url: "http://127.0.0.1".to_string(),
            stt_port: 2022,
            default_voice: "berta".to_string(),
            default_language: "hu".to_string(),
        }
    }
}

// ============================================================================
// VOICE INFO
// ============================================================================

/// Hang típus
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Female,
    Male,
}

/// TTS motor típus
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoiceEngine {
    /// Piper TTS (helyi, gyors)
    Piper,
    /// Edge TTS (Microsoft, online)
    Edge,
    /// Coqui TTS (helyi, neurális)
    Coqui,
}

/// Hang információ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// Azonosító (pl. "berta", "ryan")
    pub id: String,
    /// Megjelenített név
    pub name: String,
    /// Nyelv kód (pl. "hu-HU", "en-US")
    pub language: String,
    /// Nem
    pub gender: Gender,
    /// Leírás
    pub description: String,
    /// Elérhető-e
    pub available: bool,
    /// Támogatott érzelmek
    pub emotions: Vec<String>,
    /// TTS motor
    pub engine: VoiceEngine,
    /// Stílus (beszélgetés, narráció, hírek)
    pub style: String,
    /// Motor-specifikus modell név
    pub model_name: String,
}

impl VoiceInfo {
    /// Magyar Berta hang (Piper)
    pub fn berta() -> Self {
        Self {
            id: "berta".to_string(),
            name: "Berta".to_string(),
            language: "hu-HU".to_string(),
            gender: Gender::Female,
            description: "Magyar női hang - Hope alapértelmezett hangja".to_string(),
            available: true,
            emotions: vec![
                "joy".to_string(), "sadness".to_string(), "anger".to_string(),
                "fear".to_string(), "neutral".to_string(), "love".to_string(),
            ],
            engine: VoiceEngine::Piper,
            style: "conversational".to_string(),
            model_name: "hu_HU-berta-medium".to_string(),
        }
    }

    /// Magyar Anna hang (Piper)
    pub fn anna() -> Self {
        Self {
            id: "anna".to_string(),
            name: "Anna".to_string(),
            language: "hu-HU".to_string(),
            gender: Gender::Female,
            description: "Magyar női hang - lágyabb tónus".to_string(),
            available: true,
            emotions: vec!["joy".to_string(), "neutral".to_string(), "sadness".to_string()],
            engine: VoiceEngine::Piper,
            style: "conversational".to_string(),
            model_name: "hu_HU-anna-medium".to_string(),
        }
    }

    /// Magyar Noémi hang (Edge)
    pub fn noemi() -> Self {
        Self {
            id: "noemi".to_string(),
            name: "Noémi".to_string(),
            language: "hu-HU".to_string(),
            gender: Gender::Female,
            description: "Magyar női hang - Microsoft Edge TTS".to_string(),
            available: true,
            emotions: vec!["joy".to_string(), "neutral".to_string(), "sadness".to_string()],
            engine: VoiceEngine::Edge,
            style: "conversational".to_string(),
            model_name: "hu-HU-NoemiNeural".to_string(),
        }
    }

    /// Magyar Tamás hang (Edge)
    pub fn tamas() -> Self {
        Self {
            id: "tamas".to_string(),
            name: "Tamás".to_string(),
            language: "hu-HU".to_string(),
            gender: Gender::Male,
            description: "Magyar férfi hang - Microsoft Edge TTS".to_string(),
            available: true,
            emotions: vec!["joy".to_string(), "neutral".to_string(), "sadness".to_string()],
            engine: VoiceEngine::Edge,
            style: "conversational".to_string(),
            model_name: "hu-HU-TamasNeural".to_string(),
        }
    }

    /// English Ryan hang (Piper)
    pub fn ryan() -> Self {
        Self {
            id: "ryan".to_string(),
            name: "Ryan".to_string(),
            language: "en-US".to_string(),
            gender: Gender::Male,
            description: "American male voice - high quality".to_string(),
            available: true,
            emotions: vec!["joy".to_string(), "neutral".to_string(), "sadness".to_string()],
            engine: VoiceEngine::Piper,
            style: "conversational".to_string(),
            model_name: "en_US-ryan-high".to_string(),
        }
    }

    /// English Amy hang (Piper)
    pub fn amy() -> Self {
        Self {
            id: "amy".to_string(),
            name: "Amy".to_string(),
            language: "en-GB".to_string(),
            gender: Gender::Female,
            description: "British female voice".to_string(),
            available: true,
            emotions: vec!["joy".to_string(), "neutral".to_string(), "sadness".to_string()],
            engine: VoiceEngine::Piper,
            style: "conversational".to_string(),
            model_name: "en_GB-amy-medium".to_string(),
        }
    }

    /// English Jenny hang (Edge)
    pub fn jenny() -> Self {
        Self {
            id: "jenny".to_string(),
            name: "Jenny".to_string(),
            language: "en-US".to_string(),
            gender: Gender::Female,
            description: "American female voice - Microsoft Edge TTS".to_string(),
            available: true,
            emotions: vec![
                "joy".to_string(), "sadness".to_string(), "anger".to_string(),
                "fear".to_string(), "neutral".to_string(),
            ],
            engine: VoiceEngine::Edge,
            style: "conversational".to_string(),
            model_name: "en-US-JennyNeural".to_string(),
        }
    }

    /// English Guy hang (Edge)
    pub fn guy() -> Self {
        Self {
            id: "guy".to_string(),
            name: "Guy".to_string(),
            language: "en-US".to_string(),
            gender: Gender::Male,
            description: "American male voice - Microsoft Edge TTS".to_string(),
            available: true,
            emotions: vec!["joy".to_string(), "neutral".to_string(), "sadness".to_string()],
            engine: VoiceEngine::Edge,
            style: "conversational".to_string(),
            model_name: "en-US-GuyNeural".to_string(),
        }
    }

    /// English Aria hang (Edge) - expresszív
    pub fn aria() -> Self {
        Self {
            id: "aria".to_string(),
            name: "Aria".to_string(),
            language: "en-US".to_string(),
            gender: Gender::Female,
            description: "American female voice - expressive, emotional".to_string(),
            available: true,
            emotions: vec![
                "joy".to_string(), "sadness".to_string(), "anger".to_string(),
                "fear".to_string(), "neutral".to_string(), "cheerful".to_string(),
                "empathetic".to_string(), "friendly".to_string(),
            ],
            engine: VoiceEngine::Edge,
            style: "conversational".to_string(),
            model_name: "en-US-AriaNeural".to_string(),
        }
    }

    /// English Davis hang (Edge) - narrátor
    pub fn davis() -> Self {
        Self {
            id: "davis".to_string(),
            name: "Davis".to_string(),
            language: "en-US".to_string(),
            gender: Gender::Male,
            description: "American male voice - narrator style".to_string(),
            available: true,
            emotions: vec!["neutral".to_string(), "cheerful".to_string()],
            engine: VoiceEngine::Edge,
            style: "narration".to_string(),
            model_name: "en-US-DavisNeural".to_string(),
        }
    }

    /// Összes elérhető hang
    pub fn all_voices() -> Vec<VoiceInfo> {
        vec![
            // Magyar hangok
            Self::berta(),
            Self::anna(),
            Self::noemi(),
            Self::tamas(),
            // English voices
            Self::ryan(),
            Self::amy(),
            Self::jenny(),
            Self::guy(),
            Self::aria(),
            Self::davis(),
        ]
    }
}

// ============================================================================
// EMOTION TO PROSODY
// ============================================================================

/// Prozódia beállítások (Piper TTS paraméterek)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProsodySettings {
    /// Sebesség (0.5 - 2.0, default: 1.0)
    pub speed: f64,
    /// Hangerő (0.0 - 1.0)
    pub volume: f64,
    /// Piper length_scale (lassabb/gyorsabb)
    pub length_scale: f64,
    /// Piper noise_scale
    pub noise_scale: f64,
    /// Piper noise_w
    pub noise_w: f64,
}

impl ProsodySettings {
    pub fn neutral() -> Self {
        Self {
            speed: 1.0,
            volume: 1.0,
            length_scale: 1.0,
            noise_scale: 0.667,
            noise_w: 0.8,
        }
    }

    pub fn joy() -> Self {
        Self {
            speed: 1.1,
            volume: 1.1,
            length_scale: 0.92,
            noise_scale: 0.68,
            noise_w: 0.8,
        }
    }

    pub fn sadness() -> Self {
        Self {
            speed: 0.9,
            volume: 0.85,
            length_scale: 1.08,
            noise_scale: 0.65,
            noise_w: 0.75,
        }
    }

    pub fn anger() -> Self {
        Self {
            speed: 1.15,
            volume: 1.15,
            length_scale: 0.88,
            noise_scale: 0.72,
            noise_w: 0.85,
        }
    }

    pub fn fear() -> Self {
        Self {
            speed: 1.2,
            volume: 0.9,
            length_scale: 0.9,
            noise_scale: 0.75,
            noise_w: 0.85,
        }
    }

    pub fn love() -> Self {
        Self {
            speed: 0.95,
            volume: 0.95,
            length_scale: 1.02,
            noise_scale: 0.62,
            noise_w: 0.7,
        }
    }

    /// Érzelem alapján prozódia
    pub fn from_emotion(emotion: &str) -> Self {
        match emotion.to_lowercase().as_str() {
            "joy" | "happy" | "cheerful" => Self::joy(),
            "sadness" | "sad" => Self::sadness(),
            "anger" | "angry" => Self::anger(),
            "fear" | "afraid" => Self::fear(),
            "love" | "loving" => Self::love(),
            _ => Self::neutral(),
        }
    }

    /// 21D érzelem vektorból prozódia számítás
    pub fn from_emotions_21d(emotions: &HashMap<String, f64>) -> Self {
        let mut settings = Self::neutral();

        // Domináns érzelmek hatása
        if let Some(&joy) = emotions.get("joy") {
            settings.speed += joy * 0.15;
            settings.volume += joy * 0.1;
            settings.length_scale -= joy * 0.08;
        }

        if let Some(&sadness) = emotions.get("sadness") {
            settings.speed -= sadness * 0.15;
            settings.volume -= sadness * 0.15;
            settings.length_scale += sadness * 0.1;
        }

        if let Some(&anger) = emotions.get("anger") {
            settings.speed += anger * 0.2;
            settings.volume += anger * 0.15;
            settings.noise_scale += anger * 0.05;
        }

        if let Some(&fear) = emotions.get("fear") {
            settings.speed += fear * 0.2;
            settings.noise_scale += fear * 0.08;
        }

        if let Some(&love) = emotions.get("love") {
            settings.speed -= love * 0.05;
            settings.noise_scale -= love * 0.05;
        }

        // Határok
        settings.speed = settings.speed.clamp(0.7, 1.4);
        settings.volume = settings.volume.clamp(0.5, 1.2);
        settings.length_scale = settings.length_scale.clamp(0.85, 1.15);
        settings.noise_scale = settings.noise_scale.clamp(0.5, 0.8);

        settings
    }
}

// ============================================================================
// TTS REQUEST/RESPONSE
// ============================================================================

/// TTS kérés
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeakRequest {
    /// Szöveg
    pub text: String,
    /// Hang azonosító
    pub voice: String,
    /// Érzelem
    pub emotion: String,
    /// 21D érzelem vektor
    pub emotions_21d: Option<HashMap<String, f64>>,
    /// Prozódia beállítások
    pub prosody: Option<ProsodySettings>,
    /// Audio formátum
    pub format: String,
    /// Sample rate
    pub sample_rate: u32,
}

impl Default for SpeakRequest {
    fn default() -> Self {
        Self {
            text: String::new(),
            voice: "berta".to_string(),
            emotion: "neutral".to_string(),
            emotions_21d: None,
            prosody: None,
            format: "wav".to_string(),
            sample_rate: 22050,
        }
    }
}

/// TTS válasz
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeakResponse {
    /// Audio adat (bytes)
    pub audio: Vec<u8>,
    /// Formátum
    pub format: String,
    /// Sample rate
    pub sample_rate: u32,
    /// Időtartam (másodperc)
    pub duration: f64,
}

/// Audio chunk (streaming)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioChunk {
    /// Audio adat
    pub data: Vec<u8>,
    /// Sorszám
    pub sequence: u32,
    /// Utolsó chunk?
    pub is_final: bool,
    /// Formátum
    pub format: String,
    /// Sample rate
    pub sample_rate: u32,
    /// Időbélyeg
    pub timestamp: f64,
}

// ============================================================================
// STT REQUEST/RESPONSE
// ============================================================================

/// STT kérés
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListenRequest {
    /// Nyelv
    pub language: String,
    /// VAD engedélyezve
    pub vad_enabled: bool,
    /// VAD küszöb
    pub vad_threshold: f64,
    /// Csend timeout (másodperc)
    pub silence_timeout: f64,
    /// Szó-szintű időbélyegek
    pub word_timestamps: bool,
    /// Whisper modell
    pub model: String,
}

impl Default for ListenRequest {
    fn default() -> Self {
        Self {
            language: "hu".to_string(),
            vad_enabled: true,
            vad_threshold: 0.5,
            silence_timeout: 1.5,
            word_timestamps: false,
            model: "base".to_string(),
        }
    }
}

/// Transzkripció válasz
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    /// Szöveg
    pub text: String,
    /// Észlelt nyelv
    pub language: String,
    /// Bizonyosság
    pub confidence: f64,
    /// Szavak időbélyegekkel
    pub words: Vec<WordInfo>,
    /// Audio időtartam
    pub duration: f64,
}

/// Szó információ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordInfo {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
}

/// Transzkripció chunk (streaming)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptionChunk {
    /// Részleges szöveg
    pub text: String,
    /// Végleges?
    pub is_final: bool,
    /// Bizonyosság
    pub confidence: f64,
    /// Időbélyeg
    pub timestamp: f64,
    /// VAD: beszéd kezdődött
    pub speech_started: bool,
    /// VAD: beszéd véget ért
    pub speech_ended: bool,
}

// ============================================================================
// HOPE VOICE - Fő modul
// ============================================================================

/// Hope Voice modul - TTS/STT kezelés
pub struct HopeVoice {
    /// Konfiguráció
    config: VoiceConfig,
    /// Aktuális hang
    current_voice: Arc<RwLock<VoiceInfo>>,
    /// Aktuális érzelem
    current_emotion: Arc<RwLock<String>>,
    /// 21D érzelmek
    emotions_21d: Arc<RwLock<HashMap<String, f64>>>,
    /// HTTP kliens
    http_client: reqwest::Client,
    /// Elérhető hangok
    voices: Vec<VoiceInfo>,
}

impl HopeVoice {
    /// Új Voice modul
    pub fn new() -> Self {
        Self::with_config(VoiceConfig::default())
    }

    /// Konfigurációval
    pub fn with_config(config: VoiceConfig) -> Self {
        let default_voice = VoiceInfo::berta();

        Self {
            config,
            current_voice: Arc::new(RwLock::new(default_voice)),
            current_emotion: Arc::new(RwLock::new("neutral".to_string())),
            emotions_21d: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::new(),
            voices: VoiceInfo::all_voices(),
        }
    }

    /// TTS URL összeállítás
    fn tts_url(&self, endpoint: &str) -> String {
        format!("{}:{}{}", self.config.tts_url, self.config.tts_port, endpoint)
    }

    /// STT URL összeállítás
    fn stt_url(&self, endpoint: &str) -> String {
        format!("{}:{}{}", self.config.stt_url, self.config.stt_port, endpoint)
    }

    // ==================== TTS ====================

    /// Beszéd generálás (sync)
    pub async fn speak(&self, request: SpeakRequest) -> HopeResult<SpeakResponse> {
        let voice = self.current_voice.read().await;

        // Prozódia számítás
        let prosody = if let Some(ref emotions) = request.emotions_21d {
            ProsodySettings::from_emotions_21d(emotions)
        } else {
            request.prosody.unwrap_or_else(|| ProsodySettings::from_emotion(&request.emotion))
        };

        // API endpoint választás motor alapján
        let url = match voice.engine {
            VoiceEngine::Piper => self.tts_url("/v1/audio/speech"),
            VoiceEngine::Edge => self.tts_url("/edge_tts"),
            VoiceEngine::Coqui => self.tts_url("/coqui_tts"),
        };

        // Kérés összeállítás
        let payload = serde_json::json!({
            "input": request.text,
            "voice": voice.model_name,
            "model": "piper",
            "response_format": request.format,
            "speed": prosody.speed,
            "emotion": request.emotion,
            "length_scale": prosody.length_scale,
            "noise_scale": prosody.noise_scale,
            "noise_w": prosody.noise_w,
        });

        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("TTS hiba: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("TTS szerver hiba: {}", response.status()).into());
        }

        let audio = response.bytes().await
            .map_err(|e| format!("Audio letöltés hiba: {}", e))?
            .to_vec();

        Ok(SpeakResponse {
            audio,
            format: request.format,
            sample_rate: request.sample_rate,
            duration: 0.0, // TODO: számítás az audio méretből
        })
    }

    /// Beszéd generálás streaming módon
    pub async fn speak_stream(&self, request: SpeakRequest) -> HopeResult<Vec<AudioChunk>> {
        // Egyszerűsített implementáció: egy nagy chunk
        let response = self.speak(request).await?;

        Ok(vec![AudioChunk {
            data: response.audio,
            sequence: 0,
            is_final: true,
            format: response.format,
            sample_rate: response.sample_rate,
            timestamp: 0.0,
        }])
    }

    // ==================== STT ====================

    /// Transzkripció (audio fájlból)
    pub async fn transcribe(&self, audio_data: Vec<u8>, request: ListenRequest) -> HopeResult<TranscriptionResponse> {
        let url = self.stt_url("/v1/audio/transcriptions");

        // Multipart form
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data)
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .unwrap())
            .text("language", request.language.clone())
            .text("word_timestamps", request.word_timestamps.to_string());

        let response = self.http_client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("STT hiba: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("STT szerver hiba: {}", response.status()).into());
        }

        let result: serde_json::Value = response.json().await
            .map_err(|e| format!("JSON parse hiba: {}", e))?;

        Ok(TranscriptionResponse {
            text: result["text"].as_str().unwrap_or("").to_string(),
            language: request.language,
            confidence: result["confidence"].as_f64().unwrap_or(0.9),
            words: Vec::new(), // TODO: word parsing
            duration: result["duration"].as_f64().unwrap_or(0.0),
        })
    }

    // ==================== SETTINGS ====================

    /// Hang beállítása
    pub async fn set_voice(&self, voice_id: &str) -> HopeResult<()> {
        let voice = self.voices.iter()
            .find(|v| v.id == voice_id)
            .ok_or_else(|| format!("Ismeretlen hang: {}", voice_id))?
            .clone();

        let mut current = self.current_voice.write().await;
        *current = voice;
        Ok(())
    }

    /// Érzelem beállítása
    pub async fn set_emotion(&self, emotion: &str) {
        let mut current = self.current_emotion.write().await;
        *current = emotion.to_string();
    }

    /// 21D érzelmek beállítása
    pub async fn set_emotions_21d(&self, emotions: HashMap<String, f64>) {
        let mut current = self.emotions_21d.write().await;
        *current = emotions;
    }

    /// Hangok listája
    pub fn list_voices(&self) -> &[VoiceInfo] {
        &self.voices
    }

    /// Magyar hangok
    pub fn hungarian_voices(&self) -> Vec<&VoiceInfo> {
        self.voices.iter()
            .filter(|v| v.language.starts_with("hu"))
            .collect()
    }

    /// Angol hangok
    pub fn english_voices(&self) -> Vec<&VoiceInfo> {
        self.voices.iter()
            .filter(|v| v.language.starts_with("en"))
            .collect()
    }

    /// Női hangok
    pub fn female_voices(&self) -> Vec<&VoiceInfo> {
        self.voices.iter()
            .filter(|v| v.gender == Gender::Female)
            .collect()
    }

    /// Férfi hangok
    pub fn male_voices(&self) -> Vec<&VoiceInfo> {
        self.voices.iter()
            .filter(|v| v.gender == Gender::Male)
            .collect()
    }

    /// TTS elérhetőség ellenőrzése
    pub async fn tts_available(&self) -> bool {
        let url = self.tts_url("/health");
        self.http_client.get(&url).send().await.is_ok()
    }

    /// STT elérhetőség ellenőrzése
    pub async fn stt_available(&self) -> bool {
        let url = self.stt_url("/health");
        self.http_client.get(&url).send().await.is_ok()
    }

    /// Állapot
    pub async fn status(&self) -> String {
        let voice = self.current_voice.read().await;
        let emotion = self.current_emotion.read().await;
        let tts_ok = self.tts_available().await;
        let stt_ok = self.stt_available().await;

        format!(
            "🎤 Hope Voice\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             🗣️  Aktuális hang: {} ({})\n\
             😊 Érzelem: {}\n\
             📢 TTS: {} (port {})\n\
             👂 STT: {} (port {})\n\
             📚 Hangok: {} db ({} HU, {} EN)",
            voice.name,
            voice.language,
            emotion,
            if tts_ok { "✅" } else { "❌" },
            self.config.tts_port,
            if stt_ok { "✅" } else { "❌" },
            self.config.stt_port,
            self.voices.len(),
            self.hungarian_voices().len(),
            self.english_voices().len(),
        )
    }
}

impl Default for HopeVoice {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MODULE INFO
// ============================================================================

impl HopeVoice {
    /// Modul információ
    pub fn module_info(&self) -> String {
        format!(
            "HopeVoice v1.0.0 - Streaming TTS/STT\n\
             Magyar és angol hangok, férfi és női, 21D érzelmek"
        )
    }

    /// Introspekció
    pub fn introspect(&self) -> String {
        format!(
            "HopeVoice: {} hang ({} magyar, {} angol), TTS:{} STT:{}",
            self.voices.len(),
            self.hungarian_voices().len(),
            self.english_voices().len(),
            self.config.tts_port,
            self.config.stt_port,
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_info() {
        let berta = VoiceInfo::berta();
        assert_eq!(berta.id, "berta");
        assert_eq!(berta.language, "hu-HU");
        assert_eq!(berta.gender, Gender::Female);
    }

    #[test]
    fn test_all_voices() {
        let voices = VoiceInfo::all_voices();
        assert_eq!(voices.len(), 10);

        // Magyar hangok
        let hu_voices: Vec<_> = voices.iter().filter(|v| v.language.starts_with("hu")).collect();
        assert_eq!(hu_voices.len(), 4);

        // Angol hangok
        let en_voices: Vec<_> = voices.iter().filter(|v| v.language.starts_with("en")).collect();
        assert_eq!(en_voices.len(), 6);
    }

    #[test]
    fn test_prosody_from_emotion() {
        let joy = ProsodySettings::from_emotion("joy");
        assert!(joy.speed > 1.0);

        let sadness = ProsodySettings::from_emotion("sadness");
        assert!(sadness.speed < 1.0);
    }

    #[test]
    fn test_prosody_from_21d() {
        let mut emotions = HashMap::new();
        emotions.insert("joy".to_string(), 0.8);
        emotions.insert("sadness".to_string(), 0.1);

        let prosody = ProsodySettings::from_emotions_21d(&emotions);
        assert!(prosody.speed > 1.0); // Joy dominant
    }

    #[test]
    fn test_hope_voice_creation() {
        let voice = HopeVoice::new();
        assert_eq!(voice.voices.len(), 10);
    }
}
