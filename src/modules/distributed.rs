//! Hope Distributed Coordination - Elosztott koordináció
//!
//! Raft konszenzus, leader election, heartbeat monitoring.
//! ()=>[] - A tiszta potenciálból minden megszületik

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};

use crate::core::{HopeError, HopeResult};

// ============================================================================
// SYSTEM STATE
// ============================================================================

/// Rendszer állapotok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemState {
    Initializing,
    Healthy,
    Degraded,
    Critical,
    Recovery,
    Shutdown,
}

impl Default for SystemState {
    fn default() -> Self {
        Self::Initializing
    }
}

// ============================================================================
// CONFIG OPERATION
// ============================================================================

/// Konfigurációs műveletek
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigOperation {
    Set,
    Delete,
    Update,
    Watch,
}

// ============================================================================
// CONFIG ENTRY
// ============================================================================

/// Konfigurációs bejegyzés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// Kulcs
    pub key: String,
    /// Érték (JSON)
    pub value: serde_json::Value,
    /// Verzió
    pub version: u64,
    /// Időbélyeg
    pub timestamp: u64,
    /// Checksum
    pub checksum: String,
}

impl ConfigEntry {
    /// Új bejegyzés
    pub fn new(key: &str, value: serde_json::Value) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut entry = Self {
            key: key.to_string(),
            value,
            version: 1,
            timestamp,
            checksum: String::new(),
        };
        entry.checksum = entry.calculate_checksum();
        entry
    }

    /// Checksum számítása
    pub fn calculate_checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.key.hash(&mut hasher);
        self.value.to_string().hash(&mut hasher);
        self.version.hash(&mut hasher);
        self.timestamp.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Checksum validálása
    pub fn validate_checksum(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }

    /// Verzió növelése
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.checksum = self.calculate_checksum();
    }
}

// ============================================================================
// CONFIG CHANGE
// ============================================================================

/// Konfigurációs változás
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    /// Művelet
    pub operation: ConfigOperation,
    /// Kulcs
    pub key: String,
    /// Új érték
    pub value: Option<serde_json::Value>,
    /// Régi érték
    pub old_value: Option<serde_json::Value>,
    /// Időbélyeg
    pub timestamp: u64,
    /// Kezdeményező
    pub initiator: String,
}

impl ConfigChange {
    /// Új SET változás
    pub fn set(key: &str, value: serde_json::Value, initiator: &str) -> Self {
        Self {
            operation: ConfigOperation::Set,
            key: key.to_string(),
            value: Some(value),
            old_value: None,
            timestamp: Self::now(),
            initiator: initiator.to_string(),
        }
    }

    /// Új DELETE változás
    pub fn delete(key: &str, old_value: serde_json::Value, initiator: &str) -> Self {
        Self {
            operation: ConfigOperation::Delete,
            key: key.to_string(),
            value: None,
            old_value: Some(old_value),
            timestamp: Self::now(),
            initiator: initiator.to_string(),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================================
// NODE INFO
// ============================================================================

/// Node információ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node ID
    pub node_id: String,
    /// IP cím
    pub ip_address: String,
    /// Port
    pub port: u16,
    /// Egészséges-e
    pub healthy: bool,
    /// Utolsó heartbeat
    pub last_heartbeat: u64,
    /// Role
    pub role: NodeRole,
}

/// Node role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

impl Default for NodeRole {
    fn default() -> Self {
        Self::Follower
    }
}

// ============================================================================
// HEARTBEAT CONFIG
// ============================================================================

/// Heartbeat konfiguráció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    /// Heartbeat intervallum (ms)
    pub interval_ms: u64,
    /// Timeout (ms)
    pub timeout_ms: u64,
    /// Max hibák failover előtt
    pub max_failures: u32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1000,
            timeout_ms: 5000,
            max_failures: 3,
        }
    }
}

// ============================================================================
// ELECTION CONFIG
// ============================================================================

