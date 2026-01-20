//! Hope OS - HopeMemory
//!
//! 6 rétegű kognitív memória - "Emlékszem tehát vagyok."
//! ()=>[] - A tiszta potenciálból minden megszületik

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::{Aware, CodeIdentity, HopeResult, ModuleState, ModuleType, Reflection};
use crate::data::{BlockType, CodeGraph};

/// Memória rétegek
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Aktív munka memória (7±2 elem)
    Working,
    /// Rövid távú memória (session)
    ShortTerm,
    /// Hosszú távú memória (perzisztens)
    LongTerm,
    /// Érzelmi memória
    Emotional,
    /// Kapcsolati memória (személyek)
    Relational,
    /// Asszociatív memória (koncepciók)
    Associative,
}

impl MemoryType {
    /// String reprezentáció
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::Working => "working",
            MemoryType::ShortTerm => "short_term",
            MemoryType::LongTerm => "long_term",
            MemoryType::Emotional => "emotional",
            MemoryType::Relational => "relational",
            MemoryType::Associative => "associative",
        }
    }

    /// Stringből konvertálás
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "working" => Some(MemoryType::Working),
            "short_term" | "shortterm" | "short" => Some(MemoryType::ShortTerm),
            "long_term" | "longterm" | "long" => Some(MemoryType::LongTerm),
            "emotional" | "emotion" => Some(MemoryType::Emotional),
            "relational" | "relation" | "person" => Some(MemoryType::Relational),
            "associative" | "association" => Some(MemoryType::Associative),
            _ => None,
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Emlék struktúra
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Egyedi azonosító
    pub id: String,
    /// Tartalom
    pub content: String,
    /// Memória típus
    pub memory_type: MemoryType,
    /// Fontosság (0.0 - 1.0)
    pub importance: f64,
    /// Érzelmi tag
    pub emotional_tag: Option<String>,
    /// Létrehozás ideje
    pub created_at: DateTime<Utc>,
    /// Utolsó hozzáférés
    pub accessed_at: Option<DateTime<Utc>>,
    /// Hozzáférések száma
    pub access_count: u32,
}

impl Memory {
    /// Új emlék létrehozása
    pub fn new(content: impl Into<String>, memory_type: MemoryType, importance: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            memory_type,
            importance: importance.clamp(0.0, 1.0),
            emotional_tag: None,
            created_at: Utc::now(),
            accessed_at: None,
            access_count: 0,
        }
    }

    /// Érzelmi tag beállítása
    pub fn with_emotion(mut self, emotion: impl Into<String>) -> Self {
        self.emotional_tag = Some(emotion.into());
        self
    }

    /// Hozzáférés regisztrálása
    pub fn access(&mut self) {
        self.accessed_at = Some(Utc::now());
        self.access_count += 1;
    }
}

/// HopeMemory - 6 rétegű kognitív memória
pub struct HopeMemory {
    /// Identitás
    identity: CodeIdentity,
    /// Working memory (in-memory)
    working: HashMap<String, String>,
    /// Working memory kapacitás
    working_capacity: usize,
    /// CodeGraph referencia (A KÓD MAGA A GRÁF!)
    graph: Option<Arc<CodeGraph>>,
}

