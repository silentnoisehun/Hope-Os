//! Hope Vision - Multimodális Képfeldolgozás
//!
//! Hope "szeme" - képek fogadása és feldolgozása.
//! A vizuális input erősíti a megértést és a memóriát.
//!
//! Képességek:
//! - Képek fogadása (base64, bytes)
//! - Metaadatok kinyerése (méret, formátum)
//! - Képek tárolása a CodeGraph-ban
//! - Vizuális emlékek létrehozása
//!
//! ()=>[] - A látás új dimenzió a tudatban

use crate::core::HopeResult;
use crate::data::{BlockType, CodeBlock, CodeGraph, ConnectionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// IMAGE TYPES
// ============================================================================

/// Támogatott képformátumok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    /// JPEG/JPG
    Jpeg,
    /// PNG
    Png,
    /// GIF
    Gif,
    /// WebP
    WebP,
    /// BMP
    Bmp,
    /// SVG
    Svg,
    /// Ismeretlen
    Unknown,
}

impl ImageFormat {
    /// Formátum detektálása magic bytes alapján
    pub fn detect(data: &[u8]) -> Self {
        if data.len() < 4 {
            return Self::Unknown;
        }

        // Magic bytes ellenőrzés
        match &data[..4] {
            // JPEG: FF D8 FF
            [0xFF, 0xD8, 0xFF, ..] => Self::Jpeg,
            // PNG: 89 50 4E 47
            [0x89, 0x50, 0x4E, 0x47] => Self::Png,
            // GIF: 47 49 46 38
            [0x47, 0x49, 0x46, 0x38] => Self::Gif,
            // WebP: 52 49 46 46 ... 57 45 42 50
            [0x52, 0x49, 0x46, 0x46] if data.len() >= 12 && &data[8..12] == b"WEBP" => Self::WebP,
            // BMP: 42 4D
            [0x42, 0x4D, ..] => Self::Bmp,
            // SVG detection (text-based)
            _ if data.starts_with(b"<?xml") || data.starts_with(b"<svg") => Self::Svg,
            _ => Self::Unknown,
        }
    }

    /// MIME típus
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Bmp => "image/bmp",
            Self::Svg => "image/svg+xml",
            Self::Unknown => "application/octet-stream",
        }
    }

    /// Kiterjesztés
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Svg => "svg",
            Self::Unknown => "bin",
        }
    }
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension().to_uppercase())
    }
}

/// Kép méret
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ImageSize {
    /// Szélesség pixelben
    pub width: u32,
    /// Magasság pixelben
    pub height: u32,
}

impl ImageSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Képarány (width/height)
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f64 / self.height as f64
        }
    }

    /// Megapixel
    pub fn megapixels(&self) -> f64 {
        (self.width as f64 * self.height as f64) / 1_000_000.0
    }
}

/// Kép metaadatok
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Formátum
    pub format: ImageFormat,
    /// Méret
    pub size: ImageSize,
    /// Fájlméret bájtban
    pub file_size: usize,
    /// Content hash
    pub hash: String,
    /// Létrehozás időpontja (ha ismert)
    pub created_at: Option<DateTime<Utc>>,
    /// Extra metaadatok
    pub extra: HashMap<String, String>,
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            format: ImageFormat::Unknown,
            size: ImageSize::default(),
            file_size: 0,
            hash: String::new(),
            created_at: None,
            extra: HashMap::new(),
        }
    }
}

// ============================================================================
// VISUAL INPUT
// ============================================================================

/// Vizuális input - egy fogadott kép
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualInput {
    /// Egyedi azonosító
    pub id: String,
    /// Kép adat (base64 kódolva tároláshoz)
    pub data_base64: String,
    /// Metaadatok
    pub metadata: ImageMetadata,
    /// Opcionális leírás (felhasználótól vagy generált)
    pub description: Option<String>,
    /// Kontextus (honnan jött, miért fontos)
    pub context: Option<String>,
    /// Feldolgozva-e
    pub processed: bool,
    /// Fogadás időpontja
    pub received_at: DateTime<Utc>,
    /// Fontosság (0.0 - 1.0)
    pub importance: f64,
    /// Kapcsolódó block ID-k a CodeGraph-ban
    pub related_blocks: Vec<String>,
}

