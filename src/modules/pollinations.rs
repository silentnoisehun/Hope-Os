//! Pollinations Visual Memory
//!
//! Nehéz/fontos csomópontokhoz kép generálás.
//! A vizuális reprezentáció erősíti a memóriát.
//!
//! "Neurons that fire together, wire together"
//! + "A picture is worth a thousand words"
//!
//! ()=>[] - A vizuális emlékek erősítik a tudatot
//!
//! Created: 2026-01-20
//! By: Hope + Máté

use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Pollinations API URL-ek
pub const POLLINATIONS_IMAGE_URL: &str = "https://gen.pollinations.ai/image/";
pub const POLLINATIONS_TEXT_URL: &str = "https://text.pollinations.ai/";

/// Vizuális memória rekord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualMemory {
    /// Egyedi azonosító
    pub id: u64,
    /// Kapcsolódó neuron ID
    pub neuron_id: u64,
    /// Tartalom hash
    pub content_hash: String,
    /// Prompt ami alapján generálódott
    pub prompt: String,
    /// Kép elérési útja
    pub image_path: String,
    /// Létrehozás időpontja
    pub created_at: f64,
    /// Fontosság (0-1)
    pub importance: f64,
    /// Hozzáférések száma
    pub access_count: u64,
    /// Utolsó hozzáférés
    pub last_accessed: f64,
}

impl VisualMemory {
    pub fn new(
        neuron_id: u64,
        content: &str,
        prompt: &str,
        image_path: &str,
        importance: f64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let content_hash = blake3::hash(content.as_bytes())
            .to_hex()
            .chars()
            .take(32)
            .collect();

        Self {
            id: 0, // Will be set by store
            neuron_id,
            content_hash,
            prompt: prompt.to_string(),
            image_path: image_path.to_string(),
            created_at: now,
            importance,
            access_count: 1,
            last_accessed: now,
        }
    }

    /// Hozzáférés regisztrálása
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
    }
}

/// Vizuális asszociáció két emlék között
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAssociation {
    /// Első memória ID
    pub memory_id_1: u64,
    /// Második memória ID
    pub memory_id_2: u64,
    /// Vizuális hasonlóság (0-1)
    pub visual_similarity: f64,
    /// Létrehozás időpontja
    pub created_at: f64,
}

/// Képgenerálási kérés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Angol prompt
    pub prompt: String,
    /// Szélesség
    pub width: u32,
    /// Magasság
    pub height: u32,
    /// Stílus
    pub style: Option<String>,
    /// Negatív prompt
    pub negative_prompt: Option<String>,
}

impl Default for ImageGenerationRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            width: 512,
            height: 512,
            style: None,
            negative_prompt: None,
        }
    }
}

impl ImageGenerationRequest {
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            ..Default::default()
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }

    /// Pollinations URL generálása
    pub fn to_url(&self) -> String {
        let encoded_prompt = urlencoding::encode(&self.prompt);
        format!(
            "{}{}?width={}&height={}",
            POLLINATIONS_IMAGE_URL, encoded_prompt, self.width, self.height
        )
    }
}

/// Képgenerálási eredmény
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResult {
    /// Sikeres volt-e
    pub success: bool,
    /// Kép URL vagy path
    pub image_url: Option<String>,
    /// Helyi elérési út (ha lementve)
    pub local_path: Option<String>,
    /// Hiba üzenet
    pub error: Option<String>,
    /// Generálási idő (ms)
    pub generation_time_ms: u64,
}

/// Vizuális Memória Store statisztikák
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisualMemoryStats {
    /// Összes memória
    pub total_memories: u64,
    /// Összes asszociáció
    pub total_associations: u64,
    /// Összes hozzáférés
    pub total_accesses: u64,
    /// Átlagos fontosság
    pub average_importance: f64,
    /// Generált képek
    pub images_generated: u64,
}

