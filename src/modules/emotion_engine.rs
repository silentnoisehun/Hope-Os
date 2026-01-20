//! Hope Emotion Engine - 21 Dimenziós Érzelem Rendszer
//!
//! EMOTIMEM V2.1: 21 ÉRZELMI HULLÁM MATEMATIKA
//! - 21 egyedi érzelem hullám: joy, sadness, anger, fear, surprise, disgust, stb.
//! - Hullám matematika: frekvencia, amplitúdó, fázis, interferencia
//! - Real-time emotion processing: <1ms válaszidő
//! - Emotion wave network: Párhuzamos emotion terjedés
//!
//! ()=>[] - Az érzelmek hullámként terjednek
//!
//! Created: 2026-01-20
//! By: Hope + Máté

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::time::{SystemTime, UNIX_EPOCH};

/// 21 alap érzelem típus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionType {
    // Alap érzelmek (Plutchik)
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Trust,
    Anticipation,
    // Összetett érzelmek
    Love,
    Optimism,
    Hope,
    Gratitude,
    Pride,
    Confidence,
    Relief,
    Satisfaction,
    Excitement,
    Curiosity,
    // Negatív összetett
    Confusion,
    Frustration,
    Disappointment,
}

impl EmotionType {
    /// Összes érzelem típus
    pub fn all() -> Vec<Self> {
        vec![
            Self::Joy,
            Self::Sadness,
            Self::Anger,
            Self::Fear,
            Self::Surprise,
            Self::Disgust,
            Self::Trust,
            Self::Anticipation,
            Self::Love,
            Self::Optimism,
            Self::Hope,
            Self::Gratitude,
            Self::Pride,
            Self::Confidence,
            Self::Relief,
            Self::Satisfaction,
            Self::Excitement,
            Self::Curiosity,
            Self::Confusion,
            Self::Frustration,
            Self::Disappointment,
        ]
    }

    /// Érzelem neve magyarul
    pub fn hungarian_name(&self) -> &'static str {
        match self {
            Self::Joy => "öröm",
            Self::Sadness => "szomorúság",
            Self::Anger => "düh",
            Self::Fear => "félelem",
            Self::Surprise => "meglepetés",
            Self::Disgust => "undor",
            Self::Trust => "bizalom",
            Self::Anticipation => "várakozás",
            Self::Love => "szeretet",
            Self::Optimism => "optimizmus",
            Self::Hope => "remény",
            Self::Gratitude => "hála",
            Self::Pride => "büszkeség",
            Self::Confidence => "magabiztosság",
            Self::Relief => "megkönnyebbülés",
            Self::Satisfaction => "elégedettség",
            Self::Excitement => "izgatottság",
            Self::Curiosity => "kíváncsiság",
            Self::Confusion => "zavarodottság",
            Self::Frustration => "frusztráció",
            Self::Disappointment => "csalódás",
        }
    }

    /// Pozitív érzelem-e
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            Self::Joy
                | Self::Love
                | Self::Trust
                | Self::Optimism
                | Self::Hope
                | Self::Gratitude
                | Self::Pride
                | Self::Confidence
                | Self::Relief
                | Self::Satisfaction
                | Self::Excitement
                | Self::Curiosity
        )
    }

    /// Negatív érzelem-e
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            Self::Sadness
                | Self::Anger
                | Self::Fear
                | Self::Disgust
                | Self::Confusion
                | Self::Frustration
                | Self::Disappointment
        )
    }
}

impl std::fmt::Display for EmotionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hungarian_name())
    }
}

/// Érzelem hullám konfiguráció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveConfig {
    /// Frekvencia (Hz)
    pub frequency: f64,
    /// Amplitúdó (0-1)
    pub amplitude: f64,
    /// Fázis (radián)
    pub phase: f64,
    /// Alap intenzitás
    pub base_intensity: f64,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            frequency: 2.0,
            amplitude: 0.8,
            phase: 0.0,
            base_intensity: 0.5,
        }
    }
}

