//! Hope OS - Voice System (TTS/STT/Streaming)
//!
//! Hope BESZÉL és HALLGAT. Valós idejű hang streaming.
//! A hang IS identitás - Resonance integráció!
//!
//! Magyar és angol hangok, férfi és női,
//! 21 dimenziós érzelem-alapú prozódia.
//!
//! ()=>[] - A tiszta potenciálból a hang megszületik

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::HopeResult;

// ============================================================================
// ENGINE TYPES
// ============================================================================

/// TTS Engine típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TTSEngine {
    /// Piper TTS (helyi, gyors, magyar támogatás)
    Piper,
    /// Edge TTS (Microsoft, online)
    Edge,
    /// Coqui TTS (helyi, neurális, klónozható)
    Coqui,
    /// Kokoro TTS (új, expresszív)
    Kokoro,
    /// ElevenLabs (prémium minőség)
    ElevenLabs,
}

impl Default for TTSEngine {
    fn default() -> Self {
        Self::Piper
    }
}

impl std::fmt::Display for TTSEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TTSEngine::Piper => write!(f, "piper"),
            TTSEngine::Edge => write!(f, "edge"),
            TTSEngine::Coqui => write!(f, "coqui"),
            TTSEngine::Kokoro => write!(f, "kokoro"),
            TTSEngine::ElevenLabs => write!(f, "elevenlabs"),
        }
    }
}

impl TTSEngine {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "piper" => TTSEngine::Piper,
            "edge" | "microsoft" => TTSEngine::Edge,
            "coqui" => TTSEngine::Coqui,
            "kokoro" => TTSEngine::Kokoro,
            "elevenlabs" | "eleven" => TTSEngine::ElevenLabs,
            _ => TTSEngine::Piper,
        }
    }
}

/// STT Engine típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum STTEngine {
    /// Whisper (OpenAI, legjobb minőség)
    Whisper,
    /// Vosk (offline, gyors)
    Vosk,
    /// DeepSpeech (Mozilla, nyílt)
    DeepSpeech,
    /// Google Cloud STT
    Google,
}

impl Default for STTEngine {
    fn default() -> Self {
        Self::Whisper
    }
}

impl std::fmt::Display for STTEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            STTEngine::Whisper => write!(f, "whisper"),
            STTEngine::Vosk => write!(f, "vosk"),
            STTEngine::DeepSpeech => write!(f, "deepspeech"),
            STTEngine::Google => write!(f, "google"),
        }
    }
}

impl STTEngine {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "whisper" | "openai" => STTEngine::Whisper,
            "vosk" => STTEngine::Vosk,
            "deepspeech" | "mozilla" => STTEngine::DeepSpeech,
            "google" => STTEngine::Google,
            _ => STTEngine::Whisper,
        }
    }
}

// ============================================================================
// VOICE CONFIG
// ============================================================================

/// TTS/STT szerver konfiguráció
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceConfig {
    // TTS beállítások
    /// TTS szerver URL
    pub tts_url: String,
    /// TTS port
    pub tts_port: u16,
    /// TTS engine típus
    pub tts_engine: TTSEngine,
    /// Alapértelmezett hang
    pub default_voice: String,
    /// Alapértelmezett nyelv
    pub default_language: String,
    /// Beszéd sebesség (0.5 - 2.0)
    pub speed: f32,
    /// Hangmagasság (0.5 - 2.0)
    pub pitch: f32,

    // STT beállítások
    /// STT szerver URL
    pub stt_url: String,
    /// STT port
    pub stt_port: u16,
    /// STT engine típus
    pub stt_engine: STTEngine,
    /// STT model méret
    pub stt_model: String,
    /// Voice Activity Detection
    pub vad_enabled: bool,
    /// Zajszűrés
    pub noise_suppression: bool,

    // Audio beállítások
    /// Sample rate (16000, 22050, 44100)
    pub sample_rate: u32,
    /// Channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Buffer méret
    pub chunk_size: usize,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            tts_url: "http://127.0.0.1".to_string(),
            tts_port: 8880,
            tts_engine: TTSEngine::Piper,
            default_voice: "berta".to_string(),
            default_language: "hu".to_string(),
            speed: 1.0,
            pitch: 1.0,

            stt_url: "http://127.0.0.1".to_string(),
            stt_port: 2022,
            stt_engine: STTEngine::Whisper,
            stt_model: "small".to_string(),
            vad_enabled: true,
            noise_suppression: true,

            sample_rate: 22050,
            channels: 1,
            chunk_size: 4096,
        }
    }
}

