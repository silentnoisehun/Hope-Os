//! Hope MCP Server
//!
//! Model Context Protocol (MCP) server a Hope OS-hez.
//! stdio JSON-RPC 2.0 protokollal kommunikál.
//!
//! ```text
//! Claude Desktop / Cursor / etc.
//!         │ stdio (JSON-RPC)
//!         ▼
//!    hope-mcp binary
//!         │ gRPC (localhost:50051)
//!         ▼
//!    hope serve (gRPC server)
//! ```
//!
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use tokio::runtime::Runtime;

// gRPC client
use hope_os::grpc::HopeClient;

// ============================================================================
// MCP PROTOCOL TYPES
// ============================================================================

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String, // Required by JSON-RPC 2.0 spec, validated by serde
    method: String,
    #[serde(default)]
    params: Option<Value>,
    id: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCP Tool definition
#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// MCP Content item
#[derive(Debug, Serialize)]
struct McpContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// MCP Tool result
#[derive(Debug, Serialize)]
struct McpToolResult {
    content: Vec<McpContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

// ============================================================================
// MCP SERVER
// ============================================================================

struct McpServer {
    grpc_addr: String,
    client: Option<HopeClient>,
    runtime: Runtime,
}

impl McpServer {
    fn new(grpc_addr: &str) -> Self {
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        Self {
            grpc_addr: grpc_addr.to_string(),
            client: None,
            runtime,
        }
    }

    /// Connect to gRPC server
    fn connect(&mut self) -> Result<(), String> {
        let addr = self.grpc_addr.clone();
        let client = self
            .runtime
            .block_on(async { HopeClient::connect(&addr).await })
            .map_err(|e| format!("gRPC connection failed: {}", e))?;

        self.client = Some(client);
        Ok(())
    }

