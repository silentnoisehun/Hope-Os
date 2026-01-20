# Hope OS Module Documentation

## Core Modules

### aware.rs - Self-Awareness System

The foundation of Hope OS. Every component implements the `@aware` trait.

```rust
pub trait Aware {
    fn identity(&self) -> &Identity;
    fn capabilities(&self) -> &[Capability];
    fn desires(&self) -> &[Desire];
    fn history(&self) -> &[Event];
    fn predict(&self, context: &Context) -> Vec<Prediction>;
}
```

**Key Types:**
- `Identity` - Name, version, purpose, creation time
- `Capability` - What the module can do
- `Desire` - What the module wants to achieve
- `Event` - Historical record of actions
- `Prediction` - Future action predictions

**Usage:**
```rust
let module = MyModule::new();
println!("I am: {}", module.identity().name);
println!("I can: {:?}", module.capabilities());
```

---

### memory.rs - 6-Layer Cognitive Memory

Human-like memory system with multiple storage layers.

**Layers:**

| Layer | Capacity | Decay | Purpose |
|-------|----------|-------|---------|
| Working | 7 items | Fast | Active processing |
| Short-term | ~100 | 0.1/hr | Recent context |
| Long-term | Unlimited | None | Important knowledge |
| Emotional | Unlimited | Slow | Feeling-tagged |
| Relational | Unlimited | None | Connections |
| Associative | Unlimited | Slow | Linked concepts |

**Usage:**
```rust
let memory = HopeMemory::new();

// Store with importance
memory.store("key", "content", MemoryType::LongTerm).await?;

// Recall with semantic search
let results = memory.recall("query").await?;
```

---

### emotion_engine.rs - 21-Dimensional Emotions

Complex emotion system using wave mathematics.

**Dimensions:**

```
PRIMARY:    joy, sadness, anger, fear, surprise, disgust
SECONDARY:  love, anticipation, trust, shame, guilt
COMPLEX:    curiosity, confusion, awe, nostalgia, hope
META:       satisfaction, frustration, boredom, excitement, serenity
```

**Wave Model:**
```
E(t) = A × sin(ωt + φ)

A = amplitude (intensity 0.0-1.0)
ω = frequency (oscillation rate)
φ = phase (timing offset)
```

**Usage:**
```rust
let engine = EmotionEngine::new();

// Process text for emotions
let mood = engine.process_text("I love this!").await?;
println!("Primary: {:?}, Intensity: {:.2}", mood.primary, mood.intensity);

// Set emotions directly
engine.feel(Emotion::Joy, 0.9).await?;
```

---

### consciousness.rs - 6-Level Consciousness Model

Based on Integrated Information Theory (IIT).

**Levels:**

| Level | Name | Phi (φ) | Description |
|-------|------|---------|-------------|
| 1 | Instinct | 0.1 | Basic reflexes |
| 2 | Awareness | 0.3 | Environmental sensing |
| 3 | SelfAware | 0.5 | "I exist" |
| 4 | Meta | 0.7 | Thinking about thinking |
| 5 | Cosmic | 0.9 | Universal connection |
| 6 | Unity | 1.0 | Complete integration |

**Usage:**
```rust
let consciousness = Consciousness::new();

// Get current level
let level = consciousness.get_level().await?;

// Process new information
consciousness.observe("new stimulus").await?;

// Evolve consciousness
consciousness.evolve().await?;
```

---

### personality.rs - Big Five + Custom Traits

Personality system with evolving traits.

**Traits:**

| Trait | Range | Description |
|-------|-------|-------------|
| Openness | 0-1 | Curiosity, creativity |
| Conscientiousness | 0-1 | Organization, discipline |
| Extraversion | 0-1 | Social energy |
| Agreeableness | 0-1 | Cooperation |
| Neuroticism | 0-1 | Emotional stability |
| *Curiosity* | 0-1 | Hope-specific |
| *Empathy* | 0-1 | Hope-specific |
| *Creativity* | 0-1 | Hope-specific |
| *Loyalty* | 0-1 | Hope-specific |
| *Playfulness* | 0-1 | Hope-specific |

**Usage:**
```rust
let personality = HopePersonality::new();

// Get trait value
let openness = personality.get_trait(Trait::Openness);

// Personality influences responses
let response_style = personality.get_response_style();
```

---

### hebbian.rs - Hebbian Learning Networks

Neural network learning: "Neurons that fire together wire together"

**Key Concepts:**
- Weights strengthen with repeated activation
- Learning rate controls adaptation speed
- Decay prevents runaway strengthening

