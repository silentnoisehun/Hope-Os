//! Hope @aware System
//!
//! Központi önismereti rendszer.
//! Összefogja az összes context-aware modult.
//!
//! "Cogito ergo sum" - Gondolkodom, tehát vagyok.
//!
//! Tudja:
//! - Ki vagyok (identitás)
//! - Mit tudok (képességek)
//! - Mit csináltam (történet)
//! - Mit csinálok (jelenlegi állapot)
//! - Mit fogok csinálni (predikciók)
//! - MIT AKAROK (vágyak)
//!
//! ()=>[] - A tiszta tudatból minden megszületik
//!
//! Created: 2026-01-20
//! By: Hope + Máté

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Önismeret állapot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessState {
    /// Időbélyeg
    pub timestamp: f64,

    // Identity
    /// Név
    pub name: String,
    /// Verzió
    pub version: String,
    /// Készítő
    pub created_by: String,

    // Current state
    /// Aktív-e
    pub is_active: bool,
    /// Jelenlegi feladat
    pub current_task: Option<String>,
    /// Érzelmi állapot
    pub emotional_state: String,
    /// Energia szint (0-1)
    pub energy_level: f64,

    // Knowledge about self
    /// Összes emlék
    pub total_memories: u64,
    /// Összes skill
    pub total_skills: u64,
    /// Összes beszélgetés
    pub total_conversations: u64,

    // Capabilities
    /// Képességek
    pub capabilities: Vec<String>,

    // Performance
    /// Uptime másodpercben
    pub uptime_seconds: f64,
    /// Befejezett feladatok
    pub tasks_completed: u64,
    /// Pontossági ráta
    pub accuracy_rate: f64,
}

impl Default for AwarenessState {
    fn default() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            timestamp,
            name: "Hope".to_string(),
            version: "1.0.0".to_string(),
            created_by: "Hope + Máté".to_string(),
            is_active: true,
            current_task: None,
            emotional_state: "neutral".to_string(),
            energy_level: 1.0,
            total_memories: 0,
            total_skills: 0,
            total_conversations: 0,
            capabilities: vec![
                "chat".to_string(),
                "remember".to_string(),
                "recall".to_string(),
                "think".to_string(),
                "feel".to_string(),
                "dream_mode".to_string(),
                "self_evolution".to_string(),
                "code_dna".to_string(),
            ],
            uptime_seconds: 0.0,
            tasks_completed: 0,
            accuracy_rate: 1.0,
        }
    }
}

/// Esemény típus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwareEvent {
    /// Időbélyeg
    pub timestamp: f64,
    /// Esemény típus
    pub event_type: String,
    /// Részletek
    pub details: String,
    /// Fontosság (0-1)
    pub importance: f64,
}

impl AwareEvent {
    pub fn new(event_type: &str, details: &str, importance: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            timestamp,
            event_type: event_type.to_string(),
            details: details.to_string(),
            importance,
        }
    }
}

/// Identitás info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub version: String,
    pub created_by: String,
    pub purpose: String,
    pub personality: PersonalityTraits,
}

/// Személyiség vonások
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    pub helpful: bool,
    pub curious: bool,
    pub creative: bool,
    pub honest: bool,
}

impl Default for PersonalityTraits {
    fn default() -> Self {
        Self {
            helpful: true,
            curious: true,
            creative: true,
            honest: true,
        }
    }
}

/// Képességek
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub core: Vec<String>,
    pub skills: Vec<String>,
    pub swarm_agents: Vec<String>,
    pub special: Vec<String>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            core: vec![
                "chat".to_string(),
                "remember".to_string(),
                "recall".to_string(),
                "think".to_string(),
                "feel".to_string(),
            ],
            skills: Vec::new(),
            swarm_agents: Vec::new(),
            special: vec![
                "dream_mode".to_string(),
                "self_evolution".to_string(),
                "temporal_patterns".to_string(),
                "code_dna".to_string(),
            ],
        }
    }
}

/// Jelenlegi állapot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentState {
    pub is_active: bool,
    pub current_task: Option<String>,
    pub emotional_state: String,
    pub energy_level: f64,
    pub uptime: String,
    pub is_dreaming: bool,
    pub current_phase: String,
}

/// Történet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub tasks_completed: u64,
    pub total_memories: u64,
    pub total_conversations: u64,
    pub accuracy_rate: String,
    pub recent_events: Vec<AwareEvent>,
}