/// Leader election konfiguráció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionConfig {
    /// Election timeout min (ms)
    pub timeout_min_ms: u64,
    /// Election timeout max (ms)
    pub timeout_max_ms: u64,
    /// Leader lease time (sec)
    pub leader_lease_time: u64,
}

impl Default for ElectionConfig {
    fn default() -> Self {
        Self {
            timeout_min_ms: 150,
            timeout_max_ms: 300,
            leader_lease_time: 300,
        }
    }
}

// ============================================================================
// ORCHESTRATOR CONFIG
// ============================================================================

/// Elosztott orchestrator konfiguráció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Node ID
    pub node_id: String,
    /// IP cím
    pub ip_address: String,
    /// Port
    pub port: u16,
    /// Peer node-ok
    pub peer_nodes: Vec<NodeInfo>,
    /// Heartbeat konfiguráció
    pub heartbeat_config: HeartbeatConfig,
    /// Election konfiguráció
    pub election_config: ElectionConfig,
    /// Auto failover
    pub auto_failover: bool,
    /// Load balancing
    pub load_balancing_enabled: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            node_id: "hope-node-01".to_string(),
            ip_address: "127.0.0.1".to_string(),
            port: 50051,
            peer_nodes: Vec::new(),
            heartbeat_config: HeartbeatConfig::default(),
            election_config: ElectionConfig::default(),
            auto_failover: true,
            load_balancing_enabled: true,
        }
    }
}

// ============================================================================
// DISTRIBUTED CONFIG MANAGER
// ============================================================================

/// Distributed Config Manager statisztikák
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigManagerStats {
    pub total_keys: u64,
    pub total_changes: u64,
    pub total_watchers: u64,
    pub syncs_performed: u64,
}

/// Elosztott konfiguráció manager
pub struct DistributedConfigManager {
    /// Node ID
    node_id: String,
    /// Config store
    config_store: Arc<RwLock<HashMap<String, ConfigEntry>>>,
    /// Watchers (key -> callbacks)
    watchers: Arc<RwLock<HashMap<String, Vec<mpsc::Sender<ConfigChange>>>>>,
    /// Is leader
    is_leader: Arc<RwLock<bool>>,
    /// Running
    running: Arc<RwLock<bool>>,
    /// Stats
    stats: Arc<RwLock<ConfigManagerStats>>,
}

