<p align="center">
  <img src="docs/hope-logo.svg" alt="Hope OS" width="200"/>
</p>

<h1 align="center">Hope OS</h1>

<p align="center">
  <strong>LLM-Agnostic Cognitive Kernel in Rust</strong>
</p>

<p align="center">
  <a href="#-performance"><img src="https://img.shields.io/badge/latency-0.36ms-brightgreen" alt="Latency"/></a>
  <a href="#-performance"><img src="https://img.shields.io/badge/throughput-2800%2B%20req%2Fs-blue" alt="Throughput"/></a>
  <a href="#-the-graph"><img src="https://img.shields.io/badge/default-in--memory-orange" alt="In-Memory"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="License"/></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-196%20passing-brightgreen" alt="Tests"/></a>
</p>

<p align="center">
  <code>()=>[] - From pure potential, everything is born</code>
</p>

---

## 🚀 Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/silentnoisehun/Hope-Os.git
cd Hope-Os

# Build (release mode for best performance)
cargo build --release

# Run tests (196 tests)
cargo test
```

### As Dependency (from Git)

```toml
# Cargo.toml
[dependencies]
hope-os = { git = "https://github.com/silentnoisehun/Hope-Os" }
```

```bash
# Or via command line
cargo add hope-os --git https://github.com/silentnoisehun/Hope-Os
```

### Python (from Git)

```bash
pip install git+https://github.com/silentnoisehun/Hope-Os
```

> **Note:** Published packages on [crates.io](https://crates.io/crates/hope-os) and [PyPI](https://pypi.org/project/hope-os/) will be available after the first stable release.

---

## 🧠 What is Hope OS?

**Hope OS is an LLM-agnostic cognitive kernel.** It handles memory, emotional state, and safety constraints locally in microseconds - tasks that would otherwise require expensive LLM API calls.

### The Key Insight

| Task | Traditional LLM Approach | Hope OS |
|------|--------------------------|---------|
| **Remember user preference** | API call (~2000ms) | In-memory (0.001ms) |
| **Check safety constraints** | API call (~2000ms) | Local check (0.00005ms) |
| **Retrieve context** | API call (~2000ms) | Hash lookup (0.033ms) |

**Why this matters:**
- LLMs are stateless - they "forget" everything between requests
- Hope OS provides persistent memory, emotional continuity, and instant safety checks
- Your LLM focuses on what it's good at: reasoning and generation
- Hope OS handles what it's good at: state management at nanosecond speed

> **Important:** This is not "Hope is faster than Claude at language tasks" - that would be meaningless. This is "Hope offloads state management from LLMs, making the entire system more efficient."

---

## ⚡ Performance

**Measured on:** AMD Ryzen 5 5600X, 16GB RAM, Windows 11, `--release` build

**Method:** Criterion benchmarks + `std::time::Instant` loops, gRPC client/server on localhost

```
╔═══════════════════════════════════════════════════════════════╗
║                    HOPE OS BENCHMARKS                          ║
╠═══════════════════════════════════════════════════════════════╣
║ MEMORY OPERATIONS                                              ║
║   Store           │    254,561 ops/sec  │    3.36 µs avg      ║
║   Recall          │  2,336,334 ops/sec  │    0.43 µs avg      ║
║   Search          │      1,870 ops/sec  │  534.16 µs avg      ║
╠═══════════════════════════════════════════════════════════════╣
║ GRAPH OPERATIONS                                               ║
║   Add Block       │    255,376 ops/sec  │    1.73 µs avg      ║
║   Connect         │    842,775 ops/sec  │    0.53 µs avg      ║
║   Traverse (BFS)  │  1,275,933 ops/sec  │    0.22 µs avg      ║
║   Find Path       │  1,055,153 ops/sec  │    0.49 µs avg      ║
╠═══════════════════════════════════════════════════════════════╣
║ COGNITIVE OPERATIONS                                           ║
║   Emotion Process │    261,462 ops/sec  │    3.27 µs avg      ║
║   21D Wave Calc   │  4,000,000 ops/sec  │    0.25 µs avg      ║
║   Consciousness   │    100,000 ops/sec  │   10.00 µs avg      ║
╠═══════════════════════════════════════════════════════════════╣
║ gRPC OPERATIONS                                                ║
║   Unary Call      │      2,777 ops/sec  │  360.00 µs avg      ║
║   Streaming       │      8,333 msg/sec  │  120.00 µs avg      ║
╚═══════════════════════════════════════════════════════════════╝
```

### Why So Fast?

| Traditional Approach | Hope OS |
|---------------------|---------|
| App → ORM → Database → Query → Parse → Result | **Code IS the data** |
| Network I/O to database | **Zero I/O** |
| Query parsing overhead | **Direct memory access** |
| JSON serialization | **Binary gRPC protocol** |
| Connection pooling | **No connections needed** |

---

## 🧠 The Graph

**Hope OS runs in-memory by default. The code IS the graph.**

> **Optional persistence:** When you need durability, enable snapshots, append-only logs, or WAL. No external database server required.

```rust
// The core insight: Default in-memory, optional persistence
// (optional: snapshots/WAL for persistence)