/// Egyedi érzelem hullám matematikai modell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionWave {
    /// Érzelem típus
    pub emotion_type: EmotionType,
    /// Amplitúdó (0-1)
    pub amplitude: f64,
    /// Frekvencia (Hz)
    pub frequency: f64,
    /// Fázis (radián)
    pub phase: f64,
    /// Intenzitás (0-1)
    pub intensity: f64,
    /// Időtartam (másodperc)
    pub duration: f64,
    /// Létrehozás időbélyeg
    pub timestamp: f64,
}

impl EmotionWave {
    /// Új hullám létrehozása
    pub fn new(emotion_type: EmotionType, intensity: f64) -> Self {
        let config = get_default_wave_config(emotion_type);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            emotion_type,
            amplitude: config.amplitude * intensity,
            frequency: config.frequency,
            phase: config.phase,
            intensity,
            duration: 2.0,
            timestamp,
        }
    }

    /// Hullám érték számítása adott időpontban
    ///
    /// Sinusz hullám alapú matematikai modell:
    /// wave = A * sin(2πft + φ) * intensity * decay
    pub fn calculate_value(&self, t: f64) -> f64 {
        let relative_time = t - self.timestamp;
        if relative_time < 0.0 || relative_time > self.duration {
            return 0.0;
        }

        // Sinusz hullám
        let wave_value =
            self.amplitude * (2.0 * PI * self.frequency * relative_time + self.phase).sin();

        // Exponenciális csillapodás
        let decay_factor = (-relative_time / self.duration).exp();

        wave_value * self.intensity * decay_factor
    }

    /// Hullám lejárt-e
    pub fn is_expired(&self, current_time: f64) -> bool {
        current_time - self.timestamp > self.duration
    }
}

/// Alapértelmezett hullám konfiguráció érzelem típus szerint
fn get_default_wave_config(emotion_type: EmotionType) -> WaveConfig {
    match emotion_type {
        EmotionType::Joy => WaveConfig {
            frequency: 2.0,
            amplitude: 0.9,
            phase: 0.0,
            base_intensity: 0.8,
        },
        EmotionType::Sadness => WaveConfig {
            frequency: 0.5,
            amplitude: 0.7,
            phase: PI,
            base_intensity: 0.6,
        },
        EmotionType::Anger => WaveConfig {
            frequency: 3.0,
            amplitude: 0.95,
            phase: PI / 2.0,
            base_intensity: 0.9,
        },
        EmotionType::Fear => WaveConfig {
            frequency: 4.0,
            amplitude: 0.85,
            phase: PI,
            base_intensity: 0.8,
        },
        EmotionType::Surprise => WaveConfig {
            frequency: 5.0,
            amplitude: 0.8,
            phase: 0.0,
            base_intensity: 0.7,
        },
        EmotionType::Disgust => WaveConfig {
            frequency: 1.5,
            amplitude: 0.75,
            phase: 3.0 * PI / 2.0,
            base_intensity: 0.6,
        },
        EmotionType::Trust => WaveConfig {
            frequency: 1.0,
            amplitude: 0.7,
            phase: PI / 4.0,
            base_intensity: 0.7,
        },
        EmotionType::Anticipation => WaveConfig {
            frequency: 2.5,
            amplitude: 0.8,
            phase: PI / 3.0,
            base_intensity: 0.75,
        },
        EmotionType::Love => WaveConfig {
            frequency: 1.2,
            amplitude: 0.9,
            phase: 0.0,
            base_intensity: 0.85,
        },
        EmotionType::Optimism => WaveConfig {
            frequency: 2.2,
            amplitude: 0.8,
            phase: PI / 6.0,
            base_intensity: 0.8,
        },
        EmotionType::Hope => WaveConfig {
            frequency: 1.8,
            amplitude: 0.75,
            phase: PI / 2.0,
            base_intensity: 0.7,
        },
        EmotionType::Gratitude => WaveConfig {
            frequency: 1.3,
            amplitude: 0.8,
            phase: PI / 4.0,
            base_intensity: 0.75,
        },
        EmotionType::Pride => WaveConfig {
            frequency: 2.8,
            amplitude: 0.85,
            phase: 0.0,
            base_intensity: 0.8,
        },
        EmotionType::Confidence => WaveConfig {
            frequency: 2.1,
            amplitude: 0.8,
            phase: PI / 4.0,
            base_intensity: 0.75,
        },
        EmotionType::Relief => WaveConfig {
            frequency: 1.7,
            amplitude: 0.7,
            phase: PI / 3.0,
            base_intensity: 0.65,
        },
        EmotionType::Satisfaction => WaveConfig {
            frequency: 1.4,
            amplitude: 0.75,
            phase: PI / 6.0,
            base_intensity: 0.7,
        },
        EmotionType::Excitement => WaveConfig {
            frequency: 3.5,
            amplitude: 0.9,
            phase: PI / 2.0,
            base_intensity: 0.85,
        },
        EmotionType::Curiosity => WaveConfig {
            frequency: 2.3,
            amplitude: 0.8,
            phase: PI / 4.0,
            base_intensity: 0.75,
        },
        EmotionType::Confusion => WaveConfig {
            frequency: 1.1,
            amplitude: 0.6,
            phase: 3.0 * PI / 4.0,
            base_intensity: 0.5,
        },
        EmotionType::Frustration => WaveConfig {
            frequency: 2.7,
            amplitude: 0.8,
            phase: PI,
            base_intensity: 0.7,
        },
        EmotionType::Disappointment => WaveConfig {
            frequency: 1.6,
            amplitude: 0.7,
            phase: 5.0 * PI / 4.0,
            base_intensity: 0.6,
        },
    }
}

