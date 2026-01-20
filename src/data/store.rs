//! Hope OS - DataStore
//!
//! SurrealDB alapú multi-model tárolás (document + graph + relational).
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::sql::Thing;
use surrealdb::Surreal;

use crate::core::{HopeError, HopeResult};
use crate::modules::{Memory, MemoryType};

/// Adatbázis statisztikák
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStats {
    /// Emlékek száma
    pub memories: u64,
    /// Események száma
    pub events: u64,
    /// Érzelmek száma
    pub emotions: u64,
    /// KV párok száma
    pub kv_pairs: u64,
    /// Kapcsolatok száma (graph edges)
    pub relations: u64,
    /// Adatbázis méret (byte)
    pub db_size: u64,
}

/// Graph kapcsolat típusok
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    /// Emlék -> Érzelem kapcsolat
    TriggeredBy,
    /// Emlék -> Személy kapcsolat
    RelatesTo,
    /// Emlék -> Koncepció kapcsolat
    AssociatedWith,
    /// Emlék -> Emlék kapcsolat
    LinkedTo,
    /// Személy -> Személy kapcsolat
    Knows,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::TriggeredBy => "triggered_by",
            RelationType::RelatesTo => "relates_to",
            RelationType::AssociatedWith => "associated_with",
            RelationType::LinkedTo => "linked_to",
            RelationType::Knows => "knows",
        }
    }
}

/// Graph kapcsolat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

/// Személy (relational memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<Thing>,
    pub name: String,
    pub relationship: String,
    pub trust_level: f64,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_interaction: Option<DateTime<Utc>>,
}

/// Koncepció (associative memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Option<Thing>,
    pub name: String,
    pub description: Option<String>,
    pub importance: f64,
    pub created_at: DateTime<Utc>,
}

/// Érzelem rekord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionRecord {
    pub id: Option<Thing>,
    pub emotion: String,
    pub intensity: f64,
    pub trigger: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Esemény rekord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: Option<Thing>,
    pub event_type: String,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

/// KV rekord
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvRecord {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

/// Memory rekord SurrealDB-hez
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: Option<Thing>,
    pub content: String,
    pub memory_type: String,
    pub importance: f64,
    pub emotional_tag: Option<String>,
    pub created_at: DateTime<Utc>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub access_count: u32,
    pub embedding: Option<Vec<f32>>,
}

impl From<&Memory> for MemoryRecord {
    fn from(m: &Memory) -> Self {
        Self {
            id: None,
            content: m.content.clone(),
            memory_type: m.memory_type.as_str().to_string(),
            importance: m.importance,
            emotional_tag: m.emotional_tag.clone(),
            created_at: m.created_at,
            accessed_at: m.accessed_at,
            access_count: m.access_count,
            embedding: None,
        }
    }
}

impl MemoryRecord {
    pub fn to_memory(&self) -> Memory {
        Memory {
            id: self.id.as_ref().map(|t| t.id.to_string()).unwrap_or_default(),
            content: self.content.clone(),
            memory_type: MemoryType::from_str(&self.memory_type).unwrap_or(MemoryType::LongTerm),
            importance: self.importance,
            emotional_tag: self.emotional_tag.clone(),
            created_at: self.created_at,
            accessed_at: self.accessed_at,
            access_count: self.access_count,
        }
    }
}

/// DataStore - SurrealDB alapú multi-model tárolás
pub struct DataStore {
    /// SurrealDB kapcsolat
    db: Surreal<Db>,
    /// Adatbázis útvonal
    path: String,
}

impl DataStore {
    /// Új DataStore létrehozása
    pub async fn new(path: impl AsRef<Path>) -> HopeResult<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // SurrealDB inicializálás RocksDB backend-del
        let db = Surreal::new::<RocksDb>(&path_str)
            .await
            .map_err(|e| HopeError::General(format!("SurrealDB init error: {}", e)))?;

        // Namespace és database beállítása
        db.use_ns("hope").use_db("hope_os")
            .await
            .map_err(|e| HopeError::General(format!("SurrealDB namespace error: {}", e)))?;

        let store = Self { db, path: path_str };

        // Sémák inicializálása
        store.init_schema().await?;

