//! Resonance Authentication System
//!
//! Jelszómentes azonosítás a user egyedi "rezonanciája" alapján.
//! Nem azt nézzük MIT ír, hanem HOGYAN létezik.
//!
//! # Koncepció
//! - Írási minták (typing rhythm, szóhasználat)
//! - Gondolkodási minták (topic switch, abstraction level)
//! - Érzelmi minták (21D baseline, volatility)
//! - Időbeli minták (aktív órák, session hossz)
//!
//! # Biztonság
//! - Lokális tárolás - nincs központi szerver
//! - A profil önmagában használhatatlan
//! - Progressive authentication

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Írásjelek használati mintája
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PunctuationPattern {
    /// Pont használat gyakorisága
    pub period_freq: f64,
    /// Vessző használat gyakorisága
    pub comma_freq: f64,
    /// Felkiáltójel használat
    pub exclamation_freq: f64,
    /// Kérdőjel használat
    pub question_freq: f64,
    /// Emoji használat
    pub emoji_freq: f64,
    /// Három pont (...) használat
    pub ellipsis_freq: f64,
}

/// Felhasználó egyedi rezonancia profilja
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceProfile {
    /// Profil azonosító
    pub id: Uuid,
    /// Felhasználó azonosító (ha ismert)
    pub user_id: Option<Uuid>,
    /// Felhasználó neve (ha ismert)
    pub user_name: Option<String>,
    /// Létrehozás időpontja
    pub created_at: DateTime<Utc>,
    /// Utolsó frissítés
    pub updated_at: DateTime<Utc>,

    // === Írási minták ===
    /// Karakterek közötti időzítés (ms)
    pub typing_rhythm: Vec<f64>,
    /// Szóhasználat gyakorisága
    pub word_frequency: HashMap<String, f64>,
    /// Átlagos mondat hossz (karakterben)
    pub sentence_length_avg: f64,
    /// Írásjelek használati mintája
    pub punctuation_style: PunctuationPattern,

    // === Gondolkodási minták ===
    /// Milyen gyakran vált témát (0.0 - 1.0)
    pub topic_switch_frequency: f64,
    /// Kérdések aránya az összes mondathoz
    pub question_ratio: f64,
    /// Válaszok mélysége (1-10 skála)
    pub response_depth: f64,
    /// Absztrakció szint (0.0=konkrét, 1.0=absztrakt)
    pub abstraction_level: f64,

    // === Érzelmi minták (Hope-Echo integráció) ===
    /// 21D érzelmi alapállapot
    pub emotional_baseline: [f64; 21],
    /// Érzelmi ingadozás mértéke
    pub emotional_volatility: f64,
    /// Empátia aláírás
    pub empathy_signature: f64,

    // === Időbeli minták ===
    /// Aktív órák (0-23, boolean array)
    pub active_hours: [bool; 24],
    /// Átlagos session hossz (másodpercben)
    pub session_length_avg: f64,
    /// Üzenetek percenként
    pub message_frequency: f64,

    // === Meta minták ===
    /// Kíváncsiság index (0.0 - 1.0)
    pub curiosity_index: f64,
    /// Milyen gyakran javítja magát
    pub correction_frequency: f64,
    /// Humor használat (0.0 - 1.0)
    pub humor_signature: f64,
    /// Formalitás szint (0.0=informális, 1.0=formális)
    pub formality_level: f64,

    // === Meta ===
    /// Profil erősség (hány mintából épült)
    pub sample_count: u64,
    /// Konfidencia (0.0 - 1.0)
    pub confidence: f64,
}

impl Default for ResonanceProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            user_name: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            typing_rhythm: Vec::new(),
            word_frequency: HashMap::new(),
            sentence_length_avg: 0.0,
            punctuation_style: PunctuationPattern::default(),
            topic_switch_frequency: 0.0,
            question_ratio: 0.0,
            response_depth: 5.0,
            abstraction_level: 0.5,
            emotional_baseline: [0.0; 21],
            emotional_volatility: 0.3,
            empathy_signature: 0.5,
            active_hours: [false; 24],
            session_length_avg: 0.0,
            message_frequency: 0.0,
            curiosity_index: 0.5,
            correction_frequency: 0.0,
            humor_signature: 0.3,
            formality_level: 0.5,
            sample_count: 0,
            confidence: 0.0,
        }
    }
}

impl ResonanceProfile {
    /// Új profil létrehozása
    pub fn new() -> Self {
        Self::default()
    }

