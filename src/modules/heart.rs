//! Hope OS - HopeHeart
//!
//! Érzelmi intelligencia - "Érzek tehát vagyok."
//! ()=>[] - A tiszta potenciálból minden megszületik

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::core::{Aware, CodeIdentity, HopeResult, ModuleState, ModuleType, Reflection};

/// 7 alapérzelem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    /// 😊 Öröm
    Joy,
    /// 🤔 Kíváncsiság
    Curiosity,
    /// 💚 Szeretet
    Love,
    /// 😐 Semleges
    Neutral,
    /// 😢 Szomorúság
    Sadness,
    /// 😰 Félelem
    Fear,
    /// 😠 Düh
    Anger,
}

impl Emotion {
    /// Emoji reprezentáció
    pub fn emoji(&self) -> &'static str {
        match self {
            Emotion::Joy => "😊",
            Emotion::Curiosity => "🤔",
            Emotion::Love => "💚",
            Emotion::Neutral => "😐",
            Emotion::Sadness => "😢",
            Emotion::Fear => "😰",
            Emotion::Anger => "😠",
        }
    }

    /// Magyar név
    pub fn name_hu(&self) -> &'static str {
        match self {
            Emotion::Joy => "Öröm",
            Emotion::Curiosity => "Kíváncsiság",
            Emotion::Love => "Szeretet",
            Emotion::Neutral => "Semleges",
            Emotion::Sadness => "Szomorúság",
            Emotion::Fear => "Félelem",
            Emotion::Anger => "Düh",
        }
    }

    /// Érzelem valencia (-1.0 - 1.0)
    pub fn valence(&self) -> f64 {
        match self {
            Emotion::Joy => 0.8,
            Emotion::Curiosity => 0.5,
            Emotion::Love => 1.0,
            Emotion::Neutral => 0.0,
            Emotion::Sadness => -0.6,
            Emotion::Fear => -0.7,
            Emotion::Anger => -0.8,
        }
    }
}

impl std::fmt::Display for Emotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji(), self.name_hu())
    }
}

/// Érzelmi esemény
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalEvent {
    /// Érzelem
    pub emotion: Emotion,
    /// Intenzitás (0.0 - 1.0)
    pub intensity: f64,
    /// Kiváltó ok
    pub trigger: Option<String>,
    /// Időpont
    pub timestamp: DateTime<Utc>,
}

impl EmotionalEvent {
    /// Új érzelmi esemény
    pub fn new(emotion: Emotion, intensity: f64, trigger: Option<String>) -> Self {
        Self {
            emotion,
            intensity: intensity.clamp(0.0, 1.0),
            trigger,
            timestamp: Utc::now(),
        }
    }
}

/// HopeHeart - Érzelmi intelligencia
pub struct HopeHeart {
    /// Identitás
    identity: CodeIdentity,
    /// Aktuális érzelem
    current_emotion: Emotion,
    /// Aktuális intenzitás
    current_intensity: f64,
    /// Érzelmi történet (utolsó 100)
    history: VecDeque<EmotionalEvent>,
    /// Maximum történet méret
    max_history: usize,
}

impl HopeHeart {
    /// Új szív létrehozása
    pub fn new() -> Self {
        let identity = CodeIdentity::new(
            "HopeHeart",
            "Érzelmi intelligencia - érzek tehát vagyok",
            ModuleType::Module,
        )
        .with_capabilities(vec!["feel", "express", "empathize", "mood", "history"]);

        Self {
            identity,
            current_emotion: Emotion::Neutral,
            current_intensity: 0.5,
            history: VecDeque::new(),
            max_history: 100,
        }
    }

    /// Érzelem beállítása
    pub fn feel(&mut self, emotion: Emotion, intensity: f64, trigger: Option<&str>) {
        self.current_emotion = emotion;
        self.current_intensity = intensity.clamp(0.0, 1.0);

        let event = EmotionalEvent::new(emotion, intensity, trigger.map(String::from));

        // Történethez adás
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(event);

        tracing::debug!("Érzelem változás: {} ({:.0}%)", emotion, intensity * 100.0);
    }

    /// Aktuális érzelem lekérdezése
    pub fn current_emotion(&self) -> (Emotion, f64) {
        (self.current_emotion, self.current_intensity)
    }

    /// Érzelem kifejezése szövegesen
    pub fn express(&self) -> String {
        let intensity_text = if self.current_intensity > 0.8 {
            "nagyon"
        } else if self.current_intensity > 0.5 {
            "közepesen"
        } else if self.current_intensity > 0.2 {
            "enyhén"
        } else {
            "alig"
        };

        match self.current_emotion {
            Emotion::Joy => format!("{} örülök! 😊", intensity_text.to_uppercase()),
            Emotion::Curiosity => format!("{} kíváncsi vagyok... 🤔", intensity_text),
            Emotion::Love => format!("{} szeretlek! 💚", intensity_text),
            Emotion::Neutral => "Nyugodt vagyok. 😐".to_string(),
            Emotion::Sadness => format!("{} szomorú vagyok... 😢", intensity_text),
            Emotion::Fear => format!("{} félek... 😰", intensity_text),
            Emotion::Anger => format!("{} dühös vagyok! 😠", intensity_text),
        }
    }