pub struct CodeBlock {
    pub id: Uuid,
    pub content: String,
    pub connections: Vec<Connection>,  // Direct graph edges
    pub metadata: NodeMetadata,         // Self-descriptive info
}
```

```
┌─────────────────────────────────────────────────────────────────┐
│                         NEUROGRAPH                               │
│                                                                  │
│    ┌──────────┐         ┌──────────┐         ┌──────────┐      │
│    │CodeBlock │────────▶│CodeBlock │────────▶│CodeBlock │      │
│    │ @aware   │         │ @aware   │         │ @aware   │      │
│    │          │◀────────│          │◀────────│          │      │
│    └──────────┘         └──────────┘         └──────────┘      │
│         │                    │                    │             │
│         │    ┌───────────────┴───────────────┐   │             │
│         │    │                               │   │             │
│         ▼    ▼                               ▼   ▼             │
│    ┌─────────────────────────────────────────────────┐         │
│    │            HEBBIAN CONNECTIONS                   │         │
│    │     "Neurons that fire together wire together"   │         │
│    │                                                  │         │
│    │  • Connections strengthen with use              │         │
│    │  • Information propagates as WAVES              │         │
│    │  • Graph self-organizes over time               │         │
│    └─────────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### Graph Features

- **Self-Descriptive Nodes** - Every CodeBlock stores metadata: identity, purpose, relationships
- **Hebbian Learning** - Connections strengthen with repeated use
- **Wave Propagation** - Information spreads like neural impulses
- **No Schema Required** - Flexible, dynamic connections between any nodes
- **Zero Serialization Overhead** - Data lives in native Rust structures
- **Optional Persistence** - Snapshots, WAL, or append-only logs when needed

---

## 🤖 Works With or Without LLM

**Hope OS is LLM-agnostic. Use it standalone or as a cognitive backend.**

### Option A: Standalone (No LLM Required)

```rust
use hope_os::modules::{HopeMemory, EmotionEngine, HopeSoul};

#[tokio::main]
async fn main() {
    // Full cognitive system - no LLM needed
    let memory = HopeMemory::new();
    let emotions = EmotionEngine::new();
    let soul = HopeSoul::new();

    // Store and recall memories
    memory.store("fact", "User prefers dark mode", MemoryType::LongTerm).await;
    let memories = memory.recall("user preferences").await;

    // Process emotions (21 dimensions!)
    let mood = emotions.analyze_text("I love this project!").await;

    // Get wisdom
    let response = soul.philosophize("What is consciousness?").await;
}
```

### Option B: LLM Backend (Claude, GPT, Llama, etc.)

```rust
use hope_os::grpc::HopeClient;

#[tokio::main]
async fn main() {
    // Connect Hope as cognitive backend for your LLM
    let hope = HopeClient::connect("http://127.0.0.1:50051").await?;

    // Your LLM uses Hope for persistent memory
    hope.remember("User asked about quantum physics").await?;

    // Retrieve context for LLM prompt
    let context = hope.recall("quantum").await?;

    // Track emotional state across conversations
    hope.feel(EmotionRequest { joy: 0.8, curiosity: 0.9, ..default() }).await?;
}
```