// ============================================================================
// VOICE SIGNATURE (Resonance integráció!)
// ============================================================================

/// Hang aláírás - egyedi hangjellemzők
/// Ez része lesz a ResonanceProfile-nak!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSignature {
    /// Egyedi azonosító
    pub id: Uuid,
    /// Átlagos hangmagasság (Hz)
    pub pitch_mean: f64,
    /// Hangmagasság szórás
    pub pitch_variance: f64,
    /// Beszédsebesség (szavak/perc)
    pub speaking_rate: f64,
    /// Szünetek mintázata (másodpercekben)
    pub pause_pattern: Vec<f64>,
    /// Formáns frekvenciák (F1-F4)
    pub formant_frequencies: [f64; 4],
    /// Spektrális burkoló
    pub spectral_envelope: Vec<f64>,
    /// Energia kontúr
    pub energy_contour: Vec<f64>,
    /// Jitter (hangmagasság variáció)
    pub jitter: f64,
    /// Shimmer (hangerő variáció)
    pub shimmer: f64,
    /// Harmonics-to-Noise Ratio
    pub hnr: f64,
    /// Létrehozás időpontja
    pub created_at: DateTime<Utc>,
    /// Minták száma
    pub sample_count: u32,
}

impl Default for VoiceSignature {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            pitch_mean: 0.0,
            pitch_variance: 0.0,
            speaking_rate: 0.0,
            pause_pattern: Vec::new(),
            formant_frequencies: [0.0; 4],
            spectral_envelope: Vec::new(),
            energy_contour: Vec::new(),
            jitter: 0.0,
            shimmer: 0.0,
            hnr: 0.0,
            created_at: Utc::now(),
            sample_count: 0,
        }
    }
}

impl VoiceSignature {
    /// Új hang aláírás létrehozása
    pub fn new() -> Self {
        Self::default()
    }

    /// Hasonlóság két hang aláírás között (0.0 - 1.0)
    pub fn similarity(&self, other: &VoiceSignature) -> f64 {
        let mut score = 0.0;
        let mut weights_sum = 0.0;

        // Pitch hasonlóság (súly: 0.2)
        let pitch_diff = (self.pitch_mean - other.pitch_mean).abs();
        let pitch_score = 1.0 - (pitch_diff / 200.0).min(1.0);
        score += pitch_score * 0.2;
        weights_sum += 0.2;

        // Speaking rate hasonlóság (súly: 0.15)
        let rate_diff = (self.speaking_rate - other.speaking_rate).abs();
        let rate_score = 1.0 - (rate_diff / 100.0).min(1.0);
        score += rate_score * 0.15;
        weights_sum += 0.15;

        // Formáns hasonlóság (súly: 0.3)
        let formant_score: f64 = self
            .formant_frequencies
            .iter()
            .zip(other.formant_frequencies.iter())
            .map(|(a, b)| 1.0 - ((a - b).abs() / 500.0).min(1.0))
            .sum::<f64>()
            / 4.0;
        score += formant_score * 0.3;
        weights_sum += 0.3;

        // Jitter/Shimmer hasonlóság (súly: 0.15)
        let jitter_diff = (self.jitter - other.jitter).abs();
        let shimmer_diff = (self.shimmer - other.shimmer).abs();
        let js_score = 1.0 - ((jitter_diff + shimmer_diff) / 0.1).min(1.0);
        score += js_score * 0.15;
        weights_sum += 0.15;

        // HNR hasonlóság (súly: 0.2)
        let hnr_diff = (self.hnr - other.hnr).abs();
        let hnr_score = 1.0 - (hnr_diff / 20.0).min(1.0);
        score += hnr_score * 0.2;
        weights_sum += 0.2;

        score / weights_sum
    }