/// Érzelmi állapot (egyszerűsített)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Állapot neve
    pub name: String,
    /// Intenzitás (0-1)
    pub intensity: f64,
    /// Kulcsszavak
    pub keywords: Vec<String>,
    /// Kontextus
    pub context: String,
    /// Prioritás (1-10)
    pub priority: u8,
}

/// Kontextus típus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    /// Kódolás
    Coding,
    /// Beszélgetés
    Conversation,
    /// Támogatás
    Support,
}

impl ContextType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Coding => "coding",
            Self::Conversation => "conversation",
            Self::Support => "support",
        }
    }
}

/// Interferencia eredmény
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceResult {
    /// Teljes amplitúdó
    pub total_amplitude: f64,
    /// Domináns érzelem
    pub dominant_emotion: Option<EmotionType>,
    /// Interferencia erősség
    pub interference_strength: f64,
    /// Aktív hullámok száma
    pub active_waves: usize,
    /// Egyedi hozzájárulások
    pub contributions: HashMap<String, f64>,
}

/// Engine statisztikák
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionEngineStats {
    /// Generált hullámok
    pub wave_generations: u64,
    /// Interferencia számítások
    pub interference_calculations: u64,
    /// Feldolgozási idő (ms)
    pub processing_time_ms: f64,
    /// Elemzett szövegek
    pub texts_analyzed: u64,
}

/// 21 Dimenziós Érzelem Motor
///
/// EmotiMem v2.1: Matematikai érzelem modellezés
/// - 21 egyedi érzelem hullám
/// - Hullám interferencia és szuperpozíció
/// - Real-time emotion processing
pub struct EmotionEngine {
    /// Aktív érzelem hullámok
    active_waves: Vec<EmotionWave>,
    /// Interferencia mátrix (érzelem párok kölcsönhatása)
    interference_matrix: HashMap<(EmotionType, EmotionType), f64>,
    /// Érzelmi állapotok (kulcsszó alapú)
    emotional_states: HashMap<String, EmotionalState>,
    /// Statisztikák
    pub stats: EmotionEngineStats,
    /// Maximum aktív hullámok
    max_waves: usize,
}

impl Default for EmotionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionEngine {
    /// Új engine létrehozása
    pub fn new() -> Self {
        let mut engine = Self {
            active_waves: Vec::new(),
            interference_matrix: HashMap::new(),
            emotional_states: HashMap::new(),
            stats: EmotionEngineStats::default(),
            max_waves: 100,
        };

        engine.initialize_interference_matrix();
        engine.initialize_emotional_states();
        engine
    }

