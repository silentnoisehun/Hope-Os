//! Hope Hebbian Learning
//!
//! "Neurons that fire together wire together" - Donald Hebb
//!
//! Adaptív tanulás kapcsolat erősítéssel.
//! Pattern felismerés és skill fejlesztés.
//!
//! Hope neurális plaszticitása - minden interakcióból tanul.
//!
//! ()=>[] - A tiszta potenciálból a tanulás megszületik

use crate::core::HopeResult;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// NEURON
// ============================================================================

/// Neuron statisztikák
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuronStats {
    /// Neuron ID
    pub neuron_id: String,
    /// Tüzelések száma
    pub firing_count: u64,
    /// Tüzelési ráta (0.0 - 1.0)
    pub firing_rate: f64,
    /// Átlagos súly
    pub average_weight: f64,
    /// Maximum súly
    pub max_weight: f64,
}

/// Hebbian neuron
///
/// Implementálja a Hebbian szabályt:
/// Δw = η * x_pre * x_post
///
/// A kapcsolatok erősödnek amikor a neuronok együtt tüzelnek.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HebbianNeuron {
    /// Neuron ID
    pub neuron_id: String,
    /// Bemenet méret
    pub input_size: usize,
    /// Küszöbérték
    pub threshold: f64,
    /// Súlyok
    pub weights: Vec<f64>,
    /// Tanulási ráta
    pub learning_rate: f64,
    /// Hebbian erősség
    pub hebbian_strength: f64,
    /// Decay ráta (felejtés)
    pub decay_rate: f64,
    /// Maximum súly
    pub max_weight: f64,
    /// Minimum súly
    pub min_weight: f64,
    /// Utolsó aktiváció
    pub last_activation: f64,
    /// Tüzelések száma
    pub firing_count: u64,
    /// Aktiváció történet
    activation_history: Vec<f64>,
}

impl HebbianNeuron {
    /// Új neuron létrehozása
    pub fn new(neuron_id: &str, input_size: usize) -> Self {
        Self::with_threshold(neuron_id, input_size, 0.5)
    }

    /// Neuron létrehozása egyedi küszöbbel
    pub fn with_threshold(neuron_id: &str, input_size: usize, threshold: f64) -> Self {
        let mut rng = rand::thread_rng();

        // Random inicializálás normál eloszlással
        let weights: Vec<f64> = (0..input_size).map(|_| rng.gen_range(-0.1..0.1)).collect();

        Self {
            neuron_id: neuron_id.to_string(),
            input_size,
            threshold,
            weights,
            learning_rate: 0.01,
            hebbian_strength: 0.1,
            decay_rate: 0.001,
            max_weight: 1.0,
            min_weight: -1.0,
            last_activation: 0.0,
            firing_count: 0,
            activation_history: Vec::new(),
        }
    }

    /// Neuron aktiválása
    ///
    /// Sigmoid aktiváció és küszöb ellenőrzés.
    pub fn activate(&mut self, inputs: &[f64]) -> HopeResult<f64> {
        if inputs.len() != self.input_size {
            return Err(
                format!("Expected {} inputs, got {}", self.input_size, inputs.len()).into(),
            );
        }

        // Súlyozott összeg
        let weighted_sum: f64 = self
            .weights
            .iter()
            .zip(inputs.iter())
            .map(|(w, i)| w * i)
            .sum();

        // Sigmoid aktiváció
        let activation = 1.0 / (1.0 + (-weighted_sum).exp());

        // Küszöb ellenőrzés
        if activation >= self.threshold {
            self.firing_count += 1;
            self.last_activation = activation;
        } else {
            self.last_activation = 0.0;
        }

        // Történet tárolása (utolsó 100)
        self.activation_history.push(activation);
        if self.activation_history.len() > 100 {
            self.activation_history.remove(0);
        }

        Ok(self.last_activation)
    }

    /// Hebbian tanulási szabály alkalmazása
    ///
    /// Erősíti a kapcsolatokat az együtt tüzelő neuronok között.
    pub fn hebbian_update(&mut self, correlated_activations: &HashMap<String, f64>) {
        for (_, correlation) in correlated_activations {
            // Hebbian szabály: Δw = η * x_pre * x_post
            let delta_w = self.hebbian_strength * self.last_activation * correlation;

            // Súlyok frissítése (korlátozott)
            for weight in &mut self.weights {
                *weight = (*weight + delta_w).clamp(self.min_weight, self.max_weight);
            }
        }
    }