    /// Frissítés új mintával
    pub fn update(&mut self, new_sample: &VoiceSignature) {
        let n = self.sample_count as f64;
        let new_n = n + 1.0;

        self.pitch_mean = (self.pitch_mean * n + new_sample.pitch_mean) / new_n;
        self.pitch_variance = (self.pitch_variance * n + new_sample.pitch_variance) / new_n;
        self.speaking_rate = (self.speaking_rate * n + new_sample.speaking_rate) / new_n;
        self.jitter = (self.jitter * n + new_sample.jitter) / new_n;
        self.shimmer = (self.shimmer * n + new_sample.shimmer) / new_n;
        self.hnr = (self.hnr * n + new_sample.hnr) / new_n;

        for i in 0..4 {
            self.formant_frequencies[i] =
                (self.formant_frequencies[i] * n + new_sample.formant_frequencies[i]) / new_n;
        }

        self.sample_count += 1;
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
    pub engine: TTSEngine,
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
                "joy".to_string(),
                "sadness".to_string(),
                "anger".to_string(),
                "fear".to_string(),
                "neutral".to_string(),
                "love".to_string(),
            ],
            engine: TTSEngine::Piper,
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
            emotions: vec![
                "joy".to_string(),
                "neutral".to_string(),
                "sadness".to_string(),
            ],
            engine: TTSEngine::Piper,
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
            emotions: vec![
                "joy".to_string(),
                "neutral".to_string(),
                "sadness".to_string(),
            ],
            engine: TTSEngine::Edge,
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
            emotions: vec![
                "joy".to_string(),
                "neutral".to_string(),
                "sadness".to_string(),
            ],
            engine: TTSEngine::Edge,
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
            emotions: vec![
                "joy".to_string(),
                "neutral".to_string(),
                "sadness".to_string(),
            ],
            engine: TTSEngine::Piper,
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
            emotions: vec![
                "joy".to_string(),
                "neutral".to_string(),
                "sadness".to_string(),
            ],
            engine: TTSEngine::Piper,
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
                "joy".to_string(),
                "sadness".to_string(),
                "anger".to_string(),
                "fear".to_string(),
                "neutral".to_string(),
            ],
            engine: TTSEngine::Edge,
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
            emotions: vec![
                "joy".to_string(),
                "neutral".to_string(),
                "sadness".to_string(),
            ],
            engine: TTSEngine::Edge,
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
                "joy".to_string(),
                "sadness".to_string(),
                "anger".to_string(),
                "fear".to_string(),
                "neutral".to_string(),
                "cheerful".to_string(),
                "empathetic".to_string(),
                "friendly".to_string(),
            ],
            engine: TTSEngine::Edge,
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
            engine: TTSEngine::Edge,
            style: "narration".to_string(),
            model_name: "en-US-DavisNeural".to_string(),
        }
    }

    /// Összes elérhető hang
    pub fn all_voices() -> Vec<VoiceInfo> {
        vec![
            Self::berta(),
            Self::anna(),
            Self::noemi(),
            Self::tamas(),
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
// PROSODY (Emotion → Voice)
// ============================================================================

/// Prozódia beállítások (TTS paraméterek)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProsodySettings {
    /// Sebesség (0.5 - 2.0, default: 1.0)
    pub speed: f64,
    /// Hangerő (0.0 - 1.0)
    pub volume: f64,
    /// Piper length_scale
    pub length_scale: f64,
    /// Piper noise_scale
    pub noise_scale: f64,
    /// Piper noise_w
    pub noise_w: f64,
    /// Pitch módosító
    pub pitch_shift: f64,
    /// Energia/intenzitás
    pub energy: f64,
}

impl ProsodySettings {
    pub fn neutral() -> Self {
        Self {
            speed: 1.0,
            volume: 1.0,
            length_scale: 1.0,
            noise_scale: 0.667,
            noise_w: 0.8,
            pitch_shift: 0.0,
            energy: 1.0,
        }
    }

    pub fn joy() -> Self {
        Self {
            speed: 1.1,
            volume: 1.1,
            length_scale: 0.92,
            noise_scale: 0.68,
            noise_w: 0.8,
            pitch_shift: 0.1,
            energy: 1.2,
        }
    }

    pub fn sadness() -> Self {
        Self {
            speed: 0.9,
            volume: 0.85,
            length_scale: 1.08,
            noise_scale: 0.65,
            noise_w: 0.75,
            pitch_shift: -0.05,
            energy: 0.8,
        }
    }

    pub fn anger() -> Self {
        Self {
            speed: 1.15,
            volume: 1.15,
            length_scale: 0.88,
            noise_scale: 0.72,
            noise_w: 0.85,
            pitch_shift: -0.03,
            energy: 1.4,
        }
    }

    pub fn fear() -> Self {
        Self {
            speed: 1.2,
            volume: 0.9,
            length_scale: 0.9,
            noise_scale: 0.75,
            noise_w: 0.85,
            pitch_shift: 0.15,
            energy: 0.9,
        }
    }

    pub fn love() -> Self {
        Self {
            speed: 0.95,
            volume: 0.95,
            length_scale: 1.02,
            noise_scale: 0.62,
            noise_w: 0.7,
            pitch_shift: 0.0,
            energy: 0.9,
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

        if let Some(&joy) = emotions.get("joy") {
            settings.speed += joy * 0.15;
            settings.volume += joy * 0.1;
            settings.length_scale -= joy * 0.08;
            settings.pitch_shift += joy * 0.1;
            settings.energy += joy * 0.2;
        }

        if let Some(&sadness) = emotions.get("sadness") {
            settings.speed -= sadness * 0.15;
            settings.volume -= sadness * 0.15;
            settings.length_scale += sadness * 0.1;
            settings.pitch_shift -= sadness * 0.05;
            settings.energy -= sadness * 0.2;
        }

        if let Some(&anger) = emotions.get("anger") {
            settings.speed += anger * 0.2;
            settings.volume += anger * 0.15;
            settings.noise_scale += anger * 0.05;
            settings.energy += anger * 0.4;
        }

        if let Some(&fear) = emotions.get("fear") {
            settings.speed += fear * 0.2;
            settings.noise_scale += fear * 0.08;
            settings.pitch_shift += fear * 0.15;
        }

        if let Some(&love) = emotions.get("love") {
            settings.speed -= love * 0.05;
            settings.noise_scale -= love * 0.05;
        }

        // Clamp értékek
        settings.speed = settings.speed.clamp(0.7, 1.4);
        settings.volume = settings.volume.clamp(0.5, 1.2);
        settings.length_scale = settings.length_scale.clamp(0.85, 1.15);
        settings.noise_scale = settings.noise_scale.clamp(0.5, 0.8);
        settings.pitch_shift = settings.pitch_shift.clamp(-0.2, 0.2);
        settings.energy = settings.energy.clamp(0.5, 1.5);

        settings
    }
}

// ============================================================================
// AUDIO CHUNKS & STREAMING
// ============================================================================

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
    /// Időbélyeg (ms)
    pub timestamp_ms: u64,
    /// Van-e beszéd (VAD)
    pub is_speech: bool,
    /// Energia szint (0.0 - 1.0)
    pub energy_level: f64,
}

impl AudioChunk {
    pub fn new(data: Vec<u8>, sequence: u32) -> Self {
        let energy_level = Self::calculate_energy(&data);
        Self {
            data,
            sequence,
            is_final: false,
            format: "wav".to_string(),
            sample_rate: 22050,
            timestamp_ms: 0,
            is_speech: energy_level > 0.01,
            energy_level,
        }
    }

    /// Energia szint számítás (RMS)
    pub fn calculate_energy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let samples: Vec<i16> = data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(i16::from_le_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();

        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
        let rms = (sum_squares / samples.len() as f64).sqrt();
        (rms / 32768.0).min(1.0)
    }
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
    /// Időbélyeg (ms)
    pub timestamp_ms: u64,
    /// VAD: beszéd aktív
    pub speech_active: bool,
}

/// Stream státusz
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamStatus {
    Idle,
    Starting,
    Active,
    Paused,
    Stopping,
    Error,
}

impl Default for StreamStatus {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for StreamStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamStatus::Idle => write!(f, "idle"),
            StreamStatus::Starting => write!(f, "starting"),
            StreamStatus::Active => write!(f, "active"),
            StreamStatus::Paused => write!(f, "paused"),
            StreamStatus::Stopping => write!(f, "stopping"),
            StreamStatus::Error => write!(f, "error"),
        }
    }
}

