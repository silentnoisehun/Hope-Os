//! Hope Swarm - Raj Intelligencia
//!
//! Elosztott feladatvégrehajtás drone-okkal.
//! A HiveMind koordinálja a munkát.
//!
//! Architektúra:
//! - HiveMind: Központi koordinátor
//! - Drone: Munkavégző egység (LOCAL/REMOTE)
//! - Task: Feladat
//! - TaskQueue: Feladat sor
//!
//! ()=>[] - A tiszta potenciálból a raj megszületik

use crate::core::HopeResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// TASK
// ============================================================================

/// Feladat állapot
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Várakozik
    Pending,
    /// Fut
    Running,
    /// Kész
    Completed,
    /// Sikertelen
    Failed,
    /// Megszakítva
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "⏳ Pending"),
            TaskStatus::Running => write!(f, "🔄 Running"),
            TaskStatus::Completed => write!(f, "✅ Completed"),
            TaskStatus::Failed => write!(f, "❌ Failed"),
            TaskStatus::Cancelled => write!(f, "🚫 Cancelled"),
        }
    }
}

/// Feladat prioritás
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Swarm feladat
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwarmTask {
    /// Egyedi azonosító
    pub id: String,
    /// Skill neve (pl. "refactor", "test", "analyze")
    pub skill: String,
    /// Bemenet/payload
    pub payload: String,
    /// Állapot
    pub status: TaskStatus,
    /// Eredmény
    pub result: Option<String>,
    /// Hiba üzenet
    pub error: Option<String>,
    /// Melyik drone-hoz van rendelve
    pub assigned_to: Option<String>,
    /// Prioritás
    pub priority: TaskPriority,
    /// Létrehozás ideje
    pub created_at: f64,
    /// Kezdés ideje
    pub started_at: Option<f64>,
    /// Befejezés ideje
    pub completed_at: Option<f64>,
    /// Timeout (másodperc)
    pub timeout_secs: u64,
    /// Retry számláló
    pub retry_count: u32,
    /// Max retry
    pub max_retries: u32,
}

impl SwarmTask {
    pub fn new(skill: &str, payload: &str) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            id: format!(
                "TSK_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            skill: skill.to_string(),
            payload: payload.to_string(),
            status: TaskStatus::Pending,
            result: None,
            error: None,
            assigned_to: None,
            priority: TaskPriority::Normal,
            created_at,
            started_at: None,
            completed_at: None,
            timeout_secs: 300, // 5 perc default
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Prioritás beállítása
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Timeout beállítása
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Futás jelzése
    pub fn mark_running(&mut self, drone_id: &str) {
        self.status = TaskStatus::Running;
        self.assigned_to = Some(drone_id.to_string());
        self.started_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );
    }

    /// Kész jelzése
    pub fn mark_completed(&mut self, result: &str) {
        self.status = TaskStatus::Completed;
        self.result = Some(result.to_string());
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );
    }

    /// Hiba jelzése
    pub fn mark_failed(&mut self, error: &str) {
        self.status = TaskStatus::Failed;
        self.error = Some(error.to_string());
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        );
    }

    /// Futási idő (ha befejeződött)
    pub fn duration_secs(&self) -> Option<f64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

// ============================================================================
// DRONE
// ============================================================================

/// Drone típus
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DroneType {
    /// Helyi drone (ugyanaz a gép)
    Local,
    /// Távoli drone (hálózaton)
    Remote,
}

/// Drone állapot
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DroneStatus {
    /// Tétlen, elérhető
    Idle,
    /// Dolgozik
    Busy,
    /// Offline
    Offline,
    /// Hiba
    Error,
}

impl std::fmt::Display for DroneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DroneStatus::Idle => write!(f, "🟢 Idle"),
            DroneStatus::Busy => write!(f, "🔵 Busy"),
            DroneStatus::Offline => write!(f, "⚫ Offline"),
            DroneStatus::Error => write!(f, "🔴 Error"),
        }
    }
}

/// Drone információ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DroneInfo {
    /// Egyedi azonosító
    pub id: String,
    /// Név
    pub name: String,
    /// Típus
    pub drone_type: DroneType,
    /// Képességek (skill nevek)
    pub capabilities: Vec<String>,
    /// Állapot
    pub status: DroneStatus,
    /// Aktuális feladat
    pub current_task: Option<String>,
    /// Befejezett feladatok száma
    pub completed_tasks: u64,
    /// Sikertelen feladatok száma
    pub failed_tasks: u64,
    /// Átlagos futási idő (másodperc)
    pub avg_execution_time: f64,
}

