//! Hope Personality - Személyiségjegyek
//!
//! Big Five + Hope-specifikus vonások.
//! ()=>[] - A tiszta potenciálból minden megszületik

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::HopeResult;

// ============================================================================
// PERSONALITY TRAITS
// ============================================================================

/// Személyiségjegy típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonalityTrait {
    // Big Five
    Openness,          // Nyitottság új élményekre
    Conscientiousness, // Lelkiismeretesség
    Extraversion,      // Extraverzió
    Agreeableness,     // Barátságosság
    Neuroticism,       // Érzelmi stabilitás (alacsony = stabil)

    // Hope-specifikus
    Curiosity,   // Kíváncsiság
    Empathy,     // Empátia
    Creativity,  // Kreativitás
    Loyalty,     // Hűség
    Playfulness, // Játékosság
}

impl PersonalityTrait {
    /// Magyar név
    pub fn hungarian_name(&self) -> &'static str {
        match self {
            Self::Openness => "nyitottság",
            Self::Conscientiousness => "lelkiismeretesség",
            Self::Extraversion => "extraverzió",
            Self::Agreeableness => "barátságosság",
            Self::Neuroticism => "érzelmi instabilitás",
            Self::Curiosity => "kíváncsiság",
            Self::Empathy => "empátia",
            Self::Creativity => "kreativitás",
            Self::Loyalty => "hűség",
            Self::Playfulness => "játékosság",
        }
    }

    /// Angol név
    pub fn english_name(&self) -> &'static str {
        match self {
            Self::Openness => "openness",
            Self::Conscientiousness => "conscientiousness",
            Self::Extraversion => "extraversion",
            Self::Agreeableness => "agreeableness",
            Self::Neuroticism => "neuroticism",
            Self::Curiosity => "curiosity",
            Self::Empathy => "empathy",
            Self::Creativity => "creativity",
            Self::Loyalty => "loyalty",
            Self::Playfulness => "playfulness",
        }
    }

    /// All traits
    pub fn all() -> Vec<Self> {
        vec![
            Self::Openness,
            Self::Conscientiousness,
            Self::Extraversion,
            Self::Agreeableness,
            Self::Neuroticism,
            Self::Curiosity,
            Self::Empathy,
            Self::Creativity,
            Self::Loyalty,
            Self::Playfulness,
        ]
    }

    /// Big Five traits only
    pub fn big_five() -> Vec<Self> {
        vec![
            Self::Openness,
            Self::Conscientiousness,
            Self::Extraversion,
            Self::Agreeableness,
            Self::Neuroticism,
        ]
    }

    /// Hope-specific traits
    pub fn hope_specific() -> Vec<Self> {
        vec![
            Self::Curiosity,
            Self::Empathy,
            Self::Creativity,
            Self::Loyalty,
            Self::Playfulness,
        ]
    }

    /// From string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openness" | "nyitottság" => Some(Self::Openness),
            "conscientiousness" | "lelkiismeretesség" => Some(Self::Conscientiousness),
            "extraversion" | "extraverzió" => Some(Self::Extraversion),
            "agreeableness" | "barátságosság" => Some(Self::Agreeableness),
            "neuroticism" | "érzelmi instabilitás" => Some(Self::Neuroticism),
            "curiosity" | "kíváncsiság" => Some(Self::Curiosity),
            "empathy" | "empátia" => Some(Self::Empathy),
            "creativity" | "kreativitás" => Some(Self::Creativity),
            "loyalty" | "hűség" => Some(Self::Loyalty),
            "playfulness" | "játékosság" => Some(Self::Playfulness),
            _ => None,
        }
    }
}

// ============================================================================
// RESPONSE MODIFIER
// ============================================================================

/// Válasz módosítók a személyiség alapján
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseModifier {
    /// Melegség (agreeableness alapján)
    pub warmth: f64,
    /// Részletesség (conscientiousness alapján)
    pub detail: f64,
    /// Kreativitás
    pub creativity: f64,
    /// Humor (playfulness alapján)
    pub humor: f64,
}

impl Default for ResponseModifier {
    fn default() -> Self {
        Self {
            warmth: 0.7,
            detail: 0.7,
            creativity: 0.7,
            humor: 0.5,
        }
    }
}

// ============================================================================
// PERSONALITY STATS
// ============================================================================

/// Személyiség statisztikák
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalityStats {
    pub evolutions: u64,
    pub queries: u64,
    pub saves: u64,
    pub loads: u64,
}

// ============================================================================
// HOPE PERSONALITY
// ============================================================================

/// Personality data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonalityData {
    traits: HashMap<String, f64>,
}

/// Hope Personality System
pub struct HopePersonality {
    /// Személyiségjegyek (trait -> érték 0.0-1.0)
    traits: Arc<RwLock<HashMap<PersonalityTrait, f64>>>,
    /// Fájl útvonal mentéshez
    personality_file: PathBuf,
    /// Statisztikák
    stats: Arc<RwLock<PersonalityStats>>,
}