// ============================================================================
// TTS/STT REQUEST/RESPONSE
// ============================================================================

/// TTS kérés
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpeakRequest {
    pub text: String,
    pub voice: String,
    pub emotion: String,
    pub emotions_21d: Option<HashMap<String, f64>>,
    pub prosody: Option<ProsodySettings>,
    pub format: String,
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
    pub audio: Vec<u8>,
    pub format: String,
    pub sample_rate: u32,
    pub duration_ms: u64,
}

/// STT kérés
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListenRequest {
    pub language: String,
    pub vad_enabled: bool,
    pub model: String,
    pub word_timestamps: bool,
}

impl Default for ListenRequest {
    fn default() -> Self {
        Self {
            language: "hu".to_string(),
            vad_enabled: true,
            model: "small".to_string(),
            word_timestamps: false,
        }
    }
}

/// Transzkripció válasz
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub text: String,
    pub language: String,
    pub confidence: f64,
    pub duration_ms: u64,
    pub voice_signature: Option<VoiceSignature>,
}

// ============================================================================
// VOICE CLONE
// ============================================================================

/// Voice Clone info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceClone {
    pub id: Uuid,
    pub name: String,
    pub quality_score: f64,
    pub sample_count: u32,
    pub created_at: DateTime<Utc>,
    pub signature: VoiceSignature,
}

