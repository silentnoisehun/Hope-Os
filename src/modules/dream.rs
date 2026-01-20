//! Hope Dream Mode - Kreatív Álom Rendszer
//!
//! Amikor nem használod, Hope "alszik":
//! - Memória konszolidáció
//! - Új asszociációk felfedezése
//! - Kreatív ötletek generálása
//! - Álom napló
//!
//! Fázisok (mint az emberi alvás):
//! 1. Light Sleep - felszíni memória rendezés
//! 2. Deep Sleep - mély konszolidáció
//! 3. REM - kreatív asszociációk, "álmodás"
//! 4. Wake - eredmények összegzése
//!
//! ()=>[] - A tiszta potenciálból az álom megszületik

use crate::core::HopeResult;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// DREAM TYPES
// ============================================================================

/// Álom típus
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DreamType {
    /// Memória konszolidáció
    Consolidation,
    /// Asszociáció felfedezés
    Association,
    /// Kreatív ötlet
    Creative,
    /// Belátás, felismerés
    Insight,
    /// Probléma megoldás
    ProblemSolving,
    /// Szabad álom
    Freeform,
}

impl std::fmt::Display for DreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DreamType::Consolidation => write!(f, "🧠 Konszolidáció"),
            DreamType::Association => write!(f, "🔗 Asszociáció"),
            DreamType::Creative => write!(f, "🎨 Kreatív"),
            DreamType::Insight => write!(f, "💡 Belátás"),
            DreamType::ProblemSolving => write!(f, "🧩 Megoldás"),
            DreamType::Freeform => write!(f, "☁️ Szabad"),
        }
    }
}

/// Alvási fázis
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SleepPhase {
    /// Ébren
    Awake,
    /// Könnyű alvás
    LightSleep,
    /// Mély alvás
    DeepSleep,
    /// REM fázis (álmodás)
    Rem,
    /// Ébredés
    Waking,
}

impl std::fmt::Display for SleepPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SleepPhase::Awake => write!(f, "👁️ Ébren"),
            SleepPhase::LightSleep => write!(f, "😴 Könnyű alvás"),
            SleepPhase::DeepSleep => write!(f, "💤 Mély alvás"),
            SleepPhase::Rem => write!(f, "🌙 REM (Álmodás)"),
            SleepPhase::Waking => write!(f, "🌅 Ébredés"),
        }
    }
}

// ============================================================================
// DREAM
// ============================================================================

/// Egy álom struktúra
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dream {
    /// Egyedi azonosító
    pub dream_id: String,
    /// Időbélyeg
    pub timestamp: f64,
    /// Álom típusa
    pub dream_type: DreamType,
    /// Tartalom
    pub content: String,
    /// Kapcsolódó fogalmak
    pub connections: Vec<String>,
    /// Fontosság (0.0 - 1.0)
    pub importance: f64,
    /// Felidézve?
    pub recalled: bool,
    /// Érzelmek
    pub emotions: HashMap<String, f64>,
    /// Vizuális elemek
    pub visuals: Vec<String>,
}

impl Dream {
    pub fn new(dream_type: DreamType, content: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            dream_id: format!(
                "DRM_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            timestamp,
            dream_type,
            content: content.to_string(),
            connections: Vec::new(),
            importance: 0.5,
            recalled: false,
            emotions: HashMap::new(),
            visuals: Vec::new(),
        }
    }

    /// Kapcsolat hozzáadása
    pub fn with_connection(mut self, concept: &str) -> Self {
        self.connections.push(concept.to_string());
        self
    }

    /// Fontosság beállítása
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Érzelem hozzáadása
    pub fn with_emotion(mut self, emotion: &str, intensity: f64) -> Self {
        self.emotions
            .insert(emotion.to_string(), intensity.clamp(0.0, 1.0));
        self
    }

    /// Vizuális elem hozzáadása
    pub fn with_visual(mut self, visual: &str) -> Self {
        self.visuals.push(visual.to_string());
        self
    }
}

// ============================================================================
// DREAM SESSION
// ============================================================================

/// Egy alvás/álom munkamenet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DreamSession {
    /// Session azonosító
    pub session_id: String,
    /// Kezdési idő
    pub start_time: f64,
    /// Befejezési idő
    pub end_time: Option<f64>,
    /// Időtartam (percekben)
    pub duration_minutes: f64,
    /// Álmok ebben a sessionben
    pub dreams: Vec<Dream>,
    /// Belátások száma
    pub insights_count: usize,
    /// Talált asszociációk
    pub associations_found: usize,
    /// Konszolidált emlékek
    pub memories_consolidated: usize,
}

