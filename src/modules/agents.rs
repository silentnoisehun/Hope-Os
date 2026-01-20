//! Hope Agent Orchestrator - Multi-Agent Koordináció
//!
//! "Egy csapat, egy cél - összehangolva."
//!
//! Features:
//! - Agent lifecycle management
//! - Capability-based task distribution
//! - Communication routing (channels)
//! - Resource allocation & locking
//! - Conflict resolution
//!
//! ()=>[] - Sok agent, egy elme

use crate::core::HopeResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// AGENT STATUS & PRIORITY
// ============================================================================

/// Agent státusz
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Tétlen, feladatra vár
    Idle,
    /// Dolgozik
    Busy,
    /// Várakozik (erőforrásra, másik agentre)
    Waiting,
    /// Hiba történt
    Error,
    /// Leállítva
    Stopped,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "🟢 Idle"),
            AgentStatus::Busy => write!(f, "🔵 Busy"),
            AgentStatus::Waiting => write!(f, "🟡 Waiting"),
            AgentStatus::Error => write!(f, "🔴 Error"),
            AgentStatus::Stopped => write!(f, "⚫ Stopped"),
        }
    }
}

/// Feladat prioritás
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AgentTaskPriority {
    /// Kritikus - azonnal
    Critical = 1,
    /// Magas
    High = 2,
    /// Közepes (default)
    Medium = 3,
    /// Alacsony
    Low = 4,
}

impl Default for AgentTaskPriority {
    fn default() -> Self {
        AgentTaskPriority::Medium
    }
}

// ============================================================================
// AGENT
// ============================================================================

/// Agent képességek
pub type Capabilities = Vec<String>;

/// Agent információ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Egyedi azonosító
    pub agent_id: String,
    /// Név
    pub name: String,
    /// Képességek listája
    pub capabilities: Capabilities,
    /// Státusz
    pub status: AgentStatus,
    /// Jelenlegi feladat ID
    pub current_task: Option<String>,
    /// Létrehozás ideje
    pub created_at: f64,
    /// Befejezett feladatok száma
    pub tasks_completed: u32,
    /// Hibák száma
    pub errors: u32,
    /// Utolsó aktivitás ideje
    pub last_active: f64,
}

impl AgentInfo {
    pub fn new(name: &str, capabilities: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            agent_id: format!(
                "AGT_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            name: name.to_string(),
            capabilities,
            status: AgentStatus::Idle,
            current_task: None,
            created_at: now,
            tasks_completed: 0,
            errors: 0,
            last_active: now,
        }
    }

    /// Van-e adott képessége
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c == cap || c == "*")
    }

    /// Van-e minden szükséges képessége
    pub fn has_all_capabilities(&self, required: &[String]) -> bool {
        required.iter().all(|req| self.has_capability(req))
    }
}

/// Agent handler trait
#[async_trait]
pub trait AgentHandler: Send + Sync {
    /// Feladat végrehajtása
    async fn execute(&self, task: &AgentTask) -> HopeResult<serde_json::Value>;

    /// Üzenet fogadása
    async fn on_message(&self, message: &Message) -> HopeResult<()> {
        let _ = message;
        Ok(())
    }
}

// ============================================================================
// TASK
// ============================================================================

/// Agent feladat
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTask {
    /// Egyedi azonosító
    pub task_id: String,
    /// Feladat neve
    pub name: String,
    /// Szükséges képességek
    pub required_capabilities: Capabilities,
    /// Prioritás
    pub priority: AgentTaskPriority,
    /// Payload (bemeneti adatok)
    pub payload: serde_json::Value,
    /// Hozzárendelt agent ID
    pub assigned_agent: Option<String>,
    /// Státusz
    pub status: TaskStatus,
    /// Eredmény
    pub result: Option<serde_json::Value>,
    /// Létrehozás ideje
    pub created_at: f64,
    /// Befejezés ideje
    pub completed_at: Option<f64>,
}

/// Feladat státusz
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AgentTask {
    pub fn new(name: &str, capabilities: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            task_id: format!(
                "TSK_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            name: name.to_string(),
            required_capabilities: capabilities,
            priority: AgentTaskPriority::default(),
            payload: serde_json::Value::Null,
            assigned_agent: None,
            status: TaskStatus::Pending,
            result: None,
            created_at: now,
            completed_at: None,
        }
    }

    pub fn with_priority(mut self, priority: AgentTaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }
}