impl DistributedConfigManager {
    /// Új config manager
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            config_store: Arc::new(RwLock::new(HashMap::new())),
            watchers: Arc::new(RwLock::new(HashMap::new())),
            is_leader: Arc::new(RwLock::new(false)),
            running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(ConfigManagerStats::default())),
        }
    }

    /// Start management
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        *running = true;
    }

    /// Stop management
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Set leader status
    pub async fn set_leader(&self, is_leader: bool) {
        let mut leader = self.is_leader.write().await;
        *leader = is_leader;
    }

    /// Set config value
    pub async fn set_config(&self, key: &str, value: serde_json::Value) -> HopeResult<()> {
        let is_leader = *self.is_leader.read().await;
        if !is_leader {
            return Err(HopeError::General("Only leader can modify configuration".to_string()));
        }

        let mut store = self.config_store.write().await;
        let old_value = store.get(key).map(|e| e.value.clone());

        let entry = if let Some(existing) = store.get_mut(key) {
            existing.value = value.clone();
            existing.increment_version();
            existing.clone()
        } else {
            let entry = ConfigEntry::new(key, value.clone());
            store.insert(key.to_string(), entry.clone());
            entry
        };

        // Notify watchers
        let change = ConfigChange {
            operation: ConfigOperation::Set,
            key: key.to_string(),
            value: Some(value),
            old_value,
            timestamp: entry.timestamp,
            initiator: self.node_id.clone(),
        };
        self.notify_watchers(key, change).await;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_keys = store.len() as u64;
            stats.total_changes += 1;
        }

        Ok(())
    }

    /// Get config value
    pub async fn get_config(&self, key: &str) -> Option<serde_json::Value> {
        let store = self.config_store.read().await;
        store.get(key).map(|e| e.value.clone())
    }

    /// Delete config
    pub async fn delete_config(&self, key: &str) -> HopeResult<()> {
        let is_leader = *self.is_leader.read().await;
        if !is_leader {
            return Err(HopeError::General("Only leader can modify configuration".to_string()));
        }

        let mut store = self.config_store.write().await;
        let old_value = store.remove(key).map(|e| e.value);

        if let Some(old_val) = old_value {
            let change = ConfigChange::delete(key, old_val, &self.node_id);
            self.notify_watchers(key, change).await;
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_keys = store.len() as u64;
            stats.total_changes += 1;
        }

        Ok(())
    }

    /// Watch config key
    pub async fn watch_config(&self, key: &str) -> mpsc::Receiver<ConfigChange> {
        let (tx, rx) = mpsc::channel(100);
        let mut watchers = self.watchers.write().await;
        watchers.entry(key.to_string()).or_insert_with(Vec::new).push(tx);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_watchers += 1;
        }

        rx
    }

    /// Notify watchers
    async fn notify_watchers(&self, key: &str, change: ConfigChange) {
        let watchers = self.watchers.read().await;
        if let Some(senders) = watchers.get(key) {
            for sender in senders {
                let _ = sender.send(change.clone()).await;
            }
        }
    }

    /// Get all config keys
    pub async fn get_keys(&self) -> Vec<String> {
        let store = self.config_store.read().await;
        store.keys().cloned().collect()
    }

    /// Get config snapshot
    pub async fn get_snapshot(&self) -> HashMap<String, ConfigEntry> {
        self.config_store.read().await.clone()
    }

    /// Sync from leader
    pub async fn sync_from_leader(&self, leader_config: HashMap<String, ConfigEntry>) {
        let mut store = self.config_store.write().await;
        let mut synced = 0;

        for (key, entry) in leader_config {
            if let Some(existing) = store.get(&key) {
                if entry.version > existing.version {
                    store.insert(key, entry);
                    synced += 1;
                }
            } else {
                store.insert(key, entry);
                synced += 1;
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_keys = store.len() as u64;
            stats.syncs_performed += 1;
        }
    }

    /// Get stats
    pub async fn get_stats(&self) -> ConfigManagerStats {
        self.stats.read().await.clone()
    }
}

// ============================================================================
// HEARTBEAT MONITOR
// ============================================================================

/// Heartbeat Monitor
pub struct HeartbeatMonitor {
    /// Config
    config: HeartbeatConfig,
    /// Nodes
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    /// Running
    running: Arc<RwLock<bool>>,
    /// Failure callbacks
    on_failure: Arc<RwLock<Vec<Box<dyn Fn(&str) + Send + Sync>>>>,
    /// Recovery callbacks
    on_recovery: Arc<RwLock<Vec<Box<dyn Fn(&str) + Send + Sync>>>>,
}

impl HeartbeatMonitor {
    /// Új monitor
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            on_failure: Arc::new(RwLock::new(Vec::new())),
            on_recovery: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register node
    pub async fn register_node(&self, node: NodeInfo) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.node_id.clone(), node);
    }

    /// Record heartbeat
    pub async fn record_heartbeat(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            let was_unhealthy = !node.healthy;
            node.healthy = true;
            node.last_heartbeat = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if was_unhealthy {
                // Node recovered
                let callbacks = self.on_recovery.read().await;
                for callback in callbacks.iter() {
                    callback(node_id);
                }
            }
        }
    }

    /// Check node health
    pub async fn check_health(&self) -> HashMap<String, bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let timeout_secs = self.config.timeout_ms / 1000;
        let mut nodes = self.nodes.write().await;
        let mut health_map = HashMap::new();

        for (node_id, node) in nodes.iter_mut() {
            let was_healthy = node.healthy;
            let elapsed = now.saturating_sub(node.last_heartbeat);
            node.healthy = elapsed < timeout_secs;

            health_map.insert(node_id.clone(), node.healthy);

            if was_healthy && !node.healthy {
                // Node failed
                let callbacks = self.on_failure.read().await;
                for callback in callbacks.iter() {
                    callback(node_id);
                }
            }
        }

        health_map
    }

    /// Get healthy nodes
    pub async fn get_healthy_nodes(&self) -> Vec<String> {
        let nodes = self.nodes.read().await;
        nodes.iter()
            .filter(|(_, n)| n.healthy)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let nodes = self.nodes.read().await;
        let total = nodes.len();
        let healthy = nodes.iter().filter(|(_, n)| n.healthy).count();

        HealthStatus {
            total_nodes: total,
            healthy_nodes: healthy,
            failed_nodes: total - healthy,
            nodes: nodes.values().cloned().collect(),
        }
    }
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub failed_nodes: usize,
    pub nodes: Vec<NodeInfo>,
}

