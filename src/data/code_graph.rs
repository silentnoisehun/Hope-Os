//! Hope OS - CodeGraph
//!
//! A kod MAGA a graf. Nincs DB, nincs kulso fugges.
//! Minden CodeBlock @aware es kapcsolodik mindenhez.
//!
//! ()=>[] - A tiszta potencialbol minden megszuletik

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::core::{Aware, CodeIdentity, HopeResult, ModuleState, ModuleType, Reflection};

// ============================================================================
// CONNECTION - Kapcsolat ket CodeBlock kozott
// ============================================================================

/// Kapcsolat tipusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Fugg tole (dependency)
    DependsOn,
    /// O fugg tolem
    DependencyOf,
    /// Kapcsolodik (altalnos)
    ConnectsTo,
    /// Triggeli
    Triggers,
    /// Triggereli ot
    TriggeredBy,
    /// Tartalmazza
    Contains,
    /// Tartalmazva van
    ContainedIn,
    /// Hivatkozik ra
    References,
    /// Hivatkoznak ra
    ReferencedBy,
    /// Orokol tole
    InheritsFrom,
    /// Oroklik tole
    InheritedBy,
    /// Asszocialodik
    AssociatesWith,
    /// Emlekszik ra
    Remembers,
    /// Emlekeznek ra
    RememberedBy,
}

impl ConnectionType {
    /// Forditott kapcsolat
    pub fn inverse(&self) -> Self {
        match self {
            Self::DependsOn => Self::DependencyOf,
            Self::DependencyOf => Self::DependsOn,
            Self::ConnectsTo => Self::ConnectsTo,
            Self::Triggers => Self::TriggeredBy,
            Self::TriggeredBy => Self::Triggers,
            Self::Contains => Self::ContainedIn,
            Self::ContainedIn => Self::Contains,
            Self::References => Self::ReferencedBy,
            Self::ReferencedBy => Self::References,
            Self::InheritsFrom => Self::InheritedBy,
            Self::InheritedBy => Self::InheritsFrom,
            Self::AssociatesWith => Self::AssociatesWith,
            Self::Remembers => Self::RememberedBy,
            Self::RememberedBy => Self::Remembers,
        }
    }
}

/// Kapcsolat ket block kozott
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Cel block ID
    pub target_id: String,
    /// Kapcsolat tipusa
    pub connection_type: ConnectionType,
    /// Kapcsolat ereje (0.0 - 1.0)
    pub strength: f64,
    /// Metaadatok
    pub metadata: HashMap<String, String>,
    /// Letrehozas ideje
    pub created_at: DateTime<Utc>,
}

impl Connection {
    pub fn new(target_id: impl Into<String>, connection_type: ConnectionType) -> Self {
        Self {
            target_id: target_id.into(),
            connection_type,
            strength: 1.0,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_strength(mut self, strength: f64) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// CODE BLOCK - Onismero kod egyseg
// ============================================================================

/// CodeBlock tipusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockType {
    /// Memoria (emlekek)
    Memory,
    /// Funkcio
    Function,
    /// Adat
    Data,
    /// Esemeny
    Event,
    /// Erzelem
    Emotion,
    /// Szemely
    Person,
    /// Koncepco
    Concept,
    /// Gondolat
    Thought,
    /// Dontés
    Decision,
    /// Szabaly
    Rule,
    /// Cel
    Goal,
    /// Allapot
    State,
    /// Egyeb
    Other,
}

/// CodeBlock - Onismero kod egyseg
///
/// Minden block tudja:
/// - Ki o (identity)
/// - Mit tartalmaz (content)
/// - Kihez kapcsolodik (connections)
/// - Miert letezik (purpose)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Egyedi azonosito
    pub id: String,
    /// Nev
    pub name: String,
    /// Cel/Miert letezik
    pub purpose: String,
    /// Block tipus
    pub block_type: BlockType,
    /// Tartalom (barmi lehet - kod, adat, szoveg)
    pub content: String,
    /// Fontossag (0.0 - 1.0)
    pub importance: f64,
    /// Allapot
    pub state: BlockState,
    /// Kapcsolatok mas block-okhoz
    pub connections: Vec<Connection>,
    /// Cimkek
    pub tags: HashSet<String>,
    /// Metaadatok
    pub metadata: HashMap<String, String>,
    /// Letrehozas ideje
    pub created_at: DateTime<Utc>,
    /// Utolso modositas
    pub updated_at: DateTime<Utc>,
    /// Hozzaferes szamlalo
    pub access_count: u64,
}

/// Block allapot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockState {
    /// Aktiv
    Active,
    /// Inaktiv
    Inactive,
    /// Feldolgozas alatt
    Processing,
    /// Archivalt
    Archived,
    /// Torolt (soft delete)
    Deleted,
}