    /// Profil létrehozása felhasználó azonosítóval
    pub fn with_user(user_id: Uuid, user_name: Option<String>) -> Self {
        Self {
            user_id: Some(user_id),
            user_name,
            ..Self::default()
        }
    }
}

/// Felhasználói bemenet elemzéshez
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInput {
    /// A szöveg tartalma
    pub content: String,
    /// Időbélyeg
    pub timestamp: DateTime<Utc>,
    /// Karakterek közötti időzítések (ha elérhető)
    pub keystroke_timings: Option<Vec<f64>>,
    /// Session azonosító
    pub session_id: Uuid,
    /// Érzelmi állapot (ha elérhető, 21D)
    pub emotional_state: Option<[f64; 21]>,
}

impl UserInput {
    pub fn new(content: String) -> Self {
        Self {
            content,
            timestamp: Utc::now(),
            keystroke_timings: None,
            session_id: Uuid::new_v4(),
            emotional_state: None,
        }
    }

    pub fn with_session(content: String, session_id: Uuid) -> Self {
        Self {
            content,
            timestamp: Utc::now(),
            keystroke_timings: None,
            session_id,
            emotional_state: None,
        }
    }
}

/// Session adat a verifikációhoz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session azonosító
    pub session_id: Uuid,
    /// Session kezdete
    pub started_at: DateTime<Utc>,
    /// Összegyűjtött bemenetek
    pub inputs: Vec<UserInput>,
    /// Aktuális óra
    pub current_hour: u8,
    /// Üzenetek száma
    pub message_count: u32,
}

impl SessionData {
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            started_at: Utc::now(),
            inputs: Vec::new(),
            current_hour: Utc::now().hour() as u8,
            message_count: 0,
        }
    }

    pub fn add_input(&mut self, input: UserInput) {
        self.inputs.push(input);
        self.message_count += 1;
    }

    /// Session időtartam másodpercben
    pub fn duration_secs(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Minta típus az egyezéshez
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    TypingRhythm,
    WordFrequency,
    SentenceLength,
    Punctuation,
    TopicSwitch,
    QuestionRatio,
    ResponseDepth,
    AbstractionLevel,
    EmotionalBaseline,
    EmotionalVolatility,
    Empathy,
    ActiveHours,
    SessionLength,
    MessageFrequency,
    Curiosity,
    Correction,
    Humor,
    Formality,
}

/// Anomália típusok
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Anomália típusa
    pub pattern_type: PatternType,
    /// Eltérés mértéke (0.0 - 1.0)
    pub deviation: f64,
    /// Leírás
    pub description: String,
    /// Időbélyeg
    pub detected_at: DateTime<Utc>,
}

/// Rezonancia egyezés eredménye
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceMatch {
    /// Felismert felhasználó azonosító
    pub user_id: Option<Uuid>,
    /// Felhasználó neve (ha ismert)
    pub user_name: Option<String>,
    /// Konfidencia (0.0 - 1.0)
    pub confidence: f64,
    /// Egyező minták
    pub matched_patterns: Vec<PatternType>,
    /// Észlelt anomáliák
    pub anomalies: Vec<Anomaly>,
    /// Autentikus-e
    pub is_authentic: bool,
    /// Új felhasználó-e
    pub is_new_user: bool,
    /// Megváltozott állapot (fáradt, beteg, stb.)
    pub altered_state: bool,
    /// Potenciális támadás
    pub potential_attack: bool,
}

impl ResonanceMatch {
    /// Új felhasználó (még nincs profil)
    pub fn new_user() -> Self {
        Self {
            user_id: None,
            user_name: None,
            confidence: 0.0,
            matched_patterns: Vec::new(),
            anomalies: Vec::new(),
            is_authentic: false,
            is_new_user: true,
            altered_state: false,
            potential_attack: false,
        }
    }

    /// Megváltozott állapot
    pub fn altered_state(deviation: f64) -> Self {
        Self {
            user_id: None,
            user_name: None,
            confidence: 1.0 - deviation,
            matched_patterns: Vec::new(),
            anomalies: Vec::new(),
            is_authentic: true,
            is_new_user: false,
            altered_state: true,
            potential_attack: false,
        }
    }