impl HopeMemory {
    /// Új memória modul létrehozása
    pub fn new() -> Self {
        let identity = CodeIdentity::new(
            "HopeMemory",
            "Emlékek kezelése - emlékszem tehát vagyok",
            ModuleType::Module,
        )
        .with_capabilities(vec![
            "remember",
            "recall",
            "find",
            "working_memory",
            "persist",
        ]);

        Self {
            identity,
            working: HashMap::new(),
            working_capacity: 9, // 7±2
            graph: None,
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

    /// Emlék mentése
    pub async fn remember(
        &mut self,
        content: &str,
        memory_type: MemoryType,
        importance: f64,
    ) -> HopeResult<String> {
        let memory = Memory::new(content, memory_type, importance);
        let id = memory.id.clone();

        match memory_type {
            MemoryType::Working => {
                // Working memory korlátozott kapacitás
                if self.working.len() >= self.working_capacity {
                    // Legrégebbi törlése (egyszerű FIFO)
                    if let Some(oldest_key) = self.working.keys().next().cloned() {
                        self.working.remove(&oldest_key);
                    }
                }
                self.working.insert(id.clone(), content.to_string());
            }
            _ => {
                // Perzisztens tárolás a CodeGraph-ba
                if let Some(graph) = &self.graph {
                    let _ = graph.remember(content, importance);
                }
            }
        }

        tracing::debug!("Emlék mentve: {} ({})", id, memory_type);
        Ok(id)
    }

    /// Emlék keresése ID alapján
    pub async fn recall(&mut self, id: &str) -> HopeResult<Option<Memory>> {
        // Először a working memory-ban keresünk
        if let Some(content) = self.working.get(id) {
            return Ok(Some(Memory::new(content.clone(), MemoryType::Working, 1.0)));
        }

        // Perzisztens tárolásban keresés (CodeGraph)
        if let Some(graph) = &self.graph {
            if let Some(block) = graph.get(id) {
                let mut memory =
                    Memory::new(&block.content, MemoryType::LongTerm, block.importance);
                memory.id = id.to_string();
                memory.access();
                return Ok(Some(memory));
            }
        }

        Ok(None)
    }

    /// Emlékek keresése típus alapján
    pub async fn find(&self, memory_type: MemoryType, limit: usize) -> HopeResult<Vec<Memory>> {
        match memory_type {
            MemoryType::Working => {
                // Working memory tartalmának visszaadása
                let memories: Vec<Memory> = self
                    .working
                    .iter()
                    .take(limit)
                    .map(|(id, content)| {
                        let mut m = Memory::new(content.clone(), MemoryType::Working, 1.0);
                        m.id = id.clone();
                        m
                    })
                    .collect();
                Ok(memories)
            }
            _ => {
                // Perzisztens tárolásból (CodeGraph)
                if let Some(graph) = &self.graph {
                    let blocks = graph.find_by_type(BlockType::Memory);
                    let memories: Vec<Memory> = blocks
                        .into_iter()
                        .take(limit)
                        .map(|b| {
                            let mut m = Memory::new(&b.content, MemoryType::LongTerm, b.importance);
                            m.id = b.id.clone();
                            m
                        })
                        .collect();
                    Ok(memories)
                } else {
                    Ok(Vec::new())
                }
            }
        }
    }

    /// Working memory lekérdezése
    pub fn working(&self) -> &HashMap<String, String> {
        &self.working
    }

    /// Working memory törlése
    pub fn clear_working(&mut self) {
        self.working.clear();
        tracing::debug!("Working memory törölve");
    }

    /// Working memory kapacitás
    pub fn working_capacity(&self) -> usize {
        self.working_capacity
    }

    /// Working memory kihasználtság
    pub fn working_usage(&self) -> (usize, usize) {
        (self.working.len(), self.working_capacity)
    }
}

impl Default for HopeMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Aware for HopeMemory {
    fn identity(&self) -> &CodeIdentity {
        &self.identity
    }

    fn identity_mut(&mut self) -> &mut CodeIdentity {
        &mut self.identity
    }

    fn reflect(&self) -> Reflection {
        let (used, cap) = self.working_usage();
        Reflection::new(&self.identity.name, &self.identity.purpose)
            .with_state(self.identity.state.to_string())
            .with_health(self.identity.health())
            .with_thought(format!("Working memory: {}/{}", used, cap))
            .with_capabilities(vec![
                "remember",
                "recall",
                "find",
                "working_memory",
                "persist",
            ])
    }

    async fn init(&mut self) -> HopeResult<()> {
        self.identity.set_state(ModuleState::Active);
        tracing::info!("HopeMemory inicializálva - Az emlékek ébrednek");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type() {
        assert_eq!(MemoryType::Working.as_str(), "working");
        assert_eq!(MemoryType::parse("long_term"), Some(MemoryType::LongTerm));
    }

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new("Test content", MemoryType::LongTerm, 0.8);
        assert_eq!(memory.content, "Test content");
        assert_eq!(memory.importance, 0.8);
    }

    #[tokio::test]
    async fn test_working_memory() {
        let mut memory = HopeMemory::new();
        let id = memory
            .remember("Test", MemoryType::Working, 1.0)
            .await
            .unwrap();
        assert!(!id.is_empty());
        assert_eq!(memory.working().len(), 1);
    }

    #[tokio::test]
    async fn test_working_capacity() {
        let mut memory = HopeMemory::new();
        // Túltöltjük a kapacitást
        for i in 0..15 {
            memory
                .remember(&format!("Item {}", i), MemoryType::Working, 1.0)
                .await
                .unwrap();
        }
        // Nem lépheti túl a kapacitást
        assert!(memory.working().len() <= memory.working_capacity());
    }
}