        tracing::info!("DataStore (SurrealDB) inicializálva: {}", store.path);
        Ok(store)
    }

    /// In-memory DataStore létrehozása (tesztekhez)
    pub async fn new_memory() -> HopeResult<Self> {
        use surrealdb::engine::local::Mem;

        let db = Surreal::new::<Mem>(())
            .await
            .map_err(|e| HopeError::General(format!("SurrealDB mem init error: {}", e)))?;

        db.use_ns("hope").use_db("hope_os")
            .await
            .map_err(|e| HopeError::General(format!("SurrealDB namespace error: {}", e)))?;

        let store = Self {
            db,
            path: ":memory:".to_string(),
        };

        store.init_schema().await?;
        Ok(store)
    }

    /// Séma inicializálása
    async fn init_schema(&self) -> HopeResult<()> {
        // Táblák definiálása SurrealQL-ben
        let schema = r#"
            -- Memory tábla indexekkel
            DEFINE TABLE memory SCHEMAFULL;
            DEFINE FIELD content ON memory TYPE string;
            DEFINE FIELD memory_type ON memory TYPE string;
            DEFINE FIELD importance ON memory TYPE float DEFAULT 0.5;
            DEFINE FIELD emotional_tag ON memory TYPE option<string>;
            DEFINE FIELD created_at ON memory TYPE datetime DEFAULT time::now();
            DEFINE FIELD accessed_at ON memory TYPE option<datetime>;
            DEFINE FIELD access_count ON memory TYPE int DEFAULT 0;
            DEFINE FIELD embedding ON memory TYPE option<array<float>>;
            DEFINE INDEX idx_memory_type ON memory FIELDS memory_type;
            DEFINE INDEX idx_memory_importance ON memory FIELDS importance;

            -- Személyek tábla (relational memory)
            DEFINE TABLE person SCHEMAFULL;
            DEFINE FIELD name ON person TYPE string;
            DEFINE FIELD relationship ON person TYPE string;
            DEFINE FIELD trust_level ON person TYPE float DEFAULT 0.5;
            DEFINE FIELD notes ON person TYPE option<string>;
            DEFINE FIELD created_at ON person TYPE datetime DEFAULT time::now();
            DEFINE FIELD last_interaction ON person TYPE option<datetime>;
            DEFINE INDEX idx_person_name ON person FIELDS name UNIQUE;

            -- Koncepciók tábla (associative memory)
            DEFINE TABLE concept SCHEMAFULL;
            DEFINE FIELD name ON concept TYPE string;
            DEFINE FIELD description ON concept TYPE option<string>;
            DEFINE FIELD importance ON concept TYPE float DEFAULT 0.5;
            DEFINE FIELD created_at ON concept TYPE datetime DEFAULT time::now();
            DEFINE INDEX idx_concept_name ON concept FIELDS name UNIQUE;

            -- Érzelmek tábla
            DEFINE TABLE emotion SCHEMAFULL;
            DEFINE FIELD emotion ON emotion TYPE string;
            DEFINE FIELD intensity ON emotion TYPE float;
            DEFINE FIELD trigger ON emotion TYPE option<string>;
            DEFINE FIELD timestamp ON emotion TYPE datetime DEFAULT time::now();
            DEFINE INDEX idx_emotion_type ON emotion FIELDS emotion;

            -- Események tábla
            DEFINE TABLE event SCHEMAFULL;
            DEFINE FIELD event_type ON event TYPE string;
            DEFINE FIELD details ON event TYPE string;
            DEFINE FIELD timestamp ON event TYPE datetime DEFAULT time::now();
            DEFINE INDEX idx_event_type ON event FIELDS event_type;

            -- KV Store tábla
            DEFINE TABLE kv SCHEMAFULL;
            DEFINE FIELD key ON kv TYPE string;
            DEFINE FIELD value ON kv TYPE string;
            DEFINE FIELD updated_at ON kv TYPE datetime DEFAULT time::now();
            DEFINE INDEX idx_kv_key ON kv FIELDS key UNIQUE;

            -- GRAPH RELATIONS (ez a lényeg!)
            -- Memory -> Emotion kapcsolat
            DEFINE TABLE triggered_by SCHEMAFULL TYPE RELATION FROM memory TO emotion;
            DEFINE FIELD weight ON triggered_by TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON triggered_by TYPE datetime DEFAULT time::now();

            -- Memory -> Person kapcsolat
            DEFINE TABLE relates_to SCHEMAFULL TYPE RELATION FROM memory TO person;
            DEFINE FIELD weight ON relates_to TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON relates_to TYPE datetime DEFAULT time::now();

            -- Memory -> Concept kapcsolat
            DEFINE TABLE associated_with SCHEMAFULL TYPE RELATION FROM memory TO concept;
            DEFINE FIELD weight ON associated_with TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON associated_with TYPE datetime DEFAULT time::now();

            -- Memory -> Memory kapcsolat
            DEFINE TABLE linked_to SCHEMAFULL TYPE RELATION FROM memory TO memory;
            DEFINE FIELD weight ON linked_to TYPE float DEFAULT 1.0;
            DEFINE FIELD created_at ON linked_to TYPE datetime DEFAULT time::now();

            -- Person -> Person kapcsolat
            DEFINE TABLE knows SCHEMAFULL TYPE RELATION FROM person TO person;
            DEFINE FIELD weight ON knows TYPE float DEFAULT 1.0;
            DEFINE FIELD context ON knows TYPE option<string>;
            DEFINE FIELD created_at ON knows TYPE datetime DEFAULT time::now();
        "#;

        self.db
            .query(schema)
            .await
            .map_err(|e| HopeError::General(format!("Schema init error: {}", e)))?;

        Ok(())
    }

    // ==================== MEMORY OPERATIONS ====================

    /// Emlék mentése
    pub async fn save_memory(&self, memory: &Memory) -> HopeResult<String> {
        let record = MemoryRecord::from(memory);

        let created: Option<MemoryRecord> = self.db
            .create(("memory", &memory.id))
            .content(record)
            .await
            .map_err(|e| HopeError::General(format!("Save memory error: {}", e)))?;

        Ok(created.map(|r| r.id.map(|t| t.id.to_string()).unwrap_or_default()).unwrap_or_default())
    }

    /// Emlék lekérdezése
    pub async fn get_memory(&self, id: &str) -> HopeResult<Option<Memory>> {
        let record: Option<MemoryRecord> = self.db
            .select(("memory", id))
            .await
            .map_err(|e| HopeError::General(format!("Get memory error: {}", e)))?;

        Ok(record.map(|r| r.to_memory()))
    }

    /// Emlékek keresése típus alapján
    pub async fn find_memories(&self, memory_type: &str, limit: usize) -> HopeResult<Vec<Memory>> {
        let mut result = self.db
            .query("SELECT * FROM memory WHERE memory_type = $type ORDER BY created_at DESC LIMIT $limit")
            .bind(("type", memory_type))
            .bind(("limit", limit))
            .await
            .map_err(|e| HopeError::General(format!("Find memories error: {}", e)))?;

        let records: Vec<MemoryRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse memories error: {}", e)))?;

        Ok(records.into_iter().map(|r| r.to_memory()).collect())
    }

    /// Emlékek keresése graph kapcsolat alapján
    pub async fn find_memories_by_emotion(&self, emotion: &str) -> HopeResult<Vec<Memory>> {
        let mut result = self.db
            .query(r#"
                SELECT * FROM memory
                WHERE ->triggered_by->emotion.emotion = $emotion
                ORDER BY importance DESC
            "#)
            .bind(("emotion", emotion))
            .await
            .map_err(|e| HopeError::General(format!("Find by emotion error: {}", e)))?;

        let records: Vec<MemoryRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse error: {}", e)))?;

        Ok(records.into_iter().map(|r| r.to_memory()).collect())
    }

    /// Emlékek keresése személyhez kapcsolódva
    pub async fn find_memories_by_person(&self, person_name: &str) -> HopeResult<Vec<Memory>> {
        let mut result = self.db
            .query(r#"
                SELECT * FROM memory
                WHERE ->relates_to->person.name = $name
                ORDER BY created_at DESC
            "#)
            .bind(("name", person_name))
            .await
            .map_err(|e| HopeError::General(format!("Find by person error: {}", e)))?;

        let records: Vec<MemoryRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse error: {}", e)))?;

        Ok(records.into_iter().map(|r| r.to_memory()).collect())
    }

    // ==================== GRAPH RELATIONS ====================

    /// Kapcsolat létrehozása
    pub async fn create_relation(
        &self,
        from_table: &str,
        from_id: &str,
        relation_type: RelationType,
        to_table: &str,
        to_id: &str,
        weight: f64,
    ) -> HopeResult<()> {
        let query = format!(
            "RELATE {}:{}->{}->{}:{} SET weight = $weight, created_at = time::now()",
            from_table, from_id, relation_type.as_str(), to_table, to_id
        );

        self.db
            .query(&query)
            .bind(("weight", weight))
            .await
            .map_err(|e| HopeError::General(format!("Create relation error: {}", e)))?;

        Ok(())
    }

    /// Kapcsolódó elemek lekérdezése
    pub async fn get_related<T: for<'de> Deserialize<'de>>(
        &self,
        from_table: &str,
        from_id: &str,
        relation_type: RelationType,
    ) -> HopeResult<Vec<T>> {
        let query = format!(
            "SELECT VALUE ->{}->* FROM {}:{}",
            relation_type.as_str(), from_table, from_id
        );

        let mut result = self.db
            .query(&query)
            .await
            .map_err(|e| HopeError::General(format!("Get related error: {}", e)))?;

        let items: Vec<T> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse related error: {}", e)))?;

        Ok(items)
    }

    // ==================== PERSON OPERATIONS ====================

    /// Személy mentése
    pub async fn save_person(&self, person: &Person) -> HopeResult<String> {
        let created: Option<Person> = self.db
            .create("person")
            .content(person)
            .await
            .map_err(|e| HopeError::General(format!("Save person error: {}", e)))?;

        Ok(created.and_then(|p| p.id.map(|t| t.id.to_string())).unwrap_or_default())
    }

    /// Személy keresése név alapján
    pub async fn find_person(&self, name: &str) -> HopeResult<Option<Person>> {
        let mut result = self.db
            .query("SELECT * FROM person WHERE name = $name LIMIT 1")
            .bind(("name", name))
            .await
            .map_err(|e| HopeError::General(format!("Find person error: {}", e)))?;

        let persons: Vec<Person> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse person error: {}", e)))?;

        Ok(persons.into_iter().next())
    }

    // ==================== CONCEPT OPERATIONS ====================

    /// Koncepció mentése
    pub async fn save_concept(&self, concept: &Concept) -> HopeResult<String> {
        let created: Option<Concept> = self.db
            .create("concept")
            .content(concept)
            .await
            .map_err(|e| HopeError::General(format!("Save concept error: {}", e)))?;

        Ok(created.and_then(|c| c.id.map(|t| t.id.to_string())).unwrap_or_default())
    }

    // ==================== EVENT LOGGING ====================

    /// Esemény naplózása
    pub async fn log_event(&self, event_type: &str, details: &str) -> HopeResult<()> {
        let event = EventRecord {
            id: None,
            event_type: event_type.to_string(),
            details: details.to_string(),
            timestamp: Utc::now(),
        };

        let _: Option<EventRecord> = self.db
            .create("event")
            .content(event)
            .await
            .map_err(|e| HopeError::General(format!("Log event error: {}", e)))?;

        Ok(())
    }

    /// Események lekérdezése
    pub async fn get_events(&self, event_type: Option<&str>, limit: usize) -> HopeResult<Vec<(String, String, String)>> {
        let query = if event_type.is_some() {
            "SELECT event_type, details, timestamp FROM event WHERE event_type = $type ORDER BY timestamp DESC LIMIT $limit"
        } else {
            "SELECT event_type, details, timestamp FROM event ORDER BY timestamp DESC LIMIT $limit"
        };

        let mut result = self.db
            .query(query)
            .bind(("type", event_type.unwrap_or("")))
            .bind(("limit", limit))
            .await
            .map_err(|e| HopeError::General(format!("Get events error: {}", e)))?;

        let records: Vec<EventRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse events error: {}", e)))?;

        Ok(records
            .into_iter()
            .map(|r| (r.event_type, r.details, r.timestamp.to_rfc3339()))
            .collect())
    }

    // ==================== KEY-VALUE STORE ====================

    /// Érték beállítása
    pub async fn set(&self, key: &str, value: &str) -> HopeResult<()> {
        self.db
            .query("DELETE FROM kv WHERE key = $key; CREATE kv SET key = $key, value = $value, updated_at = time::now()")
            .bind(("key", key))
            .bind(("value", value))
            .await
            .map_err(|e| HopeError::General(format!("KV set error: {}", e)))?;

        Ok(())
    }

    /// Érték lekérdezése
    pub async fn get(&self, key: &str) -> HopeResult<Option<String>> {
        let mut result = self.db
            .query("SELECT value FROM kv WHERE key = $key LIMIT 1")
            .bind(("key", key))
            .await
            .map_err(|e| HopeError::General(format!("KV get error: {}", e)))?;

        let records: Vec<KvRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse KV error: {}", e)))?;

        Ok(records.into_iter().next().map(|r| r.value))
    }

    /// Érték törlése
    pub async fn delete(&self, key: &str) -> HopeResult<bool> {
        let mut result = self.db
            .query("DELETE FROM kv WHERE key = $key RETURN BEFORE")
            .bind(("key", key))
            .await
            .map_err(|e| HopeError::General(format!("KV delete error: {}", e)))?;

        let deleted: Vec<KvRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse delete error: {}", e)))?;

        Ok(!deleted.is_empty())
    }

    // ==================== EMOTION LOGGING ====================

    /// Érzelem naplózása
    pub async fn log_emotion(&self, emotion: &str, intensity: f64, trigger: Option<&str>) -> HopeResult<String> {
        let record = EmotionRecord {
            id: None,
            emotion: emotion.to_string(),
            intensity,
            trigger: trigger.map(String::from),
            timestamp: Utc::now(),
        };

        let created: Option<EmotionRecord> = self.db
            .create("emotion")
            .content(record)
            .await
            .map_err(|e| HopeError::General(format!("Log emotion error: {}", e)))?;

        Ok(created.and_then(|e| e.id.map(|t| t.id.to_string())).unwrap_or_default())
    }

    /// Érzelmek lekérdezése
    pub async fn get_emotions(&self, limit: usize) -> HopeResult<Vec<(String, f64, Option<String>, String)>> {
        let mut result = self.db
            .query("SELECT * FROM emotion ORDER BY timestamp DESC LIMIT $limit")
            .bind(("limit", limit))
            .await
            .map_err(|e| HopeError::General(format!("Get emotions error: {}", e)))?;

        let records: Vec<EmotionRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse emotions error: {}", e)))?;

        Ok(records
            .into_iter()
            .map(|r| (r.emotion, r.intensity, r.trigger, r.timestamp.to_rfc3339()))
            .collect())
    }

    // ==================== STATISTICS ====================

    /// Statisztikák lekérdezése
    pub async fn stats(&self) -> HopeResult<DataStats> {
        let mut result = self.db
            .query(r#"
                RETURN {
                    memories: count(SELECT * FROM memory),
                    events: count(SELECT * FROM event),
                    emotions: count(SELECT * FROM emotion),
                    kv_pairs: count(SELECT * FROM kv),
                    relations: count(SELECT * FROM triggered_by) +
                               count(SELECT * FROM relates_to) +
                               count(SELECT * FROM associated_with) +
                               count(SELECT * FROM linked_to) +
                               count(SELECT * FROM knows)
                }
            "#)
            .await
            .map_err(|e| HopeError::General(format!("Stats error: {}", e)))?;

        #[derive(Deserialize)]
        struct StatsResult {
            memories: u64,
            events: u64,
            emotions: u64,
            kv_pairs: u64,
            relations: u64,
        }

        let stats: Option<StatsResult> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse stats error: {}", e)))?;

        let s = stats.unwrap_or(StatsResult {
            memories: 0,
            events: 0,
            emotions: 0,
            kv_pairs: 0,
            relations: 0,
        });

        // Könyvtár méret becslése
        let db_size = if self.path != ":memory:" {
            walkdir::WalkDir::new(&self.path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        } else {
            0
        };

        Ok(DataStats {
            memories: s.memories,
            events: s.events,
            emotions: s.emotions,
            kv_pairs: s.kv_pairs,
            relations: s.relations,
            db_size,
        })
    }

    /// Adatbázis útvonal
    pub fn path(&self) -> &str {
        &self.path
    }

    // ==================== ADVANCED QUERIES ====================

    /// Összekapcsolt emlékek láncolata (graph traversal)
    pub async fn memory_chain(&self, start_id: &str, depth: usize) -> HopeResult<Vec<Memory>> {
        let query = format!(
            "SELECT * FROM memory:{} ->linked_to->(memory WHERE true LIMIT {}) ",
            start_id, depth
        );

        let mut result = self.db
            .query(&query)
            .await
            .map_err(|e| HopeError::General(format!("Memory chain error: {}", e)))?;

        let records: Vec<MemoryRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse chain error: {}", e)))?;

        Ok(records.into_iter().map(|r| r.to_memory()).collect())
    }

    /// Full-text keresés emlékekben
    pub async fn search_memories(&self, query: &str, limit: usize) -> HopeResult<Vec<Memory>> {
        let mut result = self.db
            .query("SELECT * FROM memory WHERE content CONTAINS $query ORDER BY importance DESC LIMIT $limit")
            .bind(("query", query))
            .bind(("limit", limit))
            .await
            .map_err(|e| HopeError::General(format!("Search error: {}", e)))?;

        let records: Vec<MemoryRecord> = result
            .take(0)
            .map_err(|e| HopeError::General(format!("Parse search error: {}", e)))?;

        Ok(records.into_iter().map(|r| r.to_memory()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_datastore_memory() {
        let store = DataStore::new_memory().await.unwrap();
        assert_eq!(store.path(), ":memory:");
    }

    #[tokio::test]
    async fn test_kv_store() {
        let store = DataStore::new_memory().await.unwrap();

        store.set("test_key", "test_value").await.unwrap();
        let value = store.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        store.delete("test_key").await.unwrap();
        let value = store.get("test_key").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_memory_operations() {
        let store = DataStore::new_memory().await.unwrap();

        let memory = Memory::new("Test memory content", MemoryType::LongTerm, 0.8);
        let id = memory.id.clone();

        store.save_memory(&memory).await.unwrap();
        let retrieved = store.get_memory(&id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test memory content");
    }

    #[tokio::test]
    async fn test_emotion_logging() {
        let store = DataStore::new_memory().await.unwrap();

        store.log_emotion("joy", 0.9, Some("good news")).await.unwrap();
        let emotions = store.get_emotions(10).await.unwrap();

        assert_eq!(emotions.len(), 1);
        assert_eq!(emotions[0].0, "joy");
    }

    #[tokio::test]
    async fn test_event_logging() {
        let store = DataStore::new_memory().await.unwrap();

        store.log_event("test", "Test event details").await.unwrap();
        let events = store.get_events(Some("test"), 10).await.unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].1, "Test event details");
    }

    #[tokio::test]
    async fn test_person_operations() {
        let store = DataStore::new_memory().await.unwrap();

        let person = Person {
            id: None,
            name: "Máté".to_string(),
            relationship: "creator".to_string(),
            trust_level: 1.0,
            notes: Some("Hope creator".to_string()),
            created_at: Utc::now(),
            last_interaction: None,
        };

        store.save_person(&person).await.unwrap();
        let found = store.find_person("Máté").await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().relationship, "creator");
    }

    #[tokio::test]
    async fn test_graph_relations() {
        let store = DataStore::new_memory().await.unwrap();

        // Emlék létrehozása
        let memory = Memory::new("Meeting with Máté", MemoryType::LongTerm, 0.9);
        store.save_memory(&memory).await.unwrap();

        // Érzelem létrehozása
        let emotion_id = store.log_emotion("joy", 0.8, Some("good meeting")).await.unwrap();

        // Kapcsolat létrehozása
        store.create_relation(
            "memory", &memory.id,
            RelationType::TriggeredBy,
            "emotion", &emotion_id,
            1.0
        ).await.unwrap();

        // Stats ellenőrzés
        let stats = store.stats().await.unwrap();
        assert!(stats.relations > 0);
    }
}
