//! Hope OS - Aware Trait
//!
//! Az önismeret alapja. Minden modul implementálja.
//! ()=>[] - A tiszta potenciálból minden megszületik

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::error::HopeResult;
use super::identity::{CodeIdentity, ModuleState};

/// Önreflexió - Mit gondolok magamról?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    /// Modul neve
    pub name: String,
    /// Modul célja
    pub purpose: String,
    /// Aktuális állapot szövegesen
    pub state: String,
    /// Egészség (0.0 - 1.0)
    pub health: f64,
    /// Aktuális gondolatok
    pub thoughts: Vec<String>,
    /// Képességek listája
    pub capabilities: Vec<String>,
}

impl Reflection {
    /// Új reflexió létrehozása
    pub fn new(name: impl Into<String>, purpose: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            purpose: purpose.into(),
            state: "unknown".to_string(),
            health: 1.0,
            thoughts: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    /// Állapot beállítása
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }

    /// Egészség beállítása
    pub fn with_health(mut self, health: f64) -> Self {
        self.health = health.clamp(0.0, 1.0);
        self
    }

    /// Gondolat hozzáadása
    pub fn with_thought(mut self, thought: impl Into<String>) -> Self {
        self.thoughts.push(thought.into());
        self
    }

    /// Több gondolat hozzáadása
    pub fn with_thoughts(mut self, thoughts: Vec<&str>) -> Self {
        self.thoughts.extend(thoughts.into_iter().map(String::from));
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

    /// Szöveges formátum
    pub fn to_text(&self) -> String {
        let mut text = format!(
            "╔═══ {} ═══╗\n║  Cél: {}\n║  Állapot: {}\n║  Egészség: {:.1}%\n",
            self.name,
            self.purpose,
            self.state,
            self.health * 100.0
        );

        if !self.thoughts.is_empty() {
            text.push_str("║  Gondolatok:\n");
            for thought in &self.thoughts {
                text.push_str(&format!("║    - {}\n", thought));
            }
        }

        if !self.capabilities.is_empty() {
            text.push_str("║  Képességek:\n");
            for cap in &self.capabilities {
                text.push_str(&format!("║    • {}\n", cap));
            }
        }

        text.push_str("╚════════════════════════════════╝");
        text
    }
}

impl std::fmt::Display for Reflection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

/// Aware Trait - Az önismeret alapja
///
/// Minden Hope modul implementálja ezt a trait-et.
/// Ez teszi lehetővé, hogy minden modul tudja:
/// - Ki vagyok?
/// - Mit csinálok?
/// - Miért létezem?
#[async_trait]
pub trait Aware: Send + Sync {
    /// Ki vagyok? - Identitás lekérdezése
    fn identity(&self) -> &CodeIdentity;

    /// Ki vagyok? - Mutable identitás
    fn identity_mut(&mut self) -> &mut CodeIdentity;

    /// Bemutatkozás
    fn introduce(&self) -> String {
        self.identity().introduce()
    }

    /// Aktuális állapot
    fn state(&self) -> ModuleState {
        self.identity().state
    }

    /// Önreflexió - Mit gondolok magamról?
    fn reflect(&self) -> Reflection {
        let identity = self.identity();
        Reflection::new(&identity.name, &identity.purpose)
            .with_state(identity.state.to_string())
            .with_health(identity.health())
            .with_capabilities(identity.capabilities.iter().map(|s| s.as_str()).collect())
    }

    /// Modul neve
    fn name(&self) -> &str {
        &self.identity().name
    }

    /// Modul inicializálása
    async fn init(&mut self) -> HopeResult<()> {
        self.identity_mut().set_state(ModuleState::Active);
        Ok(())
    }

    /// Modul leállítása
    async fn shutdown(&mut self) -> HopeResult<()> {
        self.identity_mut().set_state(ModuleState::Shutdown);
        Ok(())
    }

    /// Modul aktiválása
    fn activate(&mut self) {
        self.identity_mut().set_state(ModuleState::Active);
    }

    /// Modul deaktiválása (idle)
    fn deactivate(&mut self) {
        self.identity_mut().set_state(ModuleState::Idle);
    }

    /// Hiba beállítása
    fn set_error(&mut self) {
        self.identity_mut().set_state(ModuleState::Error);
    }

    /// Dolgozik állapot
    fn set_busy(&mut self) {
        self.identity_mut().set_state(ModuleState::Busy);
    }
}

/// Macro az Aware trait egyszerű implementálásához
#[macro_export]
macro_rules! impl_aware {
    ($type:ty, $identity_field:ident) => {
        impl $crate::core::aware::Aware for $type {
            fn identity(&self) -> &$crate::core::identity::CodeIdentity {
                &self.$identity_field
            }

            fn identity_mut(&mut self) -> &mut $crate::core::identity::CodeIdentity {
                &mut self.$identity_field
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::identity::ModuleType;

    struct TestModule {
        identity: CodeIdentity,
    }

    impl TestModule {
        fn new() -> Self {
            Self {
                identity: CodeIdentity::new("TestModule", "Tesztelés", ModuleType::Module),
            }
        }
    }

    #[async_trait]
    impl Aware for TestModule {
        fn identity(&self) -> &CodeIdentity {
            &self.identity
        }

        fn identity_mut(&mut self) -> &mut CodeIdentity {
            &mut self.identity
        }
    }

    #[test]
    fn test_reflection() {
        let reflection = Reflection::new("Test", "Testing")
            .with_state("Active")
            .with_health(0.95)
            .with_thought("All systems go");

        assert_eq!(reflection.name, "Test");
        assert_eq!(reflection.health, 0.95);
        assert_eq!(reflection.thoughts.len(), 1);
    }

    #[tokio::test]
    async fn test_aware_trait() {
        let mut module = TestModule::new();
        assert_eq!(module.state(), ModuleState::Initializing);

        module.init().await.unwrap();
        assert_eq!(module.state(), ModuleState::Active);

        module.shutdown().await.unwrap();
        assert_eq!(module.state(), ModuleState::Shutdown);
    }
}