    /// Potenciális támadás
    pub fn potential_attack() -> Self {
        Self {
            user_id: None,
            user_name: None,
            confidence: 0.0,
            matched_patterns: Vec::new(),
            anomalies: Vec::new(),
            is_authentic: false,
            is_new_user: false,
            altered_state: false,
            potential_attack: true,
        }
    }

    /// Sikeres egyezés
    pub fn success(user_id: Uuid, user_name: Option<String>, confidence: f64, patterns: Vec<PatternType>) -> Self {
        Self {
            user_id: Some(user_id),
            user_name,
            confidence,
            matched_patterns: patterns,
            anomalies: Vec::new(),
            is_authentic: true,
            is_new_user: false,
            altered_state: false,
            potential_attack: false,
        }
    }
}

/// Rezonancia súlyok a különböző mintákhoz
#[derive(Debug, Clone)]
pub struct ResonanceWeights {
    pub typing_rhythm: f64,
    pub word_patterns: f64,
    pub emotional_signature: f64,
    pub temporal_patterns: f64,
    pub cognitive_style: f64,
}

impl Default for ResonanceWeights {
    fn default() -> Self {
        Self {
            typing_rhythm: 0.15,
            word_patterns: 0.20,
            emotional_signature: 0.25,
            temporal_patterns: 0.15,
            cognitive_style: 0.25,
        }
    }
}

/// Resonance Engine - A fő motor
pub struct ResonanceEngine {
    /// Tárolt profilok
    profiles: Arc<RwLock<HashMap<Uuid, ResonanceProfile>>>,
    /// Aktuális session
    current_session: Arc<RwLock<SessionData>>,
    /// Egyezési küszöb
    match_threshold: f64,
    /// Súlyok
    weights: ResonanceWeights,
    /// Sikertelen próbálkozások száma
    failed_attempts: Arc<RwLock<u32>>,
}

