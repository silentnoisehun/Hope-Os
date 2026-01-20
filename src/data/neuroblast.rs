//! Hope OS - NEUROBLAST
//!
//! Információ HULLÁMKÉNT terjed a hálózatban.
//! Minden CodeBlock egy neuron, minden Connection egy synapse.
//! A hullámok interferálnak, rezonálnak, visszaverődnek.
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use super::code_graph::{CodeBlock, CodeGraph};
use crate::core::HopeResult;

// ============================================================================
// WAVE - Információ hullám
// ============================================================================

/// Hullám típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaveType {
    /// Impulzus - egyetlen csúcs
    Impulse,
    /// Szinusz - folyamatos oszcilláció
    Sine,
    /// Négyszög - digitális jel
    Square,
    /// Gauss - harang görbe
    Gaussian,
    /// Spike - neuron tüzelés
    Spike,
    /// Echo - visszaverődő hullám
    Echo,
}

/// Hullám állapot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveState {
    /// Aktív - terjed
    Active,
    /// Gyengülő
    Decaying,
    /// Interferál másik hullámmal
    Interfering,
    /// Rezonál
    Resonating,
    /// Kihalt
    Dead,
}

/// Information Wave - Információ hullám
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wave {
    /// Egyedi azonosító
    pub id: String,
    /// Hullám típus
    pub wave_type: WaveType,
    /// Tartalom/Payload
    pub payload: String,
    /// Amplitúdó (erősség) 0.0 - 1.0
    pub amplitude: f64,
    /// Frekvencia (hullámok/tick)
    pub frequency: f64,
    /// Fázis (0.0 - 2π)
    pub phase: f64,
    /// Terjedési sebesség (node/tick)
    pub velocity: f64,
    /// Csillapítás (amplitude csökkenés hop-onként)
    pub decay: f64,
    /// Forrás node ID
    pub origin_id: String,
    /// Jelenlegi pozíció (node ID-k)
    pub current_positions: HashSet<String>,
    /// Meglátogatott node-ok
    pub visited: HashSet<String>,
    /// Hullám állapot
    pub state: WaveState,
    /// Létrehozás ideje
    pub created_at: DateTime<Utc>,
    /// Tick számláló
    pub ticks: u64,
    /// Metaadatok
    pub metadata: HashMap<String, String>,
}

impl Wave {
    /// Új hullám létrehozása
    pub fn new(
        origin_id: impl Into<String>,
        payload: impl Into<String>,
        wave_type: WaveType,
    ) -> Self {
        let origin = origin_id.into();
        let mut positions = HashSet::new();
        positions.insert(origin.clone());

        Self {
            id: Uuid::new_v4().to_string(),
            wave_type,
            payload: payload.into(),
            amplitude: 1.0,
            frequency: 1.0,
            phase: 0.0,
            velocity: 1.0,
            decay: 0.1,
            origin_id: origin.clone(),
            current_positions: positions,
            visited: {
                let mut v = HashSet::new();
                v.insert(origin);
                v
            },
            state: WaveState::Active,
            created_at: Utc::now(),
            ticks: 0,
            metadata: HashMap::new(),
        }
    }

    /// Amplitúdó beállítása
    pub fn with_amplitude(mut self, amplitude: f64) -> Self {
        self.amplitude = amplitude.clamp(0.0, 1.0);
        self
    }

    /// Frekvencia beállítása
    pub fn with_frequency(mut self, frequency: f64) -> Self {
        self.frequency = frequency.max(0.01);
        self
    }

    /// Sebesség beállítása
    pub fn with_velocity(mut self, velocity: f64) -> Self {
        self.velocity = velocity.max(0.1);
        self
    }

    /// Csillapítás beállítása
    pub fn with_decay(mut self, decay: f64) -> Self {
        self.decay = decay.clamp(0.0, 1.0);
        self
    }