// ============================================================================
// LEADER ELECTION
// ============================================================================

/// Election state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectionState {
    Follower,
    Candidate,
    Leader,
}

/// Leader Election
pub struct LeaderElection {
    /// Node ID
    node_id: String,
    /// Peers
    peers: Vec<String>,
    /// Current term
    term: Arc<RwLock<u64>>,
    /// State
    state: Arc<RwLock<ElectionState>>,
    /// Leader ID
    leader_id: Arc<RwLock<Option<String>>>,
    /// Voted for (in current term)
    voted_for: Arc<RwLock<Option<String>>>,
    /// Config
    config: ElectionConfig,
}

impl LeaderElection {
    /// Új election
    pub fn new(node_id: &str, peers: Vec<String>, config: ElectionConfig) -> Self {
        Self {
            node_id: node_id.to_string(),
            peers,
            term: Arc::new(RwLock::new(0)),
            state: Arc::new(RwLock::new(ElectionState::Follower)),
            leader_id: Arc::new(RwLock::new(None)),
            voted_for: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Start election
    pub async fn start_election(&self) -> HopeResult<()> {
        // Increment term
        {
            let mut term = self.term.write().await;
            *term += 1;
        }

        // Become candidate
        {
            let mut state = self.state.write().await;
            *state = ElectionState::Candidate;
        }

        // Vote for self
        {
            let mut voted_for = self.voted_for.write().await;
            *voted_for = Some(self.node_id.clone());
        }

        // In real implementation, would send vote requests to peers
        // For now, if we're the only node, become leader
        if self.peers.is_empty() {
            self.become_leader().await;
        }

        Ok(())
    }

    /// Become leader
    pub async fn become_leader(&self) {
        let mut state = self.state.write().await;
        *state = ElectionState::Leader;

        let mut leader_id = self.leader_id.write().await;
        *leader_id = Some(self.node_id.clone());
    }

    /// Step down (become follower)
    pub async fn step_down(&self) {
        let mut state = self.state.write().await;
        *state = ElectionState::Follower;
    }

    /// Receive vote request
    pub async fn receive_vote_request(
        &self,
        candidate_id: &str,
        candidate_term: u64,
    ) -> (bool, u64) {
        let mut term = self.term.write().await;
        let mut voted_for = self.voted_for.write().await;

        // If candidate's term is higher, update our term
        if candidate_term > *term {
            *term = candidate_term;
            *voted_for = None;
            let mut state = self.state.write().await;
            *state = ElectionState::Follower;
        }

        // Grant vote if we haven't voted yet in this term
        let vote_granted = candidate_term >= *term
            && (voted_for.is_none() || voted_for.as_ref() == Some(&candidate_id.to_string()));

        if vote_granted {
            *voted_for = Some(candidate_id.to_string());
        }

        (vote_granted, *term)
    }

    /// Get current state
    pub async fn get_state(&self) -> ElectionState {
        *self.state.read().await
    }

    /// Get current term
    pub async fn get_term(&self) -> u64 {
        *self.term.read().await
    }

    /// Get leader ID
    pub async fn get_leader_id(&self) -> Option<String> {
        self.leader_id.read().await.clone()
    }

    /// Is leader
    pub async fn is_leader(&self) -> bool {
        *self.state.read().await == ElectionState::Leader
    }

    /// Get election status
    pub async fn get_status(&self) -> ElectionStatus {
        ElectionStatus {
            node_id: self.node_id.clone(),
            state: *self.state.read().await,
            term: *self.term.read().await,
            leader_id: self.leader_id.read().await.clone(),
            voted_for: self.voted_for.read().await.clone(),
        }
    }
}

/// Election status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionStatus {
    pub node_id: String,
    pub state: ElectionState,
    pub term: u64,
    pub leader_id: Option<String>,
    pub voted_for: Option<String>,
}

// ============================================================================
// DISTRIBUTED ORCHESTRATOR
// ============================================================================

/// Orchestrator metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchestratorMetrics {
    pub start_time: Option<u64>,
    pub total_uptime: u64,
    pub failover_count: u64,
    pub leader_changes: u64,
    pub message_count: u64,
    pub error_count: u64,
}