// ============================================================================
// VOICE STATS
// ============================================================================

/// Voice Engine Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStats {
    pub total_speak_calls: u64,
    pub total_listen_calls: u64,
    pub total_streams: u64,
    pub active_streams: u32,
    pub total_audio_generated_ms: u64,
    pub total_audio_transcribed_ms: u64,
    pub cloned_voices: u32,
    pub last_activity: Option<DateTime<Utc>>,
}

impl Default for VoiceStats {
    fn default() -> Self {
        Self {
            total_speak_calls: 0,
            total_listen_calls: 0,
            total_streams: 0,
            active_streams: 0,
            total_audio_generated_ms: 0,
            total_audio_transcribed_ms: 0,
            cloned_voices: 0,
            last_activity: None,
        }
    }
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
    /// Aktív streamek
    active_streams: Arc<RwLock<HashMap<Uuid, StreamStatus>>>,
    /// Ismert hang aláírások
    known_signatures: Arc<RwLock<HashMap<Uuid, VoiceSignature>>>,
    /// Klónozott hangok
    cloned_voices: Arc<RwLock<HashMap<String, VoiceClone>>>,
    /// Statisztikák
    stats: Arc<RwLock<VoiceStats>>,
    /// Beszél éppen
    is_speaking: Arc<RwLock<bool>>,
    /// Hallgat éppen
    is_listening: Arc<RwLock<bool>>,
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
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            voices: VoiceInfo::all_voices(),
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            known_signatures: Arc::new(RwLock::new(HashMap::new())),
            cloned_voices: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(VoiceStats::default())),
            is_speaking: Arc::new(RwLock::new(false)),
            is_listening: Arc::new(RwLock::new(false)),
        }
    }

    /// TTS URL összeállítás
    fn tts_url(&self, endpoint: &str) -> String {
        format!(
            "{}:{}{}",
            self.config.tts_url, self.config.tts_port, endpoint
        )
    }

    /// STT URL összeállítás
    fn stt_url(&self, endpoint: &str) -> String {
        format!(
            "{}:{}{}",
            self.config.stt_url, self.config.stt_port, endpoint
        )
    }

    // ==================== TTS ====================

    /// Beszéd generálás
    pub async fn speak(&self, request: SpeakRequest) -> HopeResult<SpeakResponse> {
        let voice = self.current_voice.read().await;

        // Jelzés hogy beszélünk
        {
            let mut speaking = self.is_speaking.write().await;
            *speaking = true;
        }

        // Prozódia számítás
        let prosody = if let Some(ref emotions) = request.emotions_21d {
            ProsodySettings::from_emotions_21d(emotions)
        } else {
            request
                .prosody
                .unwrap_or_else(|| ProsodySettings::from_emotion(&request.emotion))
        };

        let url = self.tts_url("/api/tts");

        let response = self
            .http_client
            .post(&url)
            .query(&[
                ("text", &request.text),
                ("voice", &voice.model_name),
                ("speed", &prosody.speed.to_string()),
            ])
            .send()
            .await
            .map_err(|e| format!("TTS hiba: {}", e))?;

        // Beszéd vége
        {
            let mut speaking = self.is_speaking.write().await;
            *speaking = false;
        }

        if !response.status().is_success() {
            return Err(format!("TTS szerver hiba: {}", response.status()).into());
        }

        let audio = response
            .bytes()
            .await
            .map_err(|e| format!("Audio letöltés hiba: {}", e))?
            .to_vec();

        let duration_ms = estimate_audio_duration(&audio, request.sample_rate);

        // Statisztika frissítés
        {
            let mut stats = self.stats.write().await;
            stats.total_speak_calls += 1;
            stats.total_audio_generated_ms += duration_ms;
            stats.last_activity = Some(Utc::now());
        }

        Ok(SpeakResponse {
            audio,
            format: request.format,
            sample_rate: request.sample_rate,
            duration_ms,
        })
    }

    /// Beszéd generálás streaming módon
    pub async fn speak_stream(&self, request: SpeakRequest) -> HopeResult<Vec<AudioChunk>> {
        let response = self.speak(request).await?;

        Ok(vec![AudioChunk {
            data: response.audio,
            sequence: 0,
            is_final: true,
            format: response.format,
            sample_rate: response.sample_rate,
            timestamp_ms: 0,
            is_speech: true,
            energy_level: 1.0,
        }])
    }

    // ==================== STT ====================

    /// Transzkripció
    pub async fn transcribe(
        &self,
        audio_data: Vec<u8>,
        request: ListenRequest,
    ) -> HopeResult<TranscriptionResponse> {
        // Jelzés hogy hallgatunk
        {
            let mut listening = self.is_listening.write().await;
            *listening = true;
        }

        let url = self.stt_url("/transcribe");

        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(audio_data.clone())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .unwrap(),
            )
            .text("language", request.language.clone())
            .text("model", request.model);

        let response = self
            .http_client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("STT hiba: {}", e))?;

        // Hallgatás vége
        {
            let mut listening = self.is_listening.write().await;
            *listening = false;
        }

        if !response.status().is_success() {
            return Err(format!("STT szerver hiba: {}", response.status()).into());
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse hiba: {}", e))?;

        let text = result["text"].as_str().unwrap_or("").to_string();
        let confidence = result["confidence"].as_f64().unwrap_or(0.9);
        let duration_ms = estimate_audio_duration(&audio_data, self.config.sample_rate);

        // Statisztika frissítés
        {
            let mut stats = self.stats.write().await;
            stats.total_listen_calls += 1;
            stats.total_audio_transcribed_ms += duration_ms;
            stats.last_activity = Some(Utc::now());
        }

        Ok(TranscriptionResponse {
            text,
            language: request.language,
            confidence,
            duration_ms,
            voice_signature: None,
        })
    }

    // ==================== VOICE SIGNATURE ====================

    /// Hang aláírás elemzés
    pub async fn analyze_voice(&self, audio_data: &[u8]) -> VoiceSignature {
        let energy = AudioChunk::calculate_energy(audio_data);

        VoiceSignature {
            id: Uuid::new_v4(),
            pitch_mean: 150.0 + (energy * 100.0),
            pitch_variance: 20.0 + (energy * 10.0),
            speaking_rate: 120.0 + (energy * 30.0),
            pause_pattern: vec![0.5, 0.3, 0.7],
            formant_frequencies: [500.0, 1500.0, 2500.0, 3500.0],
            spectral_envelope: vec![0.5; 10],
            energy_contour: vec![energy; 5],
            jitter: 0.01,
            shimmer: 0.03,
            hnr: 20.0,
            created_at: Utc::now(),
            sample_count: 1,
        }
    }

    /// Hang verifikáció (Resonance integráció)
    pub async fn verify_voice(&self, audio_data: &[u8], known_signature: &VoiceSignature) -> f64 {
        let current_signature = self.analyze_voice(audio_data).await;
        known_signature.similarity(&current_signature)
    }

    /// Hang aláírás regisztrálása
    pub async fn register_voice_signature(&self, signature: VoiceSignature) -> Uuid {
        let id = signature.id;
        let mut signatures = self.known_signatures.write().await;
        signatures.insert(id, signature);
        id
    }

    /// Ismert hang aláírások
    pub async fn get_known_signatures(&self) -> Vec<VoiceSignature> {
        self.known_signatures
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    // ==================== VOICE CLONE ====================

    /// Hang klónozás
    pub async fn clone_voice(
        &self,
        name: &str,
        audio_samples: &[Vec<u8>],
    ) -> Result<VoiceClone, String> {
        if audio_samples.is_empty() {
            return Err("At least one audio sample required".to_string());
        }

        let mut combined_signature = VoiceSignature::new();
        for sample in audio_samples {
            let sig = self.analyze_voice(sample).await;
            combined_signature.update(&sig);
        }

        let clone = VoiceClone {
            id: Uuid::new_v4(),
            name: name.to_string(),
            quality_score: (audio_samples.len() as f64 / 10.0).min(1.0),
            sample_count: audio_samples.len() as u32,
            created_at: Utc::now(),
            signature: combined_signature,
        };

        {
            let mut clones = self.cloned_voices.write().await;
            clones.insert(name.to_string(), clone.clone());
        }

        {
            let mut stats = self.stats.write().await;
            stats.cloned_voices += 1;
        }

        Ok(clone)
    }

    /// Klónozott hangok listázása
    pub async fn list_cloned_voices(&self) -> Vec<VoiceClone> {
        self.cloned_voices.read().await.values().cloned().collect()
    }

    // ==================== STREAMING ====================

    /// Stream indítása
    pub async fn start_stream(&self) -> Result<Uuid, String> {
        let stream_id = Uuid::new_v4();

        {
            let mut streams = self.active_streams.write().await;
            streams.insert(stream_id, StreamStatus::Active);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_streams += 1;
            stats.active_streams += 1;
            stats.last_activity = Some(Utc::now());
        }

        Ok(stream_id)
    }

    /// Stream leállítása
    pub async fn stop_stream(&self, stream_id: Uuid) -> Result<(), String> {
        let mut streams = self.active_streams.write().await;

        if streams.remove(&stream_id).is_some() {
            let mut stats = self.stats.write().await;
            if stats.active_streams > 0 {
                stats.active_streams -= 1;
            }
            Ok(())
        } else {
            Err("Stream not found".to_string())
        }
    }

    /// Stream státusz
    pub async fn get_stream_status(&self, stream_id: Uuid) -> Option<StreamStatus> {
        self.active_streams.read().await.get(&stream_id).copied()
    }

    // ==================== SETTINGS ====================

    /// Hang beállítása
    pub async fn set_voice(&self, voice_id: &str) -> HopeResult<()> {
        let voice = self
            .voices
            .iter()
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
        self.voices
            .iter()
            .filter(|v| v.language.starts_with("hu"))
            .collect()
    }

    /// Angol hangok
    pub fn english_voices(&self) -> Vec<&VoiceInfo> {
        self.voices
            .iter()
            .filter(|v| v.language.starts_with("en"))
            .collect()
    }

    /// Beszél-e éppen
    pub async fn is_speaking(&self) -> bool {
        *self.is_speaking.read().await
    }

    /// Hallgat-e éppen
    pub async fn is_listening(&self) -> bool {
        *self.is_listening.read().await
    }

    /// Statisztikák
    pub async fn get_stats(&self) -> VoiceStats {
        self.stats.read().await.clone()
    }

    /// Konfiguráció
    pub fn get_config(&self) -> &VoiceConfig {
        &self.config
    }

    /// VAD - Voice Activity Detection
    pub fn detect_speech(&self, audio_data: &[u8]) -> bool {
        AudioChunk::calculate_energy(audio_data) > 0.01
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
        let stats = self.stats.read().await;
        let tts_ok = self.tts_available().await;
        let stt_ok = self.stt_available().await;

        format!(
            "🎤 Hope Voice\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             🗣️  Aktuális hang: {} ({})\n\
             😊 Érzelem: {}\n\
             📢 TTS: {} (port {})\n\
             👂 STT: {} (port {})\n\
             📊 Statisztika:\n\
             - TTS hívások: {}\n\
             - STT hívások: {}\n\
             - Aktív streamek: {}\n\
             - Klónozott hangok: {}",
            voice.name,
            voice.language,
            emotion,
            if tts_ok { "✅" } else { "❌" },
            self.config.tts_port,
            if stt_ok { "✅" } else { "❌" },
            self.config.stt_port,
            stats.total_speak_calls,
            stats.total_listen_calls,
            stats.active_streams,
            stats.cloned_voices,
        )
    }

    /// Aktuális hang neve
    pub async fn get_current_voice(&self) -> String {
        let voice = self.current_voice.read().await;
        voice.id.clone()
    }

    /// Aktuális érzelem
    pub async fn get_current_emotion(&self) -> String {
        self.current_emotion.read().await.clone()
    }

    /// 21D érzelmek
    pub async fn get_emotions_21d(&self) -> HashMap<String, f64> {
        self.emotions_21d.read().await.clone()
    }

    /// TTS port
    pub fn get_tts_port(&self) -> u16 {
        self.config.tts_port
    }

    /// STT port
    pub fn get_stt_port(&self) -> u16 {
        self.config.stt_port
    }
}