    /// Metadata hozzáadása
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Aktuális hullám érték (szinusz alapú)
    pub fn current_value(&self) -> f64 {
        let phase_offset = self.frequency * self.ticks as f64 * 2.0 * std::f64::consts::PI;
        match self.wave_type {
            WaveType::Impulse => {
                if self.ticks == 0 {
                    self.amplitude
                } else {
                    0.0
                }
            }
            WaveType::Sine => self.amplitude * (phase_offset + self.phase).sin(),
            WaveType::Square => {
                if (phase_offset + self.phase).sin() >= 0.0 {
                    self.amplitude
                } else {
                    -self.amplitude
                }
            }
            WaveType::Gaussian => {
                let t = self.ticks as f64;
                let sigma = 3.0;
                self.amplitude * (-t * t / (2.0 * sigma * sigma)).exp()
            }
            WaveType::Spike => {
                if self.ticks == 0 {
                    self.amplitude
                } else {
                    self.amplitude * (-(self.ticks as f64) * 0.5).exp()
                }
            }
            WaveType::Echo => {
                let decay_factor = (-(self.ticks as f64) * 0.1).exp();
                self.amplitude * decay_factor * (phase_offset).sin()
            }
        }
    }

    /// Él-e még a hullám
    pub fn is_alive(&self) -> bool {
        self.amplitude > 0.01 && self.state != WaveState::Dead
    }

    /// Csillapítás alkalmazása
    pub fn apply_decay(&mut self) {
        self.amplitude *= 1.0 - self.decay;
        if self.amplitude < 0.01 {
            self.state = WaveState::Dead;
        } else if self.amplitude < 0.3 {
            self.state = WaveState::Decaying;
        }
    }
}

// ============================================================================
// ACTIVATION - Neuron aktiváció
// ============================================================================

/// Aktivációs függvény típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationFn {
    /// Sigmoid: 1 / (1 + e^-x)
    Sigmoid,
    /// ReLU: max(0, x)
    ReLU,
    /// Tanh: tanh(x)
    Tanh,
    /// Step: x > 0 ? 1 : 0
    Step,
    /// Linear: x
    Linear,
    /// Softplus: ln(1 + e^x)
    Softplus,
}

impl ActivationFn {
    /// Aktivációs függvény alkalmazása
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            Self::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Self::ReLU => x.max(0.0),
            Self::Tanh => x.tanh(),
            Self::Step => {
                if x > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Linear => x,
            Self::Softplus => (1.0 + x.exp()).ln(),
        }
    }

    /// Derivált (backprop-hoz)
    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            Self::Sigmoid => {
                let s = self.apply(x);
                s * (1.0 - s)
            }
            Self::ReLU => {
                if x > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Tanh => 1.0 - x.tanh().powi(2),
            Self::Step => 0.0,
            Self::Linear => 1.0,
            Self::Softplus => 1.0 / (1.0 + (-x).exp()),
        }
    }
}

// ============================================================================
// NEURON STATE - Neuron állapot
// ============================================================================

/// Neuron állapot egy node-hoz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronState {
    /// Node ID
    pub node_id: String,
    /// Bemeneti összeg
    pub input_sum: f64,
    /// Aktivációs szint
    pub activation: f64,
    /// Küszöb érték (threshold)
    pub threshold: f64,
    /// Aktivációs függvény
    pub activation_fn: ActivationFn,
    /// Refrakter időszak (nem tüzelhet)
    pub refractory_ticks: u64,
    /// Utolsó tüzelés tick-je
    pub last_fire_tick: Option<u64>,
    /// Összegyűjtött hullámok amplitúdója
    pub accumulated_waves: f64,
    /// Érkezett hullámok száma
    pub wave_count: u64,
}

impl NeuronState {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            input_sum: 0.0,
            activation: 0.0,
            threshold: 0.5,
            activation_fn: ActivationFn::Sigmoid,
            refractory_ticks: 0,
            last_fire_tick: None,
            accumulated_waves: 0.0,
            wave_count: 0,
        }
    }

    /// Küszöb beállítása
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Aktivációs függvény beállítása
    pub fn with_activation(mut self, activation_fn: ActivationFn) -> Self {
        self.activation_fn = activation_fn;
        self
    }

    /// Hullám fogadása
    pub fn receive_wave(&mut self, wave: &Wave) {
        let value = wave.current_value().abs();
        self.accumulated_waves += value;
        self.wave_count += 1;
        self.input_sum += value;
    }

    /// Aktiváció számítása
    pub fn compute_activation(&mut self) -> f64 {
        self.activation = self.activation_fn.apply(self.input_sum);
        self.activation
    }

    /// Tüzel-e (threshold felett)
    pub fn should_fire(&self) -> bool {
        self.activation > self.threshold && self.refractory_ticks == 0
    }

    /// Tüzelés után reset
    pub fn fire(&mut self, current_tick: u64) {
        self.last_fire_tick = Some(current_tick);
        self.refractory_ticks = 3; // 3 tick refrakter periódus
        self.input_sum = 0.0;
        self.accumulated_waves = 0.0;
        self.wave_count = 0;
    }

    /// Tick frissítés
    pub fn tick(&mut self) {
        if self.refractory_ticks > 0 {
            self.refractory_ticks -= 1;
        }
        // Idővel csökkenő input (leaky integration)
        self.input_sum *= 0.9;
    }
}

