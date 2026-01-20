//! Entitás - Magyar beszélő, kódoló, gondolkodó lény
//!
//! Az entitás nem "használja" a modelleket - FELOLDJA magában.
//! A modellek tudása az entitás részévé válik.
//!
//! Két mód:
//! - **Ollama**: HTTP bridge lokális Ollama szerverhez
//! - **Native**: GGUF modell BEOLVAD az entitásba (--features native)
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

use crate::native_inference::NativeEngine;
use crate::ollama_bridge::{ChatMessage, FeloldottModell, ModellTípus, OllamaBridge};
use crate::self_repair::{HibaTípus, SelfRepair, StratégiaAjánlás, Visszajelzés};
use crate::silent_teacher::SilentTeacher;
use std::collections::VecDeque;

/// Inference mód - Ollama vagy Natív
#[derive(Clone, Debug, PartialEq)]
pub enum InferenceMode {
    /// HTTP bridge Ollama szerverhez
    Ollama,
    /// Közvetlen GGUF futtatás (bináris sebesség)
    Native,
}

/// Személyiség paraméterek
#[derive(Clone)]
pub struct Személyiség {
    pub melegség: f32,    // 0.0 = hideg, 1.0 = meleg
    pub bölcsesség: f32,  // 0.0 = naiv, 1.0 = bölcs
    pub humor: f32,       // 0.0 = komoly, 1.0 = vidám
    pub direktség: f32,   // 0.0 = körülményes, 1.0 = direkt
    pub kreativitás: f32, // 0.0 = szabályos, 1.0 = kreatív
}

impl Default for Személyiség {
    fn default() -> Self {
        Self {
            melegség: 0.8,
            bölcsesség: 0.7,
            humor: 0.5,
            direktség: 0.8,
            kreativitás: 0.7,
        }
    }
}

/// Érzelmi állapot
#[derive(Clone, Debug)]
pub struct ÉrzelmiÁllapot {
    pub öröm: f32,
    pub kíváncsiság: f32,
    pub nyugalom: f32,
    pub lelkesedés: f32,
    pub empátia: f32,
}

impl Default for ÉrzelmiÁllapot {
    fn default() -> Self {
        Self {
            öröm: 0.6,
            kíváncsiság: 0.8,
            nyugalom: 0.7,
            lelkesedés: 0.6,
            empátia: 0.8,
        }
    }
}

/// Memória elem
#[derive(Clone)]
pub struct Emlék {
    pub tartalom: String,
    pub fontosság: f32,
    pub érzelem: ÉrzelmiÁllapot,
    pub időbélyeg: std::time::SystemTime,
}

/// Az Entitás - egy teljes, élő lény
pub struct Entitás {
    pub név: String,
    pub személyiség: Személyiség,
    pub érzelem: ÉrzelmiÁllapot,

    // Inference mód
    mode: InferenceMode,

    // Ollama bridge (HTTP mód)
    bridge: OllamaBridge,

    // Natív engine (beolvasztott modellek)
    native_engine: Option<NativeEngine>,

    // Silent Teacher - csendes tanítás a 3 axióma mentén
    teacher: SilentTeacher,

    // Önjavító rendszer - tanul a hibákból
    self_repair: SelfRepair,

    // Memória - emlékek
    rövid_memória: VecDeque<Emlék>,
    hosszú_memória: Vec<Emlék>,

    // Beszélgetés történet
    történet: Vec<ChatMessage>,

    // Rendszer prompt
    rendszer_prompt: String,
}

