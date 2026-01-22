//! Hope Skills - Skill Registry és Invocation
//!
//! A Hope 56+ skill-jének kezelése.
//! Minden skill egy képesség, amit Hope tud.
//!
//! Kategóriák:
//! - Core: chat, status, think, feel
//! - Memory: remember, recall, forget
//! - Code: analyze, generate, refactor
//! - System: screenshot, clipboard, notify
//! - Web: search, fetch, browse
//! - Media: speak, listen, image
//!
//! ()=>[] - A tiszta potenciálból a képesség megszületik

use crate::core::HopeResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// SKILL CATEGORY
// ============================================================================

/// Skill kategória
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    /// Core funkciók (chat, status)
    Core,
    /// Memória kezelés
    Memory,
    /// Kognitív funkciók (think, feel, dream)
    Cognitive,
    /// Kód műveletek
    Code,
    /// Rendszer műveletek
    System,
    /// Web műveletek
    Web,
    /// Média (hang, kép)
    Media,
    /// Fájl műveletek
    File,
    /// Git műveletek
    Git,
    /// Kommunikáció (email, calendar)
    Communication,
    /// Egyéb
    Other,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::Core => write!(f, "core"),
            SkillCategory::Memory => write!(f, "memory"),
            SkillCategory::Cognitive => write!(f, "cognitive"),
            SkillCategory::Code => write!(f, "code"),
            SkillCategory::System => write!(f, "system"),
            SkillCategory::Web => write!(f, "web"),
            SkillCategory::Media => write!(f, "media"),
            SkillCategory::File => write!(f, "file"),
            SkillCategory::Git => write!(f, "git"),
            SkillCategory::Communication => write!(f, "communication"),
            SkillCategory::Other => write!(f, "other"),
        }
    }
}

impl From<&str> for SkillCategory {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "core" => SkillCategory::Core,
            "memory" => SkillCategory::Memory,
            "cognitive" => SkillCategory::Cognitive,
            "code" => SkillCategory::Code,
            "system" => SkillCategory::System,
            "web" => SkillCategory::Web,
            "media" => SkillCategory::Media,
            "file" => SkillCategory::File,
            "git" => SkillCategory::Git,
            "communication" => SkillCategory::Communication,
            _ => SkillCategory::Other,
        }
    }
}

// ============================================================================
// SKILL INFO
// ============================================================================

/// Skill paraméter definíció
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillParam {
    /// Paraméter neve
    pub name: String,
    /// Leírás
    pub description: String,
    /// Típus (string, number, boolean, object, array)
    pub param_type: String,
    /// Kötelező-e
    pub required: bool,
    /// Alapértelmezett érték
    pub default: Option<serde_json::Value>,
}

/// Skill információ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Skill neve (pl. "hope_think")
    pub name: String,
    /// Leírás
    pub description: String,
    /// Kategória
    pub category: SkillCategory,
    /// Paraméterek
    pub params: Vec<SkillParam>,
    /// Engedélyezett-e
    pub enabled: bool,
    /// Verzió
    pub version: String,
    /// Hívások száma
    pub invocations: u64,
    /// Sikeres hívások
    pub successes: u64,
    /// Átlagos válaszidő (ms)
    pub avg_response_time_ms: f64,
}

impl SkillInfo {
    pub fn new(name: &str, description: &str, category: SkillCategory) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category,
            params: Vec::new(),
            enabled: true,
            version: "1.0.0".to_string(),
            invocations: 0,
            successes: 0,
            avg_response_time_ms: 0.0,
        }
    }

    pub fn with_param(mut self, param: SkillParam) -> Self {
        self.params.push(param);
        self
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Siker arány számítása
    pub fn success_rate(&self) -> f64 {
        if self.invocations == 0 {
            0.0
        } else {
            self.successes as f64 / self.invocations as f64
        }
    }
}

// ============================================================================
// SKILL HANDLER
// ============================================================================

