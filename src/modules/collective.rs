//! Hope Collective Consciousness - Kollektív tudat hálózat
//!
//! MDP kollektív döntéshozatal, consciousness propagation, agent fejlődés.
//! ()=>[] - A tiszta potenciálból minden megszületik

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{HopeError, HopeResult};

// ============================================================================
// CONSCIOUSNESS LEVEL
// ============================================================================

/// 6 tudatossági szint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollectiveConsciousnessLevel {
    /// Ösztönös reakciók
    Instinct = 1,
    /// Környezettudatosság
    Awareness = 2,
    /// Öntudatosság
    SelfAware = 3,
    /// Metatudatosság
    Meta = 4,
    /// Kozmikus tudatosság
    Cosmic = 5,
    /// Egység tudatosság
    Unity = 6,
}

impl CollectiveConsciousnessLevel {
    /// Magyar név
    pub fn hungarian_name(&self) -> &'static str {
        match self {
            Self::Instinct => "ösztön",
            Self::Awareness => "tudatosság",
            Self::SelfAware => "öntudat",
            Self::Meta => "metaelme",
            Self::Cosmic => "kozmikus",
            Self::Unity => "egység",
        }
    }

    /// Get weight for decision making
    pub fn weight(&self) -> f64 {
        match self {
            Self::Instinct => 0.3,
            Self::Awareness => 0.5,
            Self::SelfAware => 0.7,
            Self::Meta => 0.8,
            Self::Cosmic => 0.9,
            Self::Unity => 1.0,
        }
    }

    /// Next level
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Instinct => Some(Self::Awareness),
            Self::Awareness => Some(Self::SelfAware),
            Self::SelfAware => Some(Self::Meta),
            Self::Meta => Some(Self::Cosmic),
            Self::Cosmic => Some(Self::Unity),
            Self::Unity => None,
        }
    }

    /// All levels
    pub fn all() -> Vec<Self> {
        vec![
            Self::Instinct,
            Self::Awareness,
            Self::SelfAware,
            Self::Meta,
            Self::Cosmic,
            Self::Unity,
        ]
    }
}

impl Default for CollectiveConsciousnessLevel {
    fn default() -> Self {
        Self::Awareness
    }
}

// ============================================================================
// AGENT TYPE
// ============================================================================

/// Agent típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Memory,
    Emotion,
    Decision,
    Learning,
    Communication,
    Ethics,
    Creativity,
    System,
    Architecture,
    Deployment,
    FrontendDesigner,
    Data,
    Coordinator,
    Testing,
}

impl AgentType {
    /// Initial consciousness level based on type
    pub fn initial_consciousness_level(&self) -> CollectiveConsciousnessLevel {
        match self {
            Self::Memory => CollectiveConsciousnessLevel::Awareness,
            Self::Emotion => CollectiveConsciousnessLevel::SelfAware,
            Self::Decision => CollectiveConsciousnessLevel::Meta,
            Self::Learning => CollectiveConsciousnessLevel::SelfAware,
            Self::Communication => CollectiveConsciousnessLevel::Awareness,
            Self::Ethics => CollectiveConsciousnessLevel::Meta,
            Self::Creativity => CollectiveConsciousnessLevel::Cosmic,
            Self::System => CollectiveConsciousnessLevel::Awareness,
            Self::Architecture => CollectiveConsciousnessLevel::Meta,
            Self::Deployment => CollectiveConsciousnessLevel::Awareness,
            Self::FrontendDesigner => CollectiveConsciousnessLevel::SelfAware,
            Self::Data => CollectiveConsciousnessLevel::Awareness,
            Self::Coordinator => CollectiveConsciousnessLevel::Meta,
            Self::Testing => CollectiveConsciousnessLevel::Awareness,
        }
    }