impl CodeBlock {
    /// Uj CodeBlock
    pub fn new(
        name: impl Into<String>,
        purpose: impl Into<String>,
        block_type: BlockType,
        content: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            purpose: purpose.into(),
            block_type,
            content: content.into(),
            importance: 0.5,
            state: BlockState::Active,
            connections: Vec::new(),
            tags: HashSet::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            access_count: 0,
        }
    }

    /// Fontossag beallitasa
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Tag hozzaadasa
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Metadata hozzaadasa
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Kapcsolat hozzaadasa
    pub fn connect(&mut self, target_id: impl Into<String>, conn_type: ConnectionType) {
        let conn = Connection::new(target_id, conn_type);
        self.connections.push(conn);
        self.updated_at = Utc::now();
    }

    /// Kapcsolat hozzaadasa erosseggel
    pub fn connect_with_strength(
        &mut self,
        target_id: impl Into<String>,
        conn_type: ConnectionType,
        strength: f64,
    ) {
        let conn = Connection::new(target_id, conn_type).with_strength(strength);
        self.connections.push(conn);
        self.updated_at = Utc::now();
    }

    /// Kapcsolatok lekerese tipus alapjan
    pub fn get_connections(&self, conn_type: ConnectionType) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| c.connection_type == conn_type)
            .collect()
    }

    /// Osszes kapcsolt ID
    pub fn connected_ids(&self) -> Vec<&str> {
        self.connections
            .iter()
            .map(|c| c.target_id.as_str())
            .collect()
    }

    /// Hozzaferes regisztralasa
    pub fn access(&mut self) {
        self.access_count += 1;
        self.updated_at = Utc::now();
    }

    /// Block hash (tartalom alapjan)
    pub fn content_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Onreflexio - a block leirja magat
    pub fn describe(&self) -> String {
        format!(
            "CodeBlock '{}' [{}]\n\
             Purpose: {}\n\
             Type: {:?}\n\
             State: {:?}\n\
             Importance: {:.2}\n\
             Connections: {}\n\
             Tags: {:?}\n\
             Content preview: {}...",
            self.name,
            self.id,
            self.purpose,
            self.block_type,
            self.state,
            self.importance,
            self.connections.len(),
            self.tags,
            &self.content[..self.content.len().min(100)]
        )
    }
}

// ============================================================================
// CODE GRAPH - A teljes graf
// ============================================================================