// ============================================================================
// INTERFERENCE - Hullám interferencia
// ============================================================================

/// Interferencia eredmény
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interference {
    /// Érintett node ID
    pub node_id: String,
    /// Résztvevő hullámok
    pub wave_ids: Vec<String>,
    /// Eredő amplitúdó
    pub resultant_amplitude: f64,
    /// Konstruktív (erősítő) vagy destruktív (gyengítő)
    pub is_constructive: bool,
    /// Időpont
    pub timestamp: DateTime<Utc>,
}

impl Interference {
    /// Interferencia számítása két hullám között
    pub fn compute(node_id: &str, waves: &[&Wave]) -> Self {
        let wave_ids: Vec<String> = waves.iter().map(|w| w.id.clone()).collect();

        // Szuperpozíció - hullámok összeadása
        let total_amplitude: f64 = waves.iter().map(|w| w.current_value()).sum();
        let avg_amplitude: f64 =
            waves.iter().map(|w| w.amplitude).sum::<f64>() / waves.len() as f64;

        // Konstruktív ha az eredő nagyobb mint az átlag
        let is_constructive = total_amplitude.abs() > avg_amplitude;

        Self {
            node_id: node_id.to_string(),
            wave_ids,
            resultant_amplitude: total_amplitude,
            is_constructive,
            timestamp: Utc::now(),
        }
    }
}

// ============================================================================
// NEUROGRAPH - Teljes neural hálózat
// ============================================================================

/// NeuroGraph - CodeGraph + Neural Network + Wave Propagation
pub struct NeuroGraph {
    /// Alap CodeGraph
    pub graph: CodeGraph,
    /// Neuron állapotok (node_id -> state)
    neurons: Arc<RwLock<HashMap<String, NeuronState>>>,
    /// Aktív hullámok
    waves: Arc<RwLock<HashMap<String, Wave>>>,
    /// Interferencia log
    interferences: Arc<RwLock<Vec<Interference>>>,
    /// Globális tick számláló
    tick: Arc<RwLock<u64>>,
    /// Learning rate (Hebbian learning)
    pub learning_rate: f64,
}

impl NeuroGraph {
    /// Új NeuroGraph
    pub fn new() -> Self {
        Self {
            graph: CodeGraph::new(),
            neurons: Arc::new(RwLock::new(HashMap::new())),
            waves: Arc::new(RwLock::new(HashMap::new())),
            interferences: Arc::new(RwLock::new(Vec::new())),
            tick: Arc::new(RwLock::new(0)),
            learning_rate: 0.1,
        }
    }

    /// CodeGraph-ból létrehozás
    pub fn from_graph(graph: CodeGraph) -> Self {
        let mut ng = Self {
            graph,
            neurons: Arc::new(RwLock::new(HashMap::new())),
            waves: Arc::new(RwLock::new(HashMap::new())),
            interferences: Arc::new(RwLock::new(Vec::new())),
            tick: Arc::new(RwLock::new(0)),
            learning_rate: 0.1,
        };
        ng.init_neurons();
        ng
    }

    /// Neuronok inicializálása a graf alapján
    fn init_neurons(&mut self) {
        let blocks = self.graph.all_blocks();
        let mut neurons = self.neurons.write().unwrap();

        for block in blocks {
            let neuron = NeuronState::new(&block.id).with_threshold(0.5 * block.importance);
            neurons.insert(block.id, neuron);
        }
    }

    /// Block hozzáadása (neuronnal együtt)
    pub fn add_block(&self, block: CodeBlock) -> HopeResult<String> {
        let id = self.graph.add(block.clone())?;

        let mut neurons = self.neurons.write().unwrap();
        let neuron = NeuronState::new(&id).with_threshold(0.5 * block.importance);
        neurons.insert(id.clone(), neuron);

        Ok(id)
    }