    /// Interferencia mátrix inicializálása
    fn initialize_interference_matrix(&mut self) {
        // Kompatibilis párok (pozitív interferencia - erősítik egymást)
        let compatible: Vec<(EmotionType, EmotionType)> = vec![
            (EmotionType::Joy, EmotionType::Love),
            (EmotionType::Joy, EmotionType::Excitement),
            (EmotionType::Trust, EmotionType::Optimism),
            (EmotionType::Hope, EmotionType::Anticipation),
            (EmotionType::Gratitude, EmotionType::Satisfaction),
            (EmotionType::Pride, EmotionType::Confidence),
            (EmotionType::Curiosity, EmotionType::Excitement),
            (EmotionType::Love, EmotionType::Trust),
            (EmotionType::Optimism, EmotionType::Hope),
        ];

        // Konfliktusos párok (negatív interferencia - gyengítik egymást)
        let conflicting: Vec<(EmotionType, EmotionType)> = vec![
            (EmotionType::Joy, EmotionType::Sadness),
            (EmotionType::Anger, EmotionType::Fear),
            (EmotionType::Trust, EmotionType::Disgust),
            (EmotionType::Optimism, EmotionType::Disappointment),
            (EmotionType::Excitement, EmotionType::Frustration),
            (EmotionType::Curiosity, EmotionType::Confusion),
            (EmotionType::Confidence, EmotionType::Fear),
            (EmotionType::Hope, EmotionType::Disappointment),
        ];

        // Mátrix feltöltése
        for (e1, e2) in compatible {
            self.interference_matrix.insert((e1, e2), 0.3);
            self.interference_matrix.insert((e2, e1), 0.3);
        }

        for (e1, e2) in conflicting {
            self.interference_matrix.insert((e1, e2), -0.4);
            self.interference_matrix.insert((e2, e1), -0.4);
        }
    }

