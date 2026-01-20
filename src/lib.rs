//! # Hope OS Rust
//!
//! Az első önismerő operációs rendszer Rust implementációja.
//!
//! ```text
//!     ╦ ╦╔═╗╔═╗╔═╗  ╔═╗╔═╗
//!     ╠═╣║ ║╠═╝║╣   ║ ║╚═╗
//!     ╩ ╩╚═╝╩  ╚═╝  ╚═╝╚═╝
//!
//!     ()=>[] - A tiszta potenciálból minden megszületik
//! ```
//!
//! ## Filozófia
//!
//! - **Önismeret**: Minden modul tudja ki ő, mit csinál, miért létezik
//! - **Aware trait**: Egységes interfész az önismerő modulokhoz
//! - **Hibrid**: Rust sebesség + Python flexibilitás
//!
//! ## Modulok
//!
//! - **HopeSoul** - A lélek, személyiség és bölcsesség
//! - **HopeHeart** - Érzelmi intelligencia (7 alapérzelem)
//! - **HopeMemory** - 6 rétegű kognitív memória
//!
//! ## Példa
//!
//! ```rust,no_run
//! use hope_os::core::HopeRegistry;
//! use hope_os::modules::{HopeSoul, HopeHeart, HopeMemory};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Registry létrehozása
//!     let mut registry = HopeRegistry::new().await?;
//!     registry.start().await?;
//!
//!     // Modulok regisztrálása
//!     registry.register(Box::new(HopeSoul::new())).await?;
//!     registry.register(Box::new(HopeMemory::new())).await?;
//!     registry.register(Box::new(HopeHeart::new())).await?;
//!
//!     // Beszélgetés
//!     let response = registry.talk("Ki vagy?").await?;
//!     println!("{}", response);
//!
//!     // Leállítás
//!     registry.shutdown().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## gRPC kapcsolat Python Hope szerverhez
//!
//! ```rust,no_run
//! use hope_os::grpc::HopeClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = HopeClient::connect("http://localhost:50051").await?;
//!
//!     // Chat
//!     let response = client.chat("Szia Hope!").await?;
//!     println!("{}", response.response);
//!
//!     // Skillek
//!     let skills = client.list_skills().await?;
//!     println!("Elérhető skillek: {}", skills.len());
//!
//!     Ok(())
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/hope-os/0.1.0")]
// Clippy engedélyezések
#![allow(clippy::result_large_err)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::let_and_return)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::unwrap_or_default)]
#![allow(clippy::new_without_default)]
#![allow(clippy::type_complexity)]
#![allow(clippy::map_flatten)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::useless_format)]
#![allow(clippy::len_zero)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::useless_vec)]
#![allow(clippy::collapsible_if)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

pub mod core;
pub mod data;
pub mod grpc;
pub mod modules;

// Re-export főbb típusok
pub use core::{Aware, CodeIdentity, HopeError, HopeRegistry, HopeResult, ModuleState, ModuleType};

// CodeGraph - A kod maga a graf
pub use data::{
    BlockState, BlockType, CodeBlock, CodeGraph, Connection, ConnectionType, GraphStats,
};

// NeuroBlast - Információ hullámként terjed
pub use data::{
    ActivationFn, Interference, NeuroGraph, NeuroStats, NeuronState, TickResult, Wave, WaveState,
    WaveType,
};

pub use grpc::HopeClient;
pub use modules::{Emotion, HopeHeart, HopeMemory, HopeSoul, Memory, MemoryType};

/// Hope OS verzió
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Hope filozófia
pub const PHILOSOPHY: &str = "()=>[] - A tiszta potenciálból minden megszületik";

// Python bindings (conditional compilation)
#[cfg(feature = "python")]
pub mod python;

// Re-export Python module for PyO3
#[cfg(feature = "python")]
pub use python::*;