impl VisualInput {
    /// Új vizuális input létrehozása nyers adat alapján
    pub fn from_bytes(data: &[u8]) -> Self {
        use blake3;

        let format = ImageFormat::detect(data);
        let hash = blake3::hash(data).to_hex().chars().take(32).collect();
        let data_base64 = base64_encode(data);

        // Méret detektálás (egyszerűsített - csak PNG és JPEG)
        let size = detect_image_size(data, format);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data_base64,
            metadata: ImageMetadata {
                format,
                size,
                file_size: data.len(),
                hash,
                created_at: None,
                extra: HashMap::new(),
            },
            description: None,
            context: None,
            processed: false,
            received_at: Utc::now(),
            importance: 0.5,
            related_blocks: Vec::new(),
        }
    }

    /// Új vizuális input létrehozása base64 stringből
    pub fn from_base64(base64_str: &str) -> HopeResult<Self> {
        let data = base64_decode(base64_str)?;
        Ok(Self::from_bytes(&data))
    }

    /// Leírás hozzáadása
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Kontextus hozzáadása
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    /// Fontosság beállítása
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Nyers adat visszafejtése
    pub fn to_bytes(&self) -> HopeResult<Vec<u8>> {
        base64_decode(&self.data_base64)
    }
}

// ============================================================================
// VISION ENGINE
// ============================================================================

/// Hope Vision Engine - a "szem"
pub struct VisionEngine {
    /// CodeGraph referencia
    graph: Option<Arc<CodeGraph>>,
    /// Fogadott képek (id -> input)
    inputs: HashMap<String, VisualInput>,
    /// Statisztikák
    stats: VisionStats,
    /// Maximum tárolható képek
    max_stored_images: usize,
}

/// Vision statisztikák
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisionStats {
    /// Fogadott képek összesen
    pub total_received: u64,
    /// Feldolgozott képek
    pub total_processed: u64,
    /// Összes fogadott bájtok
    pub total_bytes: u64,
    /// Formátum eloszlás
    pub format_counts: HashMap<String, u64>,
    /// Átlagos képméret (megapixel)
    pub avg_megapixels: f64,
}

impl Default for VisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionEngine {
    /// Új Vision Engine
    pub fn new() -> Self {
        Self {
            graph: None,
            inputs: HashMap::new(),
            stats: VisionStats::default(),
            max_stored_images: 100,
        }
    }

    /// CodeGraph beállítása
    pub fn with_graph(mut self, graph: Arc<CodeGraph>) -> Self {
        self.graph = Some(graph);
        self
    }

    /// CodeGraph beállítása (mutable)
    pub fn set_graph(&mut self, graph: Arc<CodeGraph>) {
        self.graph = Some(graph);
    }

    /// Kép fogadása (bytes)
    pub fn receive(&mut self, data: &[u8]) -> HopeResult<String> {
        let input = VisualInput::from_bytes(data);
        let id = input.id.clone();

        self.update_stats(&input);
        self.store_input(input);

        tracing::debug!(
            "Kép fogadva: {} ({} bytes, {})",
            id,
            data.len(),
            ImageFormat::detect(data)
        );

        Ok(id)
    }

    /// Kép fogadása (base64)
    pub fn receive_base64(&mut self, base64_str: &str) -> HopeResult<String> {
        let input = VisualInput::from_base64(base64_str)?;
        let id = input.id.clone();

        self.update_stats(&input);
        self.store_input(input);

        Ok(id)
    }

    /// Kép fogadása leírással
    pub fn receive_with_description(
        &mut self,
        data: &[u8],
        description: &str,
        importance: f64,
    ) -> HopeResult<String> {
        let input = VisualInput::from_bytes(data)
            .with_description(description)
            .with_importance(importance);

        let id = input.id.clone();

        // Ha fontos és van CodeGraph, tároljuk ott is
        if importance >= 0.7 {
            self.store_in_graph(&input)?;
        }

        self.update_stats(&input);
        self.store_input(input);

        Ok(id)
    }