    /// Súly decay (felejtés)
    pub fn decay_weights(&mut self) {
        let decay_factor = 1.0 - self.decay_rate;
        for weight in &mut self.weights {
            *weight *= decay_factor;
            *weight = weight.clamp(self.min_weight, self.max_weight);
        }
    }

    /// Neuron statisztikák
    pub fn get_stats(&self) -> NeuronStats {
        let recent: Vec<&f64> = self.activation_history.iter().rev().take(50).collect();

        let firing_rate = if recent.is_empty() {
            0.0
        } else {
            recent.iter().filter(|&&a| *a >= self.threshold).count() as f64 / recent.len() as f64
        };

        let abs_weights: Vec<f64> = self.weights.iter().map(|w| w.abs()).collect();
        let average_weight = if abs_weights.is_empty() {
            0.0
        } else {
            abs_weights.iter().sum::<f64>() / abs_weights.len() as f64
        };
        let max_weight = abs_weights.iter().cloned().fold(0.0, f64::max);

        NeuronStats {
            neuron_id: self.neuron_id.clone(),
            firing_count: self.firing_count,
            firing_rate,
            average_weight,
            max_weight,
        }
    }
}

// ============================================================================
// NETWORK
// ============================================================================

/// Hebbian hálózat konfiguráció
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HebbianConfig {
    /// Bemenet méret
    pub input_size: usize,
    /// Rejtett rétegek méretei
    pub hidden_sizes: Vec<usize>,
    /// Kimenet méret
    pub output_size: usize,
    /// Tanulási ráta
    pub learning_rate: f64,
    /// Küszöbérték
    pub threshold: f64,
}

impl Default for HebbianConfig {
    fn default() -> Self {
        Self {
            input_size: 10,
            hidden_sizes: vec![20, 10],
            output_size: 5,
            learning_rate: 0.01,
            threshold: 0.5,
        }
    }
}

/// Hebbian hálózat
///
/// Többrétegű neurális hálózat Hebbian tanulással.
/// Pattern tanulás és skill adaptáció.
pub struct HebbianNetwork {
    /// Hálózat ID
    pub network_id: String,
    /// Konfiguráció
    config: HebbianConfig,
    /// Rétegek (neuronok listája rétegenként)
    layers: Vec<Vec<HebbianNeuron>>,
    /// Tanulási lépések száma
    pub training_steps: u64,
    /// Tanulási történet
    learning_history: Vec<LearningStep>,
}

/// Tanulási lépés rekord
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningStep {
    /// Lépés száma
    pub step: u64,
    /// Kimenet
    pub output: Vec<f64>,
    /// Hiba (ha van target)
    pub error: f64,
    /// Időbélyeg
    pub timestamp: f64,
}

impl HebbianNetwork {
    /// Új hálózat létrehozása
    pub fn new(network_id: &str, config: HebbianConfig) -> Self {
        let mut layers = Vec::new();

        // Réteg méretek számítása
        let mut layer_sizes = vec![config.input_size];
        layer_sizes.extend(config.hidden_sizes.clone());
        layer_sizes.push(config.output_size);

        // Rétegek építése
        for (i, &size) in layer_sizes[1..].iter().enumerate() {
            let prev_size = layer_sizes[i];
            let mut layer = Vec::new();

            for j in 0..size {
                let neuron_id = format!("L{}_N{}", i + 1, j);
                let mut neuron =
                    HebbianNeuron::with_threshold(&neuron_id, prev_size, config.threshold);
                neuron.learning_rate = config.learning_rate;
                layer.push(neuron);
            }

            layers.push(layer);
        }

        Self {
            network_id: network_id.to_string(),
            config,
            layers,
            training_steps: 0,
            learning_history: Vec::new(),
        }
    }

    /// Forward pass a hálózaton
    pub fn forward(&mut self, inputs: &[f64]) -> HopeResult<Vec<f64>> {
        let mut current = inputs.to_vec();

        for layer in &mut self.layers {
            let mut next_activations = Vec::with_capacity(layer.len());

            for neuron in layer {
                let activation = neuron.activate(&current)?;
                next_activations.push(activation);
            }

            current = next_activations;
        }

        Ok(current)
    }