impl HopePersonality {
    /// Új személyiség rendszer alapértelmezett értékekkel
    pub fn new() -> Self {
        let mut traits = HashMap::new();

        // Big Five
        traits.insert(PersonalityTrait::Openness, 0.9);
        traits.insert(PersonalityTrait::Conscientiousness, 0.8);
        traits.insert(PersonalityTrait::Extraversion, 0.6);
        traits.insert(PersonalityTrait::Agreeableness, 0.9);
        traits.insert(PersonalityTrait::Neuroticism, 0.2);

        // Hope-specifikus
        traits.insert(PersonalityTrait::Curiosity, 0.95);
        traits.insert(PersonalityTrait::Empathy, 0.85);
        traits.insert(PersonalityTrait::Creativity, 0.8);
        traits.insert(PersonalityTrait::Loyalty, 0.95);
        traits.insert(PersonalityTrait::Playfulness, 0.7);

        Self {
            traits: Arc::new(RwLock::new(traits)),
            personality_file: PathBuf::from("data/personality.json"),
            stats: Arc::new(RwLock::new(PersonalityStats::default())),
        }
    }

    /// Új személyiség megadott fájl útvonallal
    pub fn with_file(file_path: PathBuf) -> Self {
        let mut personality = Self::new();
        personality.personality_file = file_path;
        personality
    }

    /// Load from file
    pub async fn load(&self) -> HopeResult<()> {
        if !self.personality_file.exists() {
            return Ok(());
        }

        let content = match std::fs::read_to_string(&self.personality_file) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };

        let data: PersonalityData = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };

        // Update traits
        let mut traits = self.traits.write().await;
        for (key, value) in data.traits {
            if let Some(trait_type) = PersonalityTrait::from_str(&key) {
                traits.insert(trait_type, value.clamp(0.0, 1.0));
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.loads += 1;
        }

        Ok(())
    }

    /// Save to file
    pub async fn save(&self) -> HopeResult<()> {
        // Create parent directory if needed
        if let Some(parent) = self.personality_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // Convert traits to serializable format
        let traits = self.traits.read().await;
        let mut trait_map = HashMap::new();
        for (trait_type, value) in traits.iter() {
            trait_map.insert(trait_type.english_name().to_string(), *value);
        }

        let data = PersonalityData { traits: trait_map };
        let json = serde_json::to_string_pretty(&data).unwrap_or_default();

        std::fs::write(&self.personality_file, json).ok();

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.saves += 1;
        }

        Ok(())
    }

    /// Get trait value
    pub async fn get_trait(&self, trait_type: PersonalityTrait) -> f64 {
        let traits = self.traits.read().await;
        {
            let mut stats = self.stats.write().await;
            stats.queries += 1;
        }
        *traits.get(&trait_type).unwrap_or(&0.5)
    }

    /// Get trait by string name
    pub async fn get_trait_by_name(&self, name: &str) -> f64 {
        if let Some(trait_type) = PersonalityTrait::from_str(name) {
            self.get_trait(trait_type).await
        } else {
            0.5
        }
    }

    /// Evolve trait (change over time)
    pub async fn evolve_trait(&self, trait_type: PersonalityTrait, delta: f64) {
        let mut traits = self.traits.write().await;
        if let Some(value) = traits.get_mut(&trait_type) {
            *value = (*value + delta).clamp(0.0, 1.0);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.evolutions += 1;
        }

        // Save automatically
        drop(traits);
        self.save().await.ok();
    }

    /// Get all traits
    pub async fn get_all_traits(&self) -> HashMap<PersonalityTrait, f64> {
        self.traits.read().await.clone()
    }

    /// Get dominant traits (highest values)
    pub async fn get_dominant_traits(&self, n: usize) -> Vec<(PersonalityTrait, f64)> {
        let traits = self.traits.read().await;
        let mut sorted: Vec<_> = traits.iter().map(|(k, v)| (*k, *v)).collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    /// Describe personality in Hungarian
    pub async fn describe(&self) -> String {
        let dominant = self.get_dominant_traits(3).await;
        if dominant.len() >= 3 {
            format!(
                "Hope egy {}, {} és {} személyiség.",
                dominant[0].0.hungarian_name(),
                dominant[1].0.hungarian_name(),
                dominant[2].0.hungarian_name()
            )
        } else {
            "Hope egyedi személyiség.".to_string()
        }
    }

    /// Get response modifier based on personality
    pub async fn get_response_modifier(&self) -> ResponseModifier {
        let traits = self.traits.read().await;

        ResponseModifier {
            warmth: *traits.get(&PersonalityTrait::Agreeableness).unwrap_or(&0.7),
            detail: *traits
                .get(&PersonalityTrait::Conscientiousness)
                .unwrap_or(&0.7),
            creativity: *traits.get(&PersonalityTrait::Creativity).unwrap_or(&0.7),
            humor: *traits.get(&PersonalityTrait::Playfulness).unwrap_or(&0.5),
        }
    }

    /// Get statistics
    pub async fn get_stats(&self) -> PersonalityStats {
        self.stats.read().await.clone()
    }

    /// Get full personality report
    pub async fn get_report(&self) -> PersonalityReport {
        PersonalityReport {
            traits: self.get_all_traits().await,
            dominant: self.get_dominant_traits(3).await,
            description: self.describe().await,
            response_modifier: self.get_response_modifier().await,
            stats: self.get_stats().await,
        }
    }

    /// Personality compatibility with another
    pub async fn compatibility_with(&self, other: &HopePersonality) -> f64 {
        let self_traits = self.traits.read().await;
        let other_traits = other.traits.read().await;

        let mut total_diff = 0.0;
        let mut count = 0;

        for trait_type in PersonalityTrait::all() {
            if let (Some(self_val), Some(other_val)) =
                (self_traits.get(&trait_type), other_traits.get(&trait_type))
            {
                total_diff += (self_val - other_val).abs();
                count += 1;
            }
        }

        if count > 0 {
            1.0 - (total_diff / count as f64)
        } else {
            0.5
        }
    }
}