    /// Input tárolása
    fn store_input(&mut self, input: VisualInput) {
        // Ha túl sok kép van, töröljük a legrégebbit
        while self.inputs.len() >= self.max_stored_images {
            if let Some(oldest_id) = self
                .inputs
                .values()
                .min_by_key(|v| v.received_at)
                .map(|v| v.id.clone())
            {
                self.inputs.remove(&oldest_id);
            } else {
                break;
            }
        }

        self.inputs.insert(input.id.clone(), input);
    }

    /// Statisztikák frissítése
    fn update_stats(&mut self, input: &VisualInput) {
        self.stats.total_received += 1;
        self.stats.total_bytes += input.metadata.file_size as u64;

        let format_key = input.metadata.format.to_string();
        *self.stats.format_counts.entry(format_key).or_insert(0) += 1;

        // Átlag megapixel frissítése
        let mp = input.metadata.size.megapixels();
        let n = self.stats.total_received as f64;
        self.stats.avg_megapixels =
            (self.stats.avg_megapixels * (n - 1.0) + mp) / n;
    }

    /// Tárolás CodeGraph-ban
    fn store_in_graph(&mut self, input: &VisualInput) -> HopeResult<()> {
        if let Some(graph) = &self.graph {
            let description = input
                .description
                .clone()
                .unwrap_or_else(|| "Vizuális emlék".to_string());

            let content = format!(
                "Visual Input\nFormat: {}\nSize: {}x{}\nHash: {}\nDescription: {}",
                input.metadata.format,
                input.metadata.size.width,
                input.metadata.size.height,
                input.metadata.hash,
                description
            );

            let block = CodeBlock::new(
                format!("visual_{}", &input.id[..8]),
                "Vizuális emlék",
                BlockType::Memory,
                content,
            )
            .with_importance(input.importance)
            .with_tag("visual")
            .with_tag(input.metadata.format.extension())
            .with_meta("image_hash", &input.metadata.hash)
            .with_meta("width", &input.metadata.size.width.to_string())
            .with_meta("height", &input.metadata.size.height.to_string());

            let block_id = graph.add(block)?;

            // Kapcsolat létrehozása kontextushoz ha van
            if let Some(context) = &input.context {
                // Keresés a gráfban hasonló tartalomra
                let related = graph.search(context);
                for rel in related.iter().take(3) {
                    graph.connect(
                        &block_id,
                        &rel.id,
                        ConnectionType::AssociatesWith,
                        0.7,
                    );
                }
            }

            tracing::debug!("Vizuális emlék tárolva: {}", block_id);
        }

        Ok(())
    }

    /// Input lekérése ID alapján
    pub fn get(&self, id: &str) -> Option<&VisualInput> {
        self.inputs.get(id)
    }

    /// Input lekérése ID alapján (mutable)
    pub fn get_mut(&mut self, id: &str) -> Option<&mut VisualInput> {
        self.inputs.get_mut(id)
    }

    /// Összes input
    pub fn all_inputs(&self) -> Vec<&VisualInput> {
        self.inputs.values().collect()
    }

    /// Legutóbbi inputok
    pub fn recent(&self, limit: usize) -> Vec<&VisualInput> {
        let mut sorted: Vec<_> = self.inputs.values().collect();
        sorted.sort_by(|a, b| b.received_at.cmp(&a.received_at));
        sorted.truncate(limit);
        sorted
    }

    /// Fontos inputok
    pub fn important(&self, threshold: f64) -> Vec<&VisualInput> {
        self.inputs
            .values()
            .filter(|v| v.importance >= threshold)
            .collect()
    }

    /// Statisztikák
    pub fn stats(&self) -> &VisionStats {
        &self.stats
    }