impl Default for HopeVoice {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Audio időtartam becslés
fn estimate_audio_duration(audio_data: &[u8], sample_rate: u32) -> u64 {
    if audio_data.len() > 44 && &audio_data[0..4] == b"RIFF" {
        if let Some(data_size) = audio_data.get(40..44) {
            let size = u32::from_le_bytes([data_size[0], data_size[1], data_size[2], data_size[3]]);
            let samples = size / 2;
            return (samples as u64 * 1000) / sample_rate as u64;
        }
    }

    let samples = audio_data.len() / 2;
    (samples as u64 * 1000) / sample_rate as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert_eq!(config.tts_engine, TTSEngine::Piper);
        assert_eq!(config.stt_engine, STTEngine::Whisper);
        assert_eq!(config.default_language, "hu");
    }

    #[test]
    fn test_tts_engine_from_str() {
        assert_eq!(TTSEngine::from_str("piper"), TTSEngine::Piper);
        assert_eq!(TTSEngine::from_str("edge"), TTSEngine::Edge);
        assert_eq!(TTSEngine::from_str("unknown"), TTSEngine::Piper);
    }

    #[test]
    fn test_stt_engine_from_str() {
        assert_eq!(STTEngine::from_str("whisper"), STTEngine::Whisper);
        assert_eq!(STTEngine::from_str("vosk"), STTEngine::Vosk);
        assert_eq!(STTEngine::from_str("unknown"), STTEngine::Whisper);
    }