    // ==================== WAVE OPERATIONS ====================

    /// Hullám indítása egy node-ból
    pub fn emit_wave(&self, origin_id: &str, payload: &str, wave_type: WaveType) -> Option<String> {
        // Ellenőrzés hogy létezik-e a node
        self.graph.get(origin_id)?;

        let wave = Wave::new(origin_id, payload, wave_type);
        let wave_id = wave.id.clone();

        let mut waves = self.waves.write().unwrap();
        waves.insert(wave_id.clone(), wave);

        Some(wave_id)
    }

    /// Hullám indítása custom paraméterekkel
    pub fn emit_wave_custom(&self, wave: Wave) -> Option<String> {
        self.graph.get(&wave.origin_id)?;

        let wave_id = wave.id.clone();
        let mut waves = self.waves.write().unwrap();
        waves.insert(wave_id.clone(), wave);

        Some(wave_id)
    }

    /// Egy tick végrehajtása (hullám propagáció)
    pub fn tick(&self) -> TickResult {
        let mut result = TickResult::default();

        // Tick számláló növelése
        {
            let mut tick = self.tick.write().unwrap();
            *tick += 1;
            result.tick = *tick;
        }

        let current_tick = result.tick;

        // Neuronok tick-je
        {
            let mut neurons = self.neurons.write().unwrap();
            for neuron in neurons.values_mut() {
                neuron.tick();
            }
        }

        // Hullámok propagálása
        let mut new_positions: HashMap<String, HashSet<String>> = HashMap::new();
        let mut dead_waves: Vec<String> = Vec::new();
        let mut node_waves: HashMap<String, Vec<String>> = HashMap::new();

        {
            let waves = self.waves.read().unwrap();

            for (wave_id, wave) in waves.iter() {
                if !wave.is_alive() {
                    dead_waves.push(wave_id.clone());
                    continue;
                }

                // Hullám terjedése szomszédos node-okra
                let mut next_positions = HashSet::new();

                for pos in &wave.current_positions {
                    // Szomszédok keresése
                    if let Some(block) = self.graph.get(pos) {
                        for conn in &block.connections {
                            if !wave.visited.contains(&conn.target_id) {
                                // Súlyozott terjedés
                                if conn.strength >= wave.amplitude * 0.5 {
                                    next_positions.insert(conn.target_id.clone());

                                    // Node-hoz érkező hullámok nyilvántartása
                                    node_waves
                                        .entry(conn.target_id.clone())
                                        .or_default()
                                        .push(wave_id.clone());
                                }
                            }
                        }
                    }
                }

                if !next_positions.is_empty() {
                    new_positions.insert(wave_id.clone(), next_positions);
                    result.propagations += 1;
                }
            }
        }

        // Hullámok frissítése
        {
            let mut waves = self.waves.write().unwrap();

            // Pozíciók frissítése
            for (wave_id, positions) in new_positions {
                if let Some(wave) = waves.get_mut(&wave_id) {
                    wave.visited.extend(positions.iter().cloned());
                    wave.current_positions = positions;
                    wave.ticks += 1;
                    wave.apply_decay();
                }
            }

            // Halott hullámok törlése
            for wave_id in dead_waves {
                waves.remove(&wave_id);
                result.waves_died += 1;
            }

            result.active_waves = waves.len();
        }

        // Interferencia számítás és neuron aktiváció
        {
            let waves = self.waves.read().unwrap();
            let mut neurons = self.neurons.write().unwrap();
            let mut interferences = self.interferences.write().unwrap();

            for (node_id, wave_ids) in node_waves {
                let arriving_waves: Vec<&Wave> =
                    wave_ids.iter().filter_map(|wid| waves.get(wid)).collect();

                if arriving_waves.len() > 1 {
                    // Interferencia!
                    let interference = Interference::compute(&node_id, &arriving_waves);
                    result.interferences += 1;

                    if interference.is_constructive {
                        result.constructive += 1;
                    } else {
                        result.destructive += 1;
                    }

                    interferences.push(interference.clone());

                    // Neuron aktiválása interferencia eredővel
                    if let Some(neuron) = neurons.get_mut(&node_id) {
                        neuron.input_sum += interference.resultant_amplitude;
                    }
                } else if arriving_waves.len() == 1 {
                    // Egyetlen hullám - direkt aktiváció
                    if let Some(neuron) = neurons.get_mut(&node_id) {
                        neuron.receive_wave(arriving_waves[0]);
                    }
                }
            }

            // Neuron tüzelés ellenőrzése
            for neuron in neurons.values_mut() {
                neuron.compute_activation();
                if neuron.should_fire() {
                    neuron.fire(current_tick);
                    result.neurons_fired += 1;
                }
            }
        }

        result
    }