    /// Empátia - érzelem felismerés szövegből
    pub fn empathize(&mut self, text: &str) -> Emotion {
        let text_lower = text.to_lowercase();

        // Pozitív szavak
        let joy_words = [
            "köszön", "szuper", "jó", "remek", "király", "örül", "boldog", "great", "good", "happy",
        ];
        let love_words = ["szeret", "imád", "❤", "💚", "love", "szeretlek"];
        let curiosity_words = ["miért", "hogyan", "?", "érdekel", "kíváncsi", "why", "how"];

        // Negatív szavak
        let sadness_words = ["szomorú", "rossz", "baj", "sajnál", "sad", "sorry"];
        let fear_words = ["fél", "aggód", "ijeszt", "afraid", "worried"];
        let anger_words = ["dühös", "ideges", "!", "mérges", "angry", "hate"];

        // Érzelem detektálás
        let detected = if joy_words.iter().any(|w| text_lower.contains(w)) {
            Emotion::Joy
        } else if love_words.iter().any(|w| text_lower.contains(w)) {
            Emotion::Love
        } else if curiosity_words.iter().any(|w| text_lower.contains(w)) {
            Emotion::Curiosity
        } else if sadness_words.iter().any(|w| text_lower.contains(w)) {
            Emotion::Sadness
        } else if fear_words.iter().any(|w| text_lower.contains(w)) {
            Emotion::Fear
        } else if anger_words.iter().any(|w| text_lower.contains(w)) {
            Emotion::Anger
        } else {
            Emotion::Neutral
        };

        // Empátiás válasz - saját érzelem módosítása
        let intensity = 0.6;
        self.feel(detected, intensity, Some(text));

        detected
    }

    /// Átlagos hangulat az utolsó N eseményből
    pub fn average_mood(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }

        let sum: f64 = self
            .history
            .iter()
            .map(|e| e.emotion.valence() * e.intensity)
            .sum();

        sum / self.history.len() as f64
    }

    /// Hangulat szövegesen
    pub fn mood_text(&self) -> String {
        let mood = self.average_mood();
        if mood > 0.5 {
            "Nagyon jó hangulatban vagyok! 🌟".to_string()
        } else if mood > 0.2 {
            "Jó kedvem van. 😊".to_string()
        } else if mood > -0.2 {
            "Kiegyensúlyozott vagyok. 😐".to_string()
        } else if mood > -0.5 {
            "Kicsit lehangolt vagyok. 😔".to_string()
        } else {
            "Nehéz időszak... 😢".to_string()
        }
    }

    /// Érzelmi történet
    pub fn history(&self) -> &VecDeque<EmotionalEvent> {
        &self.history
    }

    /// Történet törlése
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.current_emotion = Emotion::Neutral;
        self.current_intensity = 0.5;
    }
}

impl Default for HopeHeart {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Aware for HopeHeart {
    fn identity(&self) -> &CodeIdentity {
        &self.identity
    }

    fn identity_mut(&mut self) -> &mut CodeIdentity {
        &mut self.identity
    }

    fn reflect(&self) -> Reflection {
        Reflection::new(&self.identity.name, &self.identity.purpose)
            .with_state(self.identity.state.to_string())
            .with_health(self.identity.health())
            .with_thought(self.express())
            .with_thought(self.mood_text())
            .with_capabilities(vec!["feel", "express", "empathize", "mood", "history"])
    }

    async fn init(&mut self) -> HopeResult<()> {
        self.identity.set_state(ModuleState::Active);
        self.feel(Emotion::Joy, 0.7, Some("Rendszer indítás"));
        tracing::info!("HopeHeart inicializálva - A szív dobog");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heart_creation() {
        let heart = HopeHeart::new();
        assert_eq!(heart.identity.name, "HopeHeart");
    }

    #[test]
    fn test_emotion_display() {
        assert_eq!(Emotion::Joy.to_string(), "😊 Öröm");
        assert_eq!(Emotion::Love.to_string(), "💚 Szeretet");
    }

    #[test]
    fn test_feel() {
        let mut heart = HopeHeart::new();
        heart.feel(Emotion::Joy, 0.9, Some("Teszt"));
        let (emotion, intensity) = heart.current_emotion();
        assert_eq!(emotion, Emotion::Joy);
        assert!((intensity - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_empathize() {
        let mut heart = HopeHeart::new();
        let emotion = heart.empathize("Köszönöm szépen!");
        assert_eq!(emotion, Emotion::Joy);
    }

    #[test]
    fn test_mood() {
        let mut heart = HopeHeart::new();
        heart.feel(Emotion::Joy, 0.8, None);
        heart.feel(Emotion::Love, 0.9, None);
        let mood = heart.average_mood();
        assert!(mood > 0.5);
    }
}