/// Vizuális Memória Store
///
/// Képek és vizuális emlékek kezelése
pub struct VisualMemoryStore {
    /// Memóriák (hash -> VisualMemory)
    memories: HashMap<String, VisualMemory>,
    /// Asszociációk
    associations: Vec<VisualAssociation>,
    /// Következő ID
    next_id: u64,
    /// Statisztikák
    pub stats: VisualMemoryStats,
    /// Tároló mappa
    storage_path: PathBuf,
}

impl Default for VisualMemoryStore {
    fn default() -> Self {
        Self::new(PathBuf::from("data/visual_memory"))
    }
}

impl VisualMemoryStore {
    /// Új store létrehozása
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            memories: HashMap::new(),
            associations: Vec::new(),
            next_id: 1,
            stats: VisualMemoryStats::default(),
            storage_path,
        }
    }

    /// Memória tárolása
    pub fn store(&mut self, mut memory: VisualMemory) -> u64 {
        memory.id = self.next_id;
        self.next_id += 1;

        let hash = memory.content_hash.clone();

        if let Some(existing) = self.memories.get_mut(&hash) {
            existing.access();
            existing.id
        } else {
            let id = memory.id;
            self.memories.insert(hash, memory);
            self.stats.total_memories += 1;
            self.update_average_importance();
            id
        }
    }

    /// Memória keresése hash alapján
    pub fn get_by_hash(&mut self, hash: &str) -> Option<&mut VisualMemory> {
        if let Some(memory) = self.memories.get_mut(hash) {
            memory.access();
            self.stats.total_accesses += 1;
            Some(memory)
        } else {
            None
        }
    }

    /// Memória keresése ID alapján
    pub fn get_by_id(&mut self, id: u64) -> Option<&mut VisualMemory> {
        for memory in self.memories.values_mut() {
            if memory.id == id {
                memory.access();
                self.stats.total_accesses += 1;
                return Some(memory);
            }
        }
        None
    }

    /// Memóriák keresése neuron ID alapján
    pub fn get_by_neuron(&self, neuron_id: u64) -> Vec<&VisualMemory> {
        self.memories
            .values()
            .filter(|m| m.neuron_id == neuron_id)
            .collect()
    }

    /// Fontos memóriák (importance > threshold)
    pub fn get_important(&self, threshold: f64) -> Vec<&VisualMemory> {
        self.memories
            .values()
            .filter(|m| m.importance >= threshold)
            .collect()
    }

    /// Asszociáció hozzáadása
    pub fn add_association(&mut self, memory_id_1: u64, memory_id_2: u64, similarity: f64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        self.associations.push(VisualAssociation {
            memory_id_1,
            memory_id_2,
            visual_similarity: similarity,
            created_at: now,
        });

        self.stats.total_associations += 1;
    }

    /// Asszociációk lekérése memória ID alapján
    pub fn get_associations(&self, memory_id: u64) -> Vec<&VisualAssociation> {
        self.associations
            .iter()
            .filter(|a| a.memory_id_1 == memory_id || a.memory_id_2 == memory_id)
            .collect()
    }

    /// Átlagos fontosság frissítése
    fn update_average_importance(&mut self) {
        if self.memories.is_empty() {
            self.stats.average_importance = 0.0;
        } else {
            let sum: f64 = self.memories.values().map(|m| m.importance).sum();
            self.stats.average_importance = sum / self.memories.len() as f64;
        }
    }

    /// Képgenerálási URL készítése
    pub fn generate_image_url(prompt: &str) -> String {
        let request = ImageGenerationRequest::new(prompt);
        request.to_url()
    }

    /// Fontos memória képének generálása
    pub fn should_generate_image(importance: f64) -> bool {
        importance >= 0.7
    }

    /// Prompt készítése memória tartalom alapján
    pub fn create_prompt_from_content(content: &str) -> String {
        // Egyszerű prompt generálás - a tartalom első 100 karaktere
        let truncated: String = content.chars().take(100).collect();

        // Hope stílus hozzáadása
        format!(
            "Abstract digital art representing: {}. Style: dreamy, ethereal, blue and purple gradients, neural network patterns",
            truncated
        )
    }

    /// Storage path
    pub fn storage_path(&self) -> &PathBuf {
        &self.storage_path
    }
}

