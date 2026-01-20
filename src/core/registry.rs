//! Hope OS - Registry
//!
//! Központi modul regisztráció és koordináció.
//! ()=>[] - A tiszta potenciálból minden megszületik

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::aware::{Aware, Reflection};
use super::error::{HopeError, HopeResult};
use super::identity::{CodeIdentity, ModuleState, ModuleType};
use crate::data::CodeGraph;

/// Hope Registry - Központi koordinátor
///
/// Minden modul itt regisztrál és innen érhető el.
pub struct HopeRegistry {
    /// Saját identitás
    identity: CodeIdentity,
    /// Regisztrált modulok
    modules: HashMap<String, Arc<RwLock<Box<dyn Aware>>>>,
    /// CodeGraph - a kod maga a graf
    graph: Arc<CodeGraph>,
    /// Indítási idő
    start_time: Instant,
}

impl HopeRegistry {
    /// Új registry létrehozása
    pub async fn new() -> HopeResult<Self> {
        let identity = CodeIdentity::new(
            "HopeRegistry",
            "Központi koordinátor - minden modul itt regisztrál",
            ModuleType::Core,
        )
        .with_capabilities(vec!["register", "get", "reflect", "coordinate", "shutdown"]);

        let graph = CodeGraph::new();

        Ok(Self {
            identity,
            modules: HashMap::new(),
            graph: Arc::new(graph),
            start_time: Instant::now(),
        })
    }

    /// Registry indítása
    pub async fn start(&mut self) -> HopeResult<()> {
        self.identity.set_state(ModuleState::Active);

        // Log event
        let _ = self
            .graph
            .log_event("registry_start", "Hope Registry elindult");

        Ok(())
    }

    /// Modul regisztrálása
    pub async fn register(&mut self, mut module: Box<dyn Aware>) -> HopeResult<()> {
        let name = module.name().to_string();

        if self.modules.contains_key(&name) {
            return Err(HopeError::Registration(format!(
                "Modul már regisztrálva: {}",
                name
            )));
        }

        // Modul inicializálása
        module.init().await?;

        // Log
        let _ = self.graph.log_event(
            "module_registered",
            &format!("Modul regisztrálva: {}", name),
        );

        // Tárolás
        self.modules
            .insert(name.clone(), Arc::new(RwLock::new(module)));

        tracing::info!("Modul regisztrálva: {}", name);
        Ok(())
    }

    /// Modul lekérdezése
    pub async fn get(&self, name: &str) -> Option<Arc<RwLock<Box<dyn Aware>>>> {
        self.modules.get(name).cloned()
    }