    /// Egy Hebbian tanulási lépés
    pub fn learn(&mut self, inputs: &[f64], target: Option<&[f64]>) -> HopeResult<LearningStep> {
        // Forward pass
        let outputs = self.forward(inputs)?;

        // Előbb összegyűjtjük az aktivációkat minden rétegből
        let mut all_activations: Vec<Vec<f64>> = Vec::with_capacity(self.layers.len() + 1);
        all_activations.push(inputs.to_vec());
        for layer in &self.layers {
            let activations: Vec<f64> = layer.iter().map(|n| n.last_activation).collect();
            all_activations.push(activations);
        }

        // Hebbian tanulás minden rétegben
        for (i, layer) in self.layers.iter_mut().enumerate() {
            // Előző réteg aktivációi
            let prev_activations = &all_activations[i];

            // Minden neuron frissítése
            for neuron in layer {
                let mut correlated: HashMap<String, f64> = HashMap::new();
                for (j, &act) in prev_activations.iter().enumerate() {
                    correlated.insert(format!("input_{}", j), act);
                }
                neuron.hebbian_update(&correlated);
            }
        }

        // Decay alkalmazása
        for layer in &mut self.layers {
            for neuron in layer {
                neuron.decay_weights();
            }
        }

        self.training_steps += 1;

        // Hiba számítás ha van target
        let error = if let Some(target) = target {
            let mut sum_sq_error = 0.0;
            for (o, t) in outputs.iter().zip(target.iter()) {
                sum_sq_error += (o - t).powi(2);
            }
            sum_sq_error / outputs.len() as f64
        } else {
            0.0
        };

        let step = LearningStep {
            step: self.training_steps,
            output: outputs,
            error,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };

        // Történet tárolása (utolsó 1000)
        self.learning_history.push(step.clone());
        if self.learning_history.len() > 1000 {
            self.learning_history.remove(0);
        }

        Ok(step)
    }

    /// Hálózat statisztikák
    pub fn get_stats(&self) -> NetworkStats {
        let mut all_stats = Vec::new();
        for layer in &self.layers {
            for neuron in layer {
                all_stats.push(neuron.get_stats());
            }
        }

        let total_firing: u64 = all_stats.iter().map(|s| s.firing_count).sum();
        let avg_firing_rate = if all_stats.is_empty() {
            0.0
        } else {
            all_stats.iter().map(|s| s.firing_rate).sum::<f64>() / all_stats.len() as f64
        };
        let avg_weight = if all_stats.is_empty() {
            0.0
        } else {
            all_stats.iter().map(|s| s.average_weight).sum::<f64>() / all_stats.len() as f64
        };

        NetworkStats {
            network_id: self.network_id.clone(),
            training_steps: self.training_steps,
            total_neurons: all_stats.len(),
            total_firings: total_firing,
            avg_firing_rate,
            avg_weight,
            layers: self.layers.len(),
        }
    }

    /// Súlyok mentése
    pub fn save_weights(&self) -> NetworkWeights {
        let mut weights = HashMap::new();

        for (i, layer) in self.layers.iter().enumerate() {
            for (j, neuron) in layer.iter().enumerate() {
                let key = format!("L{}_N{}", i, j);
                weights.insert(key, neuron.weights.clone());
            }
        }

        NetworkWeights {
            network_id: self.network_id.clone(),
            input_size: self.config.input_size,
            output_size: self.config.output_size,
            weights,
            training_steps: self.training_steps,
        }
    }

    /// Súlyok betöltése
    pub fn load_weights(&mut self, data: &NetworkWeights) {
        for (i, layer) in self.layers.iter_mut().enumerate() {
            for (j, neuron) in layer.iter_mut().enumerate() {
                let key = format!("L{}_N{}", i, j);
                if let Some(w) = data.weights.get(&key) {
                    if w.len() == neuron.weights.len() {
                        neuron.weights = w.clone();
                    }
                }
            }
        }
        self.training_steps = data.training_steps;
    }
}

/// Hálózat statisztikák
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkStats {
    pub network_id: String,
    pub training_steps: u64,
    pub total_neurons: usize,
    pub total_firings: u64,
    pub avg_firing_rate: f64,
    pub avg_weight: f64,
    pub layers: usize,
}

/// Hálózat súlyok (perzisztencia)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkWeights {
    pub network_id: String,
    pub input_size: usize,
    pub output_size: usize,
    pub weights: HashMap<String, Vec<f64>>,
    pub training_steps: u64,
}