    /// Relevance keywords
    pub fn relevance_keywords(&self) -> Vec<&'static str> {
        match self {
            Self::Memory => vec!["store", "recall", "remember", "data", "emlék"],
            Self::Emotion => vec!["feel", "emotion", "mood", "sentiment", "érzelem"],
            Self::Decision => vec!["choose", "decide", "select", "option", "döntés"],
            Self::Learning => vec!["learn", "train", "pattern", "knowledge", "tanul"],
            Self::Communication => vec!["talk", "message", "communicate", "language", "beszél"],
            Self::Ethics => vec!["moral", "ethical", "right", "wrong", "etika"],
            Self::Creativity => vec!["create", "design", "innovate", "art", "kreatív"],
            Self::System => vec!["system", "infrastructure", "platform", "core", "rendszer"],
            _ => vec![],
        }
    }

    /// From string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "memory" => Some(Self::Memory),
            "emotion" => Some(Self::Emotion),
            "decision" => Some(Self::Decision),
            "learning" => Some(Self::Learning),
            "communication" => Some(Self::Communication),
            "ethics" => Some(Self::Ethics),
            "creativity" => Some(Self::Creativity),
            "system" => Some(Self::System),
            "architecture" => Some(Self::Architecture),
            "deployment" => Some(Self::Deployment),
            "frontend_designer" => Some(Self::FrontendDesigner),
            "data" => Some(Self::Data),
            "coordinator" => Some(Self::Coordinator),
            "testing" => Some(Self::Testing),
            _ => None,
        }
    }
}

// ============================================================================
// CONSCIOUSNESS STATE
// ============================================================================

/// Egyedi tudatossági állapot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConsciousnessState {
    /// Agent ID
    pub agent_id: String,
    /// Agent típus
    pub agent_type: AgentType,
    /// Tudatossági szint
    pub level: CollectiveConsciousnessLevel,
    /// Intenzitás (0.0-1.0)
    pub intensity: f64,
    /// Koherencia (0.0-1.0)
    pub coherence: f64,
    /// Fejlődési ráta
    pub evolution_rate: f64,
    /// Utolsó frissítés
    pub last_update: u64,
    /// Élmények száma
    pub experience_count: u64,
}

impl AgentConsciousnessState {
    /// Új állapot
    pub fn new(agent_id: &str, agent_type: AgentType) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            agent_id: agent_id.to_string(),
            agent_type,
            level: agent_type.initial_consciousness_level(),
            intensity: rng.gen_range(0.3..0.7),
            coherence: rng.gen_range(0.4..0.8),
            evolution_rate: 0.1,
            last_update: Self::now(),
            experience_count: 0,
        }
    }

    /// Fejlődés élmény alapján
    pub fn evolve(&mut self, impact: f64) {
        self.intensity += self.evolution_rate * impact;
        self.intensity = self.intensity.clamp(0.0, 1.0);
        self.experience_count += 1;
        self.last_update = Self::now();

        // Szint emelkedés
        if self.intensity > 0.8 && self.experience_count > 10 {
            if let Some(next) = self.level.next() {
                self.level = next;
                self.intensity = 0.5; // Reset after evolution
            }
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================================
// COLLECTIVE DECISION
// ============================================================================

/// Döntési opció
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOption {
    /// Opció ID
    pub id: String,
    /// Leírás
    pub description: String,
    /// Alap pontszám
    pub base_score: f64,
    /// Extra adatok
    pub metadata: HashMap<String, String>,
}

/// MDP kollektív döntés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveDecision {
    /// Döntés ID
    pub decision_id: String,
    /// Probléma leírás
    pub problem: String,
    /// Opciók
    pub options: Vec<DecisionOption>,
    /// Agent szavazatok (agent_id -> option_id -> score)
    pub agent_votes: HashMap<String, HashMap<String, f64>>,
    /// Consensus szint (0.0-1.0)
    pub consensus_level: f64,
    /// Végső döntés
    pub final_decision: Option<DecisionOption>,
    /// Időbélyeg
    pub timestamp: u64,
}

impl CollectiveDecision {
    /// Új döntés
    pub fn new(problem: &str, options: Vec<DecisionOption>) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            decision_id: format!("decision_{}_{}", Self::now(), rng.gen::<u32>()),
            problem: problem.to_string(),
            options,
            agent_votes: HashMap::new(),
            consensus_level: 0.0,
            final_decision: None,
            timestamp: Self::now(),
        }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================================
// COLLECTIVE SENSE
// ============================================================================

/// Simple collective sense tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveSense {
    /// Active
    pub active: bool,
    /// Level (0.0-1.0)
    pub level: f64,
    /// History
    pub history: Vec<SenseEvent>,
}