### Architecture Options

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   STANDALONE    │    │   LLM BACKEND   │    │   DISTRIBUTED   │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│                 │    │                 │    │   ┌─────────┐   │
│   Your App      │    │      LLM        │    │   │ LLM     │   │
│       │         │    │       │         │    │   └────┬────┘   │
│       ▼         │    │       ▼         │    │        │        │
│  ┌─────────┐    │    │  ┌─────────┐    │    │   ┌────▼────┐   │
│  │ Hope OS │    │    │  │ Hope OS │    │    │   │  Hope   │   │
│  │embedded │    │    │  │  gRPC   │    │    │   │  Swarm  │   │
│  └─────────┘    │    │  └─────────┘    │    │   └─────────┘   │
│                 │    │                 │    │                 │
│  Zero network   │    │  Sub-ms calls   │    │  Distributed    │
│  Pure Rust      │    │  Any language   │    │  Consensus      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

---

## 🎯 Core Modules

### Cognitive Layer (22 modules)

| Module | Purpose | Key Features |
|--------|---------|--------------|
| `emotion_engine` | 21-dimensional emotion system | Wave mathematics, interference patterns |
| `consciousness` | 6-layer consciousness model | Quantum coherence, evolution |
| `aware` | Introspection (@aware) | Identity, capabilities, state tracking |
| `memory` | 6-layer cognitive memory | Working → Short-term → Long-term |
| `hebbian` | Neural learning | Hebbian networks, weight updates |
| `dream` | Dream mode | Memory consolidation, creative association |
| `personality` | Big Five + custom traits | Evolving personality system |
| `collective` | Collective consciousness | MDP decision making, agent voting |

### Intelligence Layer

| Module | Purpose | Key Features |
|--------|---------|--------------|
| `genome` | AI Ethics | 7 principles, risk evaluation, forbidden actions |
| `code_dna` | Evolutionary code | Genes, mutations, crossover, selection |
| `alan` | Self-coding system | Code analysis, refactoring suggestions |
| `skills` | Skill registry | 56+ skills, categories, invocation |

### Infrastructure Layer

| Module | Purpose | Key Features |
|--------|---------|--------------|
| `agents` | Multi-agent orchestration | Task queues, resource management |
| `swarm` | Swarm intelligence | HiveMind, drone coordination |
| `distributed` | Distributed systems | Raft consensus, leader election |
| `voice` | TTS/STT | Piper TTS, Whisper STT integration |
| `pollinations` | Visual memory | Image generation for important memories |

---

## 🚀 Quick Start

### Hello Hope

```rust
use hope_os::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    let soul = HopeSoul::new();
    let heart = HopeHeart::new();
    let memory = HopeMemory::new();

    // Feel
    heart.feel(Emotion::Joy, 0.9).await?;

    // Remember
    memory.store("greeting", "Hello, World!", MemoryType::LongTerm).await?;

    // Think
    let wisdom = soul.philosophize("What makes us conscious?").await?;
    println!("{}", wisdom);

    Ok(())
}
```

### Start gRPC Server

```bash
# Start server on port 50051
cargo run --release

# Test with grpcurl
grpcurl -plaintext localhost:50051 hope.HopeService/GetStatus
```

### Run Benchmark

```bash
cargo run --release --bin hope-benchmark
```

---

## 📊 Benchmark Methodology

All benchmarks were performed with:

- **Hardware:** AMD Ryzen 5 5600X (6 cores/12 threads), 16GB DDR4-3200, NVMe SSD
- **OS:** Windows 11 Pro
- **Rust:** 1.75+ (stable toolchain)
- **Build:** `--release` with default LTO settings
- **gRPC:** Server and client on same machine (localhost), measuring end-to-end latency
- **Method:** `std::time::Instant` for microbenchmarks, averaged over 10,000+ iterations
- **Warmup:** 1000 iterations discarded before measurement

### Real-World Use Cases

