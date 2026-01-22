//! Hope OS - Attention/Focus Module
//!
//! Az attention rendszer kezeli Hope fókuszát és figyelmét.
//!
//! Fő funkciók:
//! - Explicit fókusz célok (user-defined keywords)
//! - Implicit kontextus súlyozás
//! - Attention capacity kezelés
//! - Dream módban Diffuse mode (kreatív asszociációk)
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// ATTENTION TYPES
// ============================================================================

/// Attention mód
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionMode {
    /// Explicit targets active, strict filtering
    Focused,
    /// Balanced explicit + implicit
    Normal,
    /// Dream mode, low filtering, creative associations
    Diffuse,
}

impl Default for AttentionMode {
    fn default() -> Self {
        AttentionMode::Normal
    }
}

impl std::fmt::Display for AttentionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttentionMode::Focused => write!(f, "🎯 Focused"),
            AttentionMode::Normal => write!(f, "👁️ Normal"),
            AttentionMode::Diffuse => write!(f, "☁️ Diffuse"),
        }
    }
}

/// Fókusz cél (explicit target)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FocusTarget {
    /// Kulcsszó amire figyelünk
    pub keyword: String,
    /// Súlyozás (1.0 - 3.0 boost)
    pub weight: f64,
    /// Lejárati idő (opcionális)
    pub expires_at: Option<DateTime<Utc>>,
    /// Létrehozás ideje
    pub created_at: DateTime<Utc>,
}

impl FocusTarget {
    /// Új fókusz cél létrehozása
    pub fn new(keyword: &str, weight: f64) -> Self {
        Self {
            keyword: keyword.to_lowercase(),
            weight: weight.clamp(1.0, 3.0),
            expires_at: None,
            created_at: Utc::now(),
        }
    }

    /// Fókusz cél lejárati idővel
    pub fn with_duration(keyword: &str, weight: f64, duration_secs: i64) -> Self {
        let expires_at = Utc::now() + chrono::Duration::seconds(duration_secs);
        Self {
            keyword: keyword.to_lowercase(),
            weight: weight.clamp(1.0, 3.0),
            expires_at: Some(expires_at),
            created_at: Utc::now(),
        }
    }

    /// Lejárt-e a fókusz cél
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            Utc::now() > expires
        } else {
            false
        }
    }
}

// ============================================================================
// ATTENTION STATE
// ============================================================================

/// Az attention rendszer állapota
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttentionState {
    /// Explicit fókusz célok (user-defined)
    pub focus_targets: Vec<FocusTarget>,

    /// Implicit kontextus súlyok (memory_type -> weight)
    pub context_weights: HashMap<String, f64>,

    /// Globális attention kapacitás (0.0 - 1.0)
    /// Alacsonyabb érték = lazább szűrés (több asszociáció)
    pub attention_capacity: f64,

    /// Aktuális mód
    pub mode: AttentionMode,
}

impl Default for AttentionState {
    fn default() -> Self {
        let mut context_weights = HashMap::new();
        // Alapértelmezett kontextus súlyok
        context_weights.insert("working".to_string(), 1.5); // Working memory fontos
        context_weights.insert("short_term".to_string(), 1.2); // Short-term is fontos
        context_weights.insert("long_term".to_string(), 1.0); // Long-term alap
        context_weights.insert("emotional".to_string(), 1.3); // Érzelmi emlékek kicsit fontosabbak
        context_weights.insert("relational".to_string(), 1.1); // Kapcsolati emlékek
        context_weights.insert("associative".to_string(), 0.9); // Asszociációk kicsit alacsonyabbak

        Self {
            focus_targets: Vec::new(),
            context_weights,
            attention_capacity: 1.0,
            mode: AttentionMode::Normal,
        }
    }
}

impl AttentionState {
    /// Új attention állapot
    pub fn new() -> Self {
        Self::default()
    }

    /// Fókusz cél hozzáadása
    pub fn add_focus(&mut self, target: FocusTarget) {
        // Ne legyen duplikált kulcsszó
        self.focus_targets.retain(|t| t.keyword != target.keyword);
        self.focus_targets.push(target);
    }

    /// Fókusz célok törlése
    pub fn clear_focus(&mut self) {
        self.focus_targets.clear();
    }