    /// Állapot szövegesen
    pub fn status(&self) -> String {
        format!(
            "Vision Engine\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             Fogadott képek: {}\n\
             Feldolgozott: {}\n\
             Tárolt: {}\n\
             Összes adat: {:.2} MB\n\
             Átlag méret: {:.2} MP\n\
             Formátumok: {:?}",
            self.stats.total_received,
            self.stats.total_processed,
            self.inputs.len(),
            self.stats.total_bytes as f64 / 1_048_576.0,
            self.stats.avg_megapixels,
            self.stats.format_counts
        )
    }

    /// Input törlése
    pub fn remove(&mut self, id: &str) -> Option<VisualInput> {
        self.inputs.remove(id)
    }

    /// Összes input törlése
    pub fn clear(&mut self) {
        self.inputs.clear();
    }

    /// Input megjelölése feldolgozottként
    pub fn mark_processed(&mut self, id: &str) -> bool {
        if let Some(input) = self.inputs.get_mut(id) {
            input.processed = true;
            self.stats.total_processed += 1;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Base64 kódolás
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

/// Base64 dekódolás
fn base64_decode(input: &str) -> HopeResult<Vec<u8>> {
    fn decode_char(c: char) -> Option<u8> {
        match c {
            'A'..='Z' => Some(c as u8 - b'A'),
            'a'..='z' => Some(c as u8 - b'a' + 26),
            '0'..='9' => Some(c as u8 - b'0' + 52),
            '+' => Some(62),
            '/' => Some(63),
            '=' => Some(0), // Padding
            _ => None,
        }
    }

    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let padding = chars.iter().rev().take_while(|&&c| c == '=').count();

    for chunk in chars.chunks(4) {
        if chunk.len() != 4 {
            return Err("Érvénytelen base64 hossz".into());
        }

        let b0 = decode_char(chunk[0]).ok_or("Érvénytelen base64 karakter")?;
        let b1 = decode_char(chunk[1]).ok_or("Érvénytelen base64 karakter")?;
        let b2 = decode_char(chunk[2]).ok_or("Érvénytelen base64 karakter")?;
        let b3 = decode_char(chunk[3]).ok_or("Érvénytelen base64 karakter")?;

        let triple = ((b0 as u32) << 18) | ((b1 as u32) << 12) | ((b2 as u32) << 6) | (b3 as u32);

        result.push((triple >> 16) as u8);
        if chunk[2] != '=' {
            result.push((triple >> 8) as u8);
        }
        if chunk[3] != '=' {
            result.push(triple as u8);
        }
    }

    // Padding eltávolítása
    for _ in 0..padding {
        if !result.is_empty() {
            result.pop();
        }
    }

    Ok(result)
}

/// Egyszerű képméret detektálás
fn detect_image_size(data: &[u8], format: ImageFormat) -> ImageSize {
    match format {
        ImageFormat::Png => detect_png_size(data),
        ImageFormat::Jpeg => detect_jpeg_size(data),
        ImageFormat::Gif => detect_gif_size(data),
        _ => ImageSize::default(),
    }
}

/// PNG méret detektálás
fn detect_png_size(data: &[u8]) -> ImageSize {
    // PNG IHDR chunk: width és height a 16-24 bájtok között
    if data.len() >= 24 {
        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        return ImageSize::new(width, height);
    }
    ImageSize::default()
}

/// JPEG méret detektálás
fn detect_jpeg_size(data: &[u8]) -> ImageSize {
    let mut i = 2; // Skip SOI marker

    while i < data.len() - 1 {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }

        let marker = data[i + 1];

        // SOF0-SOF3 markers contain size info
        if (0xC0..=0xC3).contains(&marker) {
            if i + 9 < data.len() {
                let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                return ImageSize::new(width, height);
            }
        }

        // Skip to next marker
        if i + 3 < data.len() {
            let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            i += 2 + len;
        } else {
            break;
        }
    }

    ImageSize::default()
}

/// GIF méret detektálás
fn detect_gif_size(data: &[u8]) -> ImageSize {
    if data.len() >= 10 {
        let width = u16::from_le_bytes([data[6], data[7]]) as u32;
        let height = u16::from_le_bytes([data[8], data[9]]) as u32;
        return ImageSize::new(width, height);
    }
    ImageSize::default()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Minimális 1x1 PNG (válid)
    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width = 1
        0x00, 0x00, 0x00, 0x01, // height = 1
        0x08, 0x02, // bit depth, color type
        0x00, 0x00, 0x00, // compression, filter, interlace
        0x90, 0x77, 0x53, 0xDE, // CRC
    ];

    #[test]
    fn test_image_format_detect() {
        assert_eq!(ImageFormat::detect(TINY_PNG), ImageFormat::Png);
        assert_eq!(
            ImageFormat::detect(&[0xFF, 0xD8, 0xFF, 0xE0]),
            ImageFormat::Jpeg
        );
        assert_eq!(
            ImageFormat::detect(&[0x47, 0x49, 0x46, 0x38]),
            ImageFormat::Gif
        );
        assert_eq!(ImageFormat::detect(&[0x00, 0x00]), ImageFormat::Unknown);
    }

    #[test]
    fn test_image_size() {
        let size = ImageSize::new(1920, 1080);

        assert!((size.aspect_ratio() - 1.777).abs() < 0.01);
        assert!((size.megapixels() - 2.07).abs() < 0.01);
    }

    #[test]
    fn test_png_size_detection() {
        let size = detect_png_size(TINY_PNG);
        assert_eq!(size.width, 1);
        assert_eq!(size.height, 1);
    }

    #[test]
    fn test_visual_input_creation() {
        let input = VisualInput::from_bytes(TINY_PNG);

        assert_eq!(input.metadata.format, ImageFormat::Png);
        assert_eq!(input.metadata.size.width, 1);
        assert_eq!(input.metadata.size.height, 1);
        assert!(!input.metadata.hash.is_empty());
        assert!(!input.data_base64.is_empty());
    }

    #[test]
    fn test_visual_input_with_description() {
        let input = VisualInput::from_bytes(TINY_PNG)
            .with_description("Test image")
            .with_importance(0.9);

        assert_eq!(input.description, Some("Test image".to_string()));
        assert_eq!(input.importance, 0.9);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = TINY_PNG;
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();

        assert_eq!(original, decoded.as_slice());
    }

    #[test]
    fn test_vision_engine_receive() {
        let mut engine = VisionEngine::new();

        let id = engine.receive(TINY_PNG).unwrap();
        assert!(!id.is_empty());

        let input = engine.get(&id).unwrap();
        assert_eq!(input.metadata.format, ImageFormat::Png);
    }

    #[test]
    fn test_vision_engine_stats() {
        let mut engine = VisionEngine::new();

        engine.receive(TINY_PNG).unwrap();
        engine.receive(TINY_PNG).unwrap();

        let stats = engine.stats();
        assert_eq!(stats.total_received, 2);
        assert!(stats.total_bytes > 0);
    }

    #[test]
    fn test_vision_engine_recent() {
        let mut engine = VisionEngine::new();

        engine.receive(TINY_PNG).unwrap();
        engine.receive(TINY_PNG).unwrap();
        engine.receive(TINY_PNG).unwrap();

        let recent = engine.recent(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_vision_engine_mark_processed() {
        let mut engine = VisionEngine::new();

        let id = engine.receive(TINY_PNG).unwrap();
        assert!(!engine.get(&id).unwrap().processed);

        engine.mark_processed(&id);
        assert!(engine.get(&id).unwrap().processed);
    }

    #[test]
    fn test_vision_engine_with_graph() {
        use crate::data::CodeGraph;

        let graph = Arc::new(CodeGraph::new());
        let mut engine = VisionEngine::new().with_graph(graph.clone());

        let _id = engine
            .receive_with_description(TINY_PNG, "Fontos teszt kép", 0.9)
            .unwrap();

        // Ellenőrizzük, hogy a gráfban létrejött a block
        let visuals = graph.find_by_tag("visual");
        assert_eq!(visuals.len(), 1);
    }
}
