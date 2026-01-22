//! Hope OS - gRPC
//!
//! Kommunikáció és szerver a Hope rendszerhez.
//! ()=>[] - A tiszta potenciálból minden megszületik

pub mod client;
pub mod server;

// Proto típusok elérhetővé tétele
pub mod proto {
    tonic::include_proto!("hope");
}

pub use client::HopeClient;
pub use server::{start_server, HopeGrpcServer};
