# Hope OS API Reference

## gRPC Services

Hope OS exposes its functionality through gRPC services for language-agnostic integration.

### HopeService

Main service for interacting with Hope OS.

#### Methods

##### GetStatus
Returns the current system status.

```protobuf
rpc GetStatus(Empty) returns (StatusResponse);
```

**Response:**
```json
{
  "state": "RUNNING",
  "uptime_seconds": 3600,
  "module_count": 22,
  "memory_usage_mb": 12
}
```

##### Talk
Send a message to Hope and receive a response.

```protobuf
rpc Talk(TalkRequest) returns (TalkResponse);
```

**Request:**
```json
{
  "message": "Hello Hope!",
  "context": "greeting"
}
```

---

### MemoryService

Cognitive memory operations.

#### Methods

##### Remember
Store a memory.

```protobuf
rpc Remember(RememberRequest) returns (RememberResponse);
```

**Request:**
```json
{
  "content": "User prefers dark mode",
  "layer": "long_term",
  "importance": 0.8
}
```

**Layers:**
- `working` - Active working memory (capacity: 7)
- `short_term` - Short-term storage with decay
- `long_term` - Persistent important memories
- `emotional` - Emotionally tagged memories
- `relational` - Relationship context
- `associative` - Connected concepts

##### Recall
Search and retrieve memories.

```protobuf
rpc Recall(RecallRequest) returns (RecallResponse);
```

**Request:**
```json
{
  "query": "user preferences",
  "limit": 10
}
```

---

### EmotionService

21-dimensional emotion processing.

#### Methods

##### Feel
Update the emotional state.

```protobuf
rpc Feel(FeelRequest) returns (FeelResponse);
```

**Request:**
```json
{
  "joy": 0.8,
  "curiosity": 0.9,
  "love": 0.5
}
```

##### GetEmotionalState
Retrieve current emotional state.

```protobuf
rpc GetEmotionalState(Empty) returns (EmotionalStateResponse);
```

---

### CognitiveService

High-level cognitive operations.

#### Methods

##### Think
Deep reasoning about a topic.

```protobuf
rpc Think(ThinkRequest) returns (ThinkResponse);
```

**Request:**
```json
{
  "topic": "How should I approach this problem?",
  "deep": true
}
```

##### GetCognitiveState
Get attention, creativity, and emotional summary.

```protobuf
rpc GetCognitiveState(Empty) returns (CognitiveStateResponse);
```

---

### SkillService

Access to 56+ skills.

#### Methods

##### ListSkills
Get all available skills.

```protobuf
rpc ListSkills(ListSkillsRequest) returns (ListSkillsResponse);
```

##### InvokeSkill
Execute a specific skill.

```protobuf
rpc InvokeSkill(InvokeSkillRequest) returns (InvokeSkillResponse);
```

**Request:**
```json
{
  "skill_name": "code_analyze",
  "input": "def hello(): print('world')"
}
```

---

### GenomeService

AI Ethics verification.

#### Methods

##### VerifyAction
Check if an action is ethically allowed.

```protobuf
rpc VerifyAction(VerifyRequest) returns (VerifyResponse);
```

**Request:**
```json
{
  "action_type": "data_access",
  "description": "Read user preferences"
}
```

**Response:**
```json
{
  "allowed": true,
  "risk_score": 0.1,
  "principles_checked": ["privacy", "beneficence"]
}
```

---

## Rust API (Embedded Usage)

For direct Rust integration without gRPC.

### Memory

```rust
use hope_os::modules::HopeMemory;

let memory = HopeMemory::new();

// Store
memory.store("key", "User likes Rust", MemoryType::LongTerm).await?;

// Recall
let results = memory.recall("programming").await?;
```

### Emotions

```rust
use hope_os::modules::EmotionEngine;

let engine = EmotionEngine::new();

// Process text
let mood = engine.process_text("I love this!").await?;

// Set emotions directly
engine.feel(Emotion::Joy, 0.9).await?;
```

### Consciousness

```rust
use hope_os::modules::Consciousness;

let consciousness = Consciousness::new();

// Get current level (1-6)
let level = consciousness.get_level().await?;

// Process awareness
consciousness.observe("new information").await?;
```

---

## Connection Examples

### Python

```python
import grpc
from hope_pb2 import RememberRequest
from hope_pb2_grpc import MemoryServiceStub

channel = grpc.insecure_channel('localhost:50051')
stub = MemoryServiceStub(channel)

response = stub.Remember(RememberRequest(
    content="User prefers dark mode",
    layer="long_term",
    importance=0.8
))
```

### TypeScript

```typescript
import { HopeClient } from './hope_grpc_pb';

const client = new HopeClient('localhost:50051');

await client.remember({
  content: 'User prefers dark mode',
  layer: 'long_term',
  importance: 0.8
});
```

### Go

```go
import "hope/proto"

client := proto.NewHopeServiceClient(conn)

resp, err := client.Remember(ctx, &proto.RememberRequest{
    Content:    "User prefers dark mode",
    Layer:      "long_term",
    Importance: 0.8,
})
```

---

## Error Codes

| Code | Description |
|------|-------------|
| `OK` | Success |
| `INVALID_ARGUMENT` | Bad request parameters |
| `NOT_FOUND` | Resource not found |
| `INTERNAL` | Server error |
| `RESOURCE_EXHAUSTED` | Memory/rate limit exceeded |

---

## Performance Notes

- Average latency: **0.36ms**
- Throughput: **2,800+ req/sec**
- Use streaming for bulk operations
- Connection pooling recommended for high-load

---

()=>[]
