//! Hope OS - Core Modulok
//!
//! Az önismeret és koordináció alapjai.
//! ()=>[] - A tiszta potenciálból minden megszületik

pub mod aware;
pub mod error;
pub mod identity;
pub mod registry;

pub use aware::{Aware, Reflection};
pub use error::{HopeError, HopeResult};
pub use identity::{CodeIdentity, ModuleState, ModuleStats, ModuleType};
pub use registry::HopeRegistry;