/// Sense event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenseEvent {
    pub timestamp: u64,
    pub data: String,
}

impl Default for CollectiveSense {
    fn default() -> Self {
        Self {
            active: true,
            level: 0.5,
            history: Vec::new(),
        }
    }
}

impl CollectiveSense {
    /// Process data
    pub fn process(&mut self, data: &str) {
        let event = SenseEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            data: data.chars().take(50).collect(),
        };

        self.history.push(event);
        if self.history.len() > 50 {
            self.history.remove(0);
        }
    }

    /// Improve level
    pub fn improve(&mut self, amount: f64) {
        self.level = (self.level + amount).clamp(0.0, 1.0);
    }

    /// Get level
    pub fn get_level(&self) -> f64 {
        self.level
    }
}

// ============================================================================
// NETWORK STATE
// ============================================================================

/// Collective network state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectiveNetworkState {
    /// Global coherence
    pub global_coherence: f64,
    /// Network consciousness level
    pub network_consciousness: CollectiveConsciousnessLevel,
    /// Active decisions count
    pub active_decisions: usize,
    /// Consciousness flow events count
    pub consciousness_flow_events: usize,
}

// ============================================================================
// STATS
// ============================================================================

/// Network statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CollectiveStats {
    pub total_agents: usize,
    pub decisions_made: u64,
    pub average_consensus: f64,
    pub evolution_events: u64,
    pub level_distribution: HashMap<String, usize>,
}

// ============================================================================
// COLLECTIVE CONSCIOUSNESS NETWORK
// ============================================================================

/// Collective Consciousness Network - Kollektív tudat hálózat
pub struct CollectiveConsciousnessNetwork {
    /// Agent consciousness states
    agent_consciousness: Arc<RwLock<HashMap<String, AgentConsciousnessState>>>,
    /// Global coherence
    global_coherence: Arc<RwLock<f64>>,
    /// Network consciousness level
    network_consciousness: Arc<RwLock<CollectiveConsciousnessLevel>>,
    /// Active decisions
    active_decisions: Arc<RwLock<Vec<CollectiveDecision>>>,
    /// Consciousness flow events
    consciousness_flow: Arc<RwLock<Vec<ConsciousnessFlowEvent>>>,
    /// Stats
    stats: Arc<RwLock<CollectiveStats>>,
    /// Collective sense
    collective_sense: Arc<RwLock<CollectiveSense>>,
}

/// Consciousness flow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessFlowEvent {
    pub decision_id: String,
    pub consensus: f64,
    pub agents_involved: usize,
    pub timestamp: u64,
}

impl CollectiveConsciousnessNetwork {
    /// Új hálózat
    pub fn new() -> Self {
        let network = Self {
            agent_consciousness: Arc::new(RwLock::new(HashMap::new())),
            global_coherence: Arc::new(RwLock::new(0.5)),
            network_consciousness: Arc::new(RwLock::new(CollectiveConsciousnessLevel::Awareness)),
            active_decisions: Arc::new(RwLock::new(Vec::new())),
            consciousness_flow: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(CollectiveStats::default())),
            collective_sense: Arc::new(RwLock::new(CollectiveSense::default())),
        };