/// Distributed Orchestrator - Fő koordinátor
pub struct DistributedOrchestrator {
    /// Config
    config: OrchestratorConfig,
    /// System state
    system_state: Arc<RwLock<SystemState>>,
    /// Running
    running: Arc<RwLock<bool>>,
    /// Config manager
    config_manager: Arc<DistributedConfigManager>,
    /// Heartbeat monitor
    heartbeat_monitor: Arc<HeartbeatMonitor>,
    /// Leader election
    leader_election: Arc<LeaderElection>,
    /// Metrics
    metrics: Arc<RwLock<OrchestratorMetrics>>,
}

impl DistributedOrchestrator {
    /// Új orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        let peers: Vec<String> = config.peer_nodes.iter().map(|n| n.node_id.clone()).collect();

        Self {
            config_manager: Arc::new(DistributedConfigManager::new(&config.node_id)),
            heartbeat_monitor: Arc::new(HeartbeatMonitor::new(config.heartbeat_config.clone())),
            leader_election: Arc::new(LeaderElection::new(
                &config.node_id,
                peers,
                config.election_config.clone(),
            )),
            config,
            system_state: Arc::new(RwLock::new(SystemState::Initializing)),
            running: Arc::new(RwLock::new(false)),
            metrics: Arc::new(RwLock::new(OrchestratorMetrics::default())),
        }
    }

    /// Initialize system
    pub async fn initialize(&self) -> HopeResult<()> {
        // Register self node
        let self_node = NodeInfo {
            node_id: self.config.node_id.clone(),
            ip_address: self.config.ip_address.clone(),
            port: self.config.port,
            healthy: true,
            last_heartbeat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            role: NodeRole::Follower,
        };
        self.heartbeat_monitor.register_node(self_node).await;

        // Register peer nodes
        for peer in &self.config.peer_nodes {
            self.heartbeat_monitor.register_node(peer.clone()).await;
        }

        Ok(())
    }

    /// Start system
    pub async fn start(&self) -> HopeResult<()> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start config manager
        self.config_manager.start().await;

        // Start leader election if no peers (single node mode)
        if self.config.peer_nodes.is_empty() {
            self.leader_election.start_election().await?;
            self.config_manager.set_leader(true).await;
        }

        // Update state
        {
            let mut state = self.system_state.write().await;
            *state = SystemState::Healthy;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.start_time = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }

        Ok(())
    }

    /// Stop system
    pub async fn stop(&self) {
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        self.config_manager.stop().await;

        // Update state
        {
            let mut state = self.system_state.write().await;
            *state = SystemState::Shutdown;
        }

        // Calculate uptime
        {
            let mut metrics = self.metrics.write().await;
            if let Some(start) = metrics.start_time {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                metrics.total_uptime = now.saturating_sub(start);
            }
        }
    }

    /// Get system status
    pub async fn get_status(&self) -> OrchestratorStatus {
        let metrics = self.metrics.read().await;
        let uptime = if let Some(start) = metrics.start_time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now.saturating_sub(start)
        } else {
            0
        };

        OrchestratorStatus {
            system_state: *self.system_state.read().await,
            node_id: self.config.node_id.clone(),
            uptime_seconds: uptime,
            is_leader: self.leader_election.is_leader().await,
            health_status: self.heartbeat_monitor.get_health_status().await,
            election_status: self.leader_election.get_status().await,
            config_stats: self.config_manager.get_stats().await,
            metrics: metrics.clone(),
        }
    }

    /// Trigger leader election
    pub async fn trigger_election(&self) -> HopeResult<()> {
        self.leader_election.start_election().await
    }

    /// Get config manager
    pub fn config_manager(&self) -> &Arc<DistributedConfigManager> {
        &self.config_manager
    }

    /// Get heartbeat monitor
    pub fn heartbeat_monitor(&self) -> &Arc<HeartbeatMonitor> {
        &self.heartbeat_monitor
    }

    /// Get leader election
    pub fn leader_election(&self) -> &Arc<LeaderElection> {
        &self.leader_election
    }
}