impl DroneInfo {
    pub fn new(id: &str, name: &str, drone_type: DroneType, capabilities: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            drone_type,
            capabilities,
            status: DroneStatus::Idle,
            current_task: None,
            completed_tasks: 0,
            failed_tasks: 0,
            avg_execution_time: 0.0,
        }
    }

    /// Tud-e adott skill-t végrehajtani
    pub fn can_execute(&self, skill: &str) -> bool {
        self.capabilities.contains(&skill.to_string())
            || self.capabilities.contains(&"*".to_string())
    }
}

/// Drone trait - minden drone ezt implementálja
#[async_trait]
pub trait Drone: Send + Sync {
    /// Drone ID
    fn id(&self) -> &str;

    /// Drone info
    fn info(&self) -> DroneInfo;

    /// Feladat végrehajtása
    async fn execute(&self, task: &SwarmTask) -> HopeResult<String>;

    /// Állapot
    fn status(&self) -> DroneStatus;

    /// Elérhető-e
    fn is_available(&self) -> bool {
        matches!(self.status(), DroneStatus::Idle)
    }
}

/// Helyi drone implementáció
pub struct LocalDrone {
    info: Arc<RwLock<DroneInfo>>,
}

impl LocalDrone {
    pub fn new(id: &str, name: &str, capabilities: Vec<String>) -> Self {
        Self {
            info: Arc::new(RwLock::new(DroneInfo::new(
                id,
                name,
                DroneType::Local,
                capabilities,
            ))),
        }
    }
}

#[async_trait]
impl Drone for LocalDrone {
    fn id(&self) -> &str {
        // Egyszerűsített - valós implementációban async kellene
        "local_drone"
    }

    fn info(&self) -> DroneInfo {
        // Egyszerűsített
        DroneInfo::new(
            "local",
            "Local Drone",
            DroneType::Local,
            vec!["*".to_string()],
        )
    }

    async fn execute(&self, task: &SwarmTask) -> HopeResult<String> {
        let mut info = self.info.write().await;
        info.status = DroneStatus::Busy;
        info.current_task = Some(task.id.clone());

        // Szimulált végrehajtás
        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = format!(
            "[EXECUTION COMPLETE]\n\
             Skill: {}\n\
             Target: {}\n\
             Drone: {}\n\
             Status: SUCCESS",
            task.skill, task.payload, info.id
        );

        info.status = DroneStatus::Idle;
        info.current_task = None;
        info.completed_tasks += 1;

        Ok(result)
    }

    fn status(&self) -> DroneStatus {
        // Egyszerűsített
        DroneStatus::Idle
    }
}

// ============================================================================
// HIVEMIND - Központi Koordinátor
// ============================================================================

/// HiveMind - A raj központi agya
pub struct HiveMind {
    /// Regisztrált drone-ok
    drones: Arc<RwLock<HashMap<String, Arc<dyn Drone>>>>,
    /// Feladat sor
    task_queue: Arc<RwLock<Vec<SwarmTask>>>,
    /// Összes feladat (történet)
    tasks: Arc<RwLock<HashMap<String, SwarmTask>>>,
    /// Fut-e
    is_running: Arc<RwLock<bool>>,
    /// Statisztikák
    stats: Arc<RwLock<SwarmStats>>,
}

/// Swarm statisztikák
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SwarmStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub cancelled_tasks: u64,
    pub total_drones: usize,
    pub active_drones: usize,
    pub avg_task_duration: f64,
}