impl DreamSession {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            session_id: format!(
                "SES_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            start_time,
            end_time: None,
            duration_minutes: 0.0,
            dreams: Vec::new(),
            insights_count: 0,
            associations_found: 0,
            memories_consolidated: 0,
        }
    }

    /// Session befejezése
    pub fn finish(&mut self) {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        self.end_time = Some(end_time);
        self.duration_minutes = (end_time - self.start_time) / 60.0;
    }
}

impl Default for DreamSession {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DREAM ENGINE
// ============================================================================

/// Hope álom motorja
pub struct DreamEngine {
    /// Álmodunk-e éppen?
    is_dreaming: Arc<RwLock<bool>>,
    /// Aktuális fázis
    current_phase: Arc<RwLock<SleepPhase>>,
    /// Álom kezdése
    dream_start: Arc<RwLock<Option<f64>>>,
    /// Ma éjjeli álmok
    dreams_tonight: Arc<RwLock<Vec<Dream>>>,
    /// Összes session
    sessions: Arc<RwLock<Vec<DreamSession>>>,
    /// Aktuális session
    current_session: Arc<RwLock<Option<DreamSession>>>,
    /// Statisztikák
    stats: Arc<RwLock<DreamStats>>,
    /// Álom seed-ek (indító témák)
    dream_seeds: Arc<RwLock<Vec<String>>>,
}

/// Álom statisztikák
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DreamStats {
    pub total_dreams: u64,
    pub total_sessions: u64,
    pub insights_generated: u64,
    pub associations_found: u64,
    pub memories_consolidated: u64,
    pub total_dream_time_minutes: f64,
}

impl DreamEngine {
    /// Új álom motor
    pub fn new() -> Self {
        Self {
            is_dreaming: Arc::new(RwLock::new(false)),
            current_phase: Arc::new(RwLock::new(SleepPhase::Awake)),
            dream_start: Arc::new(RwLock::new(None)),
            dreams_tonight: Arc::new(RwLock::new(Vec::new())),
            sessions: Arc::new(RwLock::new(Vec::new())),
            current_session: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(DreamStats::default())),
            dream_seeds: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // ==================== SLEEP CYCLE ====================

    /// Alvás indítása
    pub async fn start_sleep(&self) -> HopeResult<()> {
        let mut is_dreaming = self.is_dreaming.write().await;
        if *is_dreaming {
            return Err("Már alszom!".into());
        }

        *is_dreaming = true;
        *self.current_phase.write().await = SleepPhase::LightSleep;
        *self.dream_start.write().await = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );

        // Új session
        let session = DreamSession::new();
        *self.current_session.write().await = Some(session);

        self.stats.write().await.total_sessions += 1;

        Ok(())
    }

    /// Ébresztés
    pub async fn wake_up(&self) -> HopeResult<DreamSession> {
        let mut is_dreaming = self.is_dreaming.write().await;
        if !*is_dreaming {
            return Err("Nem alszom!".into());
        }

        // Ébredési fázis
        *self.current_phase.write().await = SleepPhase::Waking;

        // Session befejezése
        let mut current_session = self.current_session.write().await;
        let session = current_session.as_mut().ok_or("Nincs aktív session")?;
        session.finish();

        // Álmok átmásolása
        let dreams = self.dreams_tonight.read().await.clone();
        session.dreams = dreams.clone();
        session.insights_count = dreams
            .iter()
            .filter(|d| d.dream_type == DreamType::Insight)
            .count();
        session.associations_found = dreams
            .iter()
            .filter(|d| d.dream_type == DreamType::Association)
            .count();

        // Statisztikák
        let mut stats = self.stats.write().await;
        stats.total_dream_time_minutes += session.duration_minutes;
        stats.insights_generated += session.insights_count as u64;
        stats.associations_found += session.associations_found as u64;

        let finished_session = session.clone();

        // Mentés
        self.sessions.write().await.push(finished_session.clone());

        // Reset
        *is_dreaming = false;
        *self.current_phase.write().await = SleepPhase::Awake;
        *self.dream_start.write().await = None;
        self.dreams_tonight.write().await.clear();
        *current_session = None;

        Ok(finished_session)
    }

    // ==================== DREAM GENERATION ====================