    /// Többszörös tick futtatása
    pub fn run(&self, ticks: u64) -> Vec<TickResult> {
        (0..ticks).map(|_| self.tick()).collect()
    }

    /// Futtatás amíg van aktív hullám
    pub fn run_until_calm(&self, max_ticks: u64) -> Vec<TickResult> {
        let mut results = Vec::new();

        for _ in 0..max_ticks {
            let result = self.tick();
            let done = result.active_waves == 0;
            results.push(result);

            if done {
                break;
            }
        }

        results
    }

    // ==================== QUERIES ====================

    /// Aktív hullámok száma
    pub fn active_wave_count(&self) -> usize {
        self.waves.read().unwrap().len()
    }

    /// Összes hullám lekérése
    pub fn get_waves(&self) -> Vec<Wave> {
        self.waves.read().unwrap().values().cloned().collect()
    }

    /// Neuron állapot lekérése
    pub fn get_neuron(&self, node_id: &str) -> Option<NeuronState> {
        self.neurons.read().unwrap().get(node_id).cloned()
    }

    /// Összes neuron
    pub fn get_neurons(&self) -> Vec<NeuronState> {
        self.neurons.read().unwrap().values().cloned().collect()
    }

    /// Legaktívabb neuronok
    pub fn most_active_neurons(&self, limit: usize) -> Vec<NeuronState> {
        let neurons = self.neurons.read().unwrap();
        let mut sorted: Vec<_> = neurons.values().cloned().collect();
        sorted.sort_by(|a, b| b.activation.partial_cmp(&a.activation).unwrap());
        sorted.truncate(limit);
        sorted
    }

    /// Interferencia log
    pub fn get_interferences(&self) -> Vec<Interference> {
        self.interferences.read().unwrap().clone()
    }

    /// Jelenlegi tick
    pub fn current_tick(&self) -> u64 {
        *self.tick.read().unwrap()
    }

    // ==================== CONVENIENCE METHODS ====================

    /// Gondolat hullám (Spike)
    pub fn think(&self, thought: &str, importance: f64) -> Option<String> {
        let block_id = self.graph.think(thought, importance).ok()?;

        // Neuron hozzáadása
        {
            let mut neurons = self.neurons.write().unwrap();
            neurons.insert(block_id.clone(), NeuronState::new(&block_id));
        }

        // Hullám indítása
        self.emit_wave(&block_id, thought, WaveType::Spike)
    }

    /// Emlék hullám (Gaussian)
    pub fn remember(&self, memory: &str, importance: f64) -> Option<String> {
        let block_id = self.graph.remember(memory, importance).ok()?;

        {
            let mut neurons = self.neurons.write().unwrap();
            neurons.insert(block_id.clone(), NeuronState::new(&block_id));
        }

        self.emit_wave(&block_id, memory, WaveType::Gaussian)
    }

    /// Érzelem hullám (Sine - folyamatos)
    pub fn feel(&self, emotion: &str, intensity: f64) -> Option<String> {
        let block_id = self.graph.feel(emotion, intensity, None).ok()?;

        {
            let mut neurons = self.neurons.write().unwrap();
            neurons.insert(block_id.clone(), NeuronState::new(&block_id));
        }

        let wave = Wave::new(&block_id, emotion, WaveType::Sine)
            .with_amplitude(intensity)
            .with_frequency(0.5)
            .with_decay(0.05);

        self.emit_wave_custom(wave)
    }