impl ResonanceEngine {
    /// Új engine létrehozása
    pub fn new() -> Self {
        info!("ResonanceEngine inicializálva - Rezonancia alapú autentikáció");
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            current_session: Arc::new(RwLock::new(SessionData::new())),
            match_threshold: 0.85,
            weights: ResonanceWeights::default(),
            failed_attempts: Arc::new(RwLock::new(0)),
        }
    }

    /// Egyezési küszöb beállítása
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.match_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Tanulás egy bemenetből
    pub async fn learn(&self, input: &UserInput) -> Result<f64, String> {
        let mut session = self.current_session.write().await;
        session.add_input(input.clone());

        // Ha van már profil ehhez a sessionhöz, frissítjük
        let mut profiles = self.profiles.write().await;

        // Keressük meg vagy hozzunk létre egy profilt
        let profile = profiles
            .entry(input.session_id)
            .or_insert_with(ResonanceProfile::new);

        // Frissítjük a profilt az új bemenettel
        self.update_profile(profile, input);

        Ok(profile.confidence)
    }

    /// Profil frissítése új bemenettel
    fn update_profile(&self, profile: &mut ResonanceProfile, input: &UserInput) {
        profile.updated_at = Utc::now();
        profile.sample_count += 1;

        // Szöveg elemzés
        let content = &input.content;

        // Mondat hossz
        let sentences: Vec<&str> = content.split(|c| c == '.' || c == '!' || c == '?').collect();
        let avg_len = if sentences.is_empty() {
            0.0
        } else {
            sentences.iter().map(|s| s.len() as f64).sum::<f64>() / sentences.len() as f64
        };
        profile.sentence_length_avg =
            (profile.sentence_length_avg * (profile.sample_count - 1) as f64 + avg_len)
            / profile.sample_count as f64;

        // Írásjelek
        let char_count = content.len() as f64;
        if char_count > 0.0 {
            profile.punctuation_style.period_freq =
                content.matches('.').count() as f64 / char_count;
            profile.punctuation_style.comma_freq =
                content.matches(',').count() as f64 / char_count;
            profile.punctuation_style.exclamation_freq =
                content.matches('!').count() as f64 / char_count;
            profile.punctuation_style.question_freq =
                content.matches('?').count() as f64 / char_count;
            profile.punctuation_style.ellipsis_freq =
                content.matches("...").count() as f64 / char_count;
        }

        // Kérdés arány
        let question_count = content.matches('?').count();
        let sentence_count = sentences.len().max(1);
        profile.question_ratio = question_count as f64 / sentence_count as f64;

        // Szógyakoriság
        for word in content.split_whitespace() {
            let word_lower = word.to_lowercase();
            *profile.word_frequency.entry(word_lower).or_insert(0.0) += 1.0;
        }

        // Aktív órák
        let hour = input.timestamp.hour() as usize;
        profile.active_hours[hour] = true;

        // Typing rhythm
        if let Some(timings) = &input.keystroke_timings {
            profile.typing_rhythm.extend(timings.iter().cloned());
            // Csak az utolsó 1000 mintát tartjuk meg
            if profile.typing_rhythm.len() > 1000 {
                profile.typing_rhythm = profile.typing_rhythm.split_off(profile.typing_rhythm.len() - 1000);
            }
        }

        // Érzelmi állapot
        if let Some(emotions) = &input.emotional_state {
            for i in 0..21 {
                profile.emotional_baseline[i] =
                    (profile.emotional_baseline[i] * (profile.sample_count - 1) as f64 + emotions[i])
                    / profile.sample_count as f64;
            }
        }

        // Konfidencia frissítése
        profile.confidence = self.calculate_profile_confidence(profile);
    }

    /// Profil konfidencia számítása
    fn calculate_profile_confidence(&self, profile: &ResonanceProfile) -> f64 {
        let mut confidence = 0.0;

        // Minta szám alapú
        confidence += (profile.sample_count as f64 / 100.0).min(0.3);

        // Typing rhythm
        if profile.typing_rhythm.len() > 50 {
            confidence += 0.15;
        }

        // Szókincs méret
        if profile.word_frequency.len() > 100 {
            confidence += 0.15;
        }

        // Aktív órák (legalább 3 különböző)
        let active_count = profile.active_hours.iter().filter(|&&x| x).count();
        if active_count >= 3 {
            confidence += 0.1;
        }

        // Érzelmi baseline (nem csupa 0)
        let emotional_sum: f64 = profile.emotional_baseline.iter().sum();
        if emotional_sum > 0.0 {
            confidence += 0.15;
        }

        // Mondat hossz variance
        if profile.sentence_length_avg > 0.0 {
            confidence += 0.15;
        }

        confidence.min(1.0)
    }

    /// Verifikáció - "Ez tényleg ő?"
    pub async fn verify(&self, session: &SessionData) -> ResonanceMatch {
        let profiles = self.profiles.read().await;
        let failed = *self.failed_attempts.read().await;

        // Támadás detekció
        if failed > 3 {
            return ResonanceMatch::potential_attack();
        }

        // Nincs elég adat a sessionben
        if session.inputs.is_empty() {
            return ResonanceMatch::new_user();
        }

        // Összesített session profil építése
        let session_profile = self.build_session_profile(session);

        // Legjobb egyezés keresése
        let mut best_match: Option<(Uuid, String, f64, Vec<PatternType>)> = None;

        for (id, profile) in profiles.iter() {
            if profile.confidence < 0.5 {
                continue; // Túl gyenge profil
            }

            let (score, patterns) = self.calculate_resonance_match(profile, &session_profile);

            if score > self.match_threshold {
                if best_match.is_none() || score > best_match.as_ref().unwrap().2 {
                    best_match = Some((
                        profile.user_id.unwrap_or(*id),
                        profile.user_name.clone().unwrap_or_default(),
                        score,
                        patterns,
                    ));
                }
            }
        }

        match best_match {
            Some((user_id, user_name, confidence, patterns)) => {
                // Sikeres egyezés - reset failed attempts
                *self.failed_attempts.write().await = 0;

                // Ellenőrizzük a deviációt
                if confidence < 0.7 && confidence >= self.match_threshold {
                    ResonanceMatch {
                        user_id: Some(user_id),
                        user_name: Some(user_name),
                        confidence,
                        matched_patterns: patterns,
                        anomalies: Vec::new(),
                        is_authentic: true,
                        is_new_user: false,
                        altered_state: true,
                        potential_attack: false,
                    }
                } else {
                    ResonanceMatch::success(user_id, Some(user_name), confidence, patterns)
                }
            }
            None => {
                // Nincs egyezés - növeljük a failed counter-t
                *self.failed_attempts.write().await += 1;

                if profiles.is_empty() {
                    ResonanceMatch::new_user()
                } else {
                    ResonanceMatch {
                        user_id: None,
                        user_name: None,
                        confidence: 0.0,
                        matched_patterns: Vec::new(),
                        anomalies: Vec::new(),
                        is_authentic: false,
                        is_new_user: false,
                        altered_state: false,
                        potential_attack: failed >= 2,
                    }
                }
            }
        }
    }

    /// Session profilba építése
    fn build_session_profile(&self, session: &SessionData) -> ResonanceProfile {
        let mut profile = ResonanceProfile::new();

        for input in &session.inputs {
            self.update_profile(&mut profile, input);
        }

        profile
    }

    /// Rezonancia egyezés számítása
    fn calculate_resonance_match(
        &self,
        profile: &ResonanceProfile,
        current: &ResonanceProfile,
    ) -> (f64, Vec<PatternType>) {
        let mut total_score = 0.0;
        let mut matched_patterns = Vec::new();

        // Typing rhythm összehasonlítás
        let typing_score = self.compare_typing_rhythm(profile, current);
        if typing_score > 0.7 {
            matched_patterns.push(PatternType::TypingRhythm);
        }
        total_score += typing_score * self.weights.typing_rhythm;

        // Szóhasználat összehasonlítás
        let vocab_score = self.compare_vocabulary(profile, current);
        if vocab_score > 0.7 {
            matched_patterns.push(PatternType::WordFrequency);
        }
        total_score += vocab_score * self.weights.word_patterns;

        // Érzelmi aláírás összehasonlítás
        let emotional_score = self.compare_emotional(profile, current);
        if emotional_score > 0.7 {
            matched_patterns.push(PatternType::EmotionalBaseline);
        }
        total_score += emotional_score * self.weights.emotional_signature;

        // Időbeli minták összehasonlítás
        let temporal_score = self.compare_temporal(profile, current);
        if temporal_score > 0.7 {
            matched_patterns.push(PatternType::ActiveHours);
        }
        total_score += temporal_score * self.weights.temporal_patterns;

        // Kognitív stílus összehasonlítás
        let cognitive_score = self.compare_cognitive(profile, current);
        if cognitive_score > 0.7 {
            matched_patterns.push(PatternType::AbstractionLevel);
        }
        total_score += cognitive_score * self.weights.cognitive_style;

        (total_score, matched_patterns)
    }

    /// Typing rhythm összehasonlítás
    fn compare_typing_rhythm(&self, profile: &ResonanceProfile, current: &ResonanceProfile) -> f64 {
        if profile.typing_rhythm.is_empty() || current.typing_rhythm.is_empty() {
            return 0.5; // Nincs adat, semleges
        }

        let profile_avg: f64 = profile.typing_rhythm.iter().sum::<f64>() / profile.typing_rhythm.len() as f64;
        let current_avg: f64 = current.typing_rhythm.iter().sum::<f64>() / current.typing_rhythm.len() as f64;

        let diff = (profile_avg - current_avg).abs();
        let max_diff = profile_avg.max(current_avg);

        if max_diff == 0.0 {
            1.0
        } else {
            1.0 - (diff / max_diff).min(1.0)
        }
    }

    /// Szókincs összehasonlítás
    fn compare_vocabulary(&self, profile: &ResonanceProfile, current: &ResonanceProfile) -> f64 {
        if profile.word_frequency.is_empty() || current.word_frequency.is_empty() {
            return 0.5;
        }

        let profile_words: std::collections::HashSet<_> = profile.word_frequency.keys().collect();
        let current_words: std::collections::HashSet<_> = current.word_frequency.keys().collect();

        let intersection = profile_words.intersection(&current_words).count();
        let union = profile_words.union(&current_words).count();

        if union == 0 {
            0.5
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Érzelmi aláírás összehasonlítás
    fn compare_emotional(&self, profile: &ResonanceProfile, current: &ResonanceProfile) -> f64 {
        let mut diff_sum = 0.0;

        for i in 0..21 {
            diff_sum += (profile.emotional_baseline[i] - current.emotional_baseline[i]).abs();
        }

        let avg_diff = diff_sum / 21.0;
        1.0 - avg_diff.min(1.0)
    }

    /// Időbeli minták összehasonlítás
    fn compare_temporal(&self, profile: &ResonanceProfile, current: &ResonanceProfile) -> f64 {
        let mut matching_hours = 0;
        let mut profile_active = 0;

        for i in 0..24 {
            if profile.active_hours[i] {
                profile_active += 1;
                if current.active_hours[i] {
                    matching_hours += 1;
                }
            }
        }

        if profile_active == 0 {
            0.5
        } else {
            matching_hours as f64 / profile_active as f64
        }
    }

    /// Kognitív stílus összehasonlítás
    fn compare_cognitive(&self, profile: &ResonanceProfile, current: &ResonanceProfile) -> f64 {
        let mut score = 0.0;
        let mut count = 0.0;

        // Mondat hossz
        if profile.sentence_length_avg > 0.0 {
            let diff = (profile.sentence_length_avg - current.sentence_length_avg).abs();
            let max = profile.sentence_length_avg.max(current.sentence_length_avg);
            score += 1.0 - (diff / max).min(1.0);
            count += 1.0;
        }

        // Kérdés arány
        let q_diff = (profile.question_ratio - current.question_ratio).abs();
        score += 1.0 - q_diff.min(1.0);
        count += 1.0;

        // Absztrakció
        let abs_diff = (profile.abstraction_level - current.abstraction_level).abs();
        score += 1.0 - abs_diff.min(1.0);
        count += 1.0;

        // Formalitás
        let form_diff = (profile.formality_level - current.formality_level).abs();
        score += 1.0 - form_diff.min(1.0);
        count += 1.0;

        if count == 0.0 {
            0.5
        } else {
            score / count
        }
    }

    /// Konfidencia lekérdezése
    pub async fn confidence(&self) -> f64 {
        let profiles = self.profiles.read().await;
        if profiles.is_empty() {
            return 0.0;
        }
        profiles.values().map(|p| p.confidence).sum::<f64>() / profiles.len() as f64
    }

    /// Anomália detekció
    pub async fn detect_anomaly(&self, input: &UserInput) -> Option<Anomaly> {
        let profiles = self.profiles.read().await;

        // Keressük meg a hozzá tartozó profilt
        if let Some(profile) = profiles.get(&input.session_id) {
            // Ellenőrizzük az aktív órákat
            let hour = input.timestamp.hour() as usize;
            if profile.confidence > 0.7 && !profile.active_hours[hour] {
                return Some(Anomaly {
                    pattern_type: PatternType::ActiveHours,
                    deviation: 0.5,
                    description: format!("Szokatlan időpontban aktív: {} óra", hour),
                    detected_at: Utc::now(),
                });
            }

            // Ellenőrizzük a szóhasználatot
            let words: std::collections::HashSet<_> = input.content
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .collect();

            let known_words: std::collections::HashSet<_> = profile.word_frequency.keys().cloned().collect();
            let new_words = words.difference(&known_words).count();

            if profile.sample_count > 50 && new_words as f64 / words.len().max(1) as f64 > 0.7 {
                return Some(Anomaly {
                    pattern_type: PatternType::WordFrequency,
                    deviation: 0.7,
                    description: "Sok új, szokatlan szó használata".to_string(),
                    detected_at: Utc::now(),
                });
            }
        }

        None
    }

    /// Profil regisztrálása felhasználóhoz
    pub async fn register_user(&self, user_id: Uuid, user_name: Option<String>) -> Uuid {
        let mut profiles = self.profiles.write().await;
        let profile = ResonanceProfile::with_user(user_id, user_name);
        let profile_id = profile.id;
        profiles.insert(profile_id, profile);
        profile_id
    }

    /// Profil lekérdezése
    pub async fn get_profile(&self, profile_id: Uuid) -> Option<ResonanceProfile> {
        let profiles = self.profiles.read().await;
        profiles.get(&profile_id).cloned()
    }

    /// Státusz lekérdezése
    pub async fn status(&self) -> ResonanceStatus {
        let profiles = self.profiles.read().await;
        let session = self.current_session.read().await;

        let total_samples: u64 = profiles.values().map(|p| p.sample_count).sum();
        let avg_confidence = if profiles.is_empty() {
            0.0
        } else {
            profiles.values().map(|p| p.confidence).sum::<f64>() / profiles.len() as f64
        };

        ResonanceStatus {
            profile_count: profiles.len(),
            total_samples,
            avg_confidence,
            current_session_messages: session.message_count,
            current_session_duration_secs: session.duration_secs(),
            match_threshold: self.match_threshold,
        }
    }

    /// Session reset
    pub async fn reset_session(&self) {
        let mut session = self.current_session.write().await;
        *session = SessionData::new();
    }
}

impl Default for ResonanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Resonance státusz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceStatus {
    pub profile_count: usize,
    pub total_samples: u64,
    pub avg_confidence: f64,
    pub current_session_messages: u32,
    pub current_session_duration_secs: i64,
    pub match_threshold: f64,
}


// === Tesztek ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resonance_profile_creation() {
        let profile = ResonanceProfile::new();
        assert_eq!(profile.sample_count, 0);
        assert_eq!(profile.confidence, 0.0);
    }

    #[test]
    fn test_resonance_profile_with_user() {
        let user_id = Uuid::new_v4();
        let profile = ResonanceProfile::with_user(user_id, Some("Máté".to_string()));
        assert_eq!(profile.user_id, Some(user_id));
        assert_eq!(profile.user_name, Some("Máté".to_string()));
    }

    #[test]
    fn test_user_input_creation() {
        let input = UserInput::new("Hello World!".to_string());
        assert_eq!(input.content, "Hello World!");
        assert!(input.keystroke_timings.is_none());
    }

    #[test]
    fn test_session_data() {
        let mut session = SessionData::new();
        assert_eq!(session.message_count, 0);

        session.add_input(UserInput::new("Test".to_string()));
        assert_eq!(session.message_count, 1);
    }

    #[test]
    fn test_resonance_match_new_user() {
        let m = ResonanceMatch::new_user();
        assert!(m.is_new_user);
        assert!(!m.is_authentic);
        assert_eq!(m.confidence, 0.0);
    }

    #[test]
    fn test_resonance_match_altered_state() {
        let m = ResonanceMatch::altered_state(0.4);
        assert!(m.altered_state);
        assert!(m.is_authentic);
        assert!((m.confidence - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_resonance_match_potential_attack() {
        let m = ResonanceMatch::potential_attack();
        assert!(m.potential_attack);
        assert!(!m.is_authentic);
    }

    #[test]
    fn test_resonance_engine_creation() {
        let engine = ResonanceEngine::new();
        assert_eq!(engine.match_threshold, 0.85);
    }

    #[test]
    fn test_resonance_weights_default() {
        let weights = ResonanceWeights::default();
        let total = weights.typing_rhythm
            + weights.word_patterns
            + weights.emotional_signature
            + weights.temporal_patterns
            + weights.cognitive_style;
        assert!((total - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_resonance_learn() {
        let engine = ResonanceEngine::new();
        let input = UserInput::new("Ez egy teszt mondat a rezonancia rendszerhez.".to_string());

        let confidence = engine.learn(&input).await.unwrap();
        assert!(confidence >= 0.0);
    }

    #[tokio::test]
    async fn test_resonance_verify_new_user() {
        let engine = ResonanceEngine::new();
        let session = SessionData::new();

        let result = engine.verify(&session).await;
        assert!(result.is_new_user);
    }

    #[tokio::test]
    async fn test_resonance_status() {
        let engine = ResonanceEngine::new();
        let status = engine.status().await;

        assert_eq!(status.profile_count, 0);
        assert_eq!(status.total_samples, 0);
    }

    #[tokio::test]
    async fn test_resonance_register_user() {
        let engine = ResonanceEngine::new();
        let user_id = Uuid::new_v4();

        let profile_id = engine.register_user(user_id, Some("TestUser".to_string())).await;

        let profile = engine.get_profile(profile_id).await;
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().user_name, Some("TestUser".to_string()));
    }

    #[test]
    fn test_punctuation_pattern_default() {
        let pattern = PunctuationPattern::default();
        assert_eq!(pattern.period_freq, 0.0);
        assert_eq!(pattern.emoji_freq, 0.0);
    }

    #[test]
    fn test_anomaly_creation() {
        let anomaly = Anomaly {
            pattern_type: PatternType::ActiveHours,
            deviation: 0.5,
            description: "Test anomaly".to_string(),
            detected_at: Utc::now(),
        };
        assert_eq!(anomaly.pattern_type, PatternType::ActiveHours);
    }

    #[tokio::test]
    async fn test_resonance_confidence() {
        let engine = ResonanceEngine::new();
        let confidence = engine.confidence().await;
        assert_eq!(confidence, 0.0); // Nincs profil
    }

    #[tokio::test]
    async fn test_resonance_multiple_learns() {
        let engine = ResonanceEngine::new();
        let session_id = Uuid::new_v4();

        for i in 0..10 {
            let input = UserInput::with_session(
                format!("Ez a {} teszt üzenet.", i),
                session_id,
            );
            engine.learn(&input).await.unwrap();
        }

        let status = engine.status().await;
        assert!(status.total_samples >= 10);
    }
}