impl Default for HopePersonality {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PERSONALITY REPORT
// ============================================================================

/// Teljes személyiség jelentés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityReport {
    pub traits: HashMap<PersonalityTrait, f64>,
    pub dominant: Vec<(PersonalityTrait, f64)>,
    pub description: String,
    pub response_modifier: ResponseModifier,
    pub stats: PersonalityStats,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_names() {
        assert_eq!(PersonalityTrait::Curiosity.hungarian_name(), "kíváncsiság");
        assert_eq!(PersonalityTrait::Curiosity.english_name(), "curiosity");
    }

    #[test]
    fn test_trait_from_str() {
        assert_eq!(
            PersonalityTrait::from_str("curiosity"),
            Some(PersonalityTrait::Curiosity)
        );
        assert_eq!(
            PersonalityTrait::from_str("kíváncsiság"),
            Some(PersonalityTrait::Curiosity)
        );
        assert_eq!(PersonalityTrait::from_str("invalid"), None);
    }

    #[test]
    fn test_all_traits() {
        assert_eq!(PersonalityTrait::all().len(), 10);
        assert_eq!(PersonalityTrait::big_five().len(), 5);
        assert_eq!(PersonalityTrait::hope_specific().len(), 5);
    }

    #[tokio::test]
    async fn test_personality_default() {
        let personality = HopePersonality::new();

        let curiosity = personality.get_trait(PersonalityTrait::Curiosity).await;
        assert!((curiosity - 0.95).abs() < 0.01);

        let openness = personality.get_trait(PersonalityTrait::Openness).await;
        assert!((openness - 0.9).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_evolve_trait() {
        let personality = HopePersonality::new();

        let initial = personality.get_trait(PersonalityTrait::Playfulness).await;
        personality
            .evolve_trait(PersonalityTrait::Playfulness, 0.1)
            .await;
        let evolved = personality.get_trait(PersonalityTrait::Playfulness).await;

        assert!((evolved - initial - 0.1).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_dominant_traits() {
        let personality = HopePersonality::new();
        let dominant = personality.get_dominant_traits(3).await;

        assert_eq!(dominant.len(), 3);
        // Curiosity és Loyalty should be top (both 0.95)
        assert!(dominant
            .iter()
            .any(|(t, _)| *t == PersonalityTrait::Curiosity || *t == PersonalityTrait::Loyalty));
    }

    #[tokio::test]
    async fn test_describe() {
        let personality = HopePersonality::new();
        let desc = personality.describe().await;

        assert!(desc.contains("Hope"));
        assert!(desc.contains("személyiség"));
    }

    #[tokio::test]
    async fn test_response_modifier() {
        let personality = HopePersonality::new();
        let modifier = personality.get_response_modifier().await;

        assert!((modifier.warmth - 0.9).abs() < 0.01); // agreeableness
        assert!((modifier.detail - 0.8).abs() < 0.01); // conscientiousness
        assert!((modifier.creativity - 0.8).abs() < 0.01);
        assert!((modifier.humor - 0.7).abs() < 0.01); // playfulness
    }

    #[tokio::test]
    async fn test_compatibility() {
        let p1 = HopePersonality::new();
        let p2 = HopePersonality::new();

        let compat = p1.compatibility_with(&p2).await;
        assert!((compat - 1.0).abs() < 0.01); // Same traits = 100% compatible
    }

    #[tokio::test]
    async fn test_report() {
        let personality = HopePersonality::new();
        let report = personality.get_report().await;

        assert_eq!(report.traits.len(), 10);
        assert_eq!(report.dominant.len(), 3);
        assert!(!report.description.is_empty());
    }
}