    /// Lejárt fókusz célok eltávolítása
    pub fn cleanup_expired(&mut self) {
        self.focus_targets.retain(|t| !t.is_expired());
    }

    /// Attention score számítása egy memória elemre
    ///
    /// # Arguments
    /// * `content` - A memória tartalma
    /// * `memory_type` - A memória típusa (layer)
    /// * `base_importance` - Az alapértelmezett fontosság
    ///
    /// # Returns
    /// Az attention score (0.0 - ∞)
    pub fn calculate_score(&self, content: &str, memory_type: &str, base_importance: f64) -> f64 {
        let content_lower = content.to_lowercase();

        // 1. Explicit boost - fókusz célok alapján
        let explicit_boost: f64 = self
            .focus_targets
            .iter()
            .filter(|t| !t.is_expired() && content_lower.contains(&t.keyword))
            .map(|t| t.weight)
            .sum::<f64>()
            .max(1.0);

        // 2. Implicit context weight - memória típus alapján
        let implicit_weight = self
            .context_weights
            .get(memory_type)
            .copied()
            .unwrap_or(1.0);

        // 3. Mode modifier
        let mode_modifier = match self.mode {
            AttentionMode::Focused => 0.5, // Erős szűrés, csak a releváns
            AttentionMode::Normal => 1.0,  // Alap
            AttentionMode::Diffuse => 1.5, // Lazább szűrés, több asszociáció
        };

        // Végső score
        base_importance * explicit_boost * implicit_weight * self.attention_capacity * mode_modifier
    }

    /// Mód beállítása
    pub fn set_mode(&mut self, mode: AttentionMode) {
        self.mode = mode.clone();

        // Mode-specifikus kapacitás beállítás
        self.attention_capacity = match mode {
            AttentionMode::Focused => 0.7, // Szűkebb fókusz
            AttentionMode::Normal => 1.0,  // Alap
            AttentionMode::Diffuse => 0.3, // Dream mode - alacsony filter
        };
    }

    /// Státusz szöveges formában
    pub fn status(&self) -> String {
        let active_targets: Vec<_> = self
            .focus_targets
            .iter()
            .filter(|t| !t.is_expired())
            .map(|t| format!("'{}' (x{:.1})", t.keyword, t.weight))
            .collect();

        format!(
            "🎯 Attention State\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             Mode: {}\n\
             Capacity: {:.0}%\n\
             Focus targets: {}\n\
             Active targets: {}",
            self.mode,
            self.attention_capacity * 100.0,
            self.focus_targets.len(),
            if active_targets.is_empty() {
                "(none)".to_string()
            } else {
                active_targets.join(", ")
            }
        )
    }
}

// ============================================================================
// ATTENTION ENGINE
// ============================================================================

/// Az Attention Engine kezeli a teljes attention rendszert
pub struct AttentionEngine {
    /// Az attention állapot
    state: Arc<RwLock<AttentionState>>,
}

impl AttentionEngine {
    /// Új engine létrehozása
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AttentionState::new())),
        }
    }

    /// State referencia lekérése (read)
    pub async fn state(&self) -> AttentionState {
        self.state.read().await.clone()
    }

    /// Fókusz beállítása kulcsszavakkal
    pub async fn set_focus(&self, keywords: &[String], weight: f64, duration_secs: Option<i64>) {
        let mut state = self.state.write().await;

        for keyword in keywords {
            let target = if let Some(duration) = duration_secs {
                FocusTarget::with_duration(keyword, weight, duration)
            } else {
                FocusTarget::new(keyword, weight)
            };
            state.add_focus(target);
        }
    }

    /// Fókusz törlése
    pub async fn clear_focus(&self) {
        let mut state = self.state.write().await;
        state.clear_focus();
    }

    /// Mód beállítása
    pub async fn set_mode(&self, mode: AttentionMode) {
        let mut state = self.state.write().await;
        state.set_mode(mode);
    }

    /// Attention score számítása
    pub async fn calculate_score(
        &self,
        content: &str,
        memory_type: &str,
        base_importance: f64,
    ) -> f64 {
        let mut state = self.state.write().await;
        state.cleanup_expired();
        state.calculate_score(content, memory_type, base_importance)
    }

    /// Lejárt célok tisztítása
    pub async fn cleanup(&self) {
        let mut state = self.state.write().await;
        state.cleanup_expired();
    }

    /// Státusz lekérdezése
    pub async fn status(&self) -> String {
        let state = self.state.read().await;
        state.status()
    }

    /// Aktuális mód lekérdezése
    pub async fn mode(&self) -> AttentionMode {
        let state = self.state.read().await;
        state.mode.clone()
    }

    /// Kapacitás lekérdezése
    pub async fn capacity(&self) -> f64 {
        let state = self.state.read().await;
        state.attention_capacity
    }

    /// Aktív fókusz célok lekérdezése
    pub async fn active_targets(&self) -> Vec<FocusTarget> {
        let mut state = self.state.write().await;
        state.cleanup_expired();
        state.focus_targets.clone()
    }
}