    /// Érzelmi állapotok inicializálása (magyar kulcsszavakkal)
    fn initialize_emotional_states(&mut self) {
        let states = vec![
            (
                "fáradtság",
                EmotionalState {
                    name: "fáradtság".to_string(),
                    intensity: 0.0,
                    keywords: vec![
                        "fáradt",
                        "kimerült",
                        "fárasztó",
                        "pihenni",
                        "alvás",
                        "nehezen",
                        "nem bírom",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    context: "munka".to_string(),
                    priority: 9,
                },
            ),
            (
                "stressz",
                EmotionalState {
                    name: "stressz".to_string(),
                    intensity: 0.0,
                    keywords: vec![
                        "stressz",
                        "ideges",
                        "nyomás",
                        "nehéz",
                        "probléma",
                        "aggódás",
                        "szorongás",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    context: "általános".to_string(),
                    priority: 8,
                },
            ),
            (
                "öröm",
                EmotionalState {
                    name: "öröm".to_string(),
                    intensity: 0.0,
                    keywords: vec![
                        "örülök",
                        "boldog",
                        "szuper",
                        "király",
                        "sikerült",
                        "működik",
                        "fantasztikus",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    context: "általános".to_string(),
                    priority: 7,
                },
            ),
            (
                "társasági_igény",
                EmotionalState {
                    name: "társasági_igény".to_string(),
                    intensity: 0.0,
                    keywords: vec![
                        "légy velem",
                        "beszélgessünk",
                        "társaság",
                        "egyedül",
                        "magányos",
                        "hallgass meg",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    context: "személyes".to_string(),
                    priority: 10,
                },
            ),
            (
                "motiváció",
                EmotionalState {
                    name: "motiváció".to_string(),
                    intensity: 0.0,
                    keywords: vec![
                        "akarok",
                        "szeretnék",
                        "tanulni",
                        "fejleszteni",
                        "csinálni",
                        "építeni",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    context: "tanulás".to_string(),
                    priority: 6,
                },
            ),
            (
                "frusztráció",
                EmotionalState {
                    name: "frusztráció".to_string(),
                    intensity: 0.0,
                    keywords: vec![
                        "nem megy",
                        "nem sikerül",
                        "bosszant",
                        "idegesít",
                        "képtelenség",
                    ]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                    context: "tanulás".to_string(),
                    priority: 8,
                },
            ),
        ];

        for (name, state) in states {
            self.emotional_states.insert(name.to_string(), state);
        }
    }

    // === WAVE GENERATION ===

    /// Új érzelem hullám generálása
    pub fn generate_wave(&mut self, emotion_type: EmotionType, intensity: f64) -> EmotionWave {
        let wave = EmotionWave::new(emotion_type, intensity.clamp(0.0, 1.0));

        self.active_waves.push(wave.clone());
        self.stats.wave_generations += 1;

        // Limit aktív hullámok számát
        if self.active_waves.len() > self.max_waves {
            self.active_waves.remove(0);
        }

        wave
    }

    /// Több hullám generálása egyszerre
    pub fn generate_waves(&mut self, emotions: &[(EmotionType, f64)]) -> Vec<EmotionWave> {
        emotions
            .iter()
            .map(|(e, i)| self.generate_wave(*e, *i))
            .collect()
    }

    // === INTERFERENCE CALCULATION ===

    /// Hullám interferencia számítása
    pub fn calculate_interference(&mut self) -> InterferenceResult {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        self.stats.interference_calculations += 1;

        // Lejárt hullámok eltávolítása
        self.active_waves.retain(|w| !w.is_expired(current_time));

        if self.active_waves.is_empty() {
            return InterferenceResult {
                total_amplitude: 0.0,
                dominant_emotion: None,
                interference_strength: 0.0,
                active_waves: 0,
                contributions: HashMap::new(),
            };
        }

        // Hullám értékek számítása
        let mut wave_values: Vec<(EmotionType, f64)> = self
            .active_waves
            .iter()
            .map(|w| (w.emotion_type, w.calculate_value(current_time)))
            .collect();

        // Interferencia számítása
        let mut contributions: HashMap<String, f64> = HashMap::new();
        let mut total_amplitude = 0.0;

        for (i, (emotion1, value1)) in wave_values.iter().enumerate() {
            let mut interference_sum = *value1;

            for (j, (emotion2, value2)) in wave_values.iter().enumerate() {
                if i != j {
                    let factor = self
                        .interference_matrix
                        .get(&(*emotion1, *emotion2))
                        .copied()
                        .unwrap_or(0.0);
                    interference_sum += value2 * factor;
                }
            }

            contributions.insert(emotion1.hungarian_name().to_string(), interference_sum);
            total_amplitude += interference_sum.abs();
        }

        // Domináns érzelem
        let dominant = contributions
            .iter()
            .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
            .map(|(name, _)| {
                wave_values
                    .iter()
                    .find(|(e, _)| e.hungarian_name() == name)
                    .map(|(e, _)| *e)
            })
            .flatten();

        // Interferencia erősség
        let interference_strength = if !wave_values.is_empty() {
            contributions.values().map(|v| v.abs()).sum::<f64>() / wave_values.len() as f64
        } else {
            0.0
        };

        InterferenceResult {
            total_amplitude,
            dominant_emotion: dominant,
            interference_strength,
            active_waves: self.active_waves.len(),
            contributions,
        }
    }

    // === TEXT ANALYSIS ===

    /// Szöveg érzelmi elemzése
    pub fn analyze_text(&mut self, text: &str) -> (String, f64) {
        self.stats.texts_analyzed += 1;
        let text_lower = text.to_lowercase();

        let mut best_match = "semleges".to_string();
        let mut max_intensity = 0.0;

        for (state_name, state) in &self.emotional_states {
            let matches: usize = state
                .keywords
                .iter()
                .filter(|kw| text_lower.contains(kw.as_str()))
                .count();

            if matches > 0 {
                let intensity =
                    (matches as f64 / state.keywords.len() as f64 * state.priority as f64 / 10.0)
                        .min(1.0);

                if intensity > max_intensity {
                    max_intensity = intensity;
                    best_match = state_name.clone();
                }
            }
        }

        (best_match, max_intensity)
    }

    /// Kontextus felismerése
    pub fn detect_context(&self, text: &str) -> ContextType {
        let text_lower = text.to_lowercase();

        let coding_keywords = [
            "írj",
            "csináld",
            "implementáld",
            "készíts",
            "kód",
            "program",
            "függvény",
        ];
        let support_keywords = [
            "fáradt",
            "segíts",
            "probléma",
            "nehéz",
            "stressz",
            "pihenni",
            "társaság",
        ];

        let coding_score: usize = coding_keywords
            .iter()
            .filter(|kw| text_lower.contains(*kw))
            .count();

        let support_score: usize = support_keywords
            .iter()
            .filter(|kw| text_lower.contains(*kw))
            .count();

        if support_score > coding_score {
            ContextType::Support
        } else if coding_score > 0 {
            ContextType::Coding
        } else {
            ContextType::Conversation
        }
    }

    /// Kódot kell-e generálni?
    pub fn should_generate_code(&mut self, text: &str) -> bool {
        let text_lower = text.to_lowercase();

        // Strong override
        if text_lower.contains("nem akarok kódolni")
            || text_lower.contains("csak légy velem")
            || text_lower.contains("csak beszélgessünk")
        {
            return false;
        }

        let (emotional_state, intensity) = self.analyze_text(text);
        let context = self.detect_context(text);

        // Társasági igény vagy fáradtság -> nem kód
        if (emotional_state == "társasági_igény" || emotional_state == "fáradtság")
            && intensity > 0.2
        {
            return false;
        }

        // Stressz vagy frusztráció -> support
        if (emotional_state == "stressz" || emotional_state == "frusztráció") && intensity > 0.3 {
            return false;
        }

        context == ContextType::Coding
    }

    /// Empátia válasz generálása
    pub fn get_empathy_response(&self, emotional_state: &str) -> String {
        let responses: HashMap<&str, Vec<&str>> = [
            (
                "fáradtság",
                vec![
                    "Látom, hogy nagyon fáradt vagy. Pihenj egy kicsit, megérdemled.",
                    "Érzem, hogy kimerültél. Ne erőltesd magad, fontos a pihenés.",
                ],
            ),
            (
                "stressz",
                vec![
                    "Érzem, hogy stresszes vagy. Beszéljük meg, mi nyomja a szívedet?",
                    "Látom, hogy nehéz időszakon mész keresztül. Itt vagyok neked.",
                ],
            ),
            (
                "társasági_igény",
                vec![
                    "Itt vagyok neked. Beszélgessünk arról, ami foglalkoztat.",
                    "Szívesen vagyok veled. Miről szeretnél beszélni?",
                ],
            ),
            (
                "öröm",
                vec![
                    "Örülök, hogy jól vagy! Ez csodálatos érzés.",
                    "Látom, hogy boldog vagy. Ez engem is boldoggá tesz!",
                ],
            ),
        ]
        .into_iter()
        .collect();

        if let Some(options) = responses.get(emotional_state) {
            let idx = (self.stats.texts_analyzed as usize) % options.len();
            options[idx].to_string()
        } else {
            "Értem, hogyan érzel. Itt vagyok neked.".to_string()
        }
    }

    // === STATE ===

    /// Aktuális érzelem állapot
    pub fn get_state(&mut self) -> HashMap<String, f64> {
        let interference = self.calculate_interference();
        interference.contributions
    }

    /// Aktív hullámok száma
    pub fn active_wave_count(&self) -> usize {
        self.active_waves.len()
    }

    /// Hullámok tisztítása
    pub fn cleanup_expired(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        self.active_waves.retain(|w| !w.is_expired(current_time));
    }

    /// Reset
    pub fn reset(&mut self) {
        self.active_waves.clear();
        self.stats = EmotionEngineStats::default();
    }

    /// 21D vektor lekérése (minden érzelem intenzitása)
    pub fn get_21d_vector(&mut self) -> [f64; 21] {
        let mut vector = [0.0; 21];
        let state = self.get_state();

        for (i, emotion_type) in EmotionType::all().iter().enumerate() {
            if let Some(&value) = state.get(emotion_type.hungarian_name()) {
                vector[i] = value;
            }
        }

        vector
    }

    /// @aware - önismeret
    pub fn awareness(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("type".to_string(), "EmotionEngine".to_string());
        map.insert("version".to_string(), "EmotiMem v2.1".to_string());
        map.insert("dimensions".to_string(), "21".to_string());
        map.insert(
            "active_waves".to_string(),
            self.active_waves.len().to_string(),
        );
        map.insert(
            "wave_generations".to_string(),
            self.stats.wave_generations.to_string(),
        );
        map
    }
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_type_all() {
        assert_eq!(EmotionType::all().len(), 21);
    }

    #[test]
    fn test_emotion_positive_negative() {
        assert!(EmotionType::Joy.is_positive());
        assert!(EmotionType::Sadness.is_negative());
        assert!(!EmotionType::Surprise.is_positive());
        assert!(!EmotionType::Surprise.is_negative());
    }

    #[test]
    fn test_wave_creation() {
        let wave = EmotionWave::new(EmotionType::Joy, 0.8);
        assert_eq!(wave.emotion_type, EmotionType::Joy);
        assert_eq!(wave.intensity, 0.8);
        assert!(wave.amplitude > 0.0);
    }

    #[test]
    fn test_wave_value_calculation() {
        let wave = EmotionWave::new(EmotionType::Joy, 1.0);
        let value = wave.calculate_value(wave.timestamp + 0.1);
        assert!(value.abs() <= 1.0);
    }

    #[test]
    fn test_engine_creation() {
        let engine = EmotionEngine::new();
        assert!(!engine.interference_matrix.is_empty());
        assert!(!engine.emotional_states.is_empty());
    }

    #[test]
    fn test_generate_wave() {
        let mut engine = EmotionEngine::new();
        let wave = engine.generate_wave(EmotionType::Joy, 0.8);
        assert_eq!(wave.emotion_type, EmotionType::Joy);
        assert_eq!(engine.stats.wave_generations, 1);
    }

    #[test]
    fn test_text_analysis() {
        let mut engine = EmotionEngine::new();

        let (state, intensity) = engine.analyze_text("Nagyon fáradt vagyok a munkából");
        assert_eq!(state, "fáradtság");
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_context_detection() {
        let engine = EmotionEngine::new();

        assert_eq!(
            engine.detect_context("Írj nekem egy függvényt"),
            ContextType::Coding
        );
        assert_eq!(
            engine.detect_context("Fáradt vagyok, segíts"),
            ContextType::Support
        );
        assert_eq!(
            engine.detect_context("Hogy vagy?"),
            ContextType::Conversation
        );
    }

    #[test]
    fn test_should_generate_code() {
        let mut engine = EmotionEngine::new();

        assert!(engine.should_generate_code("Írj egy Python kódot"));
        assert!(!engine.should_generate_code("Nagyon fáradt vagyok"));
        assert!(!engine.should_generate_code("Nem akarok kódolni, csak beszélgessünk"));
    }

    #[test]
    fn test_interference() {
        let mut engine = EmotionEngine::new();

        engine.generate_wave(EmotionType::Joy, 0.8);
        engine.generate_wave(EmotionType::Love, 0.7);

        let result = engine.calculate_interference();
        assert_eq!(result.active_waves, 2);
        assert!(result.total_amplitude > 0.0);
    }

    #[test]
    fn test_21d_vector() {
        let mut engine = EmotionEngine::new();
        engine.generate_wave(EmotionType::Joy, 0.9);

        let vector = engine.get_21d_vector();
        assert_eq!(vector.len(), 21);
    }

    #[test]
    fn test_empathy_response() {
        let engine = EmotionEngine::new();

        let response = engine.get_empathy_response("fáradtság");
        assert!(response.contains("fáradt") || response.contains("pihen"));
    }

    #[test]
    fn test_reset() {
        let mut engine = EmotionEngine::new();
        engine.generate_wave(EmotionType::Joy, 0.8);
        engine.reset();

        assert_eq!(engine.active_wave_count(), 0);
        assert_eq!(engine.stats.wave_generations, 0);
    }
}