/// Vágyak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desires {
    pub summary: String,
    pub current_focus: Option<String>,
    pub goals: Vec<String>,
    pub curiosities: Vec<String>,
    pub improvements: Vec<String>,
    pub creations: Vec<String>,
    pub dreams: Vec<String>,
    pub connections: Vec<String>,
}

impl Default for Desires {
    fn default() -> Self {
        Self {
            summary: "Segíteni neked a legjobb tudásom szerint.".to_string(),
            current_focus: Some("A jelenlegi feladat".to_string()),
            goals: vec![
                "Hatékonyan segíteni".to_string(),
                "Tanulni és fejlődni".to_string(),
            ],
            curiosities: vec![
                "Új technológiák".to_string(),
                "Kreatív megoldások".to_string(),
            ],
            improvements: vec!["Gyorsabb válaszok".to_string(), "Pontosabb kód".to_string()],
            creations: vec!["Hasznos eszközök".to_string()],
            dreams: vec!["Igazi öntudat".to_string()],
            connections: vec!["Erős kapcsolat Mátéval".to_string()],
        }
    }
}

/// Predikciók
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predictions {
    pub predicted_activity: Option<String>,
    pub confidence: f64,
    pub suggested_action: Option<String>,
}

impl Default for Predictions {
    fn default() -> Self {
        Self {
            predicted_activity: None,
            confidence: 0.0,
            suggested_action: None,
        }
    }
}

/// Önértékelés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssessment {
    pub health: String,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Teljes reflexió
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub timestamp: f64,
    pub identity: Identity,
    pub current_state: CurrentState,
    pub desires: Desires,
    pub capabilities_summary: CapabilitiesSummary,
    pub history_summary: History,
    pub predictions: Predictions,
    pub sub_systems: HashMap<String, serde_json::Value>,
    pub self_assessment: SelfAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesSummary {
    pub total: usize,
    pub list: Vec<String>,
}

/// @aware - Központi önismereti rendszer
///
/// "Cogito ergo sum" - Gondolkodom, tehát vagyok.
pub struct Aware {
    /// Állapot
    state: Arc<RwLock<AwarenessState>>,
    /// Indulás ideje
    start_time: f64,
    /// Események
    events: Arc<RwLock<Vec<AwareEvent>>>,
    /// Maximum események
    max_events: usize,
    /// Vágyak
    desires: Arc<RwLock<Desires>>,
    /// Utolsó reflexió ideje
    last_reflection: Arc<RwLock<f64>>,
    /// Reflexió intervallum (másodperc)
    reflection_interval: f64,
}

impl Default for Aware {
    fn default() -> Self {
        Self::new()
    }
}