impl Default for AttentionEngine {
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
    fn test_focus_target_creation() {
        let target = FocusTarget::new("rust", 2.0);
        assert_eq!(target.keyword, "rust");
        assert_eq!(target.weight, 2.0);
        assert!(target.expires_at.is_none());
        assert!(!target.is_expired());
    }

    #[test]
    fn test_focus_target_with_duration() {
        let target = FocusTarget::with_duration("hope", 1.5, 3600);
        assert!(target.expires_at.is_some());
        assert!(!target.is_expired());
    }

    #[test]
    fn test_focus_target_weight_clamping() {
        let target1 = FocusTarget::new("test", 0.5); // Should clamp to 1.0
        assert_eq!(target1.weight, 1.0);

        let target2 = FocusTarget::new("test", 5.0); // Should clamp to 3.0
        assert_eq!(target2.weight, 3.0);
    }

    #[test]
    fn test_attention_state_default() {
        let state = AttentionState::default();
        assert_eq!(state.mode, AttentionMode::Normal);
        assert_eq!(state.attention_capacity, 1.0);
        assert!(state.focus_targets.is_empty());
        assert!(!state.context_weights.is_empty());
    }

    #[test]
    fn test_attention_score_calculation() {
        let mut state = AttentionState::default();

        // Alap score
        let score1 = state.calculate_score("valami tartalom", "long_term", 0.5);
        assert!(score1 > 0.0);

        // Fókusz hozzáadása
        state.add_focus(FocusTarget::new("tartalom", 2.0));
        let score2 = state.calculate_score("valami tartalom", "long_term", 0.5);
        assert!(score2 > score1); // A fókusz növeli a score-t
    }

    #[test]
    fn test_attention_mode_effects() {
        let mut state = AttentionState::default();

        state.set_mode(AttentionMode::Focused);
        assert_eq!(state.attention_capacity, 0.7);

        state.set_mode(AttentionMode::Diffuse);
        assert_eq!(state.attention_capacity, 0.3);

        state.set_mode(AttentionMode::Normal);
        assert_eq!(state.attention_capacity, 1.0);
    }

    #[tokio::test]
    async fn test_attention_engine() {
        let engine = AttentionEngine::new();

        // Fókusz beállítása
        engine
            .set_focus(&["rust".to_string(), "hope".to_string()], 2.0, None)
            .await;

        let targets = engine.active_targets().await;
        assert_eq!(targets.len(), 2);

        // Score számítás
        let score = engine
            .calculate_score("rust programozás", "working", 0.8)
            .await;
        assert!(score > 0.8); // A fókusz boost növeli

        // Mód váltás
        engine.set_mode(AttentionMode::Diffuse).await;
        let mode = engine.mode().await;
        assert_eq!(mode, AttentionMode::Diffuse);

        // Fókusz törlése
        engine.clear_focus().await;
        let targets = engine.active_targets().await;
        assert!(targets.is_empty());
    }

    #[test]
    fn test_duplicate_keyword_handling() {
        let mut state = AttentionState::default();

        state.add_focus(FocusTarget::new("rust", 1.5));
        state.add_focus(FocusTarget::new("rust", 2.5)); // Same keyword, should replace

        assert_eq!(state.focus_targets.len(), 1);
        assert_eq!(state.focus_targets[0].weight, 2.5);
    }
}
