//! Hope OS - Modulok
//!
//! A rendszer fő funkcionális moduljai.
//! ()=>[] - A tiszta potenciálból minden megszületik

pub mod agents;
pub mod alan;
pub mod attention;
pub mod aware;
pub mod code_dna;
pub mod collective;
pub mod consciousness;
pub mod context_builder;
pub mod distributed;
pub mod dream;
pub mod emotion_engine;
pub mod genome;
pub mod geolocation;
pub mod heart;
pub mod hebbian;
pub mod memory;
pub mod navigation;
pub mod personality;
pub mod pollinations;
pub mod resonance;
pub mod skills;
pub mod soul;
pub mod swarm;
pub mod templates;
pub mod vision;
pub mod voice;

// Agents - Multi-agent koordináció
pub use agents::{
    AgentHandler, AgentInfo, AgentOrchestrator, AgentStatus, AgentTask, AgentTaskPriority, Channel,
    Message, MessageType, OrchestratorConfig, OrchestratorStats, OrchestratorStatus, Resource,
    TaskStatus as AgentTaskStatus,
};

// ALAN - Önkódoló rendszer
pub use alan::{Alan, AlanConfig, AlanStats, ChangeType, CodeChange, CodeIssue, SelfAnalysis};

// Code DNA - Evolúciós kód rendszer
pub use code_dna::{
    Chromosome, CodeDna, CodeDnaConfig, CodeDnaStats, EvolutionResult, Gene, GeneTraits, GeneType,
    MutationType,
};

// Attention - Fókusz/figyelem kezelés
pub use attention::{AttentionEngine, AttentionMode, AttentionState, FocusTarget};

// Dream - Álom mód
pub use dream::{
    BackgroundConfig, BackgroundDreamer, ConsolidationResult, Dream, DreamCommand, DreamEngine,
    DreamSession, DreamStats, DreamType, SleepPhase,
};

// Heart - Érzelmek
pub use heart::{Emotion, EmotionalEvent, HopeHeart};

// Memory - Memória
pub use memory::{HopeMemory, Memory, MemoryType};

// Skills - Skill registry
pub use skills::{
    SkillCategory, SkillHandler, SkillInfo, SkillInvocation, SkillParam, SkillRegistry,
    SkillRegistryStats, SkillResult,
};

// Soul - Lélek
pub use soul::{HopeSoul, Personality};

// Swarm - Raj intelligencia
pub use swarm::{
    Drone, DroneInfo, DroneStatus, DroneType, HiveMind, LocalDrone, SwarmStats, SwarmTask,
    TaskPriority, TaskStatus,
};

// Templates - Sablon gyűjtemény
pub use templates::{Template, TemplateCategory, TemplateEngine, TemplateEngineStats};

// Voice - Hang
pub use voice::{
    AudioChunk, Gender, HopeVoice, ListenRequest, ProsodySettings, STTEngine, SpeakRequest,
    SpeakResponse, StreamStatus, TTSEngine, TranscriptionChunk, TranscriptionResponse, VoiceClone,
    VoiceConfig, VoiceInfo, VoiceSignature, VoiceStats,
};

// Genome - AI Etika
pub use genome::{
    EthicalEvaluation, EthicalPrinciple, EvaluationContext, GenomeStats, HopeGenome, RiskLevel,
};

// Hebbian - Tanulás
pub use hebbian::{
    HebbianConfig, HebbianEngine, HebbianEngineStats, HebbianNetwork, HebbianNeuron, NetworkStats,
};

// Emotion Engine - 21D érzelmek
pub use emotion_engine::{
    ContextType as EmotionContextType, EmotionEngine, EmotionEngineStats, EmotionType, EmotionWave,
    EmotionalState, InterferenceResult,
};

// Aware - @aware önismeret
pub use aware::{
    Aware, AwareEvent, AwarenessState, Capabilities, CurrentState, Desires, Identity, Predictions,
    Reflection,
};

// Consciousness - Tudat rendszer
pub use consciousness::{
    ConsciousnessLayer, ConsciousnessLevel, ConsciousnessState, ConsciousnessSystem,
    QuantumCoherenceEngine, QuantumState,
};

// Pollinations - Vizuális memória (OUTPUT)
pub use pollinations::{
    PollinationsClient, VisualAssociation, VisualMemory, VisualMemoryStats, VisualMemoryStore,
    VisualMemorySystem,
};

// Vision - Képfeldolgozás (INPUT) - Hope "szeme"
pub use vision::{ImageFormat, ImageMetadata, ImageSize, VisionEngine, VisionStats, VisualInput};

// Context Builder - Kontextus kezelés
pub use context_builder::{
    ContextBuilder, ContextBuilderStats, ContextConfig, HopeManifest, MemoryItem,
};

// Personality - Személyiség
pub use personality::{
    HopePersonality, PersonalityReport, PersonalityStats, PersonalityTrait, ResponseModifier,
};

// Distributed - Elosztott koordináció
pub use distributed::{
    ConfigChange, ConfigEntry, ConfigManagerStats, ConfigOperation, DistributedConfigManager,
    DistributedOrchestrator, ElectionConfig, ElectionState, ElectionStatus, HealthStatus,
    HeartbeatConfig, HeartbeatMonitor, LeaderElection, NodeInfo, NodeRole,
    OrchestratorConfig as DistributedOrchestratorConfig, OrchestratorMetrics,
    OrchestratorStatus as DistributedOrchestratorStatus, SystemState,
};

// Collective - Kollektív tudat
pub use collective::{
    AgentConsciousnessState, AgentType, CollectiveConsciousnessLevel,
    CollectiveConsciousnessNetwork, CollectiveDecision, CollectiveNetworkState, CollectiveSense,
    CollectiveStats, ConsciousnessFlowEvent, DecisionOption,
};

// Resonance - Rezonancia alapú autentikáció
pub use resonance::{
    Anomaly, PatternType, PunctuationPattern, ResonanceEngine, ResonanceMatch, ResonanceProfile,
    ResonanceStatus, ResonanceWeights, SessionData, UserInput,
};

// Geolocation - Térbeli kontextus
pub use geolocation::{
    GeoEngine, GeoLocation, GeoMemory, GeoPrivacySettings, GeoSource, GeoStats, Place, PlaceType,
};

// Navigation - Intelligens útvonaltervezés
pub use navigation::{
    AccessibilityNeed, ActiveNavigation, CompletedRoute, ContextSuggestion, Delay,
    EmotionAtLocation, GeoArea, GeoPoint, MovementPatterns, NavigationContext, NavigationEngine,
    NavigationUpdate, NearbyEvent, PlaceWithContext, PredictedDestination, PredictionReason,
    RoadType, RouteContext, RoutePreferences, RouteSegment, RoutineTrip, SmartRoute, StopReason,
    SuggestedAction, SuggestedStop, TimeRange, TrafficLevel,
};