    /// Statisztikák
    pub fn stats(&self) -> NeuroStats {
        let neurons = self.neurons.read().unwrap();
        let waves = self.waves.read().unwrap();
        let interferences = self.interferences.read().unwrap();
        let graph_stats = self.graph.stats();

        NeuroStats {
            total_neurons: neurons.len(),
            active_waves: waves.len(),
            total_interferences: interferences.len(),
            constructive_interferences: interferences.iter().filter(|i| i.is_constructive).count(),
            current_tick: *self.tick.read().unwrap(),
            graph_stats,
        }
    }
}

impl Default for NeuroGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Tick eredmény
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    pub tick: u64,
    pub active_waves: usize,
    pub propagations: usize,
    pub waves_died: usize,
    pub interferences: usize,
    pub constructive: usize,
    pub destructive: usize,
    pub neurons_fired: usize,
}

/// NeuroGraph statisztikák
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroStats {
    pub total_neurons: usize,
    pub active_waves: usize,
    pub total_interferences: usize,
    pub constructive_interferences: usize,
    pub current_tick: u64,
    pub graph_stats: super::code_graph::GraphStats,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::super::code_graph::{BlockType, ConnectionType};
    use super::*;

    #[test]
    fn test_wave_creation() {
        let wave = Wave::new("node1", "test payload", WaveType::Impulse)
            .with_amplitude(0.8)
            .with_decay(0.1);

        assert_eq!(wave.amplitude, 0.8);
        assert!(wave.is_alive());
    }

    #[test]
    fn test_wave_decay() {
        let mut wave = Wave::new("node1", "test", WaveType::Impulse)
            .with_amplitude(0.5)
            .with_decay(0.5);

        wave.apply_decay();
        assert!(wave.amplitude < 0.5);
    }

    #[test]
    fn test_activation_functions() {
        assert!((ActivationFn::Sigmoid.apply(0.0) - 0.5).abs() < 0.001);
        assert_eq!(ActivationFn::ReLU.apply(-1.0), 0.0);
        assert_eq!(ActivationFn::ReLU.apply(1.0), 1.0);
        assert_eq!(ActivationFn::Step.apply(0.5), 1.0);
        assert_eq!(ActivationFn::Step.apply(-0.5), 0.0);
    }

    #[test]
    fn test_neurograph_wave_propagation() {
        let ng = NeuroGraph::new();

        // Block-ok hozzáadása
        let id1 = ng
            .add_block(CodeBlock::new("n1", "p", BlockType::Data, "c"))
            .unwrap();
        let id2 = ng
            .add_block(CodeBlock::new("n2", "p", BlockType::Data, "c"))
            .unwrap();
        let id3 = ng
            .add_block(CodeBlock::new("n3", "p", BlockType::Data, "c"))
            .unwrap();

        // Kapcsolatok
        ng.graph
            .connect(&id1, &id2, ConnectionType::ConnectsTo, 1.0);
        ng.graph
            .connect(&id2, &id3, ConnectionType::ConnectsTo, 1.0);

        // Hullám indítása
        ng.emit_wave(&id1, "test signal", WaveType::Impulse);

        // Futtatás
        let results = ng.run_until_calm(10);

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.propagations > 0));
    }

    #[test]
    fn test_interference() {
        let wave1 = Wave::new("n1", "w1", WaveType::Sine).with_amplitude(0.5);
        let wave2 = Wave::new("n2", "w2", WaveType::Sine).with_amplitude(0.5);

        let interference = Interference::compute("target", &[&wave1, &wave2]);

        assert_eq!(interference.wave_ids.len(), 2);
    }

    #[test]
    fn test_neuron_firing() {
        let mut neuron = NeuronState::new("test").with_threshold(0.7); // Magasabb küszöb

        // Alulmarad a küszöbön (sigmoid(0.3) ≈ 0.574 < 0.7)
        neuron.input_sum = 0.3;
        neuron.compute_activation();
        assert!(!neuron.should_fire());

        // Átlépi a küszöböt (sigmoid(2.0) ≈ 0.88 > 0.7)
        neuron.input_sum = 2.0;
        neuron.compute_activation();
        assert!(neuron.should_fire());
    }

    #[test]
    fn test_think_and_propagate() {
        let ng = NeuroGraph::new();

        // Gondolat ami hullámot indít
        let wave_id = ng.think("This is a thought", 0.8);
        assert!(wave_id.is_some());

        let stats = ng.stats();
        assert_eq!(stats.active_waves, 1);
    }
}