/// Skill handler trait
#[async_trait]
pub trait SkillHandler: Send + Sync {
    /// Skill végrehajtása
    async fn invoke(
        &self,
        input: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> HopeResult<SkillResult>;

    /// Skill információ
    fn info(&self) -> SkillInfo;
}

/// Skill végrehajtás eredménye
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillResult {
    /// Sikeres volt-e
    pub success: bool,
    /// Kimenet
    pub output: String,
    /// Strukturált adat
    pub data: Option<serde_json::Value>,
    /// Végrehajtási idő (ms)
    pub execution_time_ms: f64,
    /// Metaadatok
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SkillResult {
    pub fn success(output: &str) -> Self {
        Self {
            success: true,
            output: output.to_string(),
            data: None,
            execution_time_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn success_with_data(output: &str, data: serde_json::Value) -> Self {
        Self {
            success: true,
            output: output.to_string(),
            data: Some(data),
            execution_time_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            output: message.to_string(),
            data: None,
            execution_time_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_time(mut self, time_ms: f64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

// ============================================================================
// SKILL REGISTRY
// ============================================================================

/// Skill Registry - Skill-ek nyilvántartása
pub struct SkillRegistry {
    /// Regisztrált skill-ek
    skills: Arc<RwLock<HashMap<String, SkillInfo>>>,
    /// Skill handlerek
    handlers: Arc<RwLock<HashMap<String, Arc<dyn SkillHandler>>>>,
    /// Invocation history
    history: Arc<RwLock<Vec<SkillInvocation>>>,
    /// Max history méret
    max_history: usize,
}

/// Skill meghívás rekord
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillInvocation {
    /// Skill neve
    pub skill_name: String,
    /// Bemenet
    pub input: String,
    /// Eredmény
    pub result: SkillResult,
    /// Időbélyeg
    pub timestamp: f64,
}

impl SkillRegistry {
    /// Új registry létrehozása (szinkron skill init!)
    pub fn new() -> Self {
        // Először létrehozzuk a skill HashMap-et a default skill-ekkel
        let mut initial_skills = HashMap::new();
        for skill in default_skills() {
            initial_skills.insert(skill.name.clone(), skill);
        }

        Self {
            skills: Arc::new(RwLock::new(initial_skills)),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// Skill regisztrálása
    pub async fn register(&self, info: SkillInfo, handler: Option<Arc<dyn SkillHandler>>) {
        let name = info.name.clone();

        {
            let mut skills = self.skills.write().await;
            skills.insert(name.clone(), info);
        }

        if let Some(h) = handler {
            let mut handlers = self.handlers.write().await;
            handlers.insert(name, h);
        }
    }

    /// Skill eltávolítása
    pub async fn unregister(&self, name: &str) {
        {
            let mut skills = self.skills.write().await;
            skills.remove(name);
        }
        {
            let mut handlers = self.handlers.write().await;
            handlers.remove(name);
        }
    }

    /// Skill lekérése
    pub async fn get(&self, name: &str) -> Option<SkillInfo> {
        let skills = self.skills.read().await;
        skills.get(name).cloned()
    }

    /// Összes skill listázása
    pub async fn list(
        &self,
        category: Option<SkillCategory>,
        search: Option<&str>,
    ) -> Vec<SkillInfo> {
        let skills = self.skills.read().await;
        let mut list: Vec<SkillInfo> = skills.values().cloned().collect();

        // Filter by category
        if let Some(cat) = category {
            list.retain(|s| s.category == cat);
        }

        // Filter by search
        if let Some(query) = search {
            let query = query.to_lowercase();
            list.retain(|s| {
                s.name.to_lowercase().contains(&query)
                    || s.description.to_lowercase().contains(&query)
            });
        }

        // Sort by name
        list.sort_by(|a, b| a.name.cmp(&b.name));

        list
    }

    /// Skill meghívása
    pub async fn invoke(
        &self,
        name: &str,
        input: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> HopeResult<SkillResult> {
        let start_time = SystemTime::now();

        // Check if skill exists
        let skill_exists = {
            let skills = self.skills.read().await;
            skills.contains_key(name)
        };

        if !skill_exists {
            return Ok(SkillResult::error(&format!("Skill not found: {}", name)));
        }

        // Check if enabled
        let enabled = {
            let skills = self.skills.read().await;
            skills.get(name).map(|s| s.enabled).unwrap_or(false)
        };

        if !enabled {
            return Ok(SkillResult::error(&format!("Skill disabled: {}", name)));
        }

        // Execute
        let result = {
            let handlers = self.handlers.read().await;
            if let Some(handler) = handlers.get(name) {
                handler.invoke(input, params).await?
            } else {
                // Default mock result
                SkillResult::success(&format!("Skill '{}' executed with input: {}", name, input))
            }
        };

        let execution_time = start_time
            .elapsed()
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        let result = result.with_time(execution_time);

        // Update stats
        {
            let mut skills = self.skills.write().await;
            if let Some(skill) = skills.get_mut(name) {
                skill.invocations += 1;
                if result.success {
                    skill.successes += 1;
                }
                // Moving average for response time
                let n = skill.invocations as f64;
                skill.avg_response_time_ms =
                    (skill.avg_response_time_ms * (n - 1.0) + execution_time) / n;
            }
        }

        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(SkillInvocation {
                skill_name: name.to_string(),
                input: input.to_string(),
                result: result.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            });
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        Ok(result)
    }

    /// Skill engedélyezése/letiltása
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.get_mut(name) {
            skill.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Kategóriák listázása
    pub async fn list_categories(&self) -> Vec<(SkillCategory, usize)> {
        let skills = self.skills.read().await;
        let mut counts: HashMap<SkillCategory, usize> = HashMap::new();

        for skill in skills.values() {
            *counts.entry(skill.category.clone()).or_insert(0) += 1;
        }

        let mut list: Vec<(SkillCategory, usize)> = counts.into_iter().collect();
        list.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
        list
    }

    /// Invocation history
    pub async fn get_history(&self, limit: usize) -> Vec<SkillInvocation> {
        let history = self.history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Statisztikák
    pub async fn get_stats(&self) -> SkillRegistryStats {
        let skills = self.skills.read().await;
        let history = self.history.read().await;

        let total_skills = skills.len();
        let enabled_skills = skills.values().filter(|s| s.enabled).count();
        let total_invocations: u64 = skills.values().map(|s| s.invocations).sum();
        let total_successes: u64 = skills.values().map(|s| s.successes).sum();

        let avg_response_time = if total_invocations > 0 {
            skills
                .values()
                .map(|s| s.avg_response_time_ms * s.invocations as f64)
                .sum::<f64>()
                / total_invocations as f64
        } else {
            0.0
        };

        // Most used skills
        let mut skill_list: Vec<&SkillInfo> = skills.values().collect();
        skill_list.sort_by(|a, b| b.invocations.cmp(&a.invocations));
        let most_used: Vec<String> = skill_list.iter().take(5).map(|s| s.name.clone()).collect();

        SkillRegistryStats {
            total_skills,
            enabled_skills,
            total_invocations,
            total_successes,
            success_rate: if total_invocations > 0 {
                total_successes as f64 / total_invocations as f64
            } else {
                0.0
            },
            avg_response_time_ms: avg_response_time,
            history_size: history.len(),
            most_used_skills: most_used,
        }
    }

    /// Awareness
    pub async fn awareness(&self) -> HashMap<String, serde_json::Value> {
        let stats = self.get_stats().await;
        let categories = self.list_categories().await;

        let mut map = HashMap::new();
        map.insert(
            "total_skills".to_string(),
            serde_json::json!(stats.total_skills),
        );
        map.insert(
            "enabled_skills".to_string(),
            serde_json::json!(stats.enabled_skills),
        );
        map.insert(
            "total_invocations".to_string(),
            serde_json::json!(stats.total_invocations),
        );
        map.insert(
            "success_rate".to_string(),
            serde_json::json!(stats.success_rate),
        );
        map.insert(
            "avg_response_time_ms".to_string(),
            serde_json::json!(stats.avg_response_time_ms),
        );
        map.insert(
            "most_used".to_string(),
            serde_json::json!(stats.most_used_skills),
        );
        map.insert(
            "categories".to_string(),
            serde_json::json!(categories
                .iter()
                .map(|(cat, count)| serde_json::json!({
                    "category": cat.to_string(),
                    "count": count
                }))
                .collect::<Vec<_>>()),
        );
        map
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statisztikák
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillRegistryStats {
    pub total_skills: usize,
    pub enabled_skills: usize,
    pub total_invocations: u64,
    pub total_successes: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub history_size: usize,
    pub most_used_skills: Vec<String>,
}

// ============================================================================
// DEFAULT SKILLS
// ============================================================================

/// Alapértelmezett skill-ek definíciója
fn default_skills() -> Vec<SkillInfo> {
    vec![
        // === CORE ===
        SkillInfo::new("hope_talk", "Beszélgetés Hope-pal", SkillCategory::Core).with_param(
            SkillParam {
                name: "message".to_string(),
                description: "Az üzenet".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ),
        SkillInfo::new(
            "hope_status",
            "Rendszer állapot lekérdezése",
            SkillCategory::Core,
        ),
        SkillInfo::new("hope_introduce", "Hope bemutatkozása", SkillCategory::Core),
        // === MEMORY ===
        SkillInfo::new("hope_remember", "Emlék mentése", SkillCategory::Memory)
            .with_param(SkillParam {
                name: "content".to_string(),
                description: "Mit kell megjegyezni".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            })
            .with_param(SkillParam {
                name: "layer".to_string(),
                description: "Memória réteg".to_string(),
                param_type: "string".to_string(),
                required: false,
                default: Some(serde_json::json!("working")),
            }),
        SkillInfo::new("hope_recall", "Emlékek keresése", SkillCategory::Memory).with_param(
            SkillParam {
                name: "query".to_string(),
                description: "Keresési kifejezés".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ),
        // === COGNITIVE ===
        SkillInfo::new("hope_think", "Mély gondolkodás", SkillCategory::Cognitive)
            .with_param(SkillParam {
                name: "topic".to_string(),
                description: "Miről gondolkodjon".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            })
            .with_param(SkillParam {
                name: "deep".to_string(),
                description: "Mély gondolkodás mód".to_string(),
                param_type: "boolean".to_string(),
                required: false,
                default: Some(serde_json::json!(false)),
            }),
        SkillInfo::new("hope_feel", "Érzelmek beállítása", SkillCategory::Cognitive),
        SkillInfo::new(
            "hope_cognitive_state",
            "Kognitív állapot",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new("hope_dream", "Álom mód", SkillCategory::Cognitive),
        // === CODE ===
        SkillInfo::new("hope_code_analyze", "Kód elemzés", SkillCategory::Code)
            .with_param(SkillParam {
                name: "code".to_string(),
                description: "Elemzendő kód".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            })
            .with_param(SkillParam {
                name: "language".to_string(),
                description: "Programnyelv".to_string(),
                param_type: "string".to_string(),
                required: false,
                default: Some(serde_json::json!("auto")),
            }),
        SkillInfo::new("hope_code_generate", "Kód generálás", SkillCategory::Code).with_param(
            SkillParam {
                name: "description".to_string(),
                description: "Mit generáljon".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ),
        // === SYSTEM ===
        SkillInfo::new(
            "hope_screenshot",
            "Képernyőkép készítése",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_screenshot_ocr",
            "Képernyőkép + OCR",
            SkillCategory::System,
        ),
        SkillInfo::new("hope_notify", "Windows értesítés", SkillCategory::System)
            .with_param(SkillParam {
                name: "title".to_string(),
                description: "Értesítés címe".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            })
            .with_param(SkillParam {
                name: "message".to_string(),
                description: "Értesítés szövege".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            }),
        SkillInfo::new(
            "hope_clipboard_get",
            "Vágólap lekérése",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_clipboard_set",
            "Vágólap beállítása",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_system_stats",
            "Rendszer statisztikák",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_top_processes",
            "Top folyamatok",
            SkillCategory::System,
        ),
        // === WEB ===
        SkillInfo::new("hope_web_search", "Web keresés", SkillCategory::Web).with_param(
            SkillParam {
                name: "query".to_string(),
                description: "Keresési kifejezés".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ),
        SkillInfo::new("hope_web_fetch", "Weboldal letöltése", SkillCategory::Web).with_param(
            SkillParam {
                name: "url".to_string(),
                description: "URL".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            },
        ),
        SkillInfo::new(
            "hope_browser_open",
            "Böngésző megnyitása",
            SkillCategory::Web,
        ),
        SkillInfo::new("hope_browser_action", "Böngésző akció", SkillCategory::Web),
        SkillInfo::new("hope_wikipedia", "Wikipedia keresés", SkillCategory::Web),
        // === MEDIA ===
        SkillInfo::new(
            "hope_speak",
            "Szöveg felolvasása (TTS)",
            SkillCategory::Media,
        )
        .with_param(SkillParam {
            name: "text".to_string(),
            description: "Felolvasandó szöveg".to_string(),
            param_type: "string".to_string(),
            required: true,
            default: None,
        })
        .with_param(SkillParam {
            name: "voice".to_string(),
            description: "Hang (berta, anna)".to_string(),
            param_type: "string".to_string(),
            required: false,
            default: Some(serde_json::json!("berta")),
        }),
        SkillInfo::new(
            "hope_listen",
            "Beszéd felismerés (STT)",
            SkillCategory::Media,
        ),
        SkillInfo::new("hope_image_describe", "Kép leírása", SkillCategory::Media),
        SkillInfo::new(
            "hope_image_generate",
            "Kép generálása",
            SkillCategory::Media,
        ),
        SkillInfo::new("hope_music", "Zene vezérlés", SkillCategory::Media),
        // === FILE ===
        SkillInfo::new(
            "hope_rag_index",
            "Dokumentumok indexelése",
            SkillCategory::File,
        ),
        SkillInfo::new(
            "hope_rag_search",
            "Dokumentumok keresése",
            SkillCategory::File,
        ),
        SkillInfo::new(
            "hope_backup_create",
            "Backup készítése",
            SkillCategory::File,
        ),
        SkillInfo::new(
            "hope_backup_list",
            "Backup-ok listázása",
            SkillCategory::File,
        ),
        SkillInfo::new(
            "hope_backup_restore",
            "Backup visszaállítása",
            SkillCategory::File,
        ),
        // === GIT ===
        SkillInfo::new("hope_git", "Git műveletek", SkillCategory::Git).with_param(SkillParam {
            name: "command".to_string(),
            description: "Git parancs".to_string(),
            param_type: "string".to_string(),
            required: true,
            default: None,
        }),
        // === COMMUNICATION ===
        SkillInfo::new(
            "hope_email_send",
            "Email küldése",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_email_check",
            "Email ellenőrzése",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_calendar_add",
            "Naptár esemény hozzáadása",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_calendar_list",
            "Naptár események listázása",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_schedule",
            "Emlékeztető beállítása",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_list_reminders",
            "Emlékeztetők listázása",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_todo_add",
            "TODO hozzáadása",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_todo_list",
            "TODO-k listázása",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_todo_complete",
            "TODO befejezése",
            SkillCategory::Communication,
        ),
        // === OTHER ===
        SkillInfo::new("hope_weather", "Időjárás lekérdezése", SkillCategory::Other),
        SkillInfo::new("hope_news", "Hírek lekérdezése", SkillCategory::Other),
        SkillInfo::new(
            "hope_exchange_rate",
            "Árfolyam lekérdezése",
            SkillCategory::Other,
        ),
        SkillInfo::new(
            "hope_genome_status",
            "AI Genom állapot",
            SkillCategory::Other,
        ),
        SkillInfo::new(
            "hope_genome_verify",
            "Akció etikai ellenőrzése",
            SkillCategory::Other,
        ),
        SkillInfo::new(
            "hope_alan_status",
            "ALAN önkódoló állapot",
            SkillCategory::Other,
        ),
        SkillInfo::new("hope_alan_analyze", "ALAN önelemzés", SkillCategory::Other),
        SkillInfo::new(
            "hope_plugin_list",
            "Plugin-ok listázása",
            SkillCategory::Other,
        ),
        SkillInfo::new("hope_plugin_load", "Plugin betöltése", SkillCategory::Other),
        // === PYTHON HOPE SKILL-EK ===
        // Agents
        SkillInfo::new(
            "hope_agent_orchestrator",
            "Ágens orchestráció és menedzsment",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_task_planner",
            "Feladat tervezés és ütemezés",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_swarm_intelligence",
            "Raj intelligencia - több ágens koordináció",
            SkillCategory::System,
        ),
        // AI
        SkillInfo::new(
            "hope_lm_cache",
            "Nyelvi modell cache optimalizáció",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_lorax",
            "LoRA fine-tuning rendszer",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_milo",
            "Gépi tanulás asszisztens",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_olmo",
            "Open Language Model integráció",
            SkillCategory::System,
        ),
        // Cognitive (kiegészítés)
        SkillInfo::new(
            "hope_dyna_mind",
            "Dinamikus gondolkodás motor",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new(
            "hope_heart",
            "Érzelem rendszer - 21 dimenzió",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new(
            "hope_creativity",
            "Kreatív gondolkodás és ötletgenerálás",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new(
            "hope_dream_analyzer",
            "Álom elemzés és asszociációk",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new(
            "hope_emotion_analyzer",
            "Érzelem elemzés",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new(
            "hope_emotion_detector",
            "Érzelem detektálás",
            SkillCategory::Cognitive,
        ),
        SkillInfo::new(
            "hope_emotional_context",
            "Érzelmi kontextus memória",
            SkillCategory::Memory,
        ),
        // Communication
        SkillInfo::new(
            "hope_discord_bot",
            "Discord bot integráció",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_telegram_bot",
            "Telegram bot integráció",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_webhook",
            "Webhook kezelés",
            SkillCategory::Communication,
        ),
        SkillInfo::new(
            "hope_typing_monitor",
            "Gépelés és csend figyelés",
            SkillCategory::Communication,
        ),
        // Core (kiegészítés)
        SkillInfo::new("hope_config", "Konfiguráció kezelés", SkillCategory::Core),
        SkillInfo::new("hope_main", "Hope fő belépési pont", SkillCategory::Core),
        SkillInfo::new(
            "hope_mcp_core",
            "MCP (Model Context Protocol) integráció",
            SkillCategory::Core,
        ),
        SkillInfo::new(
            "hope_server",
            "gRPC szerver implementáció",
            SkillCategory::Core,
        ),
        // Development
        SkillInfo::new(
            "hope_code_fixer",
            "Kód javítás automatikusan",
            SkillCategory::Code,
        ),
        SkillInfo::new(
            "hope_document_expert",
            "Dokumentáció szakértő",
            SkillCategory::Code,
        ),
        SkillInfo::new(
            "hope_github_expert",
            "GitHub szakértő - repo kezelés",
            SkillCategory::Code,
        ),
        SkillInfo::new(
            "hope_sandbox",
            "Biztonságos kód futtatás sandbox",
            SkillCategory::Code,
        ),
        SkillInfo::new(
            "hope_testing",
            "Tesztelés és validáció",
            SkillCategory::Code,
        ),
        // Ethics
        SkillInfo::new(
            "hope_ethics",
            "Etikai döntéshozatal és értékelés",
            SkillCategory::Other,
        ),
        SkillInfo::new(
            "hope_genome_integration",
            "AI Genom - alapelvek és szabályok",
            SkillCategory::Other,
        ),
        // Infrastructure
        SkillInfo::new(
            "hope_health_check",
            "Egészség ellenőrzés és monitoring",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_hot_reload",
            "Hot reload - kód újratöltés",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_scheduler",
            "Ütemezett feladatok",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_alerting",
            "Riasztások és értesítések",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_analytics",
            "Analitika és elemzés",
            SkillCategory::System,
        ),
        SkillInfo::new(
            "hope_metrics",
            "Metrikák gyűjtése és riportálás",
            SkillCategory::System,
        ),
        // Memory (kiegészítés)
        SkillInfo::new(
            "hope_embeddings",
            "Szöveg embedding generálás",
            SkillCategory::Memory,
        ),
        SkillInfo::new(
            "hope_faiss_wrapper",
            "FAISS vektor keresés",
            SkillCategory::Memory,
        ),
        SkillInfo::new(
            "hope_chroma_memory",
            "ChromaDB memória",
            SkillCategory::Memory,
        ),
        // Voice
        SkillInfo::new(
            "hope_berta_tts",
            "Berta TTS integráció",
            SkillCategory::Media,
        ),
        SkillInfo::new(
            "hope_whisper_stt",
            "Whisper STT integráció",
            SkillCategory::Media,
        ),
        // Tools
        SkillInfo::new(
            "hope_image_gen",
            "Képgenerálás AI-val",
            SkillCategory::Media,
        ),
        SkillInfo::new(
            "hope_roadmap_manager",
            "Roadmap és projekt tervezés",
            SkillCategory::Other,
        ),
        SkillInfo::new(
            "hope_tui",
            "Terminal UI (TUI) felület",
            SkillCategory::Other,
        ),
        // Web (kiegészítés)
        SkillInfo::new(
            "hope_api_gateway",
            "API gateway és routing",
            SkillCategory::Web,
        ),
        SkillInfo::new("hope_onerec", "OneRec ajánló rendszer", SkillCategory::Web),
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_category() {
        assert_eq!(SkillCategory::from("core"), SkillCategory::Core);
        assert_eq!(SkillCategory::from("memory"), SkillCategory::Memory);
        assert_eq!(SkillCategory::from("COGNITIVE"), SkillCategory::Cognitive);
        assert_eq!(SkillCategory::from("unknown"), SkillCategory::Other);
    }

    #[test]
    fn test_skill_info_creation() {
        let skill = SkillInfo::new("hope_test", "Test skill", SkillCategory::Core)
            .with_param(SkillParam {
                name: "input".to_string(),
                description: "Input param".to_string(),
                param_type: "string".to_string(),
                required: true,
                default: None,
            })
            .with_version("2.0.0");

        assert_eq!(skill.name, "hope_test");
        assert_eq!(skill.params.len(), 1);
        assert_eq!(skill.version, "2.0.0");
    }

    #[test]
    fn test_skill_result() {
        let success = SkillResult::success("OK");
        assert!(success.success);
        assert_eq!(success.output, "OK");

        let with_data = SkillResult::success_with_data("OK", serde_json::json!({"key": "value"}));
        assert!(with_data.data.is_some());

        let error = SkillResult::error("Failed");
        assert!(!error.success);
    }

    #[test]
    fn test_skill_success_rate() {
        let mut skill = SkillInfo::new("test", "Test", SkillCategory::Core);
        assert_eq!(skill.success_rate(), 0.0);

        skill.invocations = 10;
        skill.successes = 8;
        assert!((skill.success_rate() - 0.8).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_skill_registry_creation() {
        let registry = SkillRegistry::new();

        // Wait for default skills to be registered
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let stats = registry.get_stats().await;
        assert!(stats.total_skills > 0);
    }

    #[tokio::test]
    async fn test_skill_registration() {
        let registry = SkillRegistry::new();

        let skill = SkillInfo::new("custom_skill", "Custom skill", SkillCategory::Other);
        registry.register(skill, None).await;

        let retrieved = registry.get("custom_skill").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "custom_skill");
    }

    #[tokio::test]
    async fn test_skill_listing() {
        let registry = SkillRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let all = registry.list(None, None).await;
        assert!(!all.is_empty());

        let core = registry.list(Some(SkillCategory::Core), None).await;
        assert!(core.iter().all(|s| s.category == SkillCategory::Core));

        let search = registry.list(None, Some("think")).await;
        assert!(search.iter().any(|s| s.name.contains("think")));
    }

    #[tokio::test]
    async fn test_skill_invocation() {
        let registry = SkillRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let result = registry
            .invoke("hope_status", "", &HashMap::new())
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.execution_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_skill_enable_disable() {
        let registry = SkillRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Disable
        assert!(registry.set_enabled("hope_status", false).await);

        // Try to invoke
        let result = registry
            .invoke("hope_status", "", &HashMap::new())
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.output.contains("disabled"));

        // Re-enable
        registry.set_enabled("hope_status", true).await;
    }

    #[tokio::test]
    async fn test_categories_listing() {
        let registry = SkillRegistry::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let categories = registry.list_categories().await;
        assert!(!categories.is_empty());
    }

    #[test]
    fn test_default_skills_count() {
        let skills = default_skills();
        // Ellenőrizzük, hogy van elég skill
        assert!(
            skills.len() >= 50,
            "Expected at least 50 skills, got {}",
            skills.len()
        );
    }
}