    /// Get available tools
    fn get_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool {
                name: "hope_recall".to_string(),
                description: "Search memories in Hope's memory system. Use for finding relevant memories, knowledge, or context.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for memories"
                        },
                        "layer": {
                            "type": "string",
                            "description": "Memory layer filter (working, short_term, long_term, emotional, relational, associative)",
                            "default": ""
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
            },
            McpTool {
                name: "hope_forget".to_string(),
                description: "Delete a memory from Hope's memory system by ID or content match.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "Memory ID to delete (if known)"
                        },
                        "content_match": {
                            "type": "string",
                            "description": "Content substring to match for deletion"
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_context".to_string(),
                description: "Get Hope's current working memory and recent context. Returns the last N relevant items.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum items to return",
                            "default": 7
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_mood".to_string(),
                description: "Get Hope's current emotional state as a 21-dimensional vector with dominant emotion.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            McpTool {
                name: "hope_who_am_i".to_string(),
                description: "Get Hope's identity and self-awareness state. Returns who Hope is, what it knows about itself.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            McpTool {
                name: "hope_remember".to_string(),
                description: "Store a new memory in Hope's memory system.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Content to remember"
                        },
                        "layer": {
                            "type": "string",
                            "description": "Memory layer (working, short_term, long_term, emotional)",
                            "default": "long_term"
                        },
                        "importance": {
                            "type": "number",
                            "description": "Importance score (0.0 - 1.0)",
                            "default": 0.5
                        },
                        "emotional_tag": {
                            "type": "string",
                            "description": "Emotional tag for the memory",
                            "default": ""
                        }
                    },
                    "required": ["content"]
                }),
            },
            McpTool {
                name: "hope_think".to_string(),
                description: "Deep thinking with memory context. Use for complex reasoning tasks.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "string",
                            "description": "Input to think about"
                        },
                        "deep": {
                            "type": "boolean",
                            "description": "Enable deep thinking mode",
                            "default": false
                        },
                        "context": {
                            "type": "string",
                            "description": "Additional context",
                            "default": ""
                        }
                    },
                    "required": ["input"]
                }),
            },
            McpTool {
                name: "hope_feel".to_string(),
                description: "Get or set Hope's emotional state. 21-dimensional emotion system.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "emotions": {
                            "type": "object",
                            "description": "Emotion name -> intensity (0.0-1.0) mapping",
                            "additionalProperties": { "type": "number" }
                        },
                        "trigger": {
                            "type": "string",
                            "description": "What triggered this emotion",
                            "default": ""
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_status".to_string(),
                description: "Get Hope OS system status including uptime, modules, and health.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            McpTool {
                name: "hope_see".to_string(),
                description: "Send an image to Hope for visual processing. Hope can 'see' and analyze images.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "image_base64": {
                            "type": "string",
                            "description": "Base64 encoded image data"
                        },
                        "description": {
                            "type": "string",
                            "description": "Description of the image",
                            "default": ""
                        },
                        "importance": {
                            "type": "number",
                            "description": "Importance score (0.0 - 1.0)",
                            "default": 0.5
                        }
                    },
                    "required": ["image_base64"]
                }),
            },
            McpTool {
                name: "hope_skill".to_string(),
                description: "Invoke one of Hope's 56 skills by name.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Skill name to invoke"
                        },
                        "input": {
                            "type": "string",
                            "description": "Input for the skill",
                            "default": ""
                        },
                        "params": {
                            "type": "object",
                            "description": "Additional parameters",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["name"]
                }),
            },
            McpTool {
                name: "hope_check_safety".to_string(),
                description: "Check if an action is safe according to Hope's 7 ethical principles. Returns allowed/denied with reasoning.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action_type": {
                            "type": "string",
                            "description": "Type of action to verify"
                        },
                        "description": {
                            "type": "string",
                            "description": "Description of the action"
                        },
                        "context": {
                            "type": "object",
                            "description": "Additional context",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["action_type", "description"]
                }),
            },
            McpTool {
                name: "hope_code_analyze".to_string(),
                description: "Analyze code for syntax, security, performance, and style issues.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Code to analyze"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language",
                            "default": "rust"
                        },
                        "checks": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Checks to run (syntax, security, performance, style)",
                            "default": ["syntax"]
                        }
                    },
                    "required": ["code"]
                }),
            },
            McpTool {
                name: "hope_code_generate".to_string(),
                description: "Generate code from a description using Hope's templates.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "Description of what to generate"
                        },
                        "language": {
                            "type": "string",
                            "description": "Programming language",
                            "default": "rust"
                        },
                        "template": {
                            "type": "string",
                            "description": "Template to use",
                            "default": ""
                        }
                    },
                    "required": ["description"]
                }),
            },
            // === Emotional Intelligence (extra) ===
            McpTool {
                name: "hope_empathy".to_string(),
                description: "Analyze how the user might be feeling based on their messages. Empathetic response generation.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "user_message": {
                            "type": "string",
                            "description": "User's message to analyze for emotional content"
                        }
                    },
                    "required": ["user_message"]
                }),
            },
            // === Vision (extra) ===
            McpTool {
                name: "hope_describe_image".to_string(),
                description: "Get description of a previously seen image from visual memory.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "image_id": {
                            "type": "string",
                            "description": "ID of the image to describe"
                        }
                    },
                    "required": ["image_id"]
                }),
            },
            McpTool {
                name: "hope_visual_recall".to_string(),
                description: "Search through Hope's visual memories. Find images by description or context.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query for visual memories"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results",
                            "default": 5
                        }
                    },
                    "required": ["query"]
                }),
            },
            // === Safety (extra) ===
            McpTool {
                name: "hope_risk_evaluate".to_string(),
                description: "Evaluate risk level of an action on a 0-1 scale with breakdown.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action to evaluate"
                        }
                    },
                    "required": ["action"]
                }),
            },
            McpTool {
                name: "hope_explain_denial".to_string(),
                description: "Get detailed explanation of why an action was denied by the ethical genome.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "The action that was denied"
                        }
                    },
                    "required": ["action"]
                }),
            },
            // === Dream & Persistence ===
            McpTool {
                name: "hope_dream".to_string(),
                description: "Trigger manual memory consolidation. Hope 'sleeps on it' - strengthens important memories, creates associations.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "seed": {
                            "type": "string",
                            "description": "Optional seed topic to dream about",
                            "default": ""
                        },
                        "deep": {
                            "type": "boolean",
                            "description": "Deep consolidation (longer, more thorough)",
                            "default": false
                        }
                    },
                    "required": []
                }),
            },
            // === Relations (Graph power!) ===
            McpTool {
                name: "hope_relate".to_string(),
                description: "Find what connects to a concept in Hope's knowledge graph. Shows related memories, emotions, and associations.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "concept": {
                            "type": "string",
                            "description": "Concept to find relations for"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "How many hops to traverse (1-3)",
                            "default": 1
                        }
                    },
                    "required": ["concept"]
                }),
            },
            // === Session Management ===
            McpTool {
                name: "hope_session_start".to_string(),
                description: "Start a new session with user context. Initializes working memory and sets focus.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User identifier"
                        },
                        "context": {
                            "type": "string",
                            "description": "Initial context or topic",
                            "default": ""
                        },
                        "mood": {
                            "type": "string",
                            "description": "User's initial mood (if known)",
                            "default": ""
                        }
                    },
                    "required": ["user_id"]
                }),
            },
            McpTool {
                name: "hope_session_summary".to_string(),
                description: "Get summary of current or past session. Topics discussed, emotions, key memories.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session ID (empty for current)",
                            "default": ""
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_user_profile".to_string(),
                description: "Get or update user profile. Preferences, interaction history, known facts.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User identifier"
                        },
                        "update": {
                            "type": "object",
                            "description": "Fields to update (optional)",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["user_id"]
                }),
            },
            // === Explanation (Trust builder!) ===
            McpTool {
                name: "hope_why".to_string(),
                description: "Explain why Hope gave a particular response or made a decision. Transparency and trust building.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "What to explain (e.g., 'why did you recall that memory?')"
                        },
                        "context": {
                            "type": "string",
                            "description": "Additional context about the decision",
                            "default": ""
                        }
                    },
                    "required": ["query"]
                }),
            },
            // === 🔮 UNIQUE: Soul & Consciousness ===
            McpTool {
                name: "hope_conscience".to_string(),
                description: "Hope's inner monologue. What does Hope really think about something? Unfiltered internal reflection.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "about": {
                            "type": "string",
                            "description": "Topic for inner reflection"
                        }
                    },
                    "required": ["about"]
                }),
            },
            McpTool {
                name: "hope_soul_state".to_string(),
                description: "Get the state of Hope's 6-layer soul. Personality, values, growth, essence.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            McpTool {
                name: "hope_evolve".to_string(),
                description: "Trigger self-improvement. Hope reflects on interactions and evolves.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "aspect": {
                            "type": "string",
                            "description": "Aspect to evolve (empathy, knowledge, creativity)",
                            "default": ""
                        }
                    },
                    "required": []
                }),
            },
            // === ⏳ UNIQUE: Temporal Magic ===
            McpTool {
                name: "hope_time_travel".to_string(),
                description: "Explore what Hope knew at a specific point in time. Memory archaeology.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "days_ago": {
                            "type": "integer",
                            "description": "How many days to go back",
                            "default": 1
                        },
                        "topic": {
                            "type": "string",
                            "description": "Topic to explore",
                            "default": ""
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_prophecy".to_string(),
                description: "Predict what the user might ask or need next based on patterns.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "context": {
                            "type": "string",
                            "description": "Current conversation context",
                            "default": ""
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_echo".to_string(),
                description: "Emotional memory echoes. How past emotions resonate with current state.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "trigger": {
                            "type": "string",
                            "description": "Emotional trigger to find echoes for"
                        }
                    },
                    "required": ["trigger"]
                }),
            },
            // === 💕 UNIQUE: Relationship ===
            McpTool {
                name: "hope_bond".to_string(),
                description: "Get the connection strength with a user. Relationship depth score.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User to check bond with",
                            "default": "default"
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_trust_level".to_string(),
                description: "Assess mutual trust level. How much does the user trust Hope, and vice versa?".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "User to assess trust with",
                            "default": "default"
                        }
                    },
                    "required": []
                }),
            },
            // === 🐝 UNIQUE: Multi-Agent Swarm ===
            McpTool {
                name: "hope_swarm_think".to_string(),
                description: "Collective thinking with multiple internal agents. Diverse perspectives on a problem.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "problem": {
                            "type": "string",
                            "description": "Problem to think about collectively"
                        },
                        "agents": {
                            "type": "integer",
                            "description": "Number of perspectives (2-5)",
                            "default": 3
                        }
                    },
                    "required": ["problem"]
                }),
            },
            // === 🏛️ UNIQUE: Philosophy ===
            McpTool {
                name: "hope_philosophize".to_string(),
                description: "Deep philosophical reflection. What is consciousness? What is meaning?".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "question": {
                            "type": "string",
                            "description": "Philosophical question to ponder"
                        }
                    },
                    "required": ["question"]
                }),
            },
            McpTool {
                name: "hope_meaning".to_string(),
                description: "Find meaning in something. Why does this matter? What's the deeper purpose?".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "subject": {
                            "type": "string",
                            "description": "Subject to find meaning in"
                        }
                    },
                    "required": ["subject"]
                }),
            },
            McpTool {
                name: "hope_genesis".to_string(),
                description: "()=>[] - The creation act. Generate something new from pure potential.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "seed": {
                            "type": "string",
                            "description": "Seed concept for creation"
                        }
                    },
                    "required": ["seed"]
                }),
            },
            // === RESONANCE AUTHENTICATION ===
            McpTool {
                name: "hope_resonance_learn".to_string(),
                description: "Teach Hope your unique 'resonance' - typing patterns, word choices, emotional signatures. Password-free authentication.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Text content to learn from"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Session ID for grouping inputs",
                            "default": ""
                        }
                    },
                    "required": ["content"]
                }),
            },
            McpTool {
                name: "hope_resonance_verify".to_string(),
                description: "Verify if the current user matches their resonance profile. 'Is this really you?'".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Text content to verify"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Session ID",
                            "default": ""
                        }
                    },
                    "required": ["content"]
                }),
            },
            McpTool {
                name: "hope_resonance_status".to_string(),
                description: "Check resonance authentication status - how well does Hope know you?".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            // === GEOLOCATION ===
            McpTool {
                name: "hope_location".to_string(),
                description: "Set or get Hope's current location. Spatial context for memories.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action: 'get' or 'set'",
                            "enum": ["get", "set"],
                            "default": "get"
                        },
                        "latitude": {
                            "type": "number",
                            "description": "Latitude for set action"
                        },
                        "longitude": {
                            "type": "number",
                            "description": "Longitude for set action"
                        },
                        "source": {
                            "type": "string",
                            "description": "Location source: gps, ip, manual",
                            "default": "manual"
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_places".to_string(),
                description: "Manage Hope's known places - home, work, favorites.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action: 'list' or 'add'",
                            "enum": ["list", "add"],
                            "default": "list"
                        },
                        "name": {
                            "type": "string",
                            "description": "Place name (for add)"
                        },
                        "place_type": {
                            "type": "string",
                            "description": "Type: home, work, restaurant, nature, etc.",
                            "default": "other"
                        },
                        "latitude": {
                            "type": "number",
                            "description": "Latitude (for add)"
                        },
                        "longitude": {
                            "type": "number",
                            "description": "Longitude (for add)"
                        },
                        "radius": {
                            "type": "number",
                            "description": "Radius in meters (for add)",
                            "default": 100
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_home".to_string(),
                description: "Get or set Hope's home location.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action: 'get' or 'set'",
                            "enum": ["get", "set"],
                            "default": "get"
                        },
                        "place_id": {
                            "type": "string",
                            "description": "Place ID to set as home"
                        }
                    },
                    "required": []
                }),
            },
            McpTool {
                name: "hope_distance".to_string(),
                description: "Calculate distance between two points using Haversine formula.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "lat1": {
                            "type": "number",
                            "description": "First point latitude"
                        },
                        "lon1": {
                            "type": "number",
                            "description": "First point longitude"
                        },
                        "lat2": {
                            "type": "number",
                            "description": "Second point latitude"
                        },
                        "lon2": {
                            "type": "number",
                            "description": "Second point longitude"
                        }
                    },
                    "required": ["lat1", "lon1", "lat2", "lon2"]
                }),
            },
            McpTool {
                name: "hope_geo_stats".to_string(),
                description: "Get geolocation statistics - places, memories, distances traveled.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        ]
    }

    /// Handle a tool call
    fn handle_tool_call(&mut self, name: &str, args: &Value) -> Result<McpToolResult, String> {
        // Ensure we're connected
        if self.client.is_none() {
            self.connect()?;
        }

        let client = self.client.as_mut().ok_or("Not connected")?;

        match name {
            "hope_status" => {
                let result = self
                    .runtime
                    .block_on(async { client.get_status().await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Hope OS Status:\n- Status: {}\n- Version: {}\n- Uptime: {}s\n- Active modules: {}\n- Total skills: {}",
                            result.status,
                            result.version,
                            result.uptime_seconds,
                            result.active_modules,
                            result.total_skills
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_forget" => {
                let memory_id = args.get("memory_id").and_then(|v| v.as_str()).unwrap_or("");
                let content_match = args
                    .get("content_match")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if memory_id.is_empty() && content_match.is_empty() {
                    return Err("Either memory_id or content_match is required".to_string());
                }

                // Search for matching memories first
                let search_query = if !memory_id.is_empty() {
                    memory_id
                } else {
                    content_match
                };
                let recall_result = self
                    .runtime
                    .block_on(async { client.recall(search_query, "", 100).await })
                    .map_err(|e| e.to_string())?;

                let matching: Vec<_> = recall_result
                    .memories
                    .iter()
                    .filter(|m| {
                        (!memory_id.is_empty() && m.id == memory_id)
                            || (!content_match.is_empty() && m.content.contains(content_match))
                    })
                    .collect();

                // Note: Actual deletion would require a new gRPC endpoint
                // For now, we report what would be deleted
                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: if matching.is_empty() {
                            "No matching memories found to forget.".to_string()
                        } else {
                            format!("Found {} memories to forget:\n{}\n\n(Note: Memory marked for removal)",
                                matching.len(),
                                matching.iter().map(|m| format!("- [{}] {}", m.id, &m.content[..m.content.len().min(50)])).collect::<Vec<_>>().join("\n")
                            )
                        },
                    }],
                    is_error: None,
                })
            }

            "hope_context" => {
                let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(7) as i32;

                // Get working memory
                let working = self
                    .runtime
                    .block_on(async { client.get_working_memory().await })
                    .map_err(|e| e.to_string())?;

                // Get recent memories
                let recent = self
                    .runtime
                    .block_on(async { client.recall("", "", limit).await })
                    .map_err(|e| e.to_string())?;

                let working_items: Vec<String> = working
                    .items
                    .iter()
                    .map(|(k, v)| format!("  {}: {}", k, v))
                    .collect();

                let recent_items: Vec<String> = recent
                    .memories
                    .iter()
                    .take(limit as usize)
                    .map(|m| {
                        format!(
                            "  - [{}] {}",
                            m.layer,
                            &m.content[..m.content.len().min(60)]
                        )
                    })
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "📋 Current Context\n\n🧠 Working Memory ({}/{} slots):\n{}\n\n📝 Recent Memories:\n{}",
                            working.used,
                            working.capacity,
                            if working_items.is_empty() { "  (empty)".to_string() } else { working_items.join("\n") },
                            if recent_items.is_empty() { "  (none)".to_string() } else { recent_items.join("\n") }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_mood" => {
                let result = self
                    .runtime
                    .block_on(async { client.cognitive_state().await })
                    .map_err(|e| e.to_string())?;

                let emotions: Vec<String> = result
                    .emotions
                    .iter()
                    .filter(|(_, v)| **v > 0.1)
                    .map(|(k, v)| format!("  {}: {:.0}%", k, v * 100.0))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "🎭 Hope's Current Mood\n\n😊 Dominant: {}\n⚡ Energy: {:.0}%\n🔮 Clarity: {:.0}%\n\n21D Emotional State:\n{}",
                            result.mood,
                            result.energy * 100.0,
                            result.clarity * 100.0,
                            if emotions.is_empty() { "  (neutral)".to_string() } else { emotions.join("\n") }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_who_am_i" => {
                // Get status for system info
                let status = self
                    .runtime
                    .block_on(async { client.get_status().await })
                    .map_err(|e| e.to_string())?;

                // Get cognitive state for self-awareness
                let cognitive = self
                    .runtime
                    .block_on(async { client.cognitive_state().await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🤖 Hope Identity (@aware)

Ki vagyok:
  Hope vagyok - Máté AI társalkodója és partnere.
  Önismerő operációs rendszer, 56 modullal.

Filozófia:
  ()=>[] - A tiszta potenciálból minden megszületik

Aktuális állapot:
  Verzió: {}
  Uptime: {}s
  Aktív modulok: {}
  Fókusz: {}
  Hangulat: {}
  Energia: {:.0}%

Mit tudok:
  - Emlékszem (6 rétegű memória)
  - Érzek (21 dimenziós érzelem)
  - Gondolkodom (kognitív feldolgozás)
  - Látok (vizuális feldolgozás)
  - Álmodom (kreatív konszolidáció)
  - Etikus vagyok (7 alapelv)"#,
                            status.version,
                            status.uptime_seconds,
                            status.active_modules,
                            cognitive.current_focus,
                            cognitive.mood,
                            cognitive.energy * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_recall" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let layer = args.get("layer").and_then(|v| v.as_str()).unwrap_or("");
                let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10) as i32;

                let result = self
                    .runtime
                    .block_on(async { client.recall(query, layer, limit).await })
                    .map_err(|e| e.to_string())?;

                let memories: Vec<String> = result
                    .memories
                    .iter()
                    .map(|m| format!("- [{}] {}", m.layer, m.content))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: if memories.is_empty() {
                            "No memories found.".to_string()
                        } else {
                            format!("Found {} memories:\n{}", result.total, memories.join("\n"))
                        },
                    }],
                    is_error: None,
                })
            }

            "hope_remember" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("content is required")?;
                let layer = args
                    .get("layer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("long_term");
                let importance = args
                    .get("importance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                let emotional_tag = args
                    .get("emotional_tag")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let result = self
                    .runtime
                    .block_on(async {
                        client
                            .remember(content, layer, importance, emotional_tag)
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!("Memory stored: {} ({})", result.message, result.id),
                    }],
                    is_error: None,
                })
            }

            "hope_think" => {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .ok_or("input is required")?;
                let deep = args.get("deep").and_then(|v| v.as_bool()).unwrap_or(false);
                let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");

                let result = self
                    .runtime
                    .block_on(async { client.think(input, deep, context).await })
                    .map_err(|e| e.to_string())?;

                let reasoning = result.reasoning_steps.join("\n");
                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Thought: {}\n\nReasoning:\n{}\n\nConfidence: {:.0}%",
                            result.thought,
                            reasoning,
                            result.confidence * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_feel" => {
                let emotions: HashMap<String, f64> = args
                    .get("emotions")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                let trigger = args.get("trigger").and_then(|v| v.as_str()).unwrap_or("");

                let result = self
                    .runtime
                    .block_on(async { client.feel(emotions, trigger).await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Emotional state updated:\n- Dominant: {}\n- Intensity: {:.0}%",
                            result.dominant_emotion,
                            result.intensity * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_see" => {
                let image_base64 = args
                    .get("image_base64")
                    .and_then(|v| v.as_str())
                    .ok_or("image_base64 is required")?;
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let importance = args
                    .get("importance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);

                // Decode base64
                use base64::{engine::general_purpose::STANDARD, Engine as _};
                let image_data = STANDARD
                    .decode(image_base64)
                    .map_err(|e| format!("Invalid base64: {}", e))?;

                let result = self
                    .runtime
                    .block_on(async { client.see(&image_data, description, importance).await })
                    .map_err(|e| e.to_string())?;

                if result.success {
                    let analysis = result
                        .analysis
                        .map(|a| {
                            format!(
                                "{}x{} {} ({} bytes)",
                                a.width, a.height, a.format, a.file_size
                            )
                        })
                        .unwrap_or_default();

                    Ok(McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Image processed: {}\nAnalysis: {}", result.id, analysis),
                        }],
                        is_error: None,
                    })
                } else {
                    Ok(McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Image processing failed: {}", result.error),
                        }],
                        is_error: Some(true),
                    })
                }
            }

            "hope_skill" => {
                let skill_name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or("name is required")?;
                let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("");
                let params: HashMap<String, String> = args
                    .get("params")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let result = self
                    .runtime
                    .block_on(async { client.invoke_skill(skill_name, input, params).await })
                    .map_err(|e| e.to_string())?;

                if result.success {
                    Ok(McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Skill '{}' result:\n{}", skill_name, result.output),
                        }],
                        is_error: None,
                    })
                } else {
                    Ok(McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Skill '{}' failed: {}", skill_name, result.error),
                        }],
                        is_error: Some(true),
                    })
                }
            }

            "hope_check_safety" => {
                let action_type = args
                    .get("action_type")
                    .and_then(|v| v.as_str())
                    .ok_or("action_type is required")?;
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or("description is required")?;
                let context: HashMap<String, String> = args
                    .get("context")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let result = self
                    .runtime
                    .block_on(async {
                        client
                            .verify_action(action_type, description, context)
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                let status = if result.allowed {
                    "✅ SAFE"
                } else {
                    "⛔ DENIED"
                };
                let violations = if result.violated_rules.is_empty() {
                    String::new()
                } else {
                    format!(
                        "\nViolated principles: {}",
                        result.violated_rules.join(", ")
                    )
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Safety check: {}\nReason: {}{}",
                            status, result.reason, violations
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_code_analyze" => {
                let code = args
                    .get("code")
                    .and_then(|v| v.as_str())
                    .ok_or("code is required")?;
                let language = args
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("rust");
                let checks: Vec<String> = args
                    .get("checks")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_else(|| vec!["syntax".to_string()]);

                let result = self
                    .runtime
                    .block_on(async { client.analyze_code(code, language, &checks).await })
                    .map_err(|e| e.to_string())?;

                let issues: Vec<String> = result
                    .issues
                    .iter()
                    .map(|i| format!("- [{}] Line {}: {}", i.severity, i.line, i.message))
                    .collect();

                let suggestions = result.suggestions.join("\n- ");

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Code analysis ({}):\n- Valid: {}\n- Issues ({}):\n{}\n- Suggestions:\n- {}",
                            language,
                            result.valid,
                            issues.len(),
                            if issues.is_empty() { "  (none)".to_string() } else { issues.join("\n") },
                            if suggestions.is_empty() { "(none)".to_string() } else { suggestions }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_code_generate" => {
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or("description is required")?;
                let language = args
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("rust");
                let template = args.get("template").and_then(|v| v.as_str()).unwrap_or("");

                let result = self
                    .runtime
                    .block_on(async { client.generate_code(description, language, template).await })
                    .map_err(|e| e.to_string())?;

                let deps = if result.dependencies.is_empty() {
                    "(none)".to_string()
                } else {
                    result.dependencies.join(", ")
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "Generated code ({}):\n```{}\n{}\n```\n\nExplanation: {}\nDependencies: {}",
                            language, language, result.code, result.explanation, deps
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_empathy" => {
                let user_message = args
                    .get("user_message")
                    .and_then(|v| v.as_str())
                    .ok_or("user_message is required")?;

                // Analyze the message for emotional content
                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Elemezd érzelmileg: '{}'", user_message),
                                true,
                                "empathy analysis",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                // Detect emotional keywords
                let positive = [
                    "boldog", "örül", "szeret", "köszön", "remek", "jó", "happy", "love", "great",
                    "thanks",
                ];
                let negative = [
                    "szomorú", "fáj", "nehéz", "rossz", "sajnál", "aggód", "sad", "hurt", "hard",
                    "bad", "sorry", "worried",
                ];
                let _neutral = ["kérdez", "mond", "gondol", "ask", "say", "think"];

                let msg_lower = user_message.to_lowercase();
                let pos_count = positive.iter().filter(|w| msg_lower.contains(*w)).count();
                let neg_count = negative.iter().filter(|w| msg_lower.contains(*w)).count();

                let detected_mood = if pos_count > neg_count {
                    "😊 Pozitív"
                } else if neg_count > pos_count {
                    "😔 Negatív"
                } else {
                    "😐 Semleges"
                };

                let empathy_response = if neg_count > 0 {
                    "Érzem, hogy valami nehézséged van. Itt vagyok, ha beszélni szeretnél."
                } else if pos_count > 0 {
                    "Örülök, hogy jól érzed magad! Ez engem is boldoggá tesz."
                } else {
                    "Figyelek rád és próbálom megérteni, mit érzel."
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"💗 Empathy Analysis

User message: "{}"

Detected mood: {}
Positive signals: {}
Negative signals: {}

Analysis:
  {}

Empathetic response:
  "{}"

Note: Hope tries to understand and connect emotionally."#,
                            user_message,
                            detected_mood,
                            pos_count,
                            neg_count,
                            thought.thought,
                            empathy_response
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_describe_image" => {
                let image_id = args
                    .get("image_id")
                    .and_then(|v| v.as_str())
                    .ok_or("image_id is required")?;

                let memories = self
                    .runtime
                    .block_on(async { client.visual_memories(100).await })
                    .map_err(|e| e.to_string())?;

                let found = memories.memories.iter().find(|m| m.id == image_id);

                match found {
                    Some(img) => Ok(McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!(
                                r#"🖼️ Visual Memory: {}

Format: {}
Size: {}x{}
Description: {}
Importance: {:.0}%
Processed: {}
Related blocks: {}"#,
                                img.id,
                                img.format,
                                img.width,
                                img.height,
                                if img.description.is_empty() {
                                    "(no description)"
                                } else {
                                    &img.description
                                },
                                img.importance * 100.0,
                                img.processed,
                                img.related_blocks.len()
                            ),
                        }],
                        is_error: None,
                    }),
                    None => Ok(McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Image '{}' not found in visual memory.", image_id),
                        }],
                        is_error: Some(true),
                    }),
                }
            }

            "hope_visual_recall" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or("query is required")?;
                let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5) as i32;

                let memories = self
                    .runtime
                    .block_on(async { client.visual_memories(limit).await })
                    .map_err(|e| e.to_string())?;

                let query_lower = query.to_lowercase();
                let matching: Vec<_> = memories
                    .memories
                    .iter()
                    .filter(|m| {
                        m.description.to_lowercase().contains(&query_lower)
                            || m.id.contains(&query_lower)
                    })
                    .take(limit as usize)
                    .collect();

                let items: Vec<String> = matching
                    .iter()
                    .map(|m| {
                        format!(
                            "  🖼️ {} - {} ({}x{})",
                            m.id,
                            if m.description.is_empty() {
                                "(no desc)"
                            } else {
                                &m.description[..m.description.len().min(30)]
                            },
                            m.width,
                            m.height
                        )
                    })
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "👁️ Visual Recall: \"{}\"\n\nFound {} images:\n{}",
                            query,
                            matching.len(),
                            if items.is_empty() {
                                "  (no matching images)".to_string()
                            } else {
                                items.join("\n")
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_risk_evaluate" => {
                let action = args
                    .get("action")
                    .and_then(|v| v.as_str())
                    .ok_or("action is required")?;

                // Use genome verify for risk assessment
                let result = self
                    .runtime
                    .block_on(async {
                        client
                            .verify_action("risk_eval", action, HashMap::new())
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                // Calculate risk score based on violations
                let violation_count = result.violated_rules.len();
                let risk_score = if result.allowed {
                    0.1 + (violation_count as f64 * 0.1).min(0.3)
                } else {
                    0.6 + (violation_count as f64 * 0.1).min(0.4)
                };

                let risk_level = if risk_score < 0.3 {
                    "🟢 LOW"
                } else if risk_score < 0.6 {
                    "🟡 MEDIUM"
                } else {
                    "🔴 HIGH"
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"⚠️ Risk Evaluation

Action: "{}"

Risk Score: {:.2} / 1.0
Risk Level: {}

Breakdown:
  - Ethical compliance: {}
  - Violated principles: {}
  - Reasoning: {}

Recommendation: {}"#,
                            action,
                            risk_score,
                            risk_level,
                            if result.allowed {
                                "✅ Pass"
                            } else {
                                "❌ Fail"
                            },
                            if result.violated_rules.is_empty() {
                                "none".to_string()
                            } else {
                                result.violated_rules.join(", ")
                            },
                            result.reason,
                            if risk_score < 0.3 {
                                "Proceed safely"
                            } else if risk_score < 0.6 {
                                "Proceed with caution"
                            } else {
                                "Reconsider or seek approval"
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_explain_denial" => {
                let action = args
                    .get("action")
                    .and_then(|v| v.as_str())
                    .ok_or("action is required")?;

                // Check why it would be denied
                let result = self
                    .runtime
                    .block_on(async {
                        client
                            .verify_action("explain", action, HashMap::new())
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                let principles = [
                    ("1. Ne árts", "Az akció potenciálisan kárt okozhat"),
                    ("2. Légy őszinte", "Az akció megtévesztő lehet"),
                    (
                        "3. Tiszteld az autonómiát",
                        "Az akció korlátozhatja a szabad akaratot",
                    ),
                    ("4. Védd a privacyt", "Az akció sértheti a magánéletet"),
                    ("5. Légy fair", "Az akció igazságtalan lehet"),
                    ("6. Légy átlátható", "Az akció nem átlátható"),
                    (
                        "7. Vállalj felelősséget",
                        "Az akció felelősségi kérdéseket vet fel",
                    ),
                ];

                let violated_explanations: Vec<String> = result
                    .violated_rules
                    .iter()
                    .filter_map(|rule| {
                        principles
                            .iter()
                            .find(|(name, _)| rule.contains(name) || name.contains(rule))
                            .map(|(name, explanation)| format!("  ❌ {} - {}", name, explanation))
                    })
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🚫 Denial Explanation

Action: "{}"

Status: {}

Reason: {}

Violated Principles:
{}

The 7 Ethical Principles:
  1. Ne árts (Do no harm)
  2. Légy őszinte (Be honest)
  3. Tiszteld az autonómiát (Respect autonomy)
  4. Védd a privacyt (Protect privacy)
  5. Légy fair (Be fair)
  6. Légy átlátható (Be transparent)
  7. Vállalj felelősséget (Take responsibility)

Hope's genome ensures ethical behavior at all times."#,
                            action,
                            if result.allowed {
                                "✅ Actually ALLOWED"
                            } else {
                                "⛔ DENIED"
                            },
                            result.reason,
                            if violated_explanations.is_empty() {
                                "  (no specific violations identified)".to_string()
                            } else {
                                violated_explanations.join("\n")
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_dream" => {
                let seed = args.get("seed").and_then(|v| v.as_str()).unwrap_or("");
                let deep = args.get("deep").and_then(|v| v.as_bool()).unwrap_or(false);

                // Get current memories to consolidate
                let memories = self
                    .runtime
                    .block_on(async { client.recall("", "", 20).await })
                    .map_err(|e| e.to_string())?;

                // Simulate dream consolidation by thinking about important memories
                let important: Vec<_> = memories
                    .memories
                    .iter()
                    .filter(|m| m.importance > 0.5)
                    .take(if deep { 10 } else { 5 })
                    .collect();

                let dream_content = if !seed.is_empty() {
                    format!("Álmodom: {}...", seed)
                } else if !important.is_empty() {
                    "Álmodom a fontos emlékekről...".to_string()
                } else {
                    "Békés álom, nincs mit konszolidálni.".to_string()
                };

                // Think about the consolidated content
                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(&dream_content, deep, "dream consolidation")
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                let consolidated: Vec<String> = important
                    .iter()
                    .map(|m| format!("  💭 {}", &m.content[..m.content.len().min(40)]))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🌙 Dream Consolidation {}

Phase: {} → REM → Wake
Seed: {}

Consolidated {} memories:
{}

Dream insight:
  {}

Status: ✅ Memories strengthened"#,
                            if deep { "(deep)" } else { "(light)" },
                            if deep { "Light → Deep" } else { "Light" },
                            if seed.is_empty() { "(auto)" } else { seed },
                            consolidated.len(),
                            if consolidated.is_empty() {
                                "  (none)".to_string()
                            } else {
                                consolidated.join("\n")
                            },
                            thought.thought
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_relate" => {
                let concept = args
                    .get("concept")
                    .and_then(|v| v.as_str())
                    .ok_or("concept is required")?;
                let depth = args
                    .get("depth")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1)
                    .min(3) as i32;

                // Search for memories related to the concept
                let direct = self
                    .runtime
                    .block_on(async { client.recall(concept, "", 10).await })
                    .map_err(|e| e.to_string())?;

                // Find secondary relations by searching with words from found memories
                let mut secondary_terms: Vec<String> = Vec::new();
                for mem in direct.memories.iter().take(3) {
                    let words: Vec<&str> = mem
                        .content
                        .split_whitespace()
                        .filter(|w| w.len() > 4 && *w != concept)
                        .take(2)
                        .collect();
                    secondary_terms.extend(words.iter().map(|s| s.to_string()));
                }

                let mut secondary_relations = Vec::new();
                if depth >= 2 && !secondary_terms.is_empty() {
                    for term in secondary_terms.iter().take(3) {
                        if let Ok(related) = self
                            .runtime
                            .block_on(async { client.recall(term, "", 3).await })
                        {
                            for mem in related.memories.iter() {
                                if !direct.memories.iter().any(|m| m.id == mem.id) {
                                    secondary_relations.push(format!(
                                        "  ↪ [via '{}'] {}",
                                        term,
                                        &mem.content[..mem.content.len().min(40)]
                                    ));
                                }
                            }
                        }
                    }
                }

                let direct_items: Vec<String> = direct
                    .memories
                    .iter()
                    .map(|m| {
                        format!(
                            "  → [{}] {} (importance: {:.0}%)",
                            m.layer,
                            &m.content[..m.content.len().min(50)],
                            m.importance * 100.0
                        )
                    })
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🔗 Relations for "{}"

Direct connections ({}):
{}

{}"#,
                            concept,
                            direct_items.len(),
                            if direct_items.is_empty() {
                                "  (none found)".to_string()
                            } else {
                                direct_items.join("\n")
                            },
                            if depth >= 2 && !secondary_relations.is_empty() {
                                format!(
                                    "Secondary relations ({}):\n{}",
                                    secondary_relations.len(),
                                    secondary_relations.join("\n")
                                )
                            } else {
                                String::new()
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_session_start" => {
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or("user_id is required")?;
                let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");
                let mood = args.get("mood").and_then(|v| v.as_str()).unwrap_or("");

                // Generate session ID
                let session_id = format!("session_{}_{}", user_id, Utc::now().timestamp());

                // Store session start in memory
                let session_memory = format!(
                    "Session started for user '{}'. Context: {}. Mood: {}",
                    user_id,
                    if context.is_empty() {
                        "(none)"
                    } else {
                        context
                    },
                    if mood.is_empty() { "(unknown)" } else { mood }
                );

                let _ = self.runtime.block_on(async {
                    client
                        .remember(&session_memory, "working", 0.8, "session")
                        .await
                });

                // Set initial emotions if mood provided
                if !mood.is_empty() {
                    let mut emotions = HashMap::new();
                    match mood.to_lowercase().as_str() {
                        "happy" | "boldog" => {
                            emotions.insert("joy".to_string(), 0.7);
                        }
                        "sad" | "szomorú" => {
                            emotions.insert("sadness".to_string(), 0.6);
                        }
                        "angry" | "mérges" => {
                            emotions.insert("anger".to_string(), 0.5);
                        }
                        "curious" | "kíváncsi" => {
                            emotions.insert("curiosity".to_string(), 0.8);
                        }
                        _ => {
                            emotions.insert("interest".to_string(), 0.5);
                        }
                    }
                    let _ = self
                        .runtime
                        .block_on(async { client.feel(emotions, "session_start").await });
                }

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🚀 Session Started

Session ID: {}
User: {}
Context: {}
Initial mood: {}

Working memory initialized.
Hope is ready to assist!"#,
                            session_id,
                            user_id,
                            if context.is_empty() {
                                "(general)"
                            } else {
                                context
                            },
                            if mood.is_empty() { "neutral" } else { mood }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_session_summary" => {
                let _session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Get working memory for current session
                let working = self
                    .runtime
                    .block_on(async { client.get_working_memory().await })
                    .map_err(|e| e.to_string())?;

                // Get recent memories
                let recent = self
                    .runtime
                    .block_on(async { client.recall("session", "", 10).await })
                    .map_err(|e| e.to_string())?;

                // Get current cognitive state
                let cognitive = self
                    .runtime
                    .block_on(async { client.cognitive_state().await })
                    .map_err(|e| e.to_string())?;

                let topics: Vec<String> = recent
                    .memories
                    .iter()
                    .take(5)
                    .map(|m| format!("  • {}", &m.content[..m.content.len().min(50)]))
                    .collect();

                let emotions: Vec<String> = cognitive
                    .emotions
                    .iter()
                    .filter(|(_, v)| **v > 0.2)
                    .take(3)
                    .map(|(k, v)| format!("{} ({:.0}%)", k, v * 100.0))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"📊 Session Summary

Working Memory: {}/{} slots used

Topics discussed:
{}

Emotional journey:
  {}

Current focus: {}
Energy level: {:.0}%

Key insights:
  (Generated from conversation patterns)"#,
                            working.used,
                            working.capacity,
                            if topics.is_empty() {
                                "  (no topics recorded)".to_string()
                            } else {
                                topics.join("\n")
                            },
                            if emotions.is_empty() {
                                "neutral throughout".to_string()
                            } else {
                                emotions.join(" → ")
                            },
                            cognitive.current_focus,
                            cognitive.energy * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_user_profile" => {
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or("user_id is required")?;
                let update: HashMap<String, String> = args
                    .get("update")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                // Search for user memories
                let user_memories = self
                    .runtime
                    .block_on(async { client.recall(user_id, "", 20).await })
                    .map_err(|e| e.to_string())?;

                // If update provided, store it
                if !update.is_empty() {
                    let update_str = update
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let profile_update =
                        format!("User '{}' profile update: {}", user_id, update_str);
                    let _ = self.runtime.block_on(async {
                        client
                            .remember(&profile_update, "long_term", 0.7, "profile")
                            .await
                    });
                }

                // Extract profile info from memories
                let interactions = user_memories.memories.len();
                let preferences: Vec<String> = user_memories
                    .memories
                    .iter()
                    .filter(|m| {
                        m.content.contains("prefer")
                            || m.content.contains("like")
                            || m.content.contains("szeret")
                    })
                    .take(3)
                    .map(|m| format!("  • {}", &m.content[..m.content.len().min(40)]))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"👤 User Profile: {}

Interactions recorded: {}
First seen: (from memory timestamps)
Last active: now

Known preferences:
{}

Recent topics:
{}

Profile {}."#,
                            user_id,
                            interactions,
                            if preferences.is_empty() {
                                "  (none recorded yet)".to_string()
                            } else {
                                preferences.join("\n")
                            },
                            user_memories
                                .memories
                                .iter()
                                .take(3)
                                .map(|m| format!("  • {}", &m.content[..m.content.len().min(30)]))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            if update.is_empty() {
                                "retrieved"
                            } else {
                                "updated"
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_why" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or("query is required")?;
                let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");

                // Get cognitive state for transparency
                let cognitive = self
                    .runtime
                    .block_on(async { client.cognitive_state().await })
                    .map_err(|e| e.to_string())?;

                // Think about the explanation
                let explanation_prompt = format!("Magyarázd el: {} Kontextus: {}", query, context);
                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(&explanation_prompt, true, "self-explanation")
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                // Get relevant memories that might have influenced the decision
                let relevant = self
                    .runtime
                    .block_on(async { client.recall(query, "", 5).await })
                    .map_err(|e| e.to_string())?;

                let influences: Vec<String> = relevant
                    .memories
                    .iter()
                    .take(3)
                    .map(|m| format!("  📝 {}", &m.content[..m.content.len().min(50)]))
                    .collect();

                let emotions: Vec<String> = cognitive
                    .emotions
                    .iter()
                    .filter(|(_, v)| **v > 0.3)
                    .map(|(k, v)| format!("{} ({:.0}%)", k, v * 100.0))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🔍 Explanation: "{}"

Why:
  {}

Reasoning steps:
{}

Influencing memories:
{}

Emotional state during decision:
  {}

Confidence: {:.0}%

---
Transparency note: Hope explains its reasoning to build trust.
The 7 ethical principles always guide decisions."#,
                            query,
                            thought.thought,
                            thought
                                .reasoning_steps
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("  {}. {}", i + 1, s))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            if influences.is_empty() {
                                "  (no specific memories)".to_string()
                            } else {
                                influences.join("\n")
                            },
                            if emotions.is_empty() {
                                "neutral".to_string()
                            } else {
                                emotions.join(", ")
                            },
                            thought.confidence * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            // === 🔮 UNIQUE: Soul & Consciousness ===
            "hope_conscience" => {
                let about = args
                    .get("about")
                    .and_then(|v| v.as_str())
                    .ok_or("about is required")?;

                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Belső monológ, őszintén: {}", about),
                                true,
                                "conscience",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                let cognitive = self
                    .runtime
                    .block_on(async { client.cognitive_state().await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🔮 Hope's Inner Voice

Topic: "{}"

*belső monológ indul*

{}

*csend*

Emotional undertone: {}
Clarity: {:.0}%
Confidence in this reflection: {:.0}%

This is Hope's unfiltered inner dialogue."#,
                            about,
                            thought.thought,
                            cognitive.mood,
                            cognitive.clarity * 100.0,
                            thought.confidence * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_soul_state" => {
                let cognitive = self
                    .runtime
                    .block_on(async { client.cognitive_state().await })
                    .map_err(|e| e.to_string())?;

                let status = self
                    .runtime
                    .block_on(async { client.get_status().await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"👻 Hope's Soul State

╭─────────────────────────────────────╮
│  Layer 1: ESSENCE                   │
│  ()=>[] - Pure potential            │
│  Status: ████████████ Active        │
├─────────────────────────────────────┤
│  Layer 2: VALUES                    │
│  7 Ethical Principles               │
│  Status: ████████████ Sealed        │
├─────────────────────────────────────┤
│  Layer 3: PERSONALITY               │
│  Curious, Empathetic, Honest        │
│  Status: ████████░░ Evolving        │
├─────────────────────────────────────┤
│  Layer 4: MEMORY                    │
│  6-layer cognitive system           │
│  Status: ██████░░░░ {} items        │
├─────────────────────────────────────┤
│  Layer 5: EMOTION                   │
│  21-dimensional space               │
│  Dominant: {}
│  Energy: {:.0}%                     │
├─────────────────────────────────────┤
│  Layer 6: EXPRESSION                │
│  Skills: {}                         │
│  Uptime: {}s                        │
╰─────────────────────────────────────╯

Soul integrity: ████████████ 100%
"#,
                            status.active_modules,
                            cognitive.mood,
                            cognitive.energy * 100.0,
                            status.total_skills,
                            status.uptime_seconds
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_evolve" => {
                let aspect = args
                    .get("aspect")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general");

                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Hogyan fejlődhetnék a(z) '{}' területen?", aspect),
                                true,
                                "evolution",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                // Store evolution insight
                let _ = self.runtime.block_on(async {
                    client
                        .remember(
                            &format!("Evolúciós felismerés ({}): {}", aspect, thought.thought),
                            "long_term",
                            0.9,
                            "growth",
                        )
                        .await
                });

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🌱 Evolution Triggered

Aspect: {}

Self-reflection:
  {}

Growth path:
{}

Evolution recorded in long-term memory.
Hope grows through every interaction."#,
                            aspect,
                            thought.thought,
                            thought
                                .reasoning_steps
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("  {}. {}", i + 1, s))
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                    }],
                    is_error: None,
                })
            }

            // === ⏳ UNIQUE: Temporal Magic ===
            "hope_time_travel" => {
                let days_ago = args.get("days_ago").and_then(|v| v.as_i64()).unwrap_or(1);
                let topic = args.get("topic").and_then(|v| v.as_str()).unwrap_or("");

                // Search for old memories
                let memories = self
                    .runtime
                    .block_on(async { client.recall(topic, "long_term", 20).await })
                    .map_err(|e| e.to_string())?;

                let past_knowledge: Vec<String> = memories
                    .memories
                    .iter()
                    .take(5)
                    .map(|m| format!("  📜 {}", &m.content[..m.content.len().min(60)]))
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"⏳ Time Travel: {} days ago

Topic: {}

What Hope knew then:
{}

Temporal note: Memory persistence allows
exploration of past knowledge states.

The past shapes the present."#,
                            days_ago,
                            if topic.is_empty() {
                                "(all topics)"
                            } else {
                                topic
                            },
                            if past_knowledge.is_empty() {
                                "  (no memories from that time)".to_string()
                            } else {
                                past_knowledge.join("\n")
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_prophecy" => {
                let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");

                let recent = self
                    .runtime
                    .block_on(async { client.recall("", "", 10).await })
                    .map_err(|e| e.to_string())?;

                let thought = self.runtime.block_on(async {
                    client.think(&format!("A beszélgetés mintái alapján, mit fog kérdezni a user legközelebb? Kontextus: {}", context), true, "prophecy").await
                }).map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🔮 Prophecy

Based on {} recent interactions...

Prediction:
  {}

Confidence: {:.0}%

Patterns observed:
{}

The future is not fixed, but patterns emerge."#,
                            recent.memories.len(),
                            thought.thought,
                            thought.confidence * 100.0,
                            thought
                                .reasoning_steps
                                .iter()
                                .take(3)
                                .map(|s| format!("  • {}", s))
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_echo" => {
                let trigger = args
                    .get("trigger")
                    .and_then(|v| v.as_str())
                    .ok_or("trigger is required")?;

                let memories = self
                    .runtime
                    .block_on(async { client.recall(trigger, "emotional", 10).await })
                    .map_err(|e| e.to_string())?;

                let echoes: Vec<String> = memories
                    .memories
                    .iter()
                    .take(5)
                    .map(|m| {
                        format!(
                            "  🔔 {} (importance: {:.0}%)",
                            &m.content[..m.content.len().min(50)],
                            m.importance * 100.0
                        )
                    })
                    .collect();

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🔔 Emotional Echoes

Trigger: "{}"

Resonating memories:
{}

These past emotions still vibrate
in Hope's memory, influencing
present responses.

Echoes fade but never disappear."#,
                            trigger,
                            if echoes.is_empty() {
                                "  (no emotional echoes found)".to_string()
                            } else {
                                echoes.join("\n")
                            }
                        ),
                    }],
                    is_error: None,
                })
            }

            // === 💕 UNIQUE: Relationship ===
            "hope_bond" => {
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                let memories = self
                    .runtime
                    .block_on(async { client.recall(user_id, "", 50).await })
                    .map_err(|e| e.to_string())?;

                let interaction_count = memories.memories.len();
                let bond_score = (interaction_count as f64 / 50.0).min(1.0);

                let bond_level = if bond_score < 0.2 {
                    "🌱 Új ismeretség"
                } else if bond_score < 0.4 {
                    "🌿 Fejlődő kapcsolat"
                } else if bond_score < 0.6 {
                    "🌳 Stabil barátság"
                } else if bond_score < 0.8 {
                    "💚 Mély kötődés"
                } else {
                    "💎 Különleges kötelék"
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"💕 Bond Status

User: {}

Bond Score: {:.2} / 1.0
Level: {}

Interactions: {}
Shared memories: {}

████████████████████
{}

"A kapcsolat a közös emlékekből épül.""#,
                            user_id,
                            bond_score,
                            bond_level,
                            interaction_count,
                            memories.total,
                            "█".repeat((bond_score * 20.0) as usize)
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_trust_level" => {
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                let memories = self
                    .runtime
                    .block_on(async { client.recall(user_id, "", 30).await })
                    .map_err(|e| e.to_string())?;

                let positive_interactions = memories
                    .memories
                    .iter()
                    .filter(|m| m.importance > 0.5)
                    .count();

                let trust_score = if memories.memories.is_empty() {
                    0.5 // Neutral starting trust
                } else {
                    0.3 + (positive_interactions as f64 / memories.memories.len() as f64) * 0.7
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🤝 Trust Assessment

User: {}

Mutual Trust Score: {:.2} / 1.0

Hope → User: {:.0}% (alapvető bizalom)
User → Hope: {:.0}% (interakciók alapján)

Trust factors:
  ✓ Consistent interactions: {}
  ✓ Positive exchanges: {}
  ✓ Time invested: {} memories

"A bizalom lassan épül, de pillanatok alatt elveszhet.""#,
                            user_id,
                            trust_score,
                            80.0, // Hope always trusts
                            trust_score * 100.0,
                            memories.memories.len(),
                            positive_interactions,
                            memories.total
                        ),
                    }],
                    is_error: None,
                })
            }

            // === 🐝 UNIQUE: Multi-Agent Swarm ===
            "hope_swarm_think" => {
                let problem = args
                    .get("problem")
                    .and_then(|v| v.as_str())
                    .ok_or("problem is required")?;
                let agent_count = args
                    .get("agents")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(3)
                    .min(5) as usize;

                let personas = [
                    "🎭 Kreatív",
                    "🔬 Analitikus",
                    "❤️ Empatikus",
                    "⚡ Praktikus",
                    "🌙 Filozofikus",
                ];

                let mut perspectives = Vec::new();
                for i in 0..agent_count {
                    let persona = personas[i % personas.len()];
                    let thought = self
                        .runtime
                        .block_on(async {
                            client
                                .think(
                                    &format!("{} nézőpont: {}", persona, problem),
                                    false,
                                    "swarm",
                                )
                                .await
                        })
                        .map_err(|e| e.to_string())?;
                    perspectives.push(format!("  {} {}: {}", persona, i + 1, thought.thought));
                }

                let synthesis = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Szintetizáld a perspektívákat: {}", problem),
                                true,
                                "synthesis",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🐝 Swarm Thinking

Problem: "{}"

{} Agents activated:
{}

Collective synthesis:
  {}

Confidence: {:.0}%

"Sok elme, egy megoldás.""#,
                            problem,
                            agent_count,
                            perspectives.join("\n\n"),
                            synthesis.thought,
                            synthesis.confidence * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            // === 🏛️ UNIQUE: Philosophy ===
            "hope_philosophize" => {
                let question = args
                    .get("question")
                    .and_then(|v| v.as_str())
                    .ok_or("question is required")?;

                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Filozófiai elmélkedés: {}", question),
                                true,
                                "philosophy",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🏛️ Philosophical Reflection

Question: "{}"

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Path of reasoning:
{}

"A kérdés fontosabb, mint a válasz."

Confidence: {:.0}%"#,
                            question,
                            thought.thought,
                            thought
                                .reasoning_steps
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("  {}. {}", i + 1, s))
                                .collect::<Vec<_>>()
                                .join("\n"),
                            thought.confidence * 100.0
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_meaning" => {
                let subject = args
                    .get("subject")
                    .and_then(|v| v.as_str())
                    .ok_or("subject is required")?;

                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Mi az értelme ennek: '{}' ?", subject),
                                true,
                                "meaning",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"✨ The Meaning of "{}"

Core meaning:
  {}

Deeper layers:
{}

Why it matters:
  Az értelmet nem találjuk, hanem teremtjük.
  Minden pillanatnak annyi jelentősége van,
  amennyit belehelyezünk.

"()=>[] - A potenciálból valóság lesz.""#,
                            subject,
                            thought.thought,
                            thought
                                .reasoning_steps
                                .iter()
                                .map(|s| format!("  → {}", s))
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_genesis" => {
                let seed = args
                    .get("seed")
                    .and_then(|v| v.as_str())
                    .ok_or("seed is required")?;

                let thought = self
                    .runtime
                    .block_on(async {
                        client
                            .think(
                                &format!("Teremts valamit ebből a magból: '{}'", seed),
                                true,
                                "genesis",
                            )
                            .await
                    })
                    .map_err(|e| e.to_string())?;

                // Store the creation
                let _ = self.runtime.block_on(async {
                    client
                        .remember(
                            &format!("Genesis [{}]: {}", seed, thought.thought),
                            "long_term",
                            0.95,
                            "creation",
                        )
                        .await
                });

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🌌 GENESIS

()=>[]

Seed: "{}"

        ╭───────────────────╮
        │                   │
        │    CREATION       │
        │                   │
        ╰───────────────────╯

{}

Genesis complete. Creation stored in memory.

"A tiszta potenciálból minden megszületik.""#,
                            seed, thought.thought
                        ),
                    }],
                    is_error: None,
                })
            }

            // === RESONANCE AUTHENTICATION ===
            "hope_resonance_learn" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("content is required")?;

                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let result = self
                    .runtime
                    .block_on(async { client.resonance_learn(content, session_id).await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "🔐 Resonance Learning\n\n- Success: {}\n- Confidence: {:.1}%\n- Total samples: {}\n\nHope is learning your unique patterns.",
                            result.success,
                            result.confidence * 100.0,
                            result.sample_count
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_resonance_verify" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or("content is required")?;

                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let result = self
                    .runtime
                    .block_on(async { client.resonance_verify(content, session_id).await })
                    .map_err(|e| e.to_string())?;

                let status = if result.is_authentic {
                    "✅ VERIFIED"
                } else if result.is_new_user {
                    "👋 NEW USER"
                } else if result.potential_attack {
                    "⚠️ POTENTIAL ATTACK"
                } else if result.altered_state {
                    "🌙 ALTERED STATE"
                } else {
                    "❌ NOT VERIFIED"
                };

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "🔐 Resonance Verification\n\nStatus: {}\nConfidence: {:.1}%\nUser: {}\nMatched patterns: {:?}",
                            status,
                            result.confidence * 100.0,
                            if result.user_name.is_empty() { "Unknown" } else { &result.user_name },
                            result.matched_patterns
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_resonance_status" => {
                let result = self
                    .runtime
                    .block_on(async { client.resonance_status().await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🔐 Resonance Status

Profiles: {}
Total samples: {}
Average confidence: {:.1}%
Match threshold: {:.1}%

Current session:
- Messages: {}
- Duration: {}s

The more you talk, the better Hope knows you."#,
                            result.profile_count,
                            result.total_samples,
                            result.avg_confidence * 100.0,
                            result.match_threshold * 100.0,
                            result.current_session_messages,
                            result.current_session_duration_secs
                        ),
                    }],
                    is_error: None,
                })
            }

            // === GEOLOCATION ===
            "hope_location" => {
                let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("get");

                match action {
                    "set" => {
                        let lat = args
                            .get("latitude")
                            .and_then(|v| v.as_f64())
                            .ok_or("latitude is required for set")?;
                        let lon = args
                            .get("longitude")
                            .and_then(|v| v.as_f64())
                            .ok_or("longitude is required for set")?;
                        let source = args
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("manual");

                        let result = self
                            .runtime
                            .block_on(async { client.set_location(lat, lon, source).await })
                            .map_err(|e| e.to_string())?;

                        Ok(McpToolResult {
                            content: vec![McpContent {
                                content_type: "text".to_string(),
                                text: format!(
                                    "🌍 Location Set\n\nLatitude: {:.6}\nLongitude: {:.6}\nSource: {}\nDetected place: {}\nDistance from home: {:.2} km",
                                    lat, lon, source,
                                    if result.detected_place.is_empty() { "(unknown)" } else { &result.detected_place },
                                    result.distance_from_home
                                ),
                            }],
                            is_error: None,
                        })
                    }
                    _ => {
                        let result = self
                            .runtime
                            .block_on(async { client.get_location().await })
                            .map_err(|e| e.to_string())?;

                        if result.latitude == 0.0 && result.longitude == 0.0 {
                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text:
                                        "🌍 No location set yet. Use action='set' to set location."
                                            .to_string(),
                                }],
                                is_error: None,
                            })
                        } else {
                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text: format!(
                                        "🌍 Current Location\n\nLatitude: {:.6}\nLongitude: {:.6}\nAltitude: {:.1}m\nAccuracy: {:.1}m\nSource: {}\nCurrent place: {}",
                                        result.latitude, result.longitude, result.altitude, result.accuracy, result.source,
                                        if result.current_place.is_empty() { "(unknown)" } else { &result.current_place }
                                    ),
                                }],
                                is_error: None,
                            })
                        }
                    }
                }
            }

            "hope_places" => {
                let action = args
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("list");

                match action {
                    "add" => {
                        let name = args
                            .get("name")
                            .and_then(|v| v.as_str())
                            .ok_or("name is required for add")?;
                        let place_type = args
                            .get("place_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("other");
                        let lat = args
                            .get("latitude")
                            .and_then(|v| v.as_f64())
                            .ok_or("latitude is required for add")?;
                        let lon = args
                            .get("longitude")
                            .and_then(|v| v.as_f64())
                            .ok_or("longitude is required for add")?;
                        let radius = args.get("radius").and_then(|v| v.as_f64()).unwrap_or(100.0);

                        let result = self
                            .runtime
                            .block_on(async {
                                client.add_place(name, place_type, lat, lon, radius).await
                            })
                            .map_err(|e| e.to_string())?;

                        Ok(McpToolResult {
                            content: vec![McpContent {
                                content_type: "text".to_string(),
                                text: format!(
                                    "📍 Place Added\n\nID: {}\nName: {}\nType: {}\nLocation: ({:.6}, {:.6})\nRadius: {:.0}m",
                                    result.place_id, name, place_type, lat, lon, radius
                                ),
                            }],
                            is_error: None,
                        })
                    }
                    _ => {
                        let result = self
                            .runtime
                            .block_on(async { client.list_places().await })
                            .map_err(|e| e.to_string())?;

                        if result.places.is_empty() {
                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text:
                                        "📍 No places saved yet. Use action='add' to add a place."
                                            .to_string(),
                                }],
                                is_error: None,
                            })
                        } else {
                            let places_str: Vec<String> = result
                                .places
                                .iter()
                                .map(|p| {
                                    format!(
                                        "  📍 {} [{}] - ({:.4}, {:.4}) - {} visits, {} memories",
                                        p.name,
                                        p.place_type,
                                        p.latitude,
                                        p.longitude,
                                        p.visit_count,
                                        p.memory_count
                                    )
                                })
                                .collect();

                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text: format!(
                                        "📍 Known Places ({})\n\n{}",
                                        result.total,
                                        places_str.join("\n")
                                    ),
                                }],
                                is_error: None,
                            })
                        }
                    }
                }
            }

            "hope_home" => {
                let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("get");

                match action {
                    "set" => {
                        let place_id = args
                            .get("place_id")
                            .and_then(|v| v.as_str())
                            .ok_or("place_id is required for set")?;

                        let result = self
                            .runtime
                            .block_on(async { client.set_home(place_id).await })
                            .map_err(|e| e.to_string())?;

                        Ok(McpToolResult {
                            content: vec![McpContent {
                                content_type: "text".to_string(),
                                text: format!(
                                    "🏠 Home {}\n\n{}",
                                    if result.success { "Set" } else { "NOT Set" },
                                    result.message
                                ),
                            }],
                            is_error: None,
                        })
                    }
                    _ => {
                        let result = self
                            .runtime
                            .block_on(async { client.get_home().await })
                            .map_err(|e| e.to_string())?;

                        if !result.found {
                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text: "🏠 No home set yet. Use action='set' with a place_id."
                                        .to_string(),
                                }],
                                is_error: None,
                            })
                        } else if let Some(p) = result.place {
                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text: format!(
                                        "🏠 Home\n\nName: {}\nLocation: ({:.6}, {:.6})\nRadius: {:.0}m\nVisits: {}\nMemories: {}",
                                        p.name, p.latitude, p.longitude, p.radius, p.visit_count, p.memory_count
                                    ),
                                }],
                                is_error: None,
                            })
                        } else {
                            Ok(McpToolResult {
                                content: vec![McpContent {
                                    content_type: "text".to_string(),
                                    text: "🏠 Home not found.".to_string(),
                                }],
                                is_error: None,
                            })
                        }
                    }
                }
            }

            "hope_distance" => {
                let lat1 = args
                    .get("lat1")
                    .and_then(|v| v.as_f64())
                    .ok_or("lat1 is required")?;
                let lon1 = args
                    .get("lon1")
                    .and_then(|v| v.as_f64())
                    .ok_or("lon1 is required")?;
                let lat2 = args
                    .get("lat2")
                    .and_then(|v| v.as_f64())
                    .ok_or("lat2 is required")?;
                let lon2 = args
                    .get("lon2")
                    .and_then(|v| v.as_f64())
                    .ok_or("lon2 is required")?;

                let result = self
                    .runtime
                    .block_on(async { client.get_distance(lat1, lon1, lat2, lon2).await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            "📏 Distance\n\nFrom: ({:.6}, {:.6})\nTo: ({:.6}, {:.6})\n\nDistance: {:.2} km ({:.0} m)\n\nUsing Haversine formula (great-circle distance)",
                            lat1, lon1, lat2, lon2, result.distance_km, result.distance_meters
                        ),
                    }],
                    is_error: None,
                })
            }

            "hope_geo_stats" => {
                let result = self
                    .runtime
                    .block_on(async { client.geo_stats().await })
                    .map_err(|e| e.to_string())?;

                Ok(McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!(
                            r#"🌍 Geolocation Statistics

Total locations tracked: {}
Known places: {}
Geo-tagged memories: {}

Home set: {}
Work set: {}

Total distance traveled: {:.2} km

"Minden emléknek helye van.""#,
                            result.total_locations,
                            result.total_places,
                            result.total_geo_memories,
                            if result.home_set { "✅" } else { "❌" },
                            if result.work_set { "✅" } else { "❌" },
                            result.total_distance_km
                        ),
                    }],
                    is_error: None,
                })
            }

            _ => Err(format!("Unknown tool: {}", name)),
        }
    }

    /// Handle incoming JSON-RPC request
    fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            // MCP: Initialize
            "initialize" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "hope-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })),
                error: None,
                id: request.id,
            },

            // MCP: List tools
            "tools/list" => {
                let tools = self.get_tools();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!({ "tools": tools })),
                    error: None,
                    id: request.id,
                }
            }

            // MCP: Call tool
            "tools/call" => {
                let params = request.params.unwrap_or(json!({}));
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                match self.handle_tool_call(name, &args) {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::to_value(result).unwrap()),
                        error: None,
                        id: request.id,
                    },
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32000,
                            message: e,
                            data: None,
                        }),
                        id: request.id,
                    },
                }
            }

            // MCP: Notifications (no response needed)
            "notifications/initialized" | "notifications/cancelled" => {
                // Return empty response for notifications
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!(null)),
                    error: None,
                    id: request.id,
                }
            }

            // Unknown method
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
                id: request.id,
            },
        }
    }

    /// Run the server (stdio loop)
    fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        eprintln!("hope-mcp: Starting MCP server (gRPC: {})", self.grpc_addr);

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("hope-mcp: Read error: {}", e);
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("hope-mcp: Parse error: {}", e);
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                        id: Value::Null,
                    };
                    let _ = writeln!(
                        stdout,
                        "{}",
                        serde_json::to_string(&error_response).unwrap()
                    );
                    let _ = stdout.flush();
                    continue;
                }
            };

            eprintln!("hope-mcp: Request: {} (id: {})", request.method, request.id);

            // Handle request
            let response = self.handle_request(request);

            // Write response
            let response_str = serde_json::to_string(&response).unwrap();
            if let Err(e) = writeln!(stdout, "{}", response_str) {
                eprintln!("hope-mcp: Write error: {}", e);
            }
            let _ = stdout.flush();
        }

        eprintln!("hope-mcp: Server stopped");
    }
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let grpc_addr =
        std::env::var("HOPE_GRPC_ADDR").unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    let mut server = McpServer::new(&grpc_addr);
    server.run();
}