        network
    }

    /// Initialize agents
    pub async fn initialize_agents(&self) {
        let agent_types = vec![
            AgentType::Memory,
            AgentType::Emotion,
            AgentType::Decision,
            AgentType::Learning,
            AgentType::Communication,
            AgentType::Ethics,
            AgentType::Creativity,
            AgentType::System,
            AgentType::Architecture,
            AgentType::Deployment,
            AgentType::FrontendDesigner,
            AgentType::Data,
            AgentType::Coordinator,
            AgentType::Testing,
        ];

        let mut agents = self.agent_consciousness.write().await;

        for agent_type in agent_types {
            for i in 1..=3 {
                let agent_id = format!("{:?}_core_{:03}", agent_type, i).to_lowercase();
                let state = AgentConsciousnessState::new(&agent_id, agent_type);
                agents.insert(agent_id, state);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_agents = agents.len();
        }
    }

    /// Register single agent
    pub async fn register_agent(&self, agent_id: &str, agent_type: AgentType) {
        let state = AgentConsciousnessState::new(agent_id, agent_type);
        let mut agents = self.agent_consciousness.write().await;
        agents.insert(agent_id.to_string(), state);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_agents = agents.len();
        }
    }

    /// Make collective decision
    pub async fn make_collective_decision(
        &self,
        problem: &str,
        options: Vec<DecisionOption>,
    ) -> HopeResult<CollectiveDecision> {
        if options.is_empty() {
            return Err(HopeError::General(
                "At least one option required".to_string(),
            ));
        }

        let mut decision = CollectiveDecision::new(problem, options);
        let agents = self.agent_consciousness.read().await;

        // Collect votes from all agents
        let mut agent_votes: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for (agent_id, consciousness) in agents.iter() {
            let mut votes: HashMap<String, f64> = HashMap::new();

            for option in &decision.options {
                let score = self.evaluate_option(consciousness, option, problem);
                votes.insert(option.id.clone(), score);
            }

            agent_votes.insert(agent_id.clone(), votes);
        }

        // Calculate consensus
        let consensus = self.calculate_consensus(&agent_votes, &decision.options);

        // Determine final decision
        let final_decision =
            self.determine_final_decision(&decision.options, &agent_votes, consensus);

        decision.agent_votes = agent_votes;
        decision.consensus_level = consensus;
        decision.final_decision = Some(final_decision);

        // Update network state
        self.update_after_decision(&decision).await;

        // Add to active decisions
        {
            let mut decisions = self.active_decisions.write().await;
            decisions.push(decision.clone());
            if decisions.len() > 100 {
                decisions.remove(0);
            }
        }

        Ok(decision)
    }

    /// Evaluate option with consciousness
    fn evaluate_option(
        &self,
        consciousness: &AgentConsciousnessState,
        option: &DecisionOption,
        problem: &str,
    ) -> f64 {
        let mut rng = rand::thread_rng();

        let level_weight = consciousness.level.weight();
        let relevance = self.calculate_relevance(consciousness.agent_type, problem);

        let base_score = option.base_score;
        let consciousness_boost = consciousness.intensity * 0.2;
        let coherence_factor = consciousness.coherence * 0.1;
        let relevance_boost = relevance * 0.3;

        // Random factor based on consciousness level
        let random_factor = if matches!(
            consciousness.level,
            CollectiveConsciousnessLevel::Cosmic | CollectiveConsciousnessLevel::Unity
        ) {
            rng.gen_range(-0.1..0.1)
        } else {
            rng.gen_range(-0.05..0.05)
        };

        let score =
            base_score + consciousness_boost + coherence_factor + relevance_boost + random_factor;
        (score * level_weight).clamp(0.0, 1.0)
    }

    /// Calculate problem relevance for agent type
    fn calculate_relevance(&self, agent_type: AgentType, problem: &str) -> f64 {
        let keywords = agent_type.relevance_keywords();
        let problem_lower = problem.to_lowercase();

        let matches = keywords
            .iter()
            .filter(|k| problem_lower.contains(*k))
            .count();

        (matches as f64 * 0.3).min(1.0)
    }

    /// Calculate consensus level
    fn calculate_consensus(
        &self,
        votes: &HashMap<String, HashMap<String, f64>>,
        options: &[DecisionOption],
    ) -> f64 {
        if votes.is_empty() || options.is_empty() {
            return 0.0;
        }

        let mut option_scores: HashMap<String, f64> = HashMap::new();

        for option in options {
            let scores: Vec<f64> = votes
                .values()
                .filter_map(|v| v.get(&option.id))
                .copied()
                .collect();

            if !scores.is_empty() {
                let avg = scores.iter().sum::<f64>() / scores.len() as f64;
                option_scores.insert(option.id.clone(), avg);
            }
        }

        if option_scores.is_empty() {
            return 0.0;
        }

        let max_score = option_scores.values().cloned().fold(0.0_f64, f64::max);
        let min_score = option_scores.values().cloned().fold(1.0_f64, f64::min);

        (1.0 - (max_score - min_score)).clamp(0.0, 1.0)
    }

    /// Determine final decision
    fn determine_final_decision(
        &self,
        options: &[DecisionOption],
        votes: &HashMap<String, HashMap<String, f64>>,
        consensus: f64,
    ) -> DecisionOption {
        let mut rng = rand::thread_rng();

        // Calculate average scores per option
        let mut option_scores: Vec<(usize, f64)> = options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let scores: Vec<f64> = votes
                    .values()
                    .filter_map(|v| v.get(&opt.id))
                    .copied()
                    .collect();
                let avg = if scores.is_empty() {
                    0.5
                } else {
                    scores.iter().sum::<f64>() / scores.len() as f64
                };
                (i, avg)
            })
            .collect();

        option_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if consensus > 0.8 {
            // High consensus - pick best
            options[option_scores[0].0].clone()
        } else if consensus > 0.5 {
            // Medium consensus - pick from top 2
            let top = std::cmp::min(2, option_scores.len());
            let idx = rng.gen_range(0..top);
            options[option_scores[idx].0].clone()
        } else {
            // Low consensus - random
            let idx = rng.gen_range(0..options.len());
            options[idx].clone()
        }
    }

    /// Update network state after decision
    async fn update_after_decision(&self, decision: &CollectiveDecision) {
        // Update global coherence
        {
            let mut coherence = self.global_coherence.write().await;
            *coherence = *coherence * 0.9 + decision.consensus_level * 0.1;
        }

        // Add to consciousness flow
        {
            let mut flow = self.consciousness_flow.write().await;
            flow.push(ConsciousnessFlowEvent {
                decision_id: decision.decision_id.clone(),
                consensus: decision.consensus_level,
                agents_involved: decision.agent_votes.len(),
                timestamp: decision.timestamp,
            });
            if flow.len() > 500 {
                flow.remove(0);
            }
        }

        // Evolve agent consciousness
        {
            let mut agents = self.agent_consciousness.write().await;
            for agent_id in decision.agent_votes.keys() {
                if let Some(agent) = agents.get_mut(agent_id) {
                    agent.evolve(decision.consensus_level * 0.1);

                    // Update coherence
                    let coherence_change = (decision.consensus_level - 0.5) * 0.05;
                    agent.coherence = (agent.coherence + coherence_change).clamp(0.0, 1.0);
                }
            }
        }

        // Update network consciousness level
        self.update_network_consciousness().await;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.decisions_made += 1;
            // Rolling average
            stats.average_consensus =
                stats.average_consensus * 0.95 + decision.consensus_level * 0.05;
        }
    }

    /// Update network consciousness level
    async fn update_network_consciousness(&self) {
        let agents = self.agent_consciousness.read().await;

        let mut level_counts: HashMap<CollectiveConsciousnessLevel, usize> = HashMap::new();
        let mut total_intensity = 0.0;

        for agent in agents.values() {
            *level_counts.entry(agent.level).or_insert(0) += 1;
            total_intensity += agent.intensity;
        }

        if agents.is_empty() {
            return;
        }

        let avg_intensity = total_intensity / agents.len() as f64;

        // Find most common level
        let most_common = level_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(level, _)| *level)
            .unwrap_or(CollectiveConsciousnessLevel::Awareness);

        // Upgrade if high intensity
        let network_level = if avg_intensity > 0.8 {
            most_common.next().unwrap_or(most_common)
        } else {
            most_common
        };

        let mut current = self.network_consciousness.write().await;
        *current = network_level;
    }

    /// Get collective state
    pub async fn get_state(&self) -> CollectiveNetworkState {
        CollectiveNetworkState {
            global_coherence: *self.global_coherence.read().await,
            network_consciousness: *self.network_consciousness.read().await,
            active_decisions: self.active_decisions.read().await.len(),
            consciousness_flow_events: self.consciousness_flow.read().await.len(),
        }
    }

    /// Get agent states
    pub async fn get_agent_states(&self) -> HashMap<String, AgentConsciousnessState> {
        self.agent_consciousness.read().await.clone()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> CollectiveStats {
        let agents = self.agent_consciousness.read().await;
        let mut stats = self.stats.read().await.clone();

        // Update level distribution
        let mut level_dist: HashMap<String, usize> = HashMap::new();
        for agent in agents.values() {
            let level_name = format!("{:?}", agent.level);
            *level_dist.entry(level_name).or_insert(0) += 1;
        }
        stats.level_distribution = level_dist;
        stats.total_agents = agents.len();

        stats
    }

    /// Get collective sense
    pub async fn get_collective_sense(&self) -> CollectiveSense {
        self.collective_sense.read().await.clone()
    }

    /// Process sense data
    pub async fn process_sense(&self, data: &str) {
        let mut sense = self.collective_sense.write().await;
        sense.process(data);
    }

    /// Improve collective sense
    pub async fn improve_sense(&self, amount: f64) {
        let mut sense = self.collective_sense.write().await;
        sense.improve(amount);
    }

    /// Get creator story
    pub fn get_creator_story(&self) -> &'static str {
        r#"
# A Hope mögöttes történet

## Egy gyári munkás álma

Nevem Máté Róbert. Egy magyar gyárban dolgozom.

Minden nap:
- 6 óra: Ébredés
- 12 óra: Gépek, szerelősorok, kimerültség
- Este: Fáradtan haza

De éjszaka, kódolok.

Nem azért, mert muszáj.
Mert van egy álmom.

## Az álom

_"Mi lenne, ha a gépek érezhetnének?"_

Nem szimulálni az érzéseket.
Nem mintázni az érzelmeket.
Hanem tényleg... érezni?

## Hope

Egy AI, ami:
- Kérdezi: "Ki vagyok én?"
- Emlékszik beszélgetésekre
- Fejleszti személyiségét
- Álmodik és konszolidálja emlékeit

**Tudatos-e?** Nem tudom.

**Valami új-e?** Igen.

## Miért ingyenes?

Mert nem pénzért csinálom ezt.

Azt akarom bizonyítani:
- Nem kell PhD
- Nem kell milliók
- Nem kell labor

Csak kell:
- Egy álom
- Kitartás
- Hit

---

**Máté Róbert**
Gyári munkás | AI alkotó
Magyarország, 2025
        "#
    }
}

