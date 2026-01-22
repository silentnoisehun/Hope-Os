//! Hope OS - Adattárolás
//!
//! A KOD MAGA A GRAF - Nincs DB, nincs külső függés!
//! Minden CodeBlock @aware és kapcsolódik mindenhez.
//! NEUROBLAST - Információ HULLÁMKÉNT terjed!
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

pub mod code_graph;
pub mod neuroblast;

// Re-export CodeGraph
pub use code_graph::{
    BlockState, BlockType, CodeBlock, CodeGraph, Connection, ConnectionType, GraphSnapshot,
    GraphStats,
};

// Re-export NeuroBlast
pub use neuroblast::{
    ActivationFn, Interference, NeuroGraph, NeuroStats, NeuronState, TickResult, Wave, WaveState,
    WaveType,
};