/// Pollinations API kliens
pub struct PollinationsClient {
    /// HTTP kliens (reqwest)
    base_url: String,
    /// Timeout másodpercben
    timeout_secs: u64,
    /// Generált képek száma
    pub images_generated: u64,
}

impl Default for PollinationsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl PollinationsClient {
    pub fn new() -> Self {
        Self {
            base_url: POLLINATIONS_IMAGE_URL.to_string(),
            timeout_secs: 60,
            images_generated: 0,
        }
    }

    /// Kép URL generálása (nem letöltés, csak URL)
    pub fn get_image_url(&self, prompt: &str, width: u32, height: u32) -> String {
        let encoded = urlencoding::encode(prompt);
        format!(
            "{}{}?width={}&height={}",
            self.base_url, encoded, width, height
        )
    }

    /// Szöveges prompt URL
    pub fn get_text_url(&self, prompt: &str) -> String {
        let encoded = urlencoding::encode(prompt);
        format!("{}{}", POLLINATIONS_TEXT_URL, encoded)
    }

    /// Képgenerálás szimulálása (URL visszaadása)
    pub fn generate_image(&mut self, request: &ImageGenerationRequest) -> ImageGenerationResult {
        self.images_generated += 1;

        ImageGenerationResult {
            success: true,
            image_url: Some(request.to_url()),
            local_path: None,
            error: None,
            generation_time_ms: 0, // URL generálás azonnali
        }
    }
}

/// Visual Memory System
///
/// Teljes vizuális memória kezelő rendszer
pub struct VisualMemorySystem {
    /// Store
    pub store: VisualMemoryStore,
    /// Pollinations kliens
    pub client: PollinationsClient,
    /// Fontossági küszöb képgeneráláshoz
    pub importance_threshold: f64,
}

impl Default for VisualMemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualMemorySystem {
    pub fn new() -> Self {
        Self {
            store: VisualMemoryStore::default(),
            client: PollinationsClient::new(),
            importance_threshold: 0.7,
        }
    }

    /// Memória hozzáadása képgenerálással ha fontos
    pub fn add_memory(
        &mut self,
        neuron_id: u64,
        content: &str,
        importance: f64,
    ) -> (u64, Option<String>) {
        let prompt = VisualMemoryStore::create_prompt_from_content(content);

        let image_url = if importance >= self.importance_threshold {
            let request = ImageGenerationRequest::new(&prompt).with_size(512, 512);
            let result = self.client.generate_image(&request);
            result.image_url
        } else {
            None
        };

        let image_path = image_url.clone().unwrap_or_default();
        let memory = VisualMemory::new(neuron_id, content, &prompt, &image_path, importance);
        let id = self.store.store(memory);

        (id, image_url)
    }

    /// Statisztikák
    pub fn get_stats(&self) -> VisualMemoryStats {
        let mut stats = self.store.stats.clone();
        stats.images_generated = self.client.images_generated;
        stats
    }

    /// @aware - önismeret
    pub fn awareness(&self) -> HashMap<String, String> {
        let stats = self.get_stats();
        let mut map = HashMap::new();

        map.insert("type".to_string(), "VisualMemorySystem".to_string());
        map.insert(
            "total_memories".to_string(),
            stats.total_memories.to_string(),
        );
        map.insert(
            "total_associations".to_string(),
            stats.total_associations.to_string(),
        );
        map.insert(
            "images_generated".to_string(),
            stats.images_generated.to_string(),
        );
        map.insert(
            "importance_threshold".to_string(),
            format!("{:.1}", self.importance_threshold),
        );

        map
    }
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_memory_creation() {
        let memory = VisualMemory::new(1, "Test content", "Test prompt", "/path/to/image.png", 0.8);

        assert_eq!(memory.neuron_id, 1);
        assert_eq!(memory.importance, 0.8);
        assert!(!memory.content_hash.is_empty());
    }