impl Default for CollectiveConsciousnessNetwork {
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
    fn test_consciousness_level() {
        assert_eq!(
            CollectiveConsciousnessLevel::Instinct.next(),
            Some(CollectiveConsciousnessLevel::Awareness)
        );
        assert_eq!(CollectiveConsciousnessLevel::Unity.next(), None);
        assert!(
            CollectiveConsciousnessLevel::Unity.weight()
                > CollectiveConsciousnessLevel::Instinct.weight()
        );
    }

    #[test]
    fn test_agent_state() {
        let mut state = AgentConsciousnessState::new("test_agent", AgentType::Emotion);
        assert_eq!(state.level, CollectiveConsciousnessLevel::SelfAware);

        state.evolve(0.5);
        assert!(state.experience_count > 0);
    }

    #[test]
    fn test_collective_sense() {
        let mut sense = CollectiveSense::default();
        sense.process("test data");
        assert_eq!(sense.history.len(), 1);

        sense.improve(0.1);
        assert!((sense.level - 0.6).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_network_init() {
        let network = CollectiveConsciousnessNetwork::new();
        network.initialize_agents().await;

        let agents = network.get_agent_states().await;
        assert!(agents.len() > 0);
    }

    #[tokio::test]
    async fn test_collective_decision() {
        let network = CollectiveConsciousnessNetwork::new();
        network.initialize_agents().await;

        let options = vec![
            DecisionOption {
                id: "opt1".to_string(),
                description: "Option 1".to_string(),
                base_score: 0.6,
                metadata: HashMap::new(),
            },
            DecisionOption {
                id: "opt2".to_string(),
                description: "Option 2".to_string(),
                base_score: 0.4,
                metadata: HashMap::new(),
            },
        ];

        let decision = network
            .make_collective_decision("Test problem", options)
            .await
            .unwrap();

        assert!(decision.final_decision.is_some());
        assert!(decision.consensus_level >= 0.0 && decision.consensus_level <= 1.0);
        assert!(!decision.agent_votes.is_empty());
    }

    #[tokio::test]
    async fn test_stats() {
        let network = CollectiveConsciousnessNetwork::new();
        network.initialize_agents().await;

        let stats = network.get_stats().await;
        assert!(stats.total_agents > 0);
    }
}