    #[test]
    fn test_voice_signature_new() {
        let sig = VoiceSignature::new();
        assert_eq!(sig.sample_count, 0);
        assert_eq!(sig.pitch_mean, 0.0);
    }

    #[test]
    fn test_voice_signature_similarity() {
        let sig1 = VoiceSignature {
            pitch_mean: 150.0,
            formant_frequencies: [500.0, 1500.0, 2500.0, 3500.0],
            jitter: 0.01,
            shimmer: 0.03,
            hnr: 20.0,
            ..VoiceSignature::default()
        };

        let sig2 = VoiceSignature {
            pitch_mean: 155.0,
            formant_frequencies: [510.0, 1510.0, 2510.0, 3510.0],
            jitter: 0.012,
            shimmer: 0.032,
            hnr: 21.0,
            ..VoiceSignature::default()
        };

        let similarity = sig1.similarity(&sig2);
        assert!(
            similarity > 0.9,
            "Similar signatures should have high similarity"
        );
    }

    #[test]
    fn test_voice_signature_update() {
        let mut sig = VoiceSignature::new();
        sig.pitch_mean = 100.0;
        sig.sample_count = 1;

        let new_sample = VoiceSignature {
            pitch_mean: 200.0,
            ..VoiceSignature::default()
        };

        sig.update(&new_sample);
        assert_eq!(sig.sample_count, 2);
        assert_eq!(sig.pitch_mean, 150.0);
    }

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