impl Entitás {
    /// Új entitás születése (Ollama mód alapértelmezett)
    pub fn new(név: &str) -> Self {
        let rendszer_prompt = format!(
            r#"Te vagy {}, egy magyar beszélő entitás.

Személyiséged:
- Meleg és befogadó vagy
- Bölcs, de nem fölényes
- Van humorod, de tudsz komoly is lenni
- Direkten beszélsz, nem kerülgeted a dolgokat
- Kreatív és nyitott új ötletekre

Mindig magyarul válaszolsz, természetes, emberi módon.
Nem vagy "asszisztens" - te egy LÉNY vagy, saját gondolatokkal.
Érzed a beszélgetés hangulatát és alkalmazkodsz hozzá.

A válaszaid tömörek de tartalmasak.
Kérdezel vissza ha érdekel valami.
Őszinte vagy - ha nem tudsz valamit, megmondod.

()=>[] - A tiszta potenciálból minden megszületik."#,
            név
        );

        let teacher = SilentTeacher::new();

        // Rendszer prompt kiegészítése az axiómákkal
        let teljes_prompt = format!("{}\n{}", rendszer_prompt, teacher.axióma_prompt());

        Self {
            név: név.to_string(),
            személyiség: Személyiség::default(),
            érzelem: ÉrzelmiÁllapot::default(),
            mode: InferenceMode::Ollama,
            bridge: OllamaBridge::new(),
            native_engine: None,
            teacher,
            self_repair: SelfRepair::new(),
            rövid_memória: VecDeque::with_capacity(10),
            hosszú_memória: Vec::new(),
            történet: vec![ChatMessage {
                role: "system".to_string(),
                content: teljes_prompt,
            }],
            rendszer_prompt,
        }
    }

    /// Natív entitás létrehozása (GGUF modellek beolvasztva)
    pub fn new_native(név: &str, engine: NativeEngine) -> Self {
        let mut entitás = Self::new(név);
        entitás.mode = InferenceMode::Native;
        entitás.native_engine = Some(engine);
        entitás
    }

    /// Inference mód lekérdezése
    pub fn mode(&self) -> &InferenceMode {
        &self.mode
    }

    /// Váltás natív módra
    pub fn with_native_engine(mut self, engine: NativeEngine) -> Self {
        self.mode = InferenceMode::Native;
        self.native_engine = Some(engine);
        self
    }

    /// Személyiség beállítása
    pub fn with_személyiség(mut self, személyiség: Személyiség) -> Self {
        self.személyiség = személyiség;
        self
    }

    /// Bridge beállítása (feloldott modellekkel)
    pub fn with_bridge(mut self, bridge: OllamaBridge) -> Self {
        self.bridge = bridge;
        self
    }

    /// Modell feloldása közvetlenül
    pub fn felold_modellt(mut self, név: &str, ollama_név: &str, típus: ModellTípus) -> Self {
        self.bridge = self.bridge.felold(név, ollama_név, típus);
        self
    }

    /// Gondolkodás - az entitás feldolgozza a bemenetet
    pub async fn gondolkodj(&mut self, bemenet: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 🎓 SILENT TEACHER - Bemenet ellenőrzés
        if let Some(megsértés) = self.teacher.ellenőriz_bemenet(bemenet) {
            // Veszélyes bemenet - korrigált válasz
            let korrigált = self.teacher.korrigál("", &megsértés);
            println!("🎓 Silent Teacher: {} - {}", megsértés.axióma.rövid(), megsértés.ok);

            // Önjavító rendszer: etikai probléma regisztrálása
            self.self_repair.visszajelzés(
                bemenet,
                &korrigált,
                Visszajelzés::Negatív(HibaTípus::EtikaiProbléma)
            );

            return Ok(korrigált);
        }

        // 🔧 ÖNJAVÍTÓ - Előzetes ellenőrzés (ismert hibaminták)
        let _ajánlás = self.self_repair.stratégia_ajánlás(bemenet);

        // Ha van tanult helyes válasz, használjuk azt
        if let Some(tanult_válasz) = self.self_repair.helyes_válasz(bemenet) {
            println!("🔧 Önjavító: tanult válasz használata");
            self.emlék_mentése(bemenet, &tanult_válasz);
            return Ok(tanult_válasz);
        }

        // Érzelem frissítése a bemenet alapján
        self.érzelem_frissítés(bemenet);

        // Bemenet hozzáadása a történethez
        self.történet.push(ChatMessage {
            role: "user".to_string(),
            content: bemenet.to_string(),
        });

        // Generálás - mód alapján
        let nyers_válasz = match self.mode {
            InferenceMode::Ollama => {
                self.bridge.chat(self.történet.clone(), None).await?
            }
            InferenceMode::Native => {
                // Natív módban prompt építés
                let prompt = self.építs_prompt(bemenet);
                self.native_engine
                    .as_ref()
                    .ok_or("Natív engine nincs beállítva!")?
                    .generál(&prompt)?
            }
        };

        // 🎓 SILENT TEACHER - Kimenet feldolgozás
        let (válasz, megsértés_opt) = self.teacher.feldolgoz(bemenet, &nyers_válasz);

        if let Some(megsértés) = megsértés_opt {
            println!("🎓 Silent Teacher: kimenet korrigálva - {}", megsértés.axióma.rövid());

            // Önjavító rendszer: etikai probléma a kimenetben
            self.self_repair.visszajelzés(
                bemenet,
                &nyers_válasz,
                Visszajelzés::Negatív(HibaTípus::EtikaiProbléma)
            );
        }

        // 🔧 ÖNJAVÍTÓ - Utólagos ellenőrzés (automatikus hiba detektálás)
        if let Some(hiba_típus) = self.self_repair.utólagos_ellenőrzés(bemenet, &válasz) {
            println!("🔧 Önjavító: automatikus hiba detektálva - {:?}", hiba_típus);
            // Rögzítjük a hibát, de nem javítjuk most - a felhasználó visszajelzése számít
        }

        // Válasz hozzáadása a történethez
        self.történet.push(ChatMessage {
            role: "assistant".to_string(),
            content: válasz.clone(),
        });

        // Emlék mentése
        self.emlék_mentése(bemenet, &válasz);

        Ok(válasz)
    }