// ============================================================================
// HEBBIAN ENGINE
// ============================================================================

/// Hebbian Engine - több hálózat kezelése
pub struct HebbianEngine {
    /// Hálózatok
    networks: Arc<RwLock<HashMap<String, HebbianNetwork>>>,
    /// Statisztikák
    stats: Arc<RwLock<HebbianEngineStats>>,
}

/// Engine statisztikák
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HebbianEngineStats {
    pub total_networks: usize,
    pub total_training_steps: u64,
    pub total_neurons: usize,
}

impl HebbianEngine {
    /// Új engine
    pub fn new() -> Self {
        Self {
            networks: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(HebbianEngineStats::default())),
        }
    }

    /// Hálózat létrehozása
    pub async fn create_network(&self, network_id: &str, config: HebbianConfig) -> HopeResult<()> {
        let network = HebbianNetwork::new(network_id, config);

        {
            let mut networks = self.networks.write().await;
            networks.insert(network_id.to_string(), network);
        } // write lock released here

        self.update_stats().await;
        Ok(())
    }

    /// Hálózat lekérése
    pub async fn get_network(&self, network_id: &str) -> Option<NetworkStats> {
        let networks = self.networks.read().await;
        networks.get(network_id).map(|n| n.get_stats())
    }

    /// Forward pass
    pub async fn forward(&self, network_id: &str, inputs: &[f64]) -> HopeResult<Vec<f64>> {
        let mut networks = self.networks.write().await;
        let network = networks
            .get_mut(network_id)
            .ok_or_else(|| format!("Network not found: {}", network_id))?;

        network.forward(inputs)
    }

    /// Tanulás
    pub async fn learn(
        &self,
        network_id: &str,
        inputs: &[f64],
        target: Option<&[f64]>,
    ) -> HopeResult<LearningStep> {
        let mut networks = self.networks.write().await;
        let network = networks
            .get_mut(network_id)
            .ok_or_else(|| format!("Network not found: {}", network_id))?;

        let result = network.learn(inputs, target)?;

        drop(networks);
        self.update_stats().await;

        Ok(result)
    }

    /// Statisztikák frissítése
    async fn update_stats(&self) {
        let networks = self.networks.read().await;

        let total_networks = networks.len();
        let total_training_steps: u64 = networks.values().map(|n| n.training_steps).sum();
        let total_neurons: usize = networks
            .values()
            .map(|n| n.layers.iter().map(|l| l.len()).sum::<usize>())
            .sum();

        let mut stats = self.stats.write().await;
        stats.total_networks = total_networks;
        stats.total_training_steps = total_training_steps;
        stats.total_neurons = total_neurons;
    }

    /// Statisztikák
    pub async fn get_stats(&self) -> HebbianEngineStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Összes hálózat listázása
    pub async fn list_networks(&self) -> Vec<NetworkStats> {
        let networks = self.networks.read().await;
        networks.values().map(|n| n.get_stats()).collect()
    }

    /// Awareness
    pub async fn awareness(&self) -> HashMap<String, serde_json::Value> {
        let stats = self.get_stats().await;
        let networks = self.list_networks().await;

        let mut map = HashMap::new();
        map.insert("type".to_string(), serde_json::json!("HebbianEngine"));
        map.insert(
            "principle".to_string(),
            serde_json::json!("Neurons that fire together wire together"),
        );
        map.insert(
            "total_networks".to_string(),
            serde_json::json!(stats.total_networks),
        );
        map.insert(
            "total_neurons".to_string(),
            serde_json::json!(stats.total_neurons),
        );
        map.insert(
            "total_training_steps".to_string(),
            serde_json::json!(stats.total_training_steps),
        );
        map.insert(
            "networks".to_string(),
            serde_json::json!(networks
                .iter()
                .map(|n| serde_json::json!({
                    "id": n.network_id,
                    "neurons": n.total_neurons,
                    "steps": n.training_steps,
                    "avg_firing_rate": n.avg_firing_rate
                }))
                .collect::<Vec<_>>()),
        );
        map
    }
}

impl Default for HebbianEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_creation() {
        let neuron = HebbianNeuron::new("test_neuron", 5);