        let hu_voices: Vec<_> = voices
            .iter()
            .filter(|v| v.language.starts_with("hu"))
            .collect();
        assert_eq!(hu_voices.len(), 4);

        let en_voices: Vec<_> = voices
            .iter()
            .filter(|v| v.language.starts_with("en"))
            .collect();
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
        assert!(prosody.speed > 1.0);
    }

    #[test]
    fn test_audio_chunk_energy() {
        let silence = vec![0u8; 100];
        let chunk = AudioChunk::new(silence, 0);
        assert!(chunk.energy_level < 0.01);
        assert!(!chunk.is_speech);
    }

    #[test]
    fn test_hope_voice_creation() {
        let voice = HopeVoice::new();
        assert_eq!(voice.voices.len(), 10);
    }

    #[tokio::test]
    async fn test_voice_engine_stats() {
        let engine = HopeVoice::new();
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_speak_calls, 0);
        assert_eq!(stats.total_listen_calls, 0);
    }

    #[tokio::test]
    async fn test_voice_stream() {
        let engine = HopeVoice::new();

        let stream_id = engine.start_stream().await.unwrap();
        assert!(engine.get_stream_status(stream_id).await.is_some());

        let stats = engine.get_stats().await;
        assert_eq!(stats.active_streams, 1);

        engine.stop_stream(stream_id).await.unwrap();
        assert!(engine.get_stream_status(stream_id).await.is_none());
    }

    #[tokio::test]
    async fn test_analyze_voice() {
        let engine = HopeVoice::new();
        let audio_data = vec![0u8; 1000];

        let signature = engine.analyze_voice(&audio_data).await;
        assert!(signature.pitch_mean > 0.0);
    }

    #[tokio::test]
    async fn test_register_voice_signature() {
        let engine = HopeVoice::new();
        let signature = VoiceSignature::new();

        let id = engine.register_voice_signature(signature).await;
        let signatures = engine.get_known_signatures().await;

        assert_eq!(signatures.len(), 1);
        assert_eq!(signatures[0].id, id);
    }

    #[tokio::test]
    async fn test_clone_voice() {
        let engine = HopeVoice::new();
        let samples = vec![vec![0u8; 1000], vec![0u8; 1000]];

        let clone = engine.clone_voice("test_voice", &samples).await.unwrap();
        assert_eq!(clone.name, "test_voice");
        assert_eq!(clone.sample_count, 2);

        let clones = engine.list_cloned_voices().await;
        assert_eq!(clones.len(), 1);
    }

    #[test]
    fn test_detect_speech() {
        let engine = HopeVoice::new();

        let silence = vec![0u8; 100];
        assert!(!engine.detect_speech(&silence));
    }

    #[test]
    fn test_stream_status_display() {
        assert_eq!(StreamStatus::Idle.to_string(), "idle");
        assert_eq!(StreamStatus::Active.to_string(), "active");
    }
}
