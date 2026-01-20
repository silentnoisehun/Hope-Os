//! Hope Entity - Magyar beszélő entitás
//!
//! Több Ollama modell feloldva egy entitásban:
//! - Magyar beszéd
//! - Kódolás
//! - Többnyelvű
//!
//! Nincs API költség. Minden lokálisan fut.
//!
//! # Példa
//!
//! ```rust,no_run
//! use hope_entity::{Entitás, OllamaBridge, ModellTípus};
//!
//! #[tokio::main]
//! async fn main() {
//!     let bridge = OllamaBridge::new()
//!         .felold("Magyar", "openeurollm-hungarian", ModellTípus::Magyar)
//!         .felold("Kódoló", "deepseek-coder", ModellTípus::Kódoló);
//!
//!     let mut remény = Entitás::new("Remény")
//!         .with_bridge(bridge);
//!
//!     let válasz = remény.gondolkodj("Szia! Ki vagy te?").await.unwrap();
//!     println!("{}", válasz);
//! }
//! ```

pub mod bench;
pub mod entity;
pub mod native_inference;
pub mod ollama_bridge;
pub mod self_repair;
pub mod silent_teacher;

pub use bench::{gyors_benchmark, BenchmarkEredmény, EntityBenchmark};
pub use entity::{Emlék, Entitás, InferenceMode, Személyiség, ÉrzelmiÁllapot};
pub use native_inference::{
    BeolvasztottModell, NativeEngine, NativeModelConfig, NativeModellTípus,
};
pub use ollama_bridge::{ChatMessage, FeloldottModell, ModellTípus, OllamaBridge};
pub use self_repair::{HibaTípus, SelfRepair, StratégiaAjánlás, Visszajelzés};
pub use silent_teacher::{Axióma, Megsértés, SilentTeacher, Súlyosság};