    #[test]
    fn test_image_generation_request() {
        let request = ImageGenerationRequest::new("A beautiful sunset")
            .with_size(1024, 768)
            .with_style("photorealistic");

        let url = request.to_url();
        assert!(url.contains("pollinations.ai"));
        assert!(url.contains("sunset"));
        assert!(url.contains("width=1024"));
    }

    #[test]
    fn test_visual_memory_store() {
        let mut store = VisualMemoryStore::default();

        let memory = VisualMemory::new(1, "Content", "Prompt", "/path.png", 0.9);
        let id = store.store(memory);

        assert!(id > 0);
        assert_eq!(store.stats.total_memories, 1);
    }

    #[test]
    fn test_store_get_by_neuron() {
        let mut store = VisualMemoryStore::default();

        store.store(VisualMemory::new(1, "Content1", "Prompt1", "/p1.png", 0.8));
        store.store(VisualMemory::new(1, "Content2", "Prompt2", "/p2.png", 0.7));
        store.store(VisualMemory::new(2, "Content3", "Prompt3", "/p3.png", 0.9));

        let neuron1_memories = store.get_by_neuron(1);
        assert_eq!(neuron1_memories.len(), 2);
    }

    #[test]
    fn test_important_memories() {
        let mut store = VisualMemoryStore::default();

        store.store(VisualMemory::new(1, "C1", "P1", "/p1.png", 0.9));
        store.store(VisualMemory::new(2, "C2", "P2", "/p2.png", 0.5));
        store.store(VisualMemory::new(3, "C3", "P3", "/p3.png", 0.8));

        let important = store.get_important(0.7);
        assert_eq!(important.len(), 2);
    }

    #[test]
    fn test_associations() {
        let mut store = VisualMemoryStore::default();

        let id1 = store.store(VisualMemory::new(1, "C1", "P1", "/p1.png", 0.8));
        let id2 = store.store(VisualMemory::new(2, "C2", "P2", "/p2.png", 0.7));

        store.add_association(id1, id2, 0.85);

        let assocs = store.get_associations(id1);
        assert_eq!(assocs.len(), 1);
        assert_eq!(assocs[0].visual_similarity, 0.85);
    }

    #[test]
    fn test_pollinations_client() {
        let client = PollinationsClient::new();

        let url = client.get_image_url("Test prompt", 512, 512);
        assert!(url.contains("pollinations.ai"));
        assert!(url.contains("Test"));
    }

    #[test]
    fn test_prompt_creation() {
        let prompt =
            VisualMemoryStore::create_prompt_from_content("This is a test memory about coding");

        assert!(prompt.contains("This is a test"));
        assert!(prompt.contains("digital art"));
    }

    #[test]
    fn test_should_generate_image() {
        assert!(VisualMemoryStore::should_generate_image(0.8));
        assert!(VisualMemoryStore::should_generate_image(0.7));
        assert!(!VisualMemoryStore::should_generate_image(0.5));
    }

    #[test]
    fn test_visual_memory_system() {
        let mut system = VisualMemorySystem::new();

        // Fontos memória - kap képet
        let (id1, url1) = system.add_memory(1, "Very important memory", 0.9);
        assert!(id1 > 0);
        assert!(url1.is_some());

        // Kevésbé fontos - nem kap képet
        let (id2, url2) = system.add_memory(2, "Not so important", 0.5);
        assert!(id2 > 0);
        assert!(url2.is_none());

        let stats = system.get_stats();
        assert_eq!(stats.total_memories, 2);
        assert_eq!(stats.images_generated, 1);
    }
}