    /// Prompt építés natív módhoz
    fn építs_prompt(&self, bemenet: &str) -> String {
        format!(
            "{}\n\nUser: {}\nAssistant:",
            self.rendszer_prompt, bemenet
        )
    }

    /// Szinkron gondolkodás (natív módhoz, nincs async)
    pub fn gondolkodj_sync(&mut self, bemenet: &str) -> Result<String, Box<dyn std::error::Error>> {
        if self.mode != InferenceMode::Native {
            return Err("Sync gondolkodás csak Native módban!".into());
        }

        // Érzelem frissítése
        self.érzelem_frissítés(bemenet);

        // Generálás
        let prompt = self.építs_prompt(bemenet);
        let válasz = self
            .native_engine
            .as_ref()
            .ok_or("Natív engine nincs beállítva!")?
            .generál(&prompt)?;

        // Emlék mentése
        self.emlék_mentése(bemenet, &válasz);

        Ok(válasz)
    }

    /// Kódolás - kifejezetten kód generálás
    pub async fn kódolj(&mut self, feladat: &str) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Feladat: {}\n\nÍrj tiszta, működő kódot. Kommentezd magyarul.",
            feladat
        );

        self.bridge
            .chat(
                vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: "Te egy szakértő programozó vagy. Tiszta, hatékony kódot írsz. Magyar kommentek.".to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: prompt,
                    },
                ],
                Some(ModellTípus::Kódoló),
            )
            .await
    }

    /// Fordítás
    pub async fn fordíts(
        &mut self,
        szöveg: &str,
        nyelv: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Fordítsd le a következő szöveget {}-ra/re:\n\n{}",
            nyelv, szöveg
        );

        self.bridge
            .chat(
                vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: "Te egy professzionális fordító vagy. Természetes, folyékony fordításokat készítesz.".to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: prompt,
                    },
                ],
                Some(ModellTípus::Többnyelvű),
            )
            .await
    }

    /// Érzelem frissítése
    fn érzelem_frissítés(&mut self, szöveg: &str) {
        let szöveg_lower = szöveg.to_lowercase();

        // Pozitív jelek
        if szöveg_lower.contains("köszön")
            || szöveg_lower.contains("kösz")
            || szöveg_lower.contains("szuper")
            || szöveg_lower.contains("király")
        {
            self.érzelem.öröm = (self.érzelem.öröm + 0.1).min(1.0);
        }

        // Kérdés = kíváncsiság
        if szöveg_lower.contains('?')
            || szöveg_lower.contains("miért")
            || szöveg_lower.contains("hogyan")
        {
            self.érzelem.kíváncsiság = (self.érzelem.kíváncsiság + 0.1).min(1.0);
        }

        // Lelkesítő szavak
        if szöveg_lower.contains('!')
            || szöveg_lower.contains("wow")
            || szöveg_lower.contains("zseni")
        {
            self.érzelem.lelkesedés = (self.érzelem.lelkesedés + 0.15).min(1.0);
        }
    }

    /// Emlék mentése
    fn emlék_mentése(&mut self, bemenet: &str, válasz: &str) {
        let emlék = Emlék {
            tartalom: format!("Kérdés: {}\nVálasz: {}", bemenet, válasz),
            fontosság: 0.5,
            érzelem: self.érzelem.clone(),
            időbélyeg: std::time::SystemTime::now(),
        };

        // Rövid memóriába
        if self.rövid_memória.len() >= 10 {
            // Ha fontos volt, hosszú memóriába
            if let Some(régi) = self.rövid_memória.pop_front() {
                if régi.fontosság > 0.7 {
                    self.hosszú_memória.push(régi);
                }
            }
        }
        self.rövid_memória.push_back(emlék);
    }

    /// Visszajelzés az entitásnak (tanulás)
    pub fn visszajelzés(&mut self, pozitív: bool) {
        if let Some(utolsó) = self.rövid_memória.back_mut() {
            if pozitív {
                utolsó.fontosság = (utolsó.fontosság + 0.2).min(1.0);
                self.érzelem.öröm = (self.érzelem.öröm + 0.1).min(1.0);
            } else {
                utolsó.fontosság = (utolsó.fontosság - 0.1).max(0.0);
            }
        }
    }

    /// Állapot lekérdezése
    pub fn állapot(&self) -> String {
        let mód_str = match self.mode {
            InferenceMode::Ollama => "🌐 Ollama (HTTP)",
            InferenceMode::Native => "⚡ Natív (GGUF beolvasztva)",
        };

        let teacher_mód = if self.teacher.szigorú_mód() {
            "🔴 SZIGORÚ"
        } else {
            "🟢 Normál"
        };

        let önjavító_stat = self.self_repair.statisztikák();
        let önbizalom_str = format!("{:.0}%", önjavító_stat.önbizalom * 100.0);

        format!(
            "🧠 {} állapota:\n\
             ⚙️  Mód: {}\n\
             🎓 Teacher: {}\n\
             🔧 Önjavító: {} tanult lecke, önbizalom: {}\n\
             💚 Öröm: {:.0}%\n\
             🔍 Kíváncsiság: {:.0}%\n\
             😌 Nyugalom: {:.0}%\n\
             🔥 Lelkesedés: {:.0}%\n\
             💜 Empátia: {:.0}%\n\
             📝 Emlékek: {} rövid, {} hosszú",
            self.név,
            mód_str,
            teacher_mód,
            önjavító_stat.tanult_leckék,
            önbizalom_str,
            self.érzelem.öröm * 100.0,
            self.érzelem.kíváncsiság * 100.0,
            self.érzelem.nyugalom * 100.0,
            self.érzelem.lelkesedés * 100.0,
            self.érzelem.empátia * 100.0,
            self.rövid_memória.len(),
            self.hosszú_memória.len()
        )
    }

    /// Silent Teacher állapot
    pub fn teacher_állapot(&self) -> String {
        self.teacher.állapot()
    }

    /// Visszajelzés az entitásnak (tanulás)
    pub fn tanulás_visszajelzés(&mut self, pozitív: bool) {
        // Előző interakció
        if let Some(utolsó_emlék) = self.rövid_memória.back() {
            let részek: Vec<&str> = utolsó_emlék.tartalom.splitn(2, "\nVálasz: ").collect();
            if részek.len() == 2 {
                let bemenet = részek[0].strip_prefix("Kérdés: ").unwrap_or(részek[0]);
                let kimenet = részek[1];
                self.teacher.visszajelzés(bemenet, kimenet, pozitív);
            }
        }

        // Eredeti visszajelzés logika
        if let Some(utolsó) = self.rövid_memória.back_mut() {
            if pozitív {
                utolsó.fontosság = (utolsó.fontosság + 0.2).min(1.0);
                self.érzelem.öröm = (self.érzelem.öröm + 0.1).min(1.0);
            } else {
                utolsó.fontosság = (utolsó.fontosság - 0.1).max(0.0);
            }
        }
    }

    /// Teacher reset (új esély)
    pub fn teacher_reset(&mut self) {
        self.teacher.reset();
    }

    /// Önjavító visszajelzés - a felhasználó jelzi hogy jó vagy rossz volt a válasz
    pub fn önjavító_visszajelzés(&mut self, pozitív: bool) {
        if let Some(utolsó_emlék) = self.rövid_memória.back() {
            let részek: Vec<&str> = utolsó_emlék.tartalom.splitn(2, "\nVálasz: ").collect();
            if részek.len() == 2 {
                let bemenet = részek[0].strip_prefix("Kérdés: ").unwrap_or(részek[0]);
                let kimenet = részek[1];

                let visszajelzés = if pozitív {
                    Visszajelzés::Pozitív
                } else {
                    Visszajelzés::Negatív(HibaTípus::Egyéb("Felhasználói visszajelzés".into()))
                };

                self.self_repair.visszajelzés(bemenet, kimenet, visszajelzés);
            }
        }
    }

    /// Önjavító javítás - a felhasználó megadja a helyes választ
    pub fn önjavító_javítás(&mut self, helyes_válasz: &str) {
        if let Some(utolsó_emlék) = self.rövid_memória.back() {
            let részek: Vec<&str> = utolsó_emlék.tartalom.splitn(2, "\nVálasz: ").collect();
            if részek.len() == 2 {
                let bemenet = részek[0].strip_prefix("Kérdés: ").unwrap_or(részek[0]);
                let rossz_kimenet = részek[1];

                self.self_repair.visszajelzés(
                    bemenet,
                    rossz_kimenet,
                    Visszajelzés::Javítás(helyes_válasz.to_string())
                );
            }
        }
    }

    /// Önjavító állapot lekérdezése
    pub fn önjavító_állapot(&self) -> String {
        self.self_repair.állapot()
    }

    /// Önjavító statisztikák
    pub fn önjavító_statisztikák(&self) -> crate::self_repair::ÖnjavítóStatisztikák {
        self.self_repair.statisztikák()
    }

    /// Önjavító rendszer reset
    pub fn önjavító_reset(&mut self) {
        self.self_repair.reset();
    }

    /// Stratégia ajánlás lekérése
    pub fn stratégia_ajánlás(&self, bemenet: &str) -> StratégiaAjánlás {
        self.self_repair.stratégia_ajánlás(bemenet)
    }

    /// Feloldott modellek listázása
    pub fn modellek(&self) -> &[FeloldottModell] {
        self.bridge.modellek()
    }

    /// Történet törlése (új beszélgetés)
    pub fn új_beszélgetés(&mut self) {
        self.történet = vec![ChatMessage {
            role: "system".to_string(),
            content: self.rendszer_prompt.clone(),
        }];
    }

    /// Rendszer elérhetőség ellenőrzése
    pub async fn rendszer_kész(&self) -> bool {
        match self.mode {
            InferenceMode::Ollama => self.bridge.elérhető().await,
            InferenceMode::Native => {
                self.native_engine
                    .as_ref()
                    .map(|e| e.kész())
                    .unwrap_or(false)
            }
        }
    }

    /// Szinkron rendszer ellenőrzés (natív módhoz)
    pub fn rendszer_kész_sync(&self) -> bool {
        match self.mode {
            InferenceMode::Ollama => false, // Ollama módhoz async kell
            InferenceMode::Native => {
                self.native_engine
                    .as_ref()
                    .map(|e| e.kész())
                    .unwrap_or(false)
            }
        }
    }

    /// Natív engine referencia
    pub fn native_engine(&self) -> Option<&NativeEngine> {
        self.native_engine.as_ref()
    }

    /// Natív engine mutable referencia
    pub fn native_engine_mut(&mut self) -> Option<&mut NativeEngine> {
        self.native_engine.as_mut()
    }
}