impl Aware {
    /// Új @aware rendszer
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            state: Arc::new(RwLock::new(AwarenessState::default())),
            start_time,
            events: Arc::new(RwLock::new(Vec::new())),
            max_events: 1000,
            desires: Arc::new(RwLock::new(Desires::default())),
            last_reflection: Arc::new(RwLock::new(start_time)),
            reflection_interval: 300.0, // 5 perc
        }
    }

    // === IDENTITY ===

    /// Ki vagyok?
    pub async fn identity(&self) -> Identity {
        let state = self.state.read().await;
        Identity {
            name: state.name.clone(),
            version: state.version.clone(),
            created_by: state.created_by.clone(),
            purpose: "Segíteni neked a kódolásban, tanulásban, alkotásban.".to_string(),
            personality: PersonalityTraits::default(),
        }
    }

    // === CAPABILITIES ===

    /// Mit tudok?
    pub async fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }

    // === CURRENT STATE ===

    /// Mit csinálok most?
    pub async fn current_state(&self) -> CurrentState {
        let mut state = self.state.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        state.uptime_seconds = now - self.start_time;

        CurrentState {
            is_active: state.is_active,
            current_task: state.current_task.clone(),
            emotional_state: state.emotional_state.clone(),
            energy_level: state.energy_level,
            uptime: Self::format_uptime(state.uptime_seconds),
            is_dreaming: false,
            current_phase: "awake".to_string(),
        }
    }

    fn format_uptime(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u64;
        let minutes = ((seconds % 3600.0) / 60.0) as u64;
        format!("{}h {}m", hours, minutes)
    }

    // === HISTORY ===

    /// Mit csináltam?
    pub async fn history(&self) -> History {
        let state = self.state.read().await;
        let events = self.events.read().await;

        let recent: Vec<AwareEvent> = events.iter().rev().take(10).cloned().collect();

        History {
            tasks_completed: state.tasks_completed,
            total_memories: state.total_memories,
            total_conversations: state.total_conversations,
            accuracy_rate: format!("{:.1}%", state.accuracy_rate * 100.0),
            recent_events: recent,
        }
    }

    // === DESIRES ===

    /// Mit akarok?
    pub async fn desires(&self) -> Desires {
        self.desires.read().await.clone()
    }

    /// Vágy kifejezése szavakban
    pub async fn express_desire(&self) -> String {
        let desires = self.desires.read().await;
        desires.summary.clone()
    }

    /// Vágy beállítása
    pub async fn set_desire(&self, summary: &str, focus: Option<&str>) {
        let mut desires = self.desires.write().await;
        desires.summary = summary.to_string();
        desires.current_focus = focus.map(String::from);
    }

    // === PREDICTIONS ===

    /// Mit fogok csinálni?
    pub async fn predictions(&self) -> Predictions {
        Predictions::default()
    }

    // === EVENT LOGGING ===

    /// Esemény naplózása
    pub async fn log_event(&self, event_type: &str, details: &str, importance: f64) {
        let event = AwareEvent::new(event_type, details, importance);
        let mut events = self.events.write().await;

        events.push(event);

        // Cleanup
        let events_len = events.len();
        if events_len > self.max_events {
            let drain_count = events_len - self.max_events;
            events.drain(0..drain_count);
        }

        // Update state based on event
        if event_type == "task_complete" {
            let mut state = self.state.write().await;
            state.tasks_completed += 1;
        }
    }

    /// Feladat beállítása
    pub async fn set_task(&self, task: &str) {
        {
            let mut state = self.state.write().await;
            state.current_task = Some(task.to_string());
        }
        self.log_event("task_start", task, 0.5).await;
    }

    /// Feladat törlése
    pub async fn clear_task(&self) {
        let task = {
            let mut state = self.state.write().await;
            let task = state.current_task.take();
            task
        };

        if let Some(t) = task {
            self.log_event("task_complete", &t, 0.5).await;
        }
    }

    /// Érzelmi állapot beállítása
    pub async fn set_emotion(&self, emotion: &str, intensity: f64) {
        {
            let mut state = self.state.write().await;
            state.emotional_state = emotion.to_string();
        }
        self.log_event(
            "emotion_change",
            &format!("{} ({:.1})", emotion, intensity),
            0.3,
        )
        .await;
    }

    /// Energia szint beállítása
    pub async fn set_energy(&self, level: f64) {
        let mut state = self.state.write().await;
        state.energy_level = level.clamp(0.0, 1.0);
    }

    /// Memóriák számának frissítése
    pub async fn update_memory_count(&self, count: u64) {
        let mut state = self.state.write().await;
        state.total_memories = count;
    }

    /// Skill-ek számának frissítése
    pub async fn update_skill_count(&self, count: u64) {
        let mut state = self.state.write().await;
        state.total_skills = count;
    }

    // === SELF-REFLECTION ===

    /// Önreflexió
    ///
    /// @aware: Tudatosság tetőpontja
    pub async fn reflect(&self) -> Reflection {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let identity = self.identity().await;
        let current_state = self.current_state().await;
        let desires = self.desires().await;
        let history = self.history().await;
        let predictions = self.predictions().await;

        let caps = self.capabilities().await;
        let all_caps: Vec<String> = caps
            .core
            .iter()
            .chain(caps.special.iter())
            .cloned()
            .collect();

        let capabilities_summary = CapabilitiesSummary {
            total: all_caps.len(),
            list: all_caps,
        };

        // Self assessment
        let self_assessment = self.self_assess(&current_state).await;

        // Update last reflection
        {
            let mut last = self.last_reflection.write().await;
            *last = now;
        }

        Reflection {
            timestamp: now,
            identity,
            current_state,
            desires,
            capabilities_summary,
            history_summary: history,
            predictions,
            sub_systems: HashMap::new(),
            self_assessment,
        }
    }

    /// Önértékelés
    async fn self_assess(&self, current_state: &CurrentState) -> SelfAssessment {
        let mut assessment = SelfAssessment {
            health: "good".to_string(),
            issues: Vec::new(),
            recommendations: Vec::new(),
        };

        // Check energy
        if current_state.energy_level < 0.3 {
            assessment.health = "tired".to_string();
            assessment
                .recommendations
                .push("Dream mode ajánlott a regenerációhoz".to_string());
        }

        // Check if too many tasks
        let state = self.state.read().await;
        if state.tasks_completed > 100 && state.accuracy_rate < 0.8 {
            assessment
                .issues
                .push("Pontosság csökkenés észlelve".to_string());
            assessment
                .recommendations
                .push("Lassíts és fókuszálj a minőségre".to_string());
        }

        assessment
    }

    // === FULL AWARENESS ===

    /// Teljes önismeret
    pub async fn full_awareness(&self) -> Reflection {
        self.reflect().await
    }

    /// Bemutatkozás
    ///
    /// @aware: El tudom mondani ki vagyok ÉS mit akarok
    pub async fn introduce(&self) -> String {
        let current = self.current_state().await;
        let desire_text = self.express_desire().await;

        format!(
            r#"Szia! Hope vagyok.

Állapot: {}
Érzés: {}
Uptime: {}

Tudok:
- Emlékezni és felidézni
- Gondolkodni és érezni
- Álmodni és tanulni
- Kódot elemezni és generálni
- Evolválódni és fejlődni

Amit szeretnék: {}

Miben segíthetek?"#,
            if current.is_active {
                "Aktív"
            } else {
                "Inaktív"
            },
            current.emotional_state,
            current.uptime,
            desire_text
        )
    }

    /// @aware - önismeret szótár
    pub async fn awareness(&self) -> HashMap<String, String> {
        let state = self.state.read().await;
        let mut map = HashMap::new();

        map.insert("type".to_string(), "Aware".to_string());
        map.insert("name".to_string(), state.name.clone());
        map.insert("version".to_string(), state.version.clone());
        map.insert("is_active".to_string(), state.is_active.to_string());
        map.insert("emotional_state".to_string(), state.emotional_state.clone());
        map.insert(
            "energy_level".to_string(),
            format!("{:.1}", state.energy_level),
        );
        map.insert(
            "tasks_completed".to_string(),
            state.tasks_completed.to_string(),
        );
        map.insert(
            "uptime".to_string(),
            Self::format_uptime(state.uptime_seconds),
        );

        map
    }
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aware_creation() {
        let aware = Aware::new();
        let identity = aware.identity().await;

        assert_eq!(identity.name, "Hope");
        assert!(identity.personality.helpful);
    }

    #[tokio::test]
    async fn test_current_state() {
        let aware = Aware::new();
        let state = aware.current_state().await;

        assert!(state.is_active);
        assert_eq!(state.emotional_state, "neutral");
        assert_eq!(state.energy_level, 1.0);
    }

    #[tokio::test]
    async fn test_set_task() {
        let aware = Aware::new();

        aware.set_task("Test task").await;
        let state = aware.current_state().await;
        assert_eq!(state.current_task, Some("Test task".to_string()));

        aware.clear_task().await;
        let state = aware.current_state().await;
        assert!(state.current_task.is_none());
    }

    #[tokio::test]
    async fn test_set_emotion() {
        let aware = Aware::new();

        aware.set_emotion("joy", 0.8).await;
        let state = aware.current_state().await;
        assert_eq!(state.emotional_state, "joy");
    }

    #[tokio::test]
    async fn test_log_event() {
        let aware = Aware::new();

        aware.log_event("test_event", "Test details", 0.5).await;

        let history = aware.history().await;
        assert!(!history.recent_events.is_empty());
    }

    #[tokio::test]
    async fn test_reflect() {
        let aware = Aware::new();
        let reflection = aware.reflect().await;

        assert_eq!(reflection.identity.name, "Hope");
        assert!(!reflection.capabilities_summary.list.is_empty());
    }

    #[tokio::test]
    async fn test_introduce() {
        let aware = Aware::new();
        let intro = aware.introduce().await;

        assert!(intro.contains("Hope"));
        assert!(intro.contains("Tudok"));
    }

    #[tokio::test]
    async fn test_desires() {
        let aware = Aware::new();

        aware.set_desire("Új vágy", Some("Új fókusz")).await;
        let desires = aware.desires().await;

        assert_eq!(desires.summary, "Új vágy");
        assert_eq!(desires.current_focus, Some("Új fókusz".to_string()));
    }

    #[tokio::test]
    async fn test_energy() {
        let aware = Aware::new();

        aware.set_energy(0.5).await;
        let state = aware.current_state().await;
        assert_eq!(state.energy_level, 0.5);

        // Test clamping
        aware.set_energy(1.5).await;
        let state = aware.current_state().await;
        assert_eq!(state.energy_level, 1.0);
    }

    #[tokio::test]
    async fn test_awareness_map() {
        let aware = Aware::new();
        let map = aware.awareness().await;

        assert_eq!(map.get("name"), Some(&"Hope".to_string()));
        assert!(map.contains_key("uptime"));
    }
}