/// CodeGraph - Minden CodeBlock-ot osszefog
///
/// A kod MAGA a graf. Nincs kulso DB.
pub struct CodeGraph {
    /// Identitas
    identity: CodeIdentity,
    /// Osszes block (id -> block)
    blocks: Arc<RwLock<HashMap<String, CodeBlock>>>,
    /// Index: nev -> id
    name_index: Arc<RwLock<HashMap<String, String>>>,
    /// Index: tipus -> id-k
    type_index: Arc<RwLock<HashMap<BlockType, HashSet<String>>>>,
    /// Index: tag -> id-k
    tag_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl CodeGraph {
    /// Uj ures graf
    pub fn new() -> Self {
        let identity = CodeIdentity::new(
            "CodeGraph",
            "A kod maga a graf - minden block onismero es kapcsolodik",
            ModuleType::Core,
        )
        .with_capabilities(vec![
            "store_blocks",
            "connect_blocks",
            "traverse",
            "search",
            "self_aware",
        ]);

        Self {
            identity,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ==================== CRUD OPERATIONS ====================

    /// Block hozzaadasa
    pub fn add(&self, block: CodeBlock) -> HopeResult<String> {
        let id = block.id.clone();
        let name = block.name.clone();
        let block_type = block.block_type;
        let tags: Vec<String> = block.tags.iter().cloned().collect();

        // Block mentese
        {
            let mut blocks = self.blocks.write().unwrap();
            blocks.insert(id.clone(), block);
        }

        // Indexek frissitese
        {
            let mut name_idx = self.name_index.write().unwrap();
            name_idx.insert(name, id.clone());
        }
        {
            let mut type_idx = self.type_index.write().unwrap();
            type_idx.entry(block_type).or_default().insert(id.clone());
        }
        {
            let mut tag_idx = self.tag_index.write().unwrap();
            for tag in tags {
                tag_idx.entry(tag).or_default().insert(id.clone());
            }
        }

        Ok(id)
    }

    /// Block lekerese ID alapjan
    pub fn get(&self, id: &str) -> Option<CodeBlock> {
        let mut blocks = self.blocks.write().unwrap();
        if let Some(block) = blocks.get_mut(id) {
            block.access();
            Some(block.clone())
        } else {
            None
        }
    }

    /// Block lekerese nev alapjan
    pub fn get_by_name(&self, name: &str) -> Option<CodeBlock> {
        let name_idx = self.name_index.read().unwrap();
        if let Some(id) = name_idx.get(name) {
            self.get(id)
        } else {
            None
        }
    }

    /// Block frissitese
    pub fn update(&self, id: &str, updater: impl FnOnce(&mut CodeBlock)) -> bool {
        let mut blocks = self.blocks.write().unwrap();
        if let Some(block) = blocks.get_mut(id) {
            updater(block);
            block.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Block torlese (soft delete)
    pub fn delete(&self, id: &str) -> bool {
        self.update(id, |block| {
            block.state = BlockState::Deleted;
        })
    }

    /// Block vegleges torlese
    pub fn remove(&self, id: &str) -> Option<CodeBlock> {
        let mut blocks = self.blocks.write().unwrap();
        blocks.remove(id)
    }

    // ==================== CONNECTIONS ====================

    /// Ket block osszekotese (ketiranyú)
    pub fn connect(
        &self,
        from_id: &str,
        to_id: &str,
        conn_type: ConnectionType,
        strength: f64,
    ) -> bool {
        let mut blocks = self.blocks.write().unwrap();

        // Ellenorzes hogy mindketto letezik
        if !blocks.contains_key(from_id) || !blocks.contains_key(to_id) {
            return false;
        }

        // Elso irany: from -> to
        if let Some(from_block) = blocks.get_mut(from_id) {
            from_block.connect_with_strength(to_id, conn_type, strength);
        }

        // Masodik irany: to -> from (inverse kapcsolat)
        if let Some(to_block) = blocks.get_mut(to_id) {
            to_block.connect_with_strength(from_id, conn_type.inverse(), strength);
        }

        true
    }

    /// Egyiranyu kapcsolat
    pub fn connect_one_way(
        &self,
        from_id: &str,
        to_id: &str,
        conn_type: ConnectionType,
        strength: f64,
    ) -> bool {
        let mut blocks = self.blocks.write().unwrap();

        if let Some(from_block) = blocks.get_mut(from_id) {
            from_block.connect_with_strength(to_id, conn_type, strength);
            true
        } else {
            false
        }
    }

    /// Kapcsolt block-ok lekerese
    pub fn get_connected(&self, id: &str, conn_type: Option<ConnectionType>) -> Vec<CodeBlock> {
        let blocks = self.blocks.read().unwrap();

        if let Some(block) = blocks.get(id) {
            let connected_ids: Vec<String> = if let Some(ct) = conn_type {
                block
                    .connections
                    .iter()
                    .filter(|c| c.connection_type == ct)
                    .map(|c| c.target_id.clone())
                    .collect()
            } else {
                block
                    .connections
                    .iter()
                    .map(|c| c.target_id.clone())
                    .collect()
            };

            connected_ids
                .iter()
                .filter_map(|cid| blocks.get(cid).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    // ==================== SEARCH & QUERY ====================

    /// Kereses tipus alapjan
    pub fn find_by_type(&self, block_type: BlockType) -> Vec<CodeBlock> {
        let type_idx = self.type_index.read().unwrap();
        let blocks = self.blocks.read().unwrap();

        if let Some(ids) = type_idx.get(&block_type) {
            ids.iter()
                .filter_map(|id| blocks.get(id).cloned())
                .filter(|b| b.state != BlockState::Deleted)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Kereses tag alapjan
    pub fn find_by_tag(&self, tag: &str) -> Vec<CodeBlock> {
        let tag_idx = self.tag_index.read().unwrap();
        let blocks = self.blocks.read().unwrap();

        if let Some(ids) = tag_idx.get(tag) {
            ids.iter()
                .filter_map(|id| blocks.get(id).cloned())
                .filter(|b| b.state != BlockState::Deleted)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Kereses tartalomban
    pub fn search(&self, query: &str) -> Vec<CodeBlock> {
        let blocks = self.blocks.read().unwrap();
        let query_lower = query.to_lowercase();

        blocks
            .values()
            .filter(|b| {
                b.state != BlockState::Deleted
                    && (b.name.to_lowercase().contains(&query_lower)
                        || b.content.to_lowercase().contains(&query_lower)
                        || b.purpose.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Legfontosabb block-ok
    pub fn top_important(&self, limit: usize) -> Vec<CodeBlock> {
        let blocks = self.blocks.read().unwrap();
        let mut sorted: Vec<_> = blocks
            .values()
            .filter(|b| b.state != BlockState::Deleted)
            .cloned()
            .collect();
        sorted.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        sorted.truncate(limit);
        sorted
    }

    /// Legtobbet hasznalt block-ok
    pub fn most_accessed(&self, limit: usize) -> Vec<CodeBlock> {
        let blocks = self.blocks.read().unwrap();
        let mut sorted: Vec<_> = blocks
            .values()
            .filter(|b| b.state != BlockState::Deleted)
            .cloned()
            .collect();
        sorted.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        sorted.truncate(limit);
        sorted
    }

    // ==================== GRAPH TRAVERSAL ====================

    /// BFS bejaras
    pub fn traverse_bfs(&self, start_id: &str, max_depth: usize) -> Vec<(CodeBlock, usize)> {
        let blocks = self.blocks.read().unwrap();
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();

        if let Some(start) = blocks.get(start_id) {
            queue.push_back((start.clone(), 0usize));
            visited.insert(start_id.to_string());
        }

        while let Some((block, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }

            let connected_ids: Vec<String> = block
                .connections
                .iter()
                .map(|c| c.target_id.clone())
                .collect();

            result.push((block, depth));

            for cid in connected_ids {
                if !visited.contains(&cid) {
                    visited.insert(cid.clone());
                    if let Some(connected) = blocks.get(&cid) {
                        queue.push_back((connected.clone(), depth + 1));
                    }
                }
            }
        }

        result
    }

    /// Ut kereses ket block kozott
    pub fn find_path(&self, from_id: &str, to_id: &str) -> Option<Vec<String>> {
        let blocks = self.blocks.read().unwrap();
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parents: HashMap<String, String> = HashMap::new();

        queue.push_back(from_id.to_string());
        visited.insert(from_id.to_string());

        while let Some(current_id) = queue.pop_front() {
            if current_id == to_id {
                // Ut visszaepitese
                let mut path = vec![to_id.to_string()];
                let mut current = to_id.to_string();
                while let Some(parent) = parents.get(&current) {
                    path.push(parent.clone());
                    current = parent.clone();
                }
                path.reverse();
                return Some(path);
            }

            if let Some(block) = blocks.get(&current_id) {
                for conn in &block.connections {
                    if !visited.contains(&conn.target_id) {
                        visited.insert(conn.target_id.clone());
                        parents.insert(conn.target_id.clone(), current_id.clone());
                        queue.push_back(conn.target_id.clone());
                    }
                }
            }
        }

        None
    }

    // ==================== STATISTICS ====================

    /// Graf statisztikak
    pub fn stats(&self) -> GraphStats {
        let blocks = self.blocks.read().unwrap();

        let total_blocks = blocks.len();
        let active_blocks = blocks
            .values()
            .filter(|b| b.state == BlockState::Active)
            .count();
        let total_connections: usize = blocks.values().map(|b| b.connections.len()).sum();

        let mut type_counts = HashMap::new();
        for block in blocks.values() {
            *type_counts.entry(block.block_type).or_insert(0) += 1;
        }

        GraphStats {
            total_blocks,
            active_blocks,
            total_connections,
            type_counts,
            avg_connections: if total_blocks > 0 {
                total_connections as f64 / total_blocks as f64
            } else {
                0.0
            },
        }
    }

    /// Osszes block
    pub fn all_blocks(&self) -> Vec<CodeBlock> {
        let blocks = self.blocks.read().unwrap();
        blocks.values().cloned().collect()
    }

    /// Block szam
    pub fn len(&self) -> usize {
        self.blocks.read().unwrap().len()
    }

    /// Ures-e
    pub fn is_empty(&self) -> bool {
        self.blocks.read().unwrap().is_empty()
    }
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graf statisztikak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_blocks: usize,
    pub active_blocks: usize,
    pub total_connections: usize,
    pub type_counts: HashMap<BlockType, usize>,
    pub avg_connections: f64,
}

// ============================================================================
// AWARE TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait]
impl Aware for CodeGraph {
    fn identity(&self) -> &CodeIdentity {
        &self.identity
    }

    fn identity_mut(&mut self) -> &mut CodeIdentity {
        &mut self.identity
    }

    fn reflect(&self) -> Reflection {
        let stats = self.stats();
        Reflection::new(&self.identity.name, &self.identity.purpose)
            .with_state(self.identity.state.to_string())
            .with_health(self.identity.health())
            .with_thought(format!(
                "Graf: {} block, {} kapcsolat, atlag {:.1} kapcsolat/block",
                stats.total_blocks, stats.total_connections, stats.avg_connections
            ))
            .with_capabilities(vec![
                "store_blocks",
                "connect_blocks",
                "traverse",
                "search",
                "self_aware",
            ])
    }

    async fn init(&mut self) -> HopeResult<()> {
        self.identity.set_state(ModuleState::Active);
        tracing::info!("CodeGraph inicializalva - A kod maga a graf");
        Ok(())
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS - Memory/Emotion/Person shorthand
// ============================================================================

impl CodeGraph {
    /// Emlyek hozzaadasa (Memory block)
    pub fn remember(&self, content: &str, importance: f64) -> HopeResult<String> {
        let block = CodeBlock::new(
            format!("memory_{}", Utc::now().timestamp_millis()),
            "Emlék tárolása",
            BlockType::Memory,
            content,
        )
        .with_importance(importance)
        .with_tag("memory");

        self.add(block)
    }

    /// Erzelem hozzaadasa (Emotion block)
    pub fn feel(&self, emotion: &str, intensity: f64, trigger: Option<&str>) -> HopeResult<String> {
        let mut block = CodeBlock::new(
            format!("emotion_{}_{}", emotion, Utc::now().timestamp_millis()),
            format!("Érzelem: {}", emotion),
            BlockType::Emotion,
            emotion,
        )
        .with_importance(intensity)
        .with_tag("emotion")
        .with_tag(emotion);

        if let Some(t) = trigger {
            block = block.with_meta("trigger", t);
        }

        self.add(block)
    }

    /// Szemely hozzaadasa (Person block)
    pub fn know_person(&self, name: &str, relationship: &str, trust: f64) -> HopeResult<String> {
        let block = CodeBlock::new(
            name,
            format!("Személy: {} ({})", name, relationship),
            BlockType::Person,
            format!("{}|{}|{}", name, relationship, trust),
        )
        .with_importance(trust)
        .with_tag("person")
        .with_meta("relationship", relationship);

        self.add(block)
    }

    /// Gondolat hozzaadasa (Thought block)
    pub fn think(&self, thought: &str, importance: f64) -> HopeResult<String> {
        let block = CodeBlock::new(
            format!("thought_{}", Utc::now().timestamp_millis()),
            "Gondolat rögzítése",
            BlockType::Thought,
            thought,
        )
        .with_importance(importance)
        .with_tag("thought");

        self.add(block)
    }

    /// Koncepció hozzaadasa (Concept block)
    pub fn learn_concept(
        &self,
        name: &str,
        description: &str,
        importance: f64,
    ) -> HopeResult<String> {
        let block = CodeBlock::new(
            name,
            format!("Koncepció: {}", name),
            BlockType::Concept,
            description,
        )
        .with_importance(importance)
        .with_tag("concept");

        self.add(block)
    }

    /// Esemeny hozzaadasa (Event block)
    pub fn log_event(&self, event_type: &str, details: &str) -> HopeResult<String> {
        let block = CodeBlock::new(
            format!("event_{}_{}", event_type, Utc::now().timestamp_millis()),
            format!("Esemény: {}", event_type),
            BlockType::Event,
            details,
        )
        .with_tag("event")
        .with_tag(event_type);

        self.add(block)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block_creation() {
        let block = CodeBlock::new(
            "test_block",
            "Testing purpose",
            BlockType::Data,
            "Test content",
        )
        .with_importance(0.8)
        .with_tag("test");

        assert_eq!(block.name, "test_block");
        assert_eq!(block.importance, 0.8);
        assert!(block.tags.contains("test"));
    }

    #[test]
    fn test_code_graph_add_get() {
        let graph = CodeGraph::new();

        let block = CodeBlock::new("test", "purpose", BlockType::Data, "content");
        let id = graph.add(block).unwrap();

        let retrieved = graph.get(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_code_graph_connect() {
        let graph = CodeGraph::new();

        let block1 = CodeBlock::new("block1", "p1", BlockType::Data, "c1");
        let block2 = CodeBlock::new("block2", "p2", BlockType::Data, "c2");

        let id1 = graph.add(block1).unwrap();
        let id2 = graph.add(block2).unwrap();

        graph.connect(&id1, &id2, ConnectionType::ConnectsTo, 1.0);

        let connected = graph.get_connected(&id1, None);
        assert_eq!(connected.len(), 1);
        assert_eq!(connected[0].name, "block2");

        // Inverse connection
        let connected_back = graph.get_connected(&id2, None);
        assert_eq!(connected_back.len(), 1);
        assert_eq!(connected_back[0].name, "block1");
    }

    #[test]
    fn test_code_graph_search() {
        let graph = CodeGraph::new();

        graph
            .add(CodeBlock::new(
                "hello_world",
                "p",
                BlockType::Data,
                "Hello World!",
            ))
            .unwrap();
        graph
            .add(CodeBlock::new("goodbye", "p", BlockType::Data, "Goodbye!"))
            .unwrap();

        let results = graph.search("hello");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "hello_world");
    }

    #[test]
    fn test_code_graph_traverse() {
        let graph = CodeGraph::new();

        let id1 = graph
            .add(CodeBlock::new("a", "p", BlockType::Data, "A"))
            .unwrap();
        let id2 = graph
            .add(CodeBlock::new("b", "p", BlockType::Data, "B"))
            .unwrap();
        let id3 = graph
            .add(CodeBlock::new("c", "p", BlockType::Data, "C"))
            .unwrap();

        graph.connect(&id1, &id2, ConnectionType::ConnectsTo, 1.0);
        graph.connect(&id2, &id3, ConnectionType::ConnectsTo, 1.0);

        let traversal = graph.traverse_bfs(&id1, 10);
        assert_eq!(traversal.len(), 3);
    }

    #[test]
    fn test_find_path() {
        let graph = CodeGraph::new();

        let id1 = graph
            .add(CodeBlock::new("start", "p", BlockType::Data, ""))
            .unwrap();
        let id2 = graph
            .add(CodeBlock::new("middle", "p", BlockType::Data, ""))
            .unwrap();
        let id3 = graph
            .add(CodeBlock::new("end", "p", BlockType::Data, ""))
            .unwrap();

        graph.connect(&id1, &id2, ConnectionType::ConnectsTo, 1.0);
        graph.connect(&id2, &id3, ConnectionType::ConnectsTo, 1.0);

        let path = graph.find_path(&id1, &id3);
        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }

    #[test]
    fn test_memory_emotion_person() {
        let graph = CodeGraph::new();

        let mem_id = graph.remember("Fontos emlék", 0.9).unwrap();
        let emo_id = graph.feel("joy", 0.8, Some("good news")).unwrap();
        let person_id = graph.know_person("Máté", "creator", 1.0).unwrap();

        // Connect memory to emotion and person
        graph.connect(&mem_id, &emo_id, ConnectionType::TriggeredBy, 0.9);
        graph.connect(&mem_id, &person_id, ConnectionType::References, 1.0);

        let memories = graph.find_by_type(BlockType::Memory);
        assert_eq!(memories.len(), 1);

        let emotions = graph.find_by_type(BlockType::Emotion);
        assert_eq!(emotions.len(), 1);

        let persons = graph.find_by_type(BlockType::Person);
        assert_eq!(persons.len(), 1);
    }

    #[test]
    fn test_stats() {
        let graph = CodeGraph::new();

        graph.remember("m1", 0.5).unwrap();
        graph.remember("m2", 0.5).unwrap();
        graph.feel("joy", 0.8, None).unwrap();

        let stats = graph.stats();
        assert_eq!(stats.total_blocks, 3);
        assert_eq!(stats.type_counts.get(&BlockType::Memory), Some(&2));
        assert_eq!(stats.type_counts.get(&BlockType::Emotion), Some(&1));
    }
}