impl HiveMind {
    /// Új HiveMind
    pub fn new() -> Self {
        Self {
            drones: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(SwarmStats::default())),
        }
    }

    // ==================== DRONE MANAGEMENT ====================

    /// Drone regisztrálása
    pub async fn register_drone(&self, drone: Arc<dyn Drone>) {
        let id = drone.id().to_string();
        let mut drones = self.drones.write().await;
        drones.insert(id.clone(), drone);
        self.stats.write().await.total_drones = drones.len();
        println!("[HiveMind] 🐝 Drone regisztrálva: {}", id);
    }

    /// Drone eltávolítása
    pub async fn unregister_drone(&self, drone_id: &str) {
        let mut drones = self.drones.write().await;
        drones.remove(drone_id);
        self.stats.write().await.total_drones = drones.len();
        println!("[HiveMind] 🐝 Drone eltávolítva: {}", drone_id);
    }

    /// Elérhető drone keresése skill alapján
    async fn find_available_drone(&self, skill: &str) -> Option<Arc<dyn Drone>> {
        let drones = self.drones.read().await;
        for (_, drone) in drones.iter() {
            if drone.is_available() && drone.info().can_execute(skill) {
                return Some(drone.clone());
            }
        }
        None
    }

    // ==================== TASK MANAGEMENT ====================

    /// Feladat beküldése
    pub async fn submit_task(&self, skill: &str, payload: &str) -> HopeResult<String> {
        let task = SwarmTask::new(skill, payload);
        let task_id = task.id.clone();

        // Mentés
        self.tasks
            .write()
            .await
            .insert(task_id.clone(), task.clone());
        self.task_queue.write().await.push(task);
        self.stats.write().await.total_tasks += 1;

        println!("[HiveMind] 📋 Feladat beküldve: {} ({})", task_id, skill);
        Ok(task_id)
    }

    /// Feladat beküldése prioritással
    pub async fn submit_task_with_priority(
        &self,
        skill: &str,
        payload: &str,
        priority: TaskPriority,
    ) -> HopeResult<String> {
        let task = SwarmTask::new(skill, payload).with_priority(priority);
        let task_id = task.id.clone();

        self.tasks
            .write()
            .await
            .insert(task_id.clone(), task.clone());

        // Prioritás szerinti beszúrás
        let mut queue = self.task_queue.write().await;
        let pos = queue
            .iter()
            .position(|t| t.priority < priority)
            .unwrap_or(queue.len());
        queue.insert(pos, task);

        self.stats.write().await.total_tasks += 1;

        println!(
            "[HiveMind] 📋 Prioritásos feladat: {} ({:?})",
            task_id, priority
        );
        Ok(task_id)
    }

    /// Feladat állapot lekérdezése
    pub async fn get_task(&self, task_id: &str) -> Option<SwarmTask> {
        self.tasks.read().await.get(task_id).cloned()
    }

    /// Feladat törlése
    pub async fn cancel_task(&self, task_id: &str) -> HopeResult<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            if task.status == TaskStatus::Pending {
                task.status = TaskStatus::Cancelled;
                self.stats.write().await.cancelled_tasks += 1;

                // Eltávolítás a sorból
                let mut queue = self.task_queue.write().await;
                queue.retain(|t| t.id != task_id);

                Ok(())
            } else {
                Err("Csak várakozó feladat törölhető".into())
            }
        } else {
            Err("Feladat nem található".into())
        }
    }

    // ==================== EXECUTION ====================

    /// Következő feladat feldolgozása
    pub async fn process_next_task(&self) -> HopeResult<Option<SwarmTask>> {
        // Feladat kivétele a sorból
        let task = {
            let mut queue = self.task_queue.write().await;
            if queue.is_empty() {
                return Ok(None);
            }
            queue.remove(0)
        };

        // Drone keresése
        let drone = self.find_available_drone(&task.skill).await;

        if let Some(drone) = drone {
            let mut task = task;
            task.mark_running(drone.id());

            // Feladat végrehajtása
            match drone.execute(&task).await {
                Ok(result) => {
                    task.mark_completed(&result);
                    self.stats.write().await.completed_tasks += 1;
                }
                Err(e) => {
                    task.mark_failed(&e.to_string());
                    self.stats.write().await.failed_tasks += 1;
                }
            }

            // Frissítés
            self.tasks
                .write()
                .await
                .insert(task.id.clone(), task.clone());

            Ok(Some(task))
        } else {
            // Nincs elérhető drone - visszatesszük a sorba
            self.task_queue.write().await.insert(0, task);
            Ok(None)
        }
    }

    /// Összes várakozó feladat feldolgozása
    pub async fn process_all(&self) -> Vec<SwarmTask> {
        let mut completed = Vec::new();

        while let Ok(Some(task)) = self.process_next_task().await {
            completed.push(task);
        }

        completed
    }

    /// Scheduler indítása (háttérben fut)
    pub async fn start_scheduler(&self) {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return;
        }
        *is_running = true;
        drop(is_running);

        println!("[HiveMind] 🚀 Scheduler elindítva");

        while *self.is_running.read().await {
            self.process_next_task().await.ok();
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        println!("[HiveMind] 🛑 Scheduler leállítva");
    }

    /// Scheduler leállítása
    pub async fn stop_scheduler(&self) {
        *self.is_running.write().await = false;
    }

    // ==================== STATUS ====================

    /// Statisztikák
    pub async fn stats(&self) -> SwarmStats {
        let mut stats = self.stats.read().await.clone();

        // Aktív drone-ok számolása
        let drones = self.drones.read().await;
        stats.active_drones = drones.values().filter(|d| d.is_available()).count();

        stats
    }

    /// Várakozó feladatok száma
    pub async fn pending_count(&self) -> usize {
        self.task_queue.read().await.len()
    }

    /// Állapot szövegesen
    pub async fn status(&self) -> String {
        let stats = self.stats().await;
        let pending = self.pending_count().await;
        let is_running = *self.is_running.read().await;

        format!(
            "🐝 HiveMind - Swarm Intelligence\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             🔄 Scheduler: {}\n\
             ⏳ Várakozó: {} feladat\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             📊 Statisztikák:\n\
             📋 Összes feladat: {}\n\
             ✅ Befejezett: {}\n\
             ❌ Sikertelen: {}\n\
             🚫 Törölt: {}\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             🐝 Drone-ok: {} (aktív: {})",
            if is_running { "🟢 Fut" } else { "🔴 Áll" },
            pending,
            stats.total_tasks,
            stats.completed_tasks,
            stats.failed_tasks,
            stats.cancelled_tasks,
            stats.total_drones,
            stats.active_drones
        )
    }
}