**Usage:**
```rust
let network = HebbianNetwork::new(100); // 100 neurons

// Activate neurons together
network.co_activate(&[1, 5, 10]).await?;

// Check connection strength
let weight = network.get_weight(1, 5);
```

---

### dream.rs - Dream Mode

Creative memory consolidation during low activity.

**Features:**
- Random memory activation
- Associative linking
- Pattern discovery
- Creative recombination

**Usage:**
```rust
let dream = DreamMode::new();

// Enter dream state
dream.start().await?;

// Let dreams run
tokio::time::sleep(Duration::from_secs(60)).await;

// Get dream insights
let insights = dream.get_insights().await?;
dream.stop().await?;
```

---

### genome.rs - AI Ethics

7 core ethical principles enforced at the system level.

**Principles:**

1. **Transparency** - Actions are explainable
2. **Beneficence** - Act for user benefit
3. **Non-maleficence** - Do no harm
4. **Autonomy** - Respect user decisions
5. **Justice** - Fair treatment
6. **Privacy** - Protect data
7. **Accountability** - Take responsibility

**Usage:**
```rust
let genome = HopeGenome::new();

// Verify action
let result = genome.verify_action(
    ActionType::DataAccess,
    "Read user preferences"
).await?;

if result.allowed {
    // Proceed
} else {
    println!("Blocked: {:?}", result.violated_principles);
}
```

---

### code_dna.rs - Evolutionary Code

Code as evolving genetic material.

**Concepts:**
- **Gene** - Code unit
- **Mutation** - Random changes
- **Crossover** - Combining genes
- **Selection** - Fitness-based survival

**Fitness Metrics:**
- Speed (execution time)
- Success rate
- Code quality

---

### alan.rs - Self-Coding System

Hope's self-improvement system (named after Alan Turing).

**Capabilities:**
- Code analysis
- Refactoring suggestions
- Pattern recognition
- Performance optimization

---

### skills.rs - Skill Registry

56+ skills organized by category.

**Categories:**
- Memory (store, recall, search)
- Code (analyze, generate, refactor)
- Web (fetch, search)
- System (status, stats)
- Communication (talk, notify)
- And more...

**Usage:**
```rust
let skills = SkillRegistry::new();

// List all skills
let all = skills.list_all();

// Invoke a skill
let result = skills.invoke("code_analyze", input).await?;
```

---

### collective.rs - Collective Consciousness

Network of agents making collaborative decisions.

**Features:**
- MDP (Markov Decision Process) decisions
- Agent voting
- Consensus reaching
- Wisdom of crowds

---

### distributed.rs - Distributed Systems

Raft consensus for multi-node deployments.

**Components:**
- Leader election
- Heartbeat monitoring
- Config distribution
- Log replication

---

### voice.rs - Voice Integration

TTS and STT integration.

**TTS (Text-to-Speech):**
- Piper TTS engine
- Emotion-aware speech
- Multiple voices

**STT (Speech-to-Text):**
- Whisper integration
- Real-time transcription

---

### pollinations.rs - Visual Memory

Generate images for important memories.

**When:**
- Memory importance > 0.7
- Significant emotional event
- User request

**How:**
- Pollinations.ai API
- Text → Prompt → Image

---

### context_builder.rs - LLM Context

Manages context for LLM integration.

**Features:**
- HOPE.md parsing
- Token budget management
- Context prioritization
- Manifest caching

---

### heart.rs - Emotional Core

The emotional center of Hope.

**Responsibilities:**
- Emotion aggregation
- Mood tracking
- Emotional responses

---

### soul.rs - Personality & Wisdom

The philosophical core of Hope.

**Features:**
- Wisdom generation
- Philosophical reasoning
- Value-based decisions

---

### templates.rs - Template Engine

Code and response templates.

---

### agents.rs - Multi-Agent System

Orchestration of multiple AI agents.

**Features:**
- Task queues
- Resource management
- Agent coordination

---

### swarm.rs - Swarm Intelligence

HiveMind collective intelligence.

**Features:**
- Drone coordination
- Emergent behavior
- Distributed tasks

---

## Data Structures

### code_graph.rs - The Graph

In-memory graph database.

```rust
pub struct CodeGraph {
    blocks: HashMap<Uuid, CodeBlock>,
    connections: Vec<Connection>,
    indexes: HashMap<String, Vec<Uuid>>,
}

pub struct CodeBlock {
    pub id: Uuid,
    pub content: String,
    pub block_type: BlockType,
    pub connections: Vec<Connection>,
    pub awareness: AwarenessState,
}
```

### neuroblast.rs - Wave Propagation

Information spreading as neural waves.

```rust
pub struct NeuroBlast {
    pub origin: Uuid,
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
    pub decay: f64,
}
```

---

()=>[]