/// Full orchestrator status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    pub system_state: SystemState,
    pub node_id: String,
    pub uptime_seconds: u64,
    pub is_leader: bool,
    pub health_status: HealthStatus,
    pub election_status: ElectionStatus,
    pub config_stats: ConfigManagerStats,
    pub metrics: OrchestratorMetrics,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_entry() {
        let entry = ConfigEntry::new("test.key", serde_json::json!({"value": 123}));
        assert_eq!(entry.key, "test.key");
        assert_eq!(entry.version, 1);
        assert!(entry.validate_checksum());
    }

    #[test]
    fn test_config_change() {
        let change = ConfigChange::set("key", serde_json::json!("value"), "node-1");
        assert_eq!(change.operation, ConfigOperation::Set);
        assert_eq!(change.key, "key");
    }

    #[tokio::test]
    async fn test_config_manager() {
        let manager = DistributedConfigManager::new("test-node");
        manager.start().await;
        manager.set_leader(true).await;

        manager.set_config("test.key", serde_json::json!(42)).await.unwrap();
        let value = manager.get_config("test.key").await;
        assert_eq!(value, Some(serde_json::json!(42)));

        let keys = manager.get_keys().await;
        assert_eq!(keys.len(), 1);
    }

    #[tokio::test]
    async fn test_heartbeat_monitor() {
        let monitor = HeartbeatMonitor::new(HeartbeatConfig::default());

        let node = NodeInfo {
            node_id: "test-node".to_string(),
            ip_address: "127.0.0.1".to_string(),
            port: 50051,
            healthy: true,
            last_heartbeat: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            role: NodeRole::Follower,
        };

        monitor.register_node(node).await;
        monitor.record_heartbeat("test-node").await;

        let health = monitor.check_health().await;
        assert!(health.get("test-node").copied().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_leader_election() {
        let election = LeaderElection::new(
            "node-1",
            vec![],
            ElectionConfig::default(),
        );

        assert_eq!(election.get_state().await, ElectionState::Follower);

        election.start_election().await.unwrap();
        assert_eq!(election.get_state().await, ElectionState::Leader);
        assert!(election.is_leader().await);
    }

    #[tokio::test]
    async fn test_orchestrator() {
        let config = OrchestratorConfig {
            node_id: "test-orchestrator".to_string(),
            ..Default::default()
        };

        let orchestrator = DistributedOrchestrator::new(config);
        orchestrator.initialize().await.unwrap();
        orchestrator.start().await.unwrap();

        let status = orchestrator.get_status().await;
        assert_eq!(status.system_state, SystemState::Healthy);
        assert!(status.is_leader); // Single node should be leader

        orchestrator.stop().await;
        let status = orchestrator.get_status().await;
        assert_eq!(status.system_state, SystemState::Shutdown);
    }
}
