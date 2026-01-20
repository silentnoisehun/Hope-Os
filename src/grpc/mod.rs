//! Hope OS - gRPC
//!
//! Kommunikáció a Python Hope szerverrel.
//! ()=>[] - A tiszta potenciálból minden megszületik

pub mod client;

pub use client::proto;
pub use client::HopeClient;