// ============================================================================
// COMMUNICATION
// ============================================================================

/// Üzenet típus
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    /// Broadcast csatornán
    Broadcast,
    /// Közvetlen üzenet
    Direct,
    /// Rendszer üzenet
    System,
}

/// Üzenet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Üzenet ID
    pub message_id: String,
    /// Típus
    pub msg_type: MessageType,
    /// Küldő agent ID
    pub sender: Option<String>,
    /// Címzett agent ID (direct esetén)
    pub recipient: Option<String>,
    /// Csatorna (broadcast esetén)
    pub channel: Option<String>,
    /// Tartalom
    pub content: serde_json::Value,
    /// Időbélyeg
    pub timestamp: f64,
}

impl Message {
    pub fn broadcast(channel: &str, sender: &str, content: serde_json::Value) -> Self {
        Self {
            message_id: format!(
                "MSG_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            msg_type: MessageType::Broadcast,
            sender: Some(sender.to_string()),
            recipient: None,
            channel: Some(channel.to_string()),
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    }

    pub fn direct(from: &str, to: &str, content: serde_json::Value) -> Self {
        Self {
            message_id: format!(
                "MSG_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            msg_type: MessageType::Direct,
            sender: Some(from.to_string()),
            recipient: Some(to.to_string()),
            channel: None,
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        }
    }
}

/// Kommunikációs csatorna
#[derive(Clone, Debug)]
pub struct Channel {
    /// Csatorna neve
    pub name: String,
    /// Feliratkozott agentek
    pub subscribers: Vec<String>,
    /// Üzenetek története
    pub history: Vec<Message>,
    /// Max history méret
    pub max_history: usize,
}

impl Channel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            subscribers: Vec::new(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn subscribe(&mut self, agent_id: &str) {
        if !self.subscribers.contains(&agent_id.to_string()) {
            self.subscribers.push(agent_id.to_string());
        }
    }

    pub fn unsubscribe(&mut self, agent_id: &str) {
        self.subscribers.retain(|id| id != agent_id);
    }

    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
}

// ============================================================================
// RESOURCE
// ============================================================================

/// Erőforrás
#[derive(Clone, Debug)]
pub struct Resource {
    /// Erőforrás neve
    pub name: String,
    /// Lefoglaló agent ID
    pub locked_by: Option<String>,
    /// Adat
    pub data: serde_json::Value,
}

impl Resource {
    pub fn new(name: &str, data: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            locked_by: None,
            data,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked_by.is_some()
    }

    pub fn lock(&mut self, agent_id: &str) -> bool {
        if self.locked_by.is_none() {
            self.locked_by = Some(agent_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn unlock(&mut self, agent_id: &str) -> bool {
        if self.locked_by.as_deref() == Some(agent_id) {
            self.locked_by = None;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// ORCHESTRATOR
// ============================================================================

/// Orchestrator konfiguráció
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Max várakozó feladatok
    pub max_queue_size: usize,
    /// Max befejezett feladatok history
    pub max_completed_history: usize,
    /// Scheduler interval (ms)
    pub scheduler_interval_ms: u64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            max_completed_history: 500,
            scheduler_interval_ms: 100,
        }
    }
}

/// Agent Orchestrator - Multi-agent koordinátor
pub struct AgentOrchestrator {
    /// Konfiguráció
    config: OrchestratorConfig,
    /// Regisztrált agentek
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// Agent handlerek
    handlers: Arc<RwLock<HashMap<String, Arc<dyn AgentHandler>>>>,
    /// Feladat sor (prioritás szerint rendezve)
    task_queue: Arc<RwLock<Vec<AgentTask>>>,
    /// Befejezett feladatok
    completed_tasks: Arc<RwLock<Vec<AgentTask>>>,
    /// Kommunikációs csatornák
    channels: Arc<RwLock<HashMap<String, Channel>>>,
    /// Erőforrások
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    /// Statisztikák
    stats: Arc<RwLock<OrchestratorStats>>,
    /// Fut-e az orchestrator
    running: Arc<RwLock<bool>>,
}

/// Orchestrator statisztikák
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrchestratorStats {
    /// Regisztrált agentek száma
    pub agents_registered: u32,
    /// Létrehozott feladatok
    pub tasks_created: u32,
    /// Befejezett feladatok
    pub tasks_completed: u32,
    /// Sikertelen feladatok
    pub tasks_failed: u32,
    /// Továbbított üzenetek
    pub messages_routed: u32,
}

impl AgentOrchestrator {
    /// Új orchestrator létrehozása
    pub fn new() -> Self {
        Self::with_config(OrchestratorConfig::default())
    }

    /// Orchestrator létrehozása konfigurációval
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(OrchestratorStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    // === AGENT MANAGEMENT ===

    /// Agent regisztrálása
    pub async fn register_agent(
        &self,
        name: &str,
        capabilities: Vec<String>,
        handler: Option<Arc<dyn AgentHandler>>,
    ) -> HopeResult<AgentInfo> {
        let agent = AgentInfo::new(name, capabilities);
        let agent_id = agent.agent_id.clone();

        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent.clone());
        }

        if let Some(h) = handler {
            let mut handlers = self.handlers.write().await;
            handlers.insert(agent_id.clone(), h);
        }

        {
            let mut stats = self.stats.write().await;
            stats.agents_registered += 1;
        }

        Ok(agent)
    }

    /// Agent eltávolítása
    pub async fn unregister_agent(&self, agent_id: &str) -> HopeResult<()> {
        // Remove from agents
        {
            let mut agents = self.agents.write().await;
            if let Some(mut agent) = agents.remove(agent_id) {
                agent.status = AgentStatus::Stopped;
            }
        }

        // Remove handler
        {
            let mut handlers = self.handlers.write().await;
            handlers.remove(agent_id);
        }

        // Release any held resources
        {
            let mut resources = self.resources.write().await;
            for resource in resources.values_mut() {
                if resource.locked_by.as_deref() == Some(agent_id) {
                    resource.locked_by = None;
                }
            }
        }

        // Unsubscribe from all channels
        {
            let mut channels = self.channels.write().await;
            for channel in channels.values_mut() {
                channel.unsubscribe(agent_id);
            }
        }

        Ok(())
    }

    /// Agent lekérése
    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Összes agent listázása
    pub async fn list_agents(&self, status_filter: Option<AgentStatus>) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        let mut list: Vec<AgentInfo> = agents.values().cloned().collect();

        if let Some(status) = status_filter {
            list.retain(|a| a.status == status);
        }

        list
    }

    // === TASK MANAGEMENT ===

    /// Feladat létrehozása és sorba állítása
    pub async fn create_task(
        &self,
        name: &str,
        capabilities: Vec<String>,
        priority: AgentTaskPriority,
        payload: serde_json::Value,
    ) -> HopeResult<AgentTask> {
        let task = AgentTask::new(name, capabilities)
            .with_priority(priority)
            .with_payload(payload);

        // Insert by priority
        {
            let mut queue = self.task_queue.write().await;

            let pos = queue
                .iter()
                .position(|t| t.priority > task.priority)
                .unwrap_or(queue.len());

            queue.insert(pos, task.clone());
        }

        {
            let mut stats = self.stats.write().await;
            stats.tasks_created += 1;
        }

        Ok(task)
    }

    /// Megfelelő agent keresése feladathoz
    async fn find_agent_for_task(&self, task: &AgentTask) -> Option<String> {
        let agents = self.agents.read().await;

        let mut candidates: Vec<&AgentInfo> = agents
            .values()
            .filter(|a| {
                a.status == AgentStatus::Idle && a.has_all_capabilities(&task.required_capabilities)
            })
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Rendezés: kevesebb hiba, több befejezett feladat
        candidates.sort_by(|a, b| {
            let score_a = a.errors as i32 - a.tasks_completed as i32;
            let score_b = b.errors as i32 - b.tasks_completed as i32;
            score_a.cmp(&score_b)
        });

        candidates.first().map(|a| a.agent_id.clone())
    }

    /// Feladat végrehajtása
    pub async fn execute_task(&self, task_id: &str) -> HopeResult<Option<serde_json::Value>> {
        // Find task
        let task = {
            let mut queue = self.task_queue.write().await;
            let pos = queue.iter().position(|t| t.task_id == task_id);
            pos.map(|p| queue.remove(p))
        };

        let mut task = match task {
            Some(t) => t,
            None => return Ok(None),
        };

        // Find agent
        let agent_id = match self.find_agent_for_task(&task).await {
            Some(id) => id,
            None => {
                // Put back in queue
                let mut queue = self.task_queue.write().await;
                queue.push(task);
                return Ok(None);
            }
        };

        // Assign task
        task.assigned_agent = Some(agent_id.clone());
        task.status = TaskStatus::Running;

        // Update agent status
        {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&agent_id) {
                agent.status = AgentStatus::Busy;
                agent.current_task = Some(task.task_id.clone());
                agent.last_active = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
            }
        }

        // Execute
        let result = {
            let handlers = self.handlers.read().await;
            if let Some(handler) = handlers.get(&agent_id) {
                handler.execute(&task).await
            } else {
                // Default handler
                Ok(serde_json::json!({"status": "completed", "task": task.name}))
            }
        };

        // Update task and agent
        let (task_result, success) = match result {
            Ok(res) => {
                task.status = TaskStatus::Completed;
                task.result = Some(res.clone());
                (Some(res), true)
            }
            Err(e) => {
                task.status = TaskStatus::Failed;
                task.result = Some(serde_json::json!({"error": e.to_string()}));
                (None, false)
            }
        };

        task.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );

        // Update agent
        {
            let mut agents = self.agents.write().await;
            if let Some(agent) = agents.get_mut(&agent_id) {
                agent.status = AgentStatus::Idle;
                agent.current_task = None;
                if success {
                    agent.tasks_completed += 1;
                } else {
                    agent.errors += 1;
                }
            }
        }

        // Add to completed
        {
            let mut completed = self.completed_tasks.write().await;
            completed.push(task);
            if completed.len() > self.config.max_completed_history {
                completed.remove(0);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            if success {
                stats.tasks_completed += 1;
            } else {
                stats.tasks_failed += 1;
            }
        }

        Ok(task_result)
    }

    /// Várakozó feladatok feldolgozása
    pub async fn process_queue(&self) -> HopeResult<u32> {
        let mut processed = 0;

        loop {
            // Get next pending task
            let task_id = {
                let queue = self.task_queue.read().await;
                queue
                    .iter()
                    .find(|t| t.status == TaskStatus::Pending)
                    .map(|t| t.task_id.clone())
            };

            match task_id {
                Some(id) => {
                    if self.execute_task(&id).await?.is_some() {
                        processed += 1;
                    } else {
                        break; // No available agent
                    }
                }
                None => break, // No more pending tasks
            }
        }

        Ok(processed)
    }

    // === COMMUNICATION ===

    /// Csatorna létrehozása
    pub async fn create_channel(&self, name: &str) {
        let mut channels = self.channels.write().await;
        if !channels.contains_key(name) {
            channels.insert(name.to_string(), Channel::new(name));
        }
    }

    /// Feliratkozás csatornára
    pub async fn subscribe(&self, agent_id: &str, channel_name: &str) {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(channel_name) {
            channel.subscribe(agent_id);
        } else {
            let mut channel = Channel::new(channel_name);
            channel.subscribe(agent_id);
            channels.insert(channel_name.to_string(), channel);
        }
    }

    /// Leiratkozás csatornáról
    pub async fn unsubscribe(&self, agent_id: &str, channel_name: &str) {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(channel_name) {
            channel.unsubscribe(agent_id);
        }
    }

    /// Broadcast üzenet küldése
    pub async fn broadcast(
        &self,
        channel_name: &str,
        sender_id: &str,
        content: serde_json::Value,
    ) -> HopeResult<()> {
        let message = Message::broadcast(channel_name, sender_id, content);

        // Get subscribers
        let subscribers = {
            let mut channels = self.channels.write().await;
            if let Some(channel) = channels.get_mut(channel_name) {
                channel.add_message(message.clone());
                channel
                    .subscribers
                    .iter()
                    .filter(|s| *s != sender_id)
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                return Ok(());
            }
        };

        // Notify subscribers
        let handlers = self.handlers.read().await;
        for sub_id in subscribers {
            if let Some(handler) = handlers.get(&sub_id) {
                let _ = handler.on_message(&message).await;
            }
        }

        {
            let mut stats = self.stats.write().await;
            stats.messages_routed += 1;
        }

        Ok(())
    }

    /// Közvetlen üzenet küldése
    pub async fn send_direct(
        &self,
        from_id: &str,
        to_id: &str,
        content: serde_json::Value,
    ) -> HopeResult<()> {
        let message = Message::direct(from_id, to_id, content);

        let handlers = self.handlers.read().await;
        if let Some(handler) = handlers.get(to_id) {
            handler.on_message(&message).await?;
        }

        {
            let mut stats = self.stats.write().await;
            stats.messages_routed += 1;
        }

        Ok(())
    }

    // === RESOURCES ===

    /// Erőforrás regisztrálása
    pub async fn register_resource(&self, name: &str, data: serde_json::Value) {
        let mut resources = self.resources.write().await;
        resources.insert(name.to_string(), Resource::new(name, data));
    }

    /// Erőforrás lefoglalása
    pub async fn acquire_resource(&self, agent_id: &str, resource_name: &str) -> bool {
        let mut resources = self.resources.write().await;
        if let Some(resource) = resources.get_mut(resource_name) {
            resource.lock(agent_id)
        } else {
            false
        }
    }

    /// Erőforrás felszabadítása
    pub async fn release_resource(&self, agent_id: &str, resource_name: &str) -> bool {
        let mut resources = self.resources.write().await;
        if let Some(resource) = resources.get_mut(resource_name) {
            resource.unlock(agent_id)
        } else {
            false
        }
    }

    /// Erőforrás lekérése
    pub async fn get_resource(
        &self,
        agent_id: &str,
        resource_name: &str,
    ) -> Option<serde_json::Value> {
        let resources = self.resources.read().await;
        if let Some(resource) = resources.get(resource_name) {
            // Either locked by this agent or not locked at all
            if resource.locked_by.as_deref() == Some(agent_id) || resource.locked_by.is_none() {
                return Some(resource.data.clone());
            }
        }
        None
    }

    // === STATUS ===

    /// Orchestrator státusz
    pub async fn get_status(&self) -> OrchestratorStatus {
        let agents = self.agents.read().await;
        let queue = self.task_queue.read().await;
        let completed = self.completed_tasks.read().await;
        let channels = self.channels.read().await;
        let resources = self.resources.read().await;
        let stats = self.stats.read().await;
        let running = self.running.read().await;

        OrchestratorStatus {
            running: *running,
            total_agents: agents.len(),
            idle_agents: agents
                .values()
                .filter(|a| a.status == AgentStatus::Idle)
                .count(),
            busy_agents: agents
                .values()
                .filter(|a| a.status == AgentStatus::Busy)
                .count(),
            queued_tasks: queue.len(),
            completed_tasks: completed.len(),
            channels: channels.len(),
            resources: resources.len(),
            stats: stats.clone(),
        }
    }

    /// Awareness
    pub async fn awareness(&self) -> HashMap<String, serde_json::Value> {
        let status = self.get_status().await;
        let agents = self.list_agents(None).await;

        let mut map = HashMap::new();
        map.insert("type".to_string(), serde_json::json!("AgentOrchestrator"));
        map.insert(
            "purpose".to_string(),
            serde_json::json!("Multi-agent coordination"),
        );
        map.insert("running".to_string(), serde_json::json!(status.running));
        map.insert(
            "total_agents".to_string(),
            serde_json::json!(status.total_agents),
        );
        map.insert(
            "idle_agents".to_string(),
            serde_json::json!(status.idle_agents),
        );
        map.insert(
            "busy_agents".to_string(),
            serde_json::json!(status.busy_agents),
        );
        map.insert(
            "queued_tasks".to_string(),
            serde_json::json!(status.queued_tasks),
        );
        map.insert(
            "agents".to_string(),
            serde_json::json!(agents
                .iter()
                .map(|a| serde_json::json!({
                    "id": a.agent_id,
                    "name": a.name,
                    "status": format!("{}", a.status),
                    "capabilities": a.capabilities,
                    "tasks_completed": a.tasks_completed
                }))
                .collect::<Vec<_>>()),
        );
        map
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrator státusz
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    pub running: bool,
    pub total_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub queued_tasks: usize,
    pub completed_tasks: usize,
    pub channels: usize,
    pub resources: usize,
    pub stats: OrchestratorStats,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_creation() {
        let agent = AgentInfo::new(
            "test_agent",
            vec!["analyze".to_string(), "refactor".to_string()],
        );

        assert!(!agent.agent_id.is_empty());
        assert_eq!(agent.name, "test_agent");
        assert_eq!(agent.capabilities.len(), 2);
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn test_agent_capabilities() {
        let agent = AgentInfo::new("test", vec!["code".to_string(), "test".to_string()]);

        assert!(agent.has_capability("code"));
        assert!(agent.has_capability("test"));
        assert!(!agent.has_capability("deploy"));
        assert!(agent.has_all_capabilities(&vec!["code".to_string(), "test".to_string()]));
    }

    #[test]
    fn test_wildcard_capability() {
        let agent = AgentInfo::new("super_agent", vec!["*".to_string()]);

        assert!(agent.has_capability("anything"));
        assert!(agent.has_capability("code"));
        assert!(agent.has_all_capabilities(&vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string()
        ]));
    }

    #[test]
    fn test_task_creation() {
        let task = AgentTask::new("analyze_code", vec!["analyze".to_string()])
            .with_priority(AgentTaskPriority::High)
            .with_payload(serde_json::json!({"file": "main.rs"}));

        assert!(!task.task_id.is_empty());
        assert_eq!(task.priority, AgentTaskPriority::High);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_message_creation() {
        let broadcast =
            Message::broadcast("updates", "agent_1", serde_json::json!({"type": "status"}));
        assert!(matches!(broadcast.msg_type, MessageType::Broadcast));
        assert_eq!(broadcast.channel, Some("updates".to_string()));

        let direct = Message::direct("agent_1", "agent_2", serde_json::json!({"hello": "world"}));
        assert!(matches!(direct.msg_type, MessageType::Direct));
        assert_eq!(direct.recipient, Some("agent_2".to_string()));
    }

    #[test]
    fn test_channel() {
        let mut channel = Channel::new("test_channel");

        channel.subscribe("agent_1");
        channel.subscribe("agent_2");
        assert_eq!(channel.subscribers.len(), 2);

        channel.unsubscribe("agent_1");
        assert_eq!(channel.subscribers.len(), 1);
    }

    #[test]
    fn test_resource_locking() {
        let mut resource =
            Resource::new("database", serde_json::json!({"connection": "localhost"}));

        assert!(!resource.is_locked());
        assert!(resource.lock("agent_1"));
        assert!(resource.is_locked());
        assert!(!resource.lock("agent_2")); // Already locked

        assert!(resource.unlock("agent_1"));
        assert!(!resource.is_locked());
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = AgentOrchestrator::new();
        let status = orchestrator.get_status().await;

        assert_eq!(status.total_agents, 0);
        assert_eq!(status.queued_tasks, 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let orchestrator = AgentOrchestrator::new();

        let agent = orchestrator
            .register_agent("coder", vec!["code".to_string(), "test".to_string()], None)
            .await
            .unwrap();

        assert_eq!(agent.name, "coder");

        let agents = orchestrator.list_agents(None).await;
        assert_eq!(agents.len(), 1);
    }

    #[tokio::test]
    async fn test_task_creation_and_queue() {
        let orchestrator = AgentOrchestrator::new();

        // Create tasks with different priorities
        orchestrator
            .create_task(
                "low_task",
                vec!["code".to_string()],
                AgentTaskPriority::Low,
                serde_json::json!({}),
            )
            .await
            .unwrap();

        orchestrator
            .create_task(
                "high_task",
                vec!["code".to_string()],
                AgentTaskPriority::High,
                serde_json::json!({}),
            )
            .await
            .unwrap();

        let status = orchestrator.get_status().await;
        assert_eq!(status.queued_tasks, 2);
    }

    #[tokio::test]
    async fn test_channel_communication() {
        let orchestrator = AgentOrchestrator::new();

        orchestrator.create_channel("updates").await;
        orchestrator.subscribe("agent_1", "updates").await;
        orchestrator.subscribe("agent_2", "updates").await;

        // Broadcast should work
        orchestrator
            .broadcast("updates", "agent_1", serde_json::json!({"msg": "hello"}))
            .await
            .unwrap();

        let status = orchestrator.get_status().await;
        assert_eq!(status.stats.messages_routed, 1);
    }

    #[tokio::test]
    async fn test_resource_management() {
        let orchestrator = AgentOrchestrator::new();

        orchestrator
            .register_resource("db", serde_json::json!({"url": "localhost"}))
            .await;

        assert!(orchestrator.acquire_resource("agent_1", "db").await);
        assert!(!orchestrator.acquire_resource("agent_2", "db").await);

        let data = orchestrator.get_resource("agent_1", "db").await;
        assert!(data.is_some());

        assert!(orchestrator.release_resource("agent_1", "db").await);
        assert!(orchestrator.acquire_resource("agent_2", "db").await);
    }
}
