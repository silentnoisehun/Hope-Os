//! Hope OS - Modul Identitás
//!
//! Minden modul egyedi identitással rendelkezik.
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Modul típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    /// Alapvető rendszer komponens
    Core,
    /// Általános modul
    Module,
    /// Háttérszolgáltatás
    Service,
    /// Önálló ügynök
    Agent,
    /// Adatkezelő
    Data,
}

impl fmt::Display for ModuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleType::Core => write!(f, "Core"),
            ModuleType::Module => write!(f, "Module"),
            ModuleType::Service => write!(f, "Service"),
            ModuleType::Agent => write!(f, "Agent"),
            ModuleType::Data => write!(f, "Data"),
        }
    }
}

/// Modul állapotok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    /// ⏳ Inicializálás alatt
    Initializing,
    /// 🟢 Aktív és működik
    Active,
    /// 💤 Tétlen, de elérhető
    Idle,
    /// 🔄 Dolgozik
    Busy,
    /// 😴 Alszik (energia takarékos)
    Sleeping,
    /// 🔴 Hiba történt
    Error,
    /// ⭕ Leállt
    Shutdown,
}

impl ModuleState {
    /// Emoji reprezentáció
    pub fn emoji(&self) -> &'static str {
        match self {
            ModuleState::Initializing => "⏳",
            ModuleState::Active => "🟢",
            ModuleState::Idle => "💤",
            ModuleState::Busy => "🔄",
            ModuleState::Sleeping => "😴",
            ModuleState::Error => "🔴",
            ModuleState::Shutdown => "⭕",
        }
    }

    /// Magyar leírás
    pub fn description_hu(&self) -> &'static str {
        match self {
            ModuleState::Initializing => "Inicializálás",
            ModuleState::Active => "Aktív",
            ModuleState::Idle => "Tétlen",
            ModuleState::Busy => "Dolgozik",
            ModuleState::Sleeping => "Alszik",
            ModuleState::Error => "Hiba",
            ModuleState::Shutdown => "Leállt",
        }
    }
}

impl fmt::Display for ModuleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.emoji(), self.description_hu())
    }
}

/// Modul statisztikák
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleStats {
    /// Hívások száma
    pub calls: u64,
    /// Sikeres hívások
    pub successes: u64,
    /// Hibák száma
    pub errors: u64,
    /// Összesített válaszidő (ms)
    pub total_response_time_ms: u64,
    /// Utolsó hívás ideje
    pub last_call: Option<DateTime<Utc>>,
}

impl ModuleStats {
    /// Új statisztika
    pub fn new() -> Self {
        Self::default()
    }

    /// Sikeres hívás regisztrálása
    pub fn record_success(&mut self, response_time_ms: u64) {
        self.calls += 1;
        self.successes += 1;
        self.total_response_time_ms += response_time_ms;
        self.last_call = Some(Utc::now());
    }

    /// Hibás hívás regisztrálása
    pub fn record_error(&mut self) {
        self.calls += 1;
        self.errors += 1;
        self.last_call = Some(Utc::now());
    }

    /// Átlagos válaszidő
    pub fn avg_response_time_ms(&self) -> f64 {
        if self.successes == 0 {
            0.0
        } else {
            self.total_response_time_ms as f64 / self.successes as f64
        }
    }

    /// Sikerességi arány
    pub fn success_rate(&self) -> f64 {
        if self.calls == 0 {
            1.0
        } else {
            self.successes as f64 / self.calls as f64
        }
    }
}

/// Modul identitás - Ki vagyok?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIdentity {
    /// Egyedi azonosító
    pub id: Uuid,
    /// Ki vagyok? - Név
    pub name: String,
    /// Miért vagyok? - Cél
    pub purpose: String,
    /// Modul típus
    pub module_type: ModuleType,
    /// Mikor születtem?
    pub created_at: DateTime<Utc>,
    /// Verzió
    pub version: String,
    /// Függőségek
    pub dependencies: Vec<String>,
    /// Képességek
    pub capabilities: Vec<String>,
    /// Aktuális állapot
    pub state: ModuleState,
    /// Statisztikák
    pub stats: ModuleStats,
}

impl CodeIdentity {
    /// Új identitás létrehozása
    pub fn new(
        name: impl Into<String>,
        purpose: impl Into<String>,
        module_type: ModuleType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            purpose: purpose.into(),
            module_type,
            created_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            state: ModuleState::Initializing,
            stats: ModuleStats::new(),
        }
    }

    /// Függőség hozzáadása
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Képesség hozzáadása
    pub fn with_capability(mut self, cap: impl Into<String>) -> Self {
        self.capabilities.push(cap.into());
        self
    }

    /// Több képesség hozzáadása
    pub fn with_capabilities(mut self, caps: Vec<&str>) -> Self {
        self.capabilities.extend(caps.into_iter().map(String::from));
        self
    }

    /// Állapot beállítása
    pub fn set_state(&mut self, state: ModuleState) {
        self.state = state;
    }

    /// Bemutatkozás
    pub fn introduce(&self) -> String {
        format!(
            "{} vagyok ({}). {}",
            self.name, self.module_type, self.purpose
        )
    }

    /// Életkor percekben
    pub fn age_minutes(&self) -> i64 {
        let now = Utc::now();
        (now - self.created_at).num_minutes()
    }

    /// Egészségi mutató (0.0 - 1.0)
    pub fn health(&self) -> f64 {
        match self.state {
            ModuleState::Active | ModuleState::Idle => {
                // Alapból 1.0, csökken a hibák arányával
                let error_penalty = if self.stats.calls > 0 {
                    self.stats.errors as f64 / self.stats.calls as f64 * 0.5
                } else {
                    0.0
                };
                (1.0 - error_penalty).max(0.0)
            }
            ModuleState::Busy => 0.9,
            ModuleState::Initializing => 0.8,
            ModuleState::Sleeping => 0.7,
            ModuleState::Error => 0.3,
            ModuleState::Shutdown => 0.0,
        }
    }
}

impl fmt::Display for CodeIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = self.id.to_string();
        write!(
            f,
            "[{}] {} ({}) - {} - {}",
            &id_str[..8],
            self.name,
            self.module_type,
            self.state,
            self.purpose
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation() {
        let identity = CodeIdentity::new("TestModule", "Tesztelés", ModuleType::Module);
        assert_eq!(identity.name, "TestModule");
        assert_eq!(identity.state, ModuleState::Initializing);
    }

    #[test]
    fn test_module_state_display() {
        assert_eq!(ModuleState::Active.to_string(), "🟢 Aktív");
        assert_eq!(ModuleState::Error.to_string(), "🔴 Hiba");
    }

    #[test]
    fn test_stats() {
        let mut stats = ModuleStats::new();
        stats.record_success(100);
        stats.record_success(200);
        stats.record_error();

        assert_eq!(stats.calls, 3);
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.avg_response_time_ms(), 150.0);
    }
}