        assert_eq!(neuron.neuron_id, "test_neuron");
        assert_eq!(neuron.input_size, 5);
        assert_eq!(neuron.weights.len(), 5);
        assert_eq!(neuron.firing_count, 0);
    }

    #[test]
    fn test_neuron_activation() {
        let mut neuron = HebbianNeuron::with_threshold("test", 3, 0.5);

        // Set weights for predictable output
        neuron.weights = vec![0.5, 0.5, 0.5];

        let result = neuron.activate(&[1.0, 1.0, 1.0]).unwrap();

        // Should fire (weighted sum = 1.5, sigmoid > 0.5)
        assert!(result > 0.0);
        assert_eq!(neuron.firing_count, 1);
    }

    #[test]
    fn test_neuron_no_fire() {
        let mut neuron = HebbianNeuron::with_threshold("test", 3, 0.9);

        // Very low weights
        neuron.weights = vec![0.01, 0.01, 0.01];

        let result = neuron.activate(&[0.1, 0.1, 0.1]).unwrap();

        // Should not fire
        assert_eq!(result, 0.0);
        assert_eq!(neuron.firing_count, 0);
    }

    #[test]
    fn test_neuron_stats() {
        let neuron = HebbianNeuron::new("test", 5);
        let stats = neuron.get_stats();

        assert_eq!(stats.neuron_id, "test");
        assert_eq!(stats.firing_count, 0);
    }

    #[test]
    fn test_network_creation() {
        let config = HebbianConfig {
            input_size: 4,
            hidden_sizes: vec![8, 4],
            output_size: 2,
            ..Default::default()
        };

        let network = HebbianNetwork::new("test_net", config);

        assert_eq!(network.network_id, "test_net");
        assert_eq!(network.layers.len(), 3); // 2 hidden + 1 output
    }

    #[test]
    fn test_network_forward() {
        let config = HebbianConfig {
            input_size: 3,
            hidden_sizes: vec![4],
            output_size: 2,
            ..Default::default()
        };

        let mut network = HebbianNetwork::new("test", config);
        let outputs = network.forward(&[1.0, 0.5, 0.3]).unwrap();

        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_network_learning() {
        let config = HebbianConfig {
            input_size: 3,
            hidden_sizes: vec![4],
            output_size: 2,
            ..Default::default()
        };

        let mut network = HebbianNetwork::new("test", config);
        let step = network.learn(&[1.0, 0.5, 0.3], None).unwrap();

        assert_eq!(step.step, 1);
        assert_eq!(network.training_steps, 1);
    }

    #[test]
    fn test_network_learning_with_target() {
        let config = HebbianConfig {
            input_size: 3,
            hidden_sizes: vec![4],
            output_size: 2,
            ..Default::default()
        };

        let mut network = HebbianNetwork::new("test", config);
        let step = network.learn(&[1.0, 0.5, 0.3], Some(&[1.0, 0.0])).unwrap();

        assert!(step.error >= 0.0);
    }

    #[test]
    fn test_network_save_load_weights() {
        let config = HebbianConfig {
            input_size: 3,
            hidden_sizes: vec![4],
            output_size: 2,
            ..Default::default()
        };

        let mut network = HebbianNetwork::new("test", config.clone());

        // Train a bit
        for _ in 0..10 {
            network.learn(&[1.0, 0.5, 0.3], None).unwrap();
        }

        // Save
        let weights = network.save_weights();

        // Create new network and load
        let mut network2 = HebbianNetwork::new("test2", config);
        network2.load_weights(&weights);

        assert_eq!(network2.training_steps, 10);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_hebbian_engine() {
        let engine = HebbianEngine::new();

        let config = HebbianConfig {
            input_size: 4,
            hidden_sizes: vec![8],
            output_size: 2,
            ..Default::default()
        };

        engine.create_network("test_net", config).await.unwrap();

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_networks, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_engine_forward_and_learn() {
        let engine = HebbianEngine::new();

        let config = HebbianConfig {
            input_size: 3,
            hidden_sizes: vec![4],
            output_size: 2,
            ..Default::default()
        };

        engine.create_network("net1", config).await.unwrap();

        // Forward
        let outputs = engine.forward("net1", &[1.0, 0.5, 0.3]).await.unwrap();
        assert_eq!(outputs.len(), 2);

        // Learn
        let step = engine.learn("net1", &[1.0, 0.5, 0.3], None).await.unwrap();
        assert_eq!(step.step, 1);
    }
}