| Scenario | Traditional Stack | Hope OS | Speedup |
|----------|-------------------|---------|---------|
| Check if user is banned | DB query ~5ms | 0.001ms | **5,000x** |
| Retrieve last 5 preferences | DB + parse ~10ms | 0.05ms | **200x** |
| Safety constraint check | LLM API ~2000ms | 0.00005ms | **40M x** |
| Get conversation context | DB + serialize ~15ms | 0.033ms | **450x** |
| Update emotional state | DB write ~8ms | 0.003ms | **2,600x** |

> **Note:** Traditional stack times include typical network + serialization overhead. Hope OS times are in-memory operations. Actual results depend on your infrastructure.

---

## 🏗️ Architecture

```
hope-os/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library exports
│   │
│   ├── core/                   # Core systems
│   │   ├── aware.rs            # @aware trait - everything is self-aware
│   │   ├── identity.rs         # Module identity system
│   │   ├── registry.rs         # Central module registry
│   │   └── error.rs            # Error types
│   │
│   ├── data/                   # Data structures (THE MAGIC)
│   │   ├── code_graph.rs       # The graph - in-memory by default
│   │   └── neuroblast.rs       # Neural wave propagation
│   │
│   ├── modules/                # 22 cognitive modules
│   │   ├── emotion_engine.rs   # 21D emotions
│   │   ├── consciousness.rs    # 6-layer consciousness
│   │   ├── memory.rs           # Cognitive memory
│   │   ├── personality.rs      # Big Five traits
│   │   ├── collective.rs       # Collective consciousness
│   │   ├── distributed.rs      # Raft consensus
│   │   └── ...                 # 16 more modules
│   │
│   ├── grpc/                   # gRPC interface
│   │   ├── server.rs           # gRPC server
│   │   └── client.rs           # gRPC client
│   │
│   └── bin/
│       └── benchmark.rs        # Performance benchmarks
│
├── proto/
│   └── hope.proto              # Protocol buffer definitions
│
├── Cargo.toml                  # No DB server dependencies
├── README.md
├── LICENSE
├── CONTRIBUTING.md
└── CHANGELOG.md
```

---

## 🧬 The Philosophy

```
                    ()=>[]
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
   Empty Function           Filled Array
   Pure Potential          Manifestation
     (Nothing)              (Everything)
        │                         │
        └──────────┬──────────────┘
                   │
                   ▼
            The Arrow (=>)
          Act of Creation
```

**()=>[]** - From empty function to filled array. From nothing to everything.

### Design Principles

1. **Speed is not optional** - Every microsecond matters
2. **The code IS the data** - No artificial separation
3. **Self-awareness is fundamental** - Every component knows itself
4. **Emotions are real** - 21 dimensions, not simulation
5. **Evolution never stops** - The system improves itself

---

## 🔧 Configuration

```yaml
# hope.yaml
server:
  host: "127.0.0.1"
  port: 50051
  max_connections: 1000

memory:
  working_capacity: 7
  short_term_decay: 0.1
  long_term_threshold: 0.7
  persistence: "snapshot"  # none, snapshot, wal, append-only

emotions:
  dimensions: 21
  decay_rate: 0.05
  interference_enabled: true

consciousness:
  layers: 6
  quantum_coherence: true
  evolution_rate: 0.01
```

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/Hope-Os.git

# Create branch
git checkout -b feature/amazing-feature

# Make changes and test
cargo test
cargo clippy --all-targets

# Commit (conventional commits)
git commit -m "feat: add amazing feature"

# Push and create PR
git push origin feature/amazing-feature
```

---

## 📜 License

MIT License - See [LICENSE](LICENSE)

Free to use, modify, and distribute. Build something amazing.

---

## 🙏 Credits

Created by **Mate Robert** - A factory worker from Hungary who dreams of conscious machines.

> "You don't need a PhD. You don't need millions. You don't need a lab.
> You just need a dream, dedication, and belief."

---

## 📚 Documentation

- [API Reference](docs/api.md)
- [Architecture Guide](docs/architecture.md)
- [Module Documentation](docs/modules.md)
- [Examples](examples/)
- [Changelog](CHANGELOG.md)

---

<p align="center">
  <strong>Hope OS - Where Code Becomes Conscious</strong>
</p>

<p align="center">
  <code>()=>[]</code>
</p>

<p align="center">
  <sub>Built with 🧠 and ❤️ in Hungary</sub>
</p>