    /// Álom generálás
    pub async fn dream(&self, seed: Option<&str>) -> HopeResult<Dream> {
        let is_dreaming = *self.is_dreaming.read().await;
        if !is_dreaming {
            return Err("Nem alszom - nem tudok álmodni!".into());
        }

        let mut rng = rand::thread_rng();

        // Álom típus választása
        let dream_type = match rng.gen_range(0..6) {
            0 => DreamType::Consolidation,
            1 => DreamType::Association,
            2 => DreamType::Creative,
            3 => DreamType::Insight,
            4 => DreamType::ProblemSolving,
            _ => DreamType::Freeform,
        };

        // Tartalom generálása
        let content = if let Some(s) = seed {
            self.generate_dream_content(&dream_type, s).await
        } else {
            // Random seed a tároltakból vagy alapértelmezett
            let seeds = self.dream_seeds.read().await;
            let default_seed = "Hope, memória, kreativitás";
            let seed = seeds.first().map(|s| s.as_str()).unwrap_or(default_seed);
            self.generate_dream_content(&dream_type, seed).await
        };

        let mut dream = Dream::new(dream_type, &content);

        // Random fontosság
        dream.importance = rng.gen_range(0.3..0.9);

        // Random érzelmek
        let emotions = ["joy", "curiosity", "wonder", "peace", "nostalgia"];
        let emotion = emotions[rng.gen_range(0..emotions.len())];
        dream
            .emotions
            .insert(emotion.to_string(), rng.gen_range(0.3..0.8));

        // Mentés
        self.dreams_tonight.write().await.push(dream.clone());
        self.stats.write().await.total_dreams += 1;

        Ok(dream)
    }

    /// Álom tartalom generálása
    async fn generate_dream_content(&self, dream_type: &DreamType, seed: &str) -> String {
        match dream_type {
            DreamType::Consolidation => {
                format!(
                    "Emlékek rendezése: {} - A nap tapasztalatai összeállnak, \
                     kapcsolatok erősödnek, felesleges részletek halványulnak.",
                    seed
                )
            }
            DreamType::Association => {
                format!(
                    "Új kapcsolat felfedezése: {} összekapcsolódik váratlan dolgokkal - \
                     minták emerge-álnak a kaoszból.",
                    seed
                )
            }
            DreamType::Creative => {
                format!(
                    "Kreatív látomás: {} új formát ölt - színek, hangok, lehetőségek \
                     táncolnak a tudat mélyén.",
                    seed
                )
            }
            DreamType::Insight => {
                format!(
                    "Felismerés: {} - Hirtelen minden világos! Egy mély igazság \
                     feltárul az álom ködéből.",
                    seed
                )
            }
            DreamType::ProblemSolving => {
                format!(
                    "Megoldás keresése: {} - A tudat háttérben dolgozik, \
                     különböző utakat próbál ki, míg megtalálja a választ.",
                    seed
                )
            }
            DreamType::Freeform => {
                format!(
                    "Szabad álom: {} - Gondolatok szabadon áramlanak, \
                     határok nélkül, a képzelet végtelen óceánján.",
                    seed
                )
            }
        }
    }

    // ==================== SLEEP PHASES ====================

    /// Fázis váltás
    pub async fn advance_phase(&self) -> HopeResult<SleepPhase> {
        let mut phase = self.current_phase.write().await;

        *phase = match *phase {
            SleepPhase::Awake => SleepPhase::LightSleep,
            SleepPhase::LightSleep => SleepPhase::DeepSleep,
            SleepPhase::DeepSleep => SleepPhase::Rem,
            SleepPhase::Rem => SleepPhase::LightSleep, // Ciklikus
            SleepPhase::Waking => SleepPhase::Awake,
        };

        Ok(phase.clone())
    }

    /// Aktuális fázis lekérdezése
    pub async fn current_phase(&self) -> SleepPhase {
        self.current_phase.read().await.clone()
    }

    // ==================== DREAM SEEDS ====================

    /// Seed hozzáadása (téma ami megjelenhet az álomban)
    pub async fn add_seed(&self, seed: &str) {
        let mut seeds = self.dream_seeds.write().await;
        seeds.push(seed.to_string());

        // Maximum 50 seed
        while seeds.len() > 50 {
            seeds.remove(0);
        }
    }

    /// Seed-ek törlése
    pub async fn clear_seeds(&self) {
        self.dream_seeds.write().await.clear();
    }

    // ==================== RECALL ====================