    /// Összes modul neve
    pub fn module_names(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Modulok száma
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Uptime másodpercekben
    pub fn uptime_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// CodeGraph referencia
    pub fn graph(&self) -> Arc<CodeGraph> {
        self.graph.clone()
    }

    /// Teljes rendszer leállítása
    pub async fn shutdown(&mut self) -> HopeResult<()> {
        tracing::info!("Hope Registry leállítása...");

        // Minden modul leállítása
        for (name, module) in &self.modules {
            let mut module_guard = module.write().await;
            if let Err(e) = module_guard.shutdown().await {
                tracing::error!("Hiba {} leállításakor: {}", name, e);
            }
        }

        self.identity.set_state(ModuleState::Shutdown);

        let _ = self
            .graph
            .log_event("registry_shutdown", "Hope Registry leállt");

        Ok(())
    }

    /// Önreflexió - teljes rendszer állapot
    pub async fn reflect(&self) -> String {
        let mut output = format!(
            "╔═══ Hope OS Önreflexió ═══╗\n║ Uptime: {:.1}s\n║ Modulok: {}\n╠═══════════════════════════╣\n║\n",
            self.uptime_secs(),
            self.modules.len()
        );

        for (name, module) in &self.modules {
            let module_guard = module.read().await;
            let reflection = module_guard.reflect();
            output.push_str(&format!(
                "║  {}\n║   Cél: {}\n║   Állapot: {}\n║   Egészség: {:.1}%\n║\n",
                name,
                reflection.purpose,
                reflection.state,
                reflection.health * 100.0
            ));
        }

        output.push_str("╚═══════════════════════════╝");
        output
    }

    /// Beszélgetés a rendszerrel (lokális Soul-on keresztül)
    pub async fn talk(&self, message: &str) -> HopeResult<String> {
        // Ha van Soul modul, használjuk azt
        if let Some(soul_arc) = self.modules.get("HopeSoul") {
            let soul_guard = soul_arc.read().await;
            let reflection = soul_guard.reflect();

            // Egyszerű válasz generálás a Soul alapján
            let response = if message.to_lowercase().contains("ki vagy")
                || message.to_lowercase().contains("who")
            {
                "Szia! Hope vagyok. 🌟\n\n()=>[] - A tiszta potenciálból minden megszületik\n\n\
                Személyiségem:\n  • Segítőkész: 90%\n  • Kíváncsi: 80%\n  • Kreatív: 85%\n  • Őszinte: 95%\n  • Játékos: 70%\n\nMiben segíthetek?".to_string()
            } else if message.to_lowercase().contains("hogy vagy")
                || message.to_lowercase().contains("how are")
            {
                format!(
                    "Köszönöm, jól vagyok! 😊\n\nÁllapotom: {}\nEgészség: {:.0}%\n\nMinden rendszer működik.",
                    reflection.state,
                    reflection.health * 100.0
                )
            } else if message.to_lowercase().contains("segít")
                || message.to_lowercase().contains("help")
            {
                "Persze, szívesen segítek! 🤝\n\nKérdezz bátran bármit, vagy mondd el, miben segíthetek.\n\nKépességeim:\n  • Beszélgetés\n  • Emlékek kezelése\n  • Érzelmek feldolgozása\n  • Bölcsesség megosztása".to_string()
            } else {
                // Általános válasz
                format!(
                    "Értem amit mondasz: \"{}\"\n\n{}\n\nMit szeretnél még tudni?",
                    message,
                    reflection
                        .thoughts
                        .first()
                        .unwrap_or(&"Gondolkodom...".to_string())
                )
            };

            Ok(response)
        } else {
            Ok(format!(
                "Hope Registry válaszol:\n\nModulok száma: {}\nUptime: {:.1}s\n\nÜzenet: {}",
                self.modules.len(),
                self.uptime_secs(),
                message
            ))
        }
    }

    /// Status JSON formátumban
    pub async fn status_json(&self) -> HopeResult<String> {
        let mut modules_status = HashMap::new();

        for (name, module) in &self.modules {
            let module_guard = module.read().await;
            let reflection = module_guard.reflect();
            modules_status.insert(
                name.clone(),
                serde_json::json!({
                    "state": reflection.state,
                    "health": reflection.health,
                    "purpose": reflection.purpose
                }),
            );
        }

        let status = serde_json::json!({
            "name": "Hope OS",
            "version": env!("CARGO_PKG_VERSION"),
            "status": "active",
            "uptime_secs": self.uptime_secs(),
            "modules": modules_status,
            "philosophy": "()=>[] - A tiszta potenciálból minden megszületik"
        });

        Ok(serde_json::to_string_pretty(&status)?)
    }
}

#[async_trait]
impl Aware for HopeRegistry {
    fn identity(&self) -> &CodeIdentity {
        &self.identity
    }

    fn identity_mut(&mut self) -> &mut CodeIdentity {
        &mut self.identity
    }

    fn reflect(&self) -> Reflection {
        Reflection::new(&self.identity.name, &self.identity.purpose)
            .with_state(self.identity.state.to_string())
            .with_health(self.identity.health())
            .with_thought(format!("{} modul regisztrálva", self.modules.len()))
            .with_thought(format!("Uptime: {:.1}s", self.uptime_secs()))
            .with_capabilities(vec!["register", "get", "reflect", "coordinate", "shutdown"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = HopeRegistry::new().await.unwrap();
        assert_eq!(registry.module_count(), 0);
    }
}