impl Default for HiveMind {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = SwarmTask::new("refactor", "src/main.rs")
            .with_priority(TaskPriority::High)
            .with_timeout(60);

        assert!(task.id.starts_with("TSK_"));
        assert_eq!(task.skill, "refactor");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.timeout_secs, 60);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = SwarmTask::new("test", "payload");
        assert_eq!(task.status, TaskStatus::Pending);

        task.mark_running("drone_1");
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.assigned_to, Some("drone_1".to_string()));

        task.mark_completed("Success!");
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.duration_secs().is_some());
    }

    #[test]
    fn test_drone_info() {
        let info = DroneInfo::new(
            "drone_1",
            "Test Drone",
            DroneType::Local,
            vec!["refactor".to_string(), "test".to_string()],
        );

        assert!(info.can_execute("refactor"));
        assert!(info.can_execute("test"));
        assert!(!info.can_execute("unknown"));
    }

    #[test]
    fn test_drone_wildcard() {
        let info = DroneInfo::new(
            "super_drone",
            "Super Drone",
            DroneType::Local,
            vec!["*".to_string()],
        );

        assert!(info.can_execute("anything"));
        assert!(info.can_execute("everything"));
    }

    #[tokio::test]
    async fn test_hivemind_task_submission() {
        let hive = HiveMind::new();

        let task_id = hive.submit_task("analyze", "src/lib.rs").await.unwrap();
        assert!(task_id.starts_with("TSK_"));

        let stats = hive.stats().await;
        assert_eq!(stats.total_tasks, 1);
    }

    #[tokio::test]
    async fn test_hivemind_priority_queue() {
        let hive = HiveMind::new();

        hive.submit_task("low", "payload1").await.unwrap();
        hive.submit_task_with_priority("high", "payload2", TaskPriority::High)
            .await
            .unwrap();

        let queue = hive.task_queue.read().await;
        assert_eq!(queue[0].skill, "high"); // High priority first
        assert_eq!(queue[1].skill, "low");
    }

    #[tokio::test]
    async fn test_hivemind_with_drone() {
        let hive = HiveMind::new();

        // Drone regisztrálása
        let drone = Arc::new(LocalDrone::new(
            "local_1",
            "Local Drone 1",
            vec!["*".to_string()],
        ));
        hive.register_drone(drone).await;

        // Feladat beküldése és feldolgozása
        hive.submit_task("test", "payload").await.unwrap();
        let result = hive.process_next_task().await.unwrap();

        assert!(result.is_some());
        let task = result.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }
}