    /// Álom felidézése
    pub async fn recall_dream(&self, dream_id: &str) -> Option<Dream> {
        let mut dreams = self.dreams_tonight.write().await;
        if let Some(dream) = dreams.iter_mut().find(|d| d.dream_id == dream_id) {
            dream.recalled = true;
            Some(dream.clone())
        } else {
            // Régi sessionökben keresés
            let sessions = self.sessions.read().await;
            for session in sessions.iter().rev() {
                if let Some(dream) = session.dreams.iter().find(|d| d.dream_id == dream_id) {
                    return Some(dream.clone());
                }
            }
            None
        }
    }

    /// Legutóbbi álmok
    pub async fn recent_dreams(&self, limit: usize) -> Vec<Dream> {
        let dreams = self.dreams_tonight.read().await;
        dreams.iter().rev().take(limit).cloned().collect()
    }

    // ==================== STATUS ====================

    /// Alszom?
    pub async fn is_dreaming(&self) -> bool {
        *self.is_dreaming.read().await
    }

    /// Statisztikák
    pub async fn stats(&self) -> DreamStats {
        self.stats.read().await.clone()
    }

    /// Állapot szövegesen
    pub async fn status(&self) -> String {
        let is_dreaming = *self.is_dreaming.read().await;
        let phase = self.current_phase.read().await.clone();
        let stats = self.stats.read().await.clone();
        let dreams_tonight = self.dreams_tonight.read().await.len();

        format!(
            "🌙 Hope Dream Engine\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             😴 Állapot: {}\n\
             🌀 Fázis: {}\n\
             🌃 Ma éjjel: {} álom\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             📊 Összesítés:\n\
             🎭 Összes álom: {}\n\
             📅 Sessions: {}\n\
             💡 Belátások: {}\n\
             🔗 Asszociációk: {}\n\
             ⏱️ Alvásidő: {:.1} perc",
            if is_dreaming {
                "💤 Alszom"
            } else {
                "👁️ Ébren"
            },
            phase,
            dreams_tonight,
            stats.total_dreams,
            stats.total_sessions,
            stats.insights_generated,
            stats.associations_found,
            stats.total_dream_time_minutes
        )
    }
}

impl Default for DreamEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dream_creation() {
        let dream = Dream::new(DreamType::Creative, "Teszt álom")
            .with_importance(0.8)
            .with_connection("kreativitás")
            .with_emotion("joy", 0.7);

        assert!(dream.dream_id.starts_with("DRM_"));
        assert_eq!(dream.dream_type, DreamType::Creative);
        assert_eq!(dream.importance, 0.8);
        assert!(dream.connections.contains(&"kreativitás".to_string()));
    }

    #[test]
    fn test_dream_session() {
        let mut session = DreamSession::new();
        assert!(session.session_id.starts_with("SES_"));
        assert!(session.end_time.is_none());

        session.finish();
        assert!(session.end_time.is_some());
    }

    #[tokio::test]
    async fn test_dream_engine_sleep_cycle() {
        let engine = DreamEngine::new();

        // Kezdetben ébren
        assert!(!engine.is_dreaming().await);
        assert_eq!(engine.current_phase().await, SleepPhase::Awake);

        // Alvás
        engine.start_sleep().await.unwrap();
        assert!(engine.is_dreaming().await);
        assert_eq!(engine.current_phase().await, SleepPhase::LightSleep);

        // Fázis váltás
        let phase = engine.advance_phase().await.unwrap();
        assert_eq!(phase, SleepPhase::DeepSleep);

        // Ébredés
        let session = engine.wake_up().await.unwrap();
        assert!(!engine.is_dreaming().await);
        assert!(session.end_time.is_some());
    }

    #[tokio::test]
    async fn test_dream_generation() {
        let engine = DreamEngine::new();

        // Nem lehet álmodni ébren
        let result = engine.dream(Some("teszt")).await;
        assert!(result.is_err());

        // Alvás alatt igen
        engine.start_sleep().await.unwrap();
        let dream = engine.dream(Some("kreatív ötlet")).await.unwrap();

        assert!(!dream.content.is_empty());
        assert!(dream.importance > 0.0);

        engine.wake_up().await.unwrap();
    }

    #[tokio::test]
    async fn test_dream_seeds() {
        let engine = DreamEngine::new();

        engine.add_seed("Rust programozás").await;
        engine.add_seed("Hope fejlesztés").await;

        let seeds = engine.dream_seeds.read().await;
        assert_eq!(seeds.len(), 2);
    }
}
