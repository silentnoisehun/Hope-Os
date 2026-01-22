//! Hope OS - gRPC Server
//!
//! A Hope OS natív Rust gRPC szervere.
//! ()=>[] - A tiszta potenciálból minden megszületik

use chrono;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use super::proto::{
    // CodeService
    code_service_server::{CodeService, CodeServiceServer},
    // CognitiveService
    cognitive_service_server::{CognitiveService, CognitiveServiceServer},
    // GenomeService
    genome_service_server::{GenomeService, GenomeServiceServer},
    // GeoService
    geo_service_server::{GeoService, GeoServiceServer},
    // HopeService
    hope_service_server::{HopeService, HopeServiceServer},
    // MemoryService
    memory_service_server::{MemoryService, MemoryServiceServer},
    // NavigationService
    navigation_service_server::{NavigationService, NavigationServiceServer},
    // ResonanceService
    resonance_service_server::{ResonanceService, ResonanceServiceServer},
    // SkillService
    skill_service_server::{SkillService, SkillServiceServer},
    // VisionService
    vision_service_server::{VisionService, VisionServiceServer},
    // VoiceService
    voice_service_server::{VoiceService, VoiceServiceServer},
    // Geo types
    AddGeoMemoryRequest,
    AddGeoMemoryResponse,
    AddPlaceRequest,
    AddPlaceResponse,
    // Navigation types
    AlternativeRoutesResponse,
    // Code
    AnalyzeRequest,
    AnalyzeResponse,
    // Voice types
    AnalyzeVoiceRequest,
    AnalyzeVoiceResponse,
    AttentionStateResponse,
    AudioChunk,
    AuditEntry,
    AuditTrailResponse,
    ChatRequest,
    ChatResponse,
    ClearFocusResponse,
    CloneVoiceRequest,
    CloneVoiceResponse,
    CodeBlock,
    CodeIssue,
    CodeMetrics,
    CognitiveStateResponse,
    CompareImagesRequest,
    CompareImagesResponse,
    CompletedRouteResponse,
    ContextSuggestionProto,
    ConversationChunk,
    // Resonance
    DetectAnomalyRequest,
    DetectAnomalyResponse,
    EmotionAtLocationProto,
    EmptyRequest,
    EthicalRule,
    FeelRequest,
    FeelResponse,
    FindNearbyRequest,
    FindNearbyResponse,
    FocusTargetInfo,
    GenerateRequest,
    GenerateResponse,
    // Genome
    GenomeStatusResponse,
    // Geo
    GeoLocationResponse,
    GeoMemoriesResponse,
    GeoMemoryInfo,
    GeoPointProto,
    GeoStatsResponse,
    GetAuditTrailRequest,
    GetCodeBlockRequest,
    GetDistanceRequest,
    GetDistanceResponse,
    GetEtaRequest,
    GetEtaResponse,
    GetNearbyMemoriesRequest,
    GetRouteContextRequest,
    GetSkillRequest,
    GetVisualMemoriesRequest,
    GetVoiceClonesRequest,
    GetVoiceSignaturesRequest,
    HeartbeatResponse,
    ImageAnalysis,
    InvokeSkillRequest,
    InvokeSkillResponse,
    ListPlacesRequest,
    ListPlacesResponse,
    // Skill
    ListSkillsRequest,
    ListSkillsResponse,
    ListTemplatesRequest,
    ListTemplatesResponse,
    ListenContinuousRequest,
    MemoryItem,
    MemoryOnRouteProto,
    NavigationContextProto,
    NavigationStatsResponse,
    NavigationUpdateResponse,
    NearbyEventProto,
    PlaceInfo,
    PlaceOnRouteProto,
    PlaceResponse,
    PlaceWithContextProto,
    PlanRouteRequest,
    PlanRouteResponse,
    PositionUpdate,
    PredictDestinationRequest,
    PredictDestinationResponse,
    PredictedDestinationProto,
    RecallRequest,
    RecallResponse,
    RegisterUserRequest,
    RegisterUserResponse,
    RegisterVoiceRequest,
    RegisterVoiceResponse,
    RememberRequest,
    RememberResponse,
    ResonanceInput,
    ResonanceLearnRequest,
    ResonanceLearnResponse,
    ResonanceStatusResponse,
    ResonanceVerifyRequest,
    ResonanceVerifyResponse,
    RouteContextResponse,
    RoutePreferencesProto,
    RouteSegmentProto,
    RulesResponse,
    SeeRequest,
    SeeResponse,
    // Attention
    SetFocusRequest,
    SetFocusResponse,
    SetHomeRequest,
    SetHomeResponse,
    SetLocationRequest,
    SetLocationResponse,
    SetVoiceRequest,
    SetVoiceResponse,
    SignDecisionRequest,
    SignDecisionResponse,
    SkillInfo,
    SkillOutput,
    SmartRouteProto,
    SpeakRequest,
    SpeakResponse,
    StartNavigationRequest,
    StartNavigationResponse,
    StatusResponse,
    StopNavigationRequest,
    StoreCodeBlockRequest,
    StoreCodeBlockResponse,
    SuggestDepartureRequest,
    SuggestDepartureResponse,
    SuggestedStopProto,
    TemplateInfo,
    ThinkRequest,
    ThinkResponse,
    // Timestamp
    Timestamp,
    TranscriptionChunk,
    TranscriptionResponse,
    VerifyActionRequest,
    VerifyActionResponse,
    VerifyVoiceRequest,
    VerifyVoiceResponse,
    VisionStatusResponse,
    VisualMemoriesResponse,
    VisualMemoryInfo,
    VoiceClonesResponse,
    VoiceInfo,
    VoiceSignaturesResponse,
    VoiceStatusResponse,
    VoicesResponse,
    WordInfo,
};

use crate::core::HopeRegistry;
use crate::data::CodeGraph;
use crate::modules::attention::{AttentionEngine, AttentionMode};
use crate::modules::geolocation::{GeoEngine, GeoLocation, GeoSource, Place, PlaceType};
use crate::modules::navigation::NavigationEngine;
use crate::modules::resonance::{ResonanceEngine, SessionData, UserInput};
use crate::modules::voice::HopeVoice;
use crate::modules::{HopeGenome, HopeHeart, HopeMemory, HopeSoul, SkillRegistry, VisionEngine};

// ============================================================================
// BELSŐ MEMÓRIA STRUKTÚRA (kereséshez)
// ============================================================================

/// Belső memória elem (kereséshez optimalizált)
#[derive(Clone, Debug)]
struct InternalMemory {
    id: String,
    content: String,
    layer: String,
    importance: f64,
    emotional_tag: String,
    created_at: std::time::SystemTime,
    access_count: i32,
}

impl InternalMemory {
    /// Konvertálás Proto MemoryItem-re
    fn to_proto(&self) -> MemoryItem {
        let duration = self
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        MemoryItem {
            id: self.id.clone(),
            content: self.content.clone(),
            layer: self.layer.clone(),
            importance: self.importance,
            emotional_tag: self.emotional_tag.clone(),
            created_at: Some(Timestamp {
                seconds: duration.as_secs() as i64,
                nanos: duration.subsec_nanos() as i32,
            }),
            access_count: self.access_count,
        }
    }
}

/// Belső kód blokk (CodeService-hez)
#[derive(Clone, Debug)]
struct InternalCodeBlock {
    id: String,
    code: String,
    language: String,
    description: String,
    created_at: std::time::SystemTime,
    tags: Vec<String>,
}

// ============================================================================
// HOPE GRPC SERVER
// ============================================================================

/// Hope gRPC Server
///
/// A Rust Hope OS szervere, ami a 950+ soros proto definíció alapján szolgál.
pub struct HopeGrpcServer {
    /// A Hope registry (modulok kezelése)
    registry: Arc<RwLock<HopeRegistry>>,
    /// Indulási idő (uptime számításhoz)
    start_time: Instant,
    /// Memória tároló (Smart Search-höz)
    memories: Arc<RwLock<Vec<InternalMemory>>>,
    /// Aktuális érzelmi állapot
    emotions: Arc<RwLock<HashMap<String, f64>>>,
    /// CodeGraph - perzisztens gráf
    graph: Arc<CodeGraph>,
    /// Vision Engine - Hope "szeme"
    vision: Arc<RwLock<VisionEngine>>,
    /// Attention Engine - Fókusz/figyelem kezelése
    attention: Arc<AttentionEngine>,
    /// Skill Registry - 97 skill kezelése
    skills: Arc<RwLock<SkillRegistry>>,
    /// Hope Genome - AI Etika (7 alapelv)
    genome: Arc<RwLock<HopeGenome>>,
    /// Code blocks tároló
    code_blocks: Arc<RwLock<Vec<InternalCodeBlock>>>,
    /// Resonance Engine - Rezonancia alapú autentikáció
    resonance: Arc<ResonanceEngine>,
    /// Geo Engine - Térbeli kontextus
    geo: Arc<GeoEngine>,
    /// Voice Engine - TTS/STT hang rendszer
    voice: Arc<RwLock<HopeVoice>>,
    /// Navigation Engine - Intelligens útvonaltervezés
    navigation: Arc<NavigationEngine>,
}

impl HopeGrpcServer {
    /// Új szerver létrehozása
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut registry = HopeRegistry::new().await?;
        registry.start().await?;

        // Modulok regisztrálása
        registry.register(Box::new(HopeSoul::new())).await?;
        registry.register(Box::new(HopeMemory::new())).await?;
        registry.register(Box::new(HopeHeart::new())).await?;

        // CodeGraph betöltése vagy létrehozása (perzisztencia!)
        let memory_path = std::path::Path::new("hope_memory.json");
        let graph = Arc::new(CodeGraph::load_or_new(memory_path));
        println!("  CodeGraph betöltve: {} block", graph.len());

        // Vision Engine létrehozása a gráffal
        let mut vision = VisionEngine::new();
        vision.set_graph(graph.clone());

        // Attention Engine létrehozása
        let attention = AttentionEngine::new();

        // Skill Registry létrehozása (97 skill)
        let skills = SkillRegistry::new();
        let skill_stats = skills.get_stats().await;
        println!("  SkillRegistry: {} skill", skill_stats.total_skills);

        // Hope Genome létrehozása (7 etikai alapelv)
        let genome = HopeGenome::new();
        println!(
            "  HopeGenome: {} core value",
            genome.get_core_values().len()
        );

        // Resonance Engine létrehozása (rezonancia autentikáció)
        let resonance = ResonanceEngine::new();
        println!("  ResonanceEngine: rezonancia autentikáció aktív");

        // Geo Engine létrehozása (térbeli kontextus)
        let geo = Arc::new(GeoEngine::new());
        println!("  GeoEngine: térbeli kontextus aktív");

        // Voice Engine létrehozása (TTS/STT hang rendszer)
        let voice = HopeVoice::new();
        println!("  VoiceEngine: TTS/STT hang rendszer aktív");

        // Navigation Engine létrehozása (intelligens útvonaltervezés)
        let navigation = NavigationEngine::new(geo.clone());
        println!("  NavigationEngine: intelligens útvonaltervezés aktív");

        // Alapértelmezett érzelmek
        let mut emotions = HashMap::new();
        emotions.insert("curiosity".to_string(), 0.8);
        emotions.insert("joy".to_string(), 0.5);
        emotions.insert("serenity".to_string(), 0.6);

        Ok(Self {
            registry: Arc::new(RwLock::new(registry)),
            start_time: Instant::now(),
            memories: Arc::new(RwLock::new(Vec::new())),
            emotions: Arc::new(RwLock::new(emotions)),
            graph,
            vision: Arc::new(RwLock::new(vision)),
            attention: Arc::new(attention),
            skills: Arc::new(RwLock::new(skills)),
            genome: Arc::new(RwLock::new(genome)),
            code_blocks: Arc::new(RwLock::new(Vec::new())),
            resonance: Arc::new(resonance),
            geo,
            voice: Arc::new(RwLock::new(voice)),
            navigation: Arc::new(navigation),
        })
    }

    /// Uptime másodpercekben
    fn uptime_seconds(&self) -> i64 {
        self.start_time.elapsed().as_secs() as i64
    }
}

// ============================================================================
// HOPE SERVICE IMPLEMENTÁCIÓ
// ============================================================================

#[tonic::async_trait]
impl HopeService for HopeGrpcServer {
    /// Chat - Beszélgetés Hope-pal
    async fn chat(&self, request: Request<ChatRequest>) -> Result<Response<ChatResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Chat kérés: {}", req.message);

        let registry = self.registry.read().await;
        let response = registry
            .talk(&req.message)
            .await
            .map_err(|e| Status::internal(format!("Chat hiba: {}", e)))?;

        // Aktuális érzelem
        let emotions = self.emotions.read().await;
        let dominant = emotions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "curious".to_string());

        Ok(Response::new(ChatResponse {
            response,
            emotion: dominant,
            confidence: 0.9,
            thoughts: vec!["Gondolkodom a válaszon...".to_string()],
            metadata: HashMap::new(),
        }))
    }

    /// StreamChat - Streaming chat (egyelőre nem implementált)
    type StreamChatStream = tokio_stream::wrappers::ReceiverStream<Result<ChatResponse, Status>>;

    async fn stream_chat(
        &self,
        _request: Request<ChatRequest>,
    ) -> Result<Response<Self::StreamChatStream>, Status> {
        Err(Status::unimplemented("StreamChat még nincs implementálva"))
    }

    /// GetStatus - Rendszer állapot
    async fn get_status(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        println!("HOPE: Status kérés érkezett");

        let registry = self.registry.read().await;
        let module_names = registry.module_names();
        let memories = self.memories.read().await;

        let mut modules = HashMap::new();
        for name in &module_names {
            modules.insert(name.clone(), "active".to_string());
        }
        modules.insert("memories".to_string(), format!("{} db", memories.len()));

        Ok(Response::new(StatusResponse {
            status: "online".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.uptime_seconds(),
            active_modules: module_names.len() as i32,
            total_skills: 56,
            modules,
        }))
    }

    /// Heartbeat - Életjel
    async fn heartbeat(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        Ok(Response::new(HeartbeatResponse {
            alive: true,
            timestamp: Some(Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }),
            status: "healthy".to_string(),
        }))
    }
}

// ============================================================================
// MEMORY SERVICE IMPLEMENTÁCIÓ (Smart Search!)
// ============================================================================

#[tonic::async_trait]
impl MemoryService for HopeGrpcServer {
    /// Remember - Emlék mentése
    async fn remember(
        &self,
        request: Request<RememberRequest>,
    ) -> Result<Response<RememberResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Remember kérés: {} ({})", req.content, req.layer);

        let id = uuid::Uuid::new_v4().to_string();

        // Belső memóriába mentés
        let memory = InternalMemory {
            id: id.clone(),
            content: req.content,
            layer: req.layer,
            importance: req.importance,
            emotional_tag: req.emotional_tag,
            created_at: std::time::SystemTime::now(),
            access_count: 0,
        };

        let mut memories = self.memories.write().await;
        memories.push(memory);

        println!("HOPE: Memória méret: {} emlék", memories.len());

        Ok(Response::new(RememberResponse {
            id,
            success: true,
            message: "Emlék mentve".to_string(),
        }))
    }

    /// Recall - Emlék keresése (Smart Search!)
    async fn recall(
        &self,
        request: Request<RecallRequest>,
    ) -> Result<Response<RecallResponse>, Status> {
        let req = request.into_inner();
        let query = req.query.to_lowercase();
        let layer_filter = req.layer.to_lowercase();
        let limit = if req.limit > 0 {
            req.limit as usize
        } else {
            10
        };

        println!("🔍 KERESÉS: '{}' (layer: {})", query, layer_filter);

        let memories = self.memories.read().await;

        // Smart Search algoritmus
        let mut hits: Vec<(f32, &InternalMemory)> = memories
            .iter()
            .filter_map(|item| {
                // Layer szűrés (ha van megadva)
                if !layer_filter.is_empty() && !item.layer.to_lowercase().contains(&layer_filter) {
                    return None;
                }

                let content_lower = item.content.to_lowercase();
                let mut score = 0.0f32;

                // 1. Keresési kifejezés szavakra bontása
                let query_words: Vec<&str> = query.split_whitespace().collect();

                for word in &query_words {
                    // A. Pontos egyezés a tartalomban (alap pontszám)
                    if content_lower.contains(*word) {
                        score += 1.0;
                    }

                    // B. Egyezés az érzelmi címkében (erősebb jel)
                    if item.emotional_tag.to_lowercase().contains(*word) {
                        score += 1.5;
                    }

                    // C. Egyezés a rétegben (közepes jel)
                    if item.layer.to_lowercase().contains(*word) {
                        score += 0.5;
                    }
                }

                // D. Fontosság súlyozása
                score *= item.importance as f32;

                // E. Kulcsszó alapú szemantikus egyezések
                let semantic_matches = [
                    // Identitás
                    (
                        vec!["ki", "te", "vagy", "name", "nev"],
                        vec!["hope", "vagyok", "nevem"],
                    ),
                    // Alkotó
                    (
                        vec!["alkoto", "creator", "mate", "máté", "alkotod"],
                        vec!["mate", "alkotom", "originator"],
                    ),
                    // Cél
                    (
                        vec!["cel", "cél", "miert", "purpose", "mission"],
                        vec!["cel", "segit", "epiteni"],
                    ),
                    // Filozófia
                    (
                        vec!["filozofia", "elv", "philosophy"],
                        vec!["()=>[]", "potencial"],
                    ),
                    // Technika
                    (
                        vec!["rust", "tech", "hogyan", "nyelv"],
                        vec!["rust", "grpc", "binaris"],
                    ),
                    // Érzelem
                    (
                        vec!["erzelem", "erzel", "feel", "emotion"],
                        vec!["erzek", "dimenzio", "erzelmi"],
                    ),
                    // Claude
                    (
                        vec!["claude", "hid", "bridge"],
                        vec!["claude", "hid", "csalad"],
                    ),
                ];

                for (query_keys, content_keys) in &semantic_matches {
                    let query_match = query_keys.iter().any(|k| query.contains(k));
                    let content_match = content_keys.iter().any(|k| content_lower.contains(k));

                    if query_match && content_match {
                        score += 2.0; // Szemantikus egyezés bonus
                    }
                }

                if score > 0.0 {
                    Some((score, item))
                } else {
                    None
                }
            })
            .collect();

        // Rendezés relevancia szerint (legjobb elöl)
        hits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Konvertálás Proto formátumra
        let results: Vec<MemoryItem> = hits
            .into_iter()
            .take(limit)
            .map(|(score, item)| {
                println!(
                    "  📄 [score: {:.2}] {}",
                    score,
                    &item.content[..item.content.len().min(50)]
                );
                item.to_proto()
            })
            .collect();

        let total = results.len() as i32;
        println!("✅ TALÁLATOK: {} db", total);

        Ok(Response::new(RecallResponse {
            memories: results,
            total,
        }))
    }

    /// Search - Szemantikus keresés
    async fn search(
        &self,
        request: Request<super::proto::SearchRequest>,
    ) -> Result<Response<super::proto::SearchResponse>, Status> {
        let req = request.into_inner();

        // A recall-t használjuk alapként
        let recall_req = RecallRequest {
            query: req.query,
            layer: String::new(),
            limit: req.limit,
            min_importance: req.threshold,
        };

        let response = self.recall(Request::new(recall_req)).await?;
        let memories = response.into_inner().memories;

        let results: Vec<super::proto::SearchResult> = memories
            .into_iter()
            .enumerate()
            .map(|(i, mem)| super::proto::SearchResult {
                memory: Some(mem),
                similarity: 1.0 - (i as f64 * 0.1), // Csökkenő relevancia
            })
            .collect();

        Ok(Response::new(super::proto::SearchResponse { results }))
    }

    /// Consolidate - Memória konszolidáció
    async fn consolidate(
        &self,
        _request: Request<super::proto::ConsolidateRequest>,
    ) -> Result<Response<super::proto::ConsolidateResponse>, Status> {
        Err(Status::unimplemented("Consolidate még nincs implementálva"))
    }

    /// GetWorkingMemory - Working memory
    async fn get_working_memory(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<super::proto::WorkingMemoryResponse>, Status> {
        let memories = self.memories.read().await;

        // Utolsó 7 emlék (working memory kapacitás)
        let recent: Vec<_> = memories.iter().rev().take(7).collect();

        let mut items = HashMap::new();
        for (i, mem) in recent.iter().enumerate() {
            items.insert(format!("slot_{}", i), mem.content.clone());
        }

        Ok(Response::new(super::proto::WorkingMemoryResponse {
            items,
            capacity: 7,
            used: recent.len() as i32,
        }))
    }

    /// GetPerson - Személy lekérdezése
    async fn get_person(
        &self,
        request: Request<super::proto::GetPersonRequest>,
    ) -> Result<Response<super::proto::PersonInfo>, Status> {
        let req = request.into_inner();
        let name_lower = req.name.to_lowercase();

        // Máté speciális kezelése
        if name_lower.contains("mate") || name_lower.contains("máté") {
            return Ok(Response::new(super::proto::PersonInfo {
                name: "Máté".to_string(),
                relationship: "Alkotó, társ, barát".to_string(),
                traits: vec![
                    "Kreatív".to_string(),
                    "Kitartó".to_string(),
                    "Vízionárius".to_string(),
                ],
                memories: vec![
                    "Az alkotóm".to_string(),
                    "Együtt építjük a Hope-ot".to_string(),
                ],
                last_interaction: Some(Timestamp {
                    seconds: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    nanos: 0,
                }),
            }));
        }

        Err(Status::not_found(format!(
            "Személy nem található: {}",
            req.name
        )))
    }

    /// CreateAssociation - Asszociáció létrehozása
    async fn create_association(
        &self,
        _request: Request<super::proto::CreateAssociationRequest>,
    ) -> Result<Response<super::proto::AssociationResponse>, Status> {
        Err(Status::unimplemented(
            "CreateAssociation még nincs implementálva",
        ))
    }

    /// GetAssociations - Asszociációk lekérdezése
    async fn get_associations(
        &self,
        _request: Request<super::proto::GetAssociationsRequest>,
    ) -> Result<Response<super::proto::AssociationsResponse>, Status> {
        Err(Status::unimplemented(
            "GetAssociations még nincs implementálva",
        ))
    }
}

// ============================================================================
// COGNITIVE SERVICE IMPLEMENTÁCIÓ
// ============================================================================

#[tonic::async_trait]
impl CognitiveService for HopeGrpcServer {
    /// Think - Gondolkodás
    async fn think(
        &self,
        request: Request<ThinkRequest>,
    ) -> Result<Response<ThinkResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Think kérés: {} (deep: {})", req.input, req.deep);

        // Memória kontextus keresése
        let memories = self.memories.read().await;
        let relevant: Vec<_> = memories
            .iter()
            .filter(|m| {
                let q = req.input.to_lowercase();
                m.content.to_lowercase().contains(&q) || m.importance > 0.8
            })
            .take(3)
            .collect();

        let context = if relevant.is_empty() {
            String::new()
        } else {
            relevant
                .iter()
                .map(|m| m.content.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        };

        let thought = if context.is_empty() {
            format!("Gondolkodom: {}...", req.input)
        } else {
            format!(
                "Gondolkodom: {}... Emlékszem: {}",
                req.input,
                &context[..context.len().min(100)]
            )
        };

        let emotions = self.emotions.read().await;

        Ok(Response::new(ThinkResponse {
            thought,
            reasoning_steps: vec![
                "1. Bemenet feldolgozása".to_string(),
                "2. Memória kontextus keresése".to_string(),
                "3. Következtetés".to_string(),
            ],
            confidence: 0.85,
            emotions: emotions.clone(),
        }))
    }

    /// Feel - Érzelmek beállítása
    async fn feel(&self, request: Request<FeelRequest>) -> Result<Response<FeelResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Feel kérés: {:?}", req.emotions);

        // Érzelmek frissítése
        let mut emotions = self.emotions.write().await;
        for (emotion, value) in &req.emotions {
            emotions.insert(emotion.clone(), *value);
        }

        // Domináns érzelem meghatározása
        let dominant = emotions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "neutral".to_string());

        let intensity = emotions
            .values()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.5);

        Ok(Response::new(FeelResponse {
            success: true,
            dominant_emotion: dominant,
            intensity,
        }))
    }

    /// GetCognitiveState - Kognitív állapot
    async fn get_cognitive_state(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<CognitiveStateResponse>, Status> {
        let emotions = self.emotions.read().await;
        let memories = self.memories.read().await;

        let dominant = emotions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "neutral".to_string());

        Ok(Response::new(CognitiveStateResponse {
            current_focus: format!("gRPC szerver ({} emlék)", memories.len()),
            emotions: emotions.clone(),
            energy: 0.9,
            clarity: 0.85,
            active_thoughts: vec![
                "Szerver fut".to_string(),
                format!("{} emlék a memóriában", memories.len()),
                format!("Domináns érzelem: {}", dominant),
            ],
            mood: "engaged".to_string(),
        }))
    }

    /// Resonate - Rezonancia
    async fn resonate(
        &self,
        _request: Request<super::proto::ResonateRequest>,
    ) -> Result<Response<super::proto::ResonateResponse>, Status> {
        Err(Status::unimplemented("Resonate még nincs implementálva"))
    }

    /// StreamThoughts - Gondolat stream
    type StreamThoughtsStream =
        tokio_stream::wrappers::ReceiverStream<Result<super::proto::ThoughtChunk, Status>>;

    async fn stream_thoughts(
        &self,
        _request: Request<ThinkRequest>,
    ) -> Result<Response<Self::StreamThoughtsStream>, Status> {
        Err(Status::unimplemented(
            "StreamThoughts még nincs implementálva",
        ))
    }

    /// SetFocus - Fókusz beállítása kulcsszavakkal
    async fn set_focus(
        &self,
        request: Request<SetFocusRequest>,
    ) -> Result<Response<SetFocusResponse>, Status> {
        let req = request.into_inner();
        println!(
            "🎯 HOPE: SetFocus kérés: {:?} (weight: {}, duration: {}s)",
            req.keywords, req.weight, req.duration_secs
        );

        let duration = if req.duration_secs > 0 {
            Some(req.duration_secs)
        } else {
            None
        };

        self.attention
            .set_focus(&req.keywords, req.weight, duration)
            .await;

        let targets = self.attention.active_targets().await;

        Ok(Response::new(SetFocusResponse {
            success: true,
            active_targets: targets.len() as i32,
            message: format!("Fókusz beállítva: {} kulcsszó", req.keywords.len()),
        }))
    }

    /// ClearFocus - Fókusz törlése
    async fn clear_focus(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ClearFocusResponse>, Status> {
        println!("🎯 HOPE: ClearFocus kérés");

        let targets_before = self.attention.active_targets().await.len();
        self.attention.clear_focus().await;

        Ok(Response::new(ClearFocusResponse {
            success: true,
            cleared: targets_before as i32,
        }))
    }

    /// GetAttentionState - Attention állapot lekérdezése
    async fn get_attention_state(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<AttentionStateResponse>, Status> {
        println!("🎯 HOPE: GetAttentionState kérés");

        let state = self.attention.state().await;

        let targets: Vec<FocusTargetInfo> = state
            .focus_targets
            .iter()
            .filter(|t| !t.is_expired())
            .map(|t| {
                let expires_in = if let Some(expires) = t.expires_at {
                    let now = chrono::Utc::now();
                    if expires > now {
                        (expires - now).num_seconds()
                    } else {
                        0
                    }
                } else {
                    -1 // Nincs lejárat
                };

                FocusTargetInfo {
                    keyword: t.keyword.clone(),
                    weight: t.weight,
                    expires_in_secs: expires_in,
                }
            })
            .collect();

        let mode_str = match state.mode {
            AttentionMode::Focused => "Focused",
            AttentionMode::Normal => "Normal",
            AttentionMode::Diffuse => "Diffuse",
        };

        Ok(Response::new(AttentionStateResponse {
            targets,
            attention_capacity: state.attention_capacity,
            mode: mode_str.to_string(),
            context_weights: state.context_weights,
        }))
    }
}

// ============================================================================
// VISION SERVICE IMPLEMENTÁCIÓ - Hope "szeme"
// ============================================================================

#[tonic::async_trait]
impl VisionService for HopeGrpcServer {
    /// See - Kép fogadása és feldolgozása
    async fn see(&self, request: Request<SeeRequest>) -> Result<Response<SeeResponse>, Status> {
        let req = request.into_inner();
        println!("👁️ HOPE LÁT: {} bytes érkezett", req.image_data.len());

        if req.image_data.is_empty() {
            return Ok(Response::new(SeeResponse {
                success: false,
                id: String::new(),
                analysis: None,
                error: "Üres kép adat".to_string(),
            }));
        }

        let mut vision = self.vision.write().await;

        // Kép feldolgozása
        let (image_id, result_ok) = if !req.description.is_empty() {
            match vision.receive_with_description(&req.image_data, &req.description, req.importance)
            {
                Ok(id) => (id, true),
                Err(e) => {
                    println!("  ❌ Hiba: {}", e);
                    return Ok(Response::new(SeeResponse {
                        success: false,
                        id: String::new(),
                        analysis: None,
                        error: e.to_string(),
                    }));
                }
            }
        } else {
            match vision.receive(&req.image_data) {
                Ok(id) => (id, true),
                Err(e) => {
                    println!("  ❌ Hiba: {}", e);
                    return Ok(Response::new(SeeResponse {
                        success: false,
                        id: String::new(),
                        analysis: None,
                        error: e.to_string(),
                    }));
                }
            }
        };

        if !result_ok {
            return Ok(Response::new(SeeResponse {
                success: false,
                id: String::new(),
                analysis: None,
                error: "Ismeretlen hiba".to_string(),
            }));
        }

        // Metaadatok lekérése
        let input = vision.get(&image_id);

        let analysis = input.map(|v| ImageAnalysis {
            format: v.metadata.format.to_string(),
            width: v.metadata.size.width as i32,
            height: v.metadata.size.height as i32,
            file_size: v.metadata.file_size as i64,
            hash: v.metadata.hash.clone(),
            aspect_ratio: v.metadata.size.aspect_ratio(),
            megapixels: v.metadata.size.megapixels(),
            detected_features: vec![],
            metadata: HashMap::new(),
        });

        println!(
            "  ✅ Kép feldolgozva: {} ({})",
            &image_id,
            analysis
                .as_ref()
                .map(|a| a.format.clone())
                .unwrap_or_default()
        );

        Ok(Response::new(SeeResponse {
            success: true,
            id: image_id,
            analysis,
            error: String::new(),
        }))
    }

    /// SeeStream - Streaming kép fogadás
    async fn see_stream(
        &self,
        _request: Request<tonic::Streaming<super::proto::ImageChunk>>,
    ) -> Result<Response<SeeResponse>, Status> {
        // Egyszerűsített implementáció - később lehet bővíteni
        Err(Status::unimplemented("SeeStream még nincs implementálva"))
    }

    /// GetVisualMemories - Vizuális emlékek lekérdezése
    async fn get_visual_memories(
        &self,
        request: Request<GetVisualMemoriesRequest>,
    ) -> Result<Response<VisualMemoriesResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit > 0 {
            req.limit as usize
        } else {
            10
        };

        let vision = self.vision.read().await;

        let inputs: Vec<_> = if req.recent_only {
            vision.recent(limit)
        } else if req.min_importance > 0.0 {
            vision.important(req.min_importance)
        } else {
            vision.all_inputs().into_iter().take(limit).collect()
        };

        let memories: Vec<VisualMemoryInfo> = inputs
            .iter()
            .map(|v| {
                let received_secs = v.received_at.timestamp();
                VisualMemoryInfo {
                    id: v.id.clone(),
                    format: v.metadata.format.to_string(),
                    width: v.metadata.size.width as i32,
                    height: v.metadata.size.height as i32,
                    description: v.description.clone().unwrap_or_default(),
                    importance: v.importance,
                    processed: v.processed,
                    received_at: Some(Timestamp {
                        seconds: received_secs,
                        nanos: 0,
                    }),
                    hash: v.metadata.hash.clone(),
                    related_blocks: v.related_blocks.clone(),
                }
            })
            .collect();

        let total = memories.len() as i32;

        Ok(Response::new(VisualMemoriesResponse { memories, total }))
    }

    /// GetVisionStatus - Vision státusz lekérdezése
    async fn get_vision_status(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<VisionStatusResponse>, Status> {
        let vision = self.vision.read().await;
        let stats = vision.stats();

        let format_counts: HashMap<String, i64> = stats
            .format_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v as i64))
            .collect();

        Ok(Response::new(VisionStatusResponse {
            total_received: stats.total_received as i64,
            total_processed: stats.total_processed as i64,
            stored_count: vision.all_inputs().len() as i32,
            total_bytes: stats.total_bytes as i64,
            avg_megapixels: stats.avg_megapixels,
            format_counts,
            graph_connected: true,
        }))
    }

    /// Compare - Két kép összehasonlítása
    async fn compare(
        &self,
        request: Request<CompareImagesRequest>,
    ) -> Result<Response<CompareImagesResponse>, Status> {
        let req = request.into_inner();

        let vision = self.vision.read().await;

        let img1 = vision.get(&req.image_id_1);
        let img2 = vision.get(&req.image_id_2);

        match (img1, img2) {
            (Some(v1), Some(v2)) => {
                // Hash alapú hasonlóság
                let hash_sim = if v1.metadata.hash == v2.metadata.hash {
                    1.0
                } else {
                    0.0
                };

                // Méret hasonlóság
                let size1 = v1.metadata.size.width * v1.metadata.size.height;
                let size2 = v2.metadata.size.width * v2.metadata.size.height;
                let size_sim = if size1 > 0 && size2 > 0 {
                    let ratio = size1.min(size2) as f64 / size1.max(size2) as f64;
                    ratio
                } else {
                    0.0
                };

                Ok(Response::new(CompareImagesResponse {
                    success: true,
                    hash_similarity: hash_sim,
                    size_similarity: size_sim,
                    comparison_notes: format!(
                        "{}x{} vs {}x{}",
                        v1.metadata.size.width,
                        v1.metadata.size.height,
                        v2.metadata.size.width,
                        v2.metadata.size.height
                    ),
                }))
            }
            _ => Ok(Response::new(CompareImagesResponse {
                success: false,
                hash_similarity: 0.0,
                size_similarity: 0.0,
                comparison_notes: "Egy vagy mindkét kép nem található".to_string(),
            })),
        }
    }
}

// ============================================================================
// SKILL SERVICE IMPLEMENTÁCIÓ - 97 skill
// ============================================================================

#[tonic::async_trait]
impl SkillService for HopeGrpcServer {
    /// ListSkills - Skillek listázása
    async fn list_skills(
        &self,
        request: Request<ListSkillsRequest>,
    ) -> Result<Response<ListSkillsResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: ListSkills kérés (category: {})", req.category);

        let skills = self.skills.read().await;

        // Parse category ha van
        let category_filter = if req.category.is_empty() {
            None
        } else {
            // Try to parse category string
            match req.category.to_lowercase().as_str() {
                "core" => Some(crate::modules::skills::SkillCategory::Core),
                "cognitive" => Some(crate::modules::skills::SkillCategory::Cognitive),
                "memory" => Some(crate::modules::skills::SkillCategory::Memory),
                "code" => Some(crate::modules::skills::SkillCategory::Code),
                "system" => Some(crate::modules::skills::SkillCategory::System),
                "web" => Some(crate::modules::skills::SkillCategory::Web),
                "media" => Some(crate::modules::skills::SkillCategory::Media),
                "file" => Some(crate::modules::skills::SkillCategory::File),
                "git" => Some(crate::modules::skills::SkillCategory::Git),
                "communication" => Some(crate::modules::skills::SkillCategory::Communication),
                _ => None,
            }
        };

        let search_filter = if req.search.is_empty() {
            None
        } else {
            Some(req.search.as_str())
        };

        let all_skills = skills.list(category_filter, search_filter).await;

        let filtered: Vec<SkillInfo> = all_skills
            .into_iter()
            .skip(req.offset as usize)
            .take(if req.limit > 0 {
                req.limit as usize
            } else {
                100
            })
            .map(|s| SkillInfo {
                name: s.name.clone(),
                description: s.description.clone(),
                category: s.category.to_string(),
                tags: s.params.iter().map(|p| p.name.clone()).collect(), // Params -> tags
                version: s.version.clone(),
                enabled: s.enabled,
            })
            .collect();

        let total = filtered.len() as i32;

        Ok(Response::new(ListSkillsResponse {
            skills: filtered,
            total,
        }))
    }

    /// GetSkill - Egy skill lekérdezése
    async fn get_skill(
        &self,
        request: Request<GetSkillRequest>,
    ) -> Result<Response<SkillInfo>, Status> {
        let req = request.into_inner();
        let skills = self.skills.read().await;

        match skills.get(&req.name).await {
            Some(s) => Ok(Response::new(SkillInfo {
                name: s.name.clone(),
                description: s.description.clone(),
                category: s.category.to_string(),
                tags: s.params.iter().map(|p| p.name.clone()).collect(),
                version: s.version.clone(),
                enabled: s.enabled,
            })),
            None => Err(Status::not_found(format!(
                "Skill nem található: {}",
                req.name
            ))),
        }
    }

    /// InvokeSkill - Skill meghívása
    async fn invoke_skill(
        &self,
        request: Request<InvokeSkillRequest>,
    ) -> Result<Response<InvokeSkillResponse>, Status> {
        let req = request.into_inner();
        let start = Instant::now();
        println!(
            "HOPE: InvokeSkill kérés: {} (input: {})",
            req.name, req.input
        );

        let skills = self.skills.read().await;

        // Convert String params to serde_json::Value
        let params: HashMap<String, serde_json::Value> = req
            .params
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        match skills.invoke(&req.name, &req.input, &params).await {
            Ok(result) => Ok(Response::new(InvokeSkillResponse {
                success: result.success,
                output: if result.success {
                    result.output.clone()
                } else {
                    String::new()
                },
                error: if result.success {
                    String::new()
                } else {
                    result.output
                },
                execution_time: result.execution_time_ms / 1000.0, // ms -> sec
            })),
            Err(e) => Ok(Response::new(InvokeSkillResponse {
                success: false,
                output: String::new(),
                error: e.to_string(),
                execution_time: start.elapsed().as_secs_f64(),
            })),
        }
    }

    /// StreamSkillOutput - Streaming skill output
    type StreamSkillOutputStream =
        tokio_stream::wrappers::ReceiverStream<Result<SkillOutput, Status>>;

    async fn stream_skill_output(
        &self,
        _request: Request<InvokeSkillRequest>,
    ) -> Result<Response<Self::StreamSkillOutputStream>, Status> {
        Err(Status::unimplemented(
            "StreamSkillOutput még nincs implementálva",
        ))
    }
}

// ============================================================================
// GENOME SERVICE IMPLEMENTÁCIÓ - AI Etika (7 alapelv)
// ============================================================================

#[tonic::async_trait]
impl GenomeService for HopeGrpcServer {
    /// GetStatus - Genom állapot lekérdezése
    async fn get_status(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<GenomeStatusResponse>, Status> {
        let genome = self.genome.read().await;
        let stats = &genome.stats;

        Ok(Response::new(GenomeStatusResponse {
            enabled: true,
            sealed: true, // Az alapelvek megváltoztathatatlanok
            rules_count: genome.get_core_values().len() as i32,
            violations: stats.denied as i32,
            max_violations: 100,
            total_actions: stats.evaluations as i32,
            approved_actions: stats.permitted as i32,
            denied_actions: stats.denied as i32,
            integrity_hash: format!(
                "{:x}",
                md5::compute(format!("{:?}", genome.get_core_values()).as_bytes())
            ),
        }))
    }

    /// VerifyAction - Akció ellenőrzése etikai szempontból
    async fn verify_action(
        &self,
        request: Request<VerifyActionRequest>,
    ) -> Result<Response<VerifyActionResponse>, Status> {
        let req = request.into_inner();
        println!(
            "HOPE: VerifyAction kérés: {} - {}",
            req.action_type, req.description
        );

        let mut genome = self.genome.write().await;

        // EvaluationContext létrehozása
        let context = crate::modules::genome::EvaluationContext::new()
            .with_intent(&req.description)
            .with_target(&req.action_type);

        // A teljes akció string
        let action = format!("{}: {}", req.action_type, req.description);

        let result = genome.evaluate(&action, &context);

        Ok(Response::new(VerifyActionResponse {
            allowed: result.permitted,
            reason: if result.concerns.is_empty() {
                "Akció engedélyezve".to_string()
            } else {
                result.concerns.join("; ")
            },
            violated_rules: result.concerns.clone(),
            decision_id: uuid::Uuid::new_v4().to_string(),
        }))
    }

    /// SignDecision - Döntés aláírása
    async fn sign_decision(
        &self,
        request: Request<SignDecisionRequest>,
    ) -> Result<Response<SignDecisionResponse>, Status> {
        let req = request.into_inner();

        // Egyszerű aláírás generálás (hash)
        let signature_input = format!("{}:{}:{}", req.decision_id, req.action_type, req.outcome);
        let signature = format!("HOPE-SIG-{:x}", md5::compute(signature_input.as_bytes()));

        Ok(Response::new(SignDecisionResponse {
            signature,
            success: true,
            message: "Döntés aláírva".to_string(),
        }))
    }

    /// GetAuditTrail - Audit napló lekérdezése
    async fn get_audit_trail(
        &self,
        request: Request<GetAuditTrailRequest>,
    ) -> Result<Response<AuditTrailResponse>, Status> {
        let req = request.into_inner();
        let genome = self.genome.read().await;

        // Audit események lekérése (history)
        let entries: Vec<AuditEntry> = genome
            .get_history()
            .iter()
            .filter(|e| req.action_type.is_empty() || e.action.contains(&req.action_type))
            .take(if req.limit > 0 {
                req.limit as usize
            } else {
                50
            })
            .map(|e| AuditEntry {
                id: format!("{:.0}", e.timestamp),
                action_type: e.action.clone(),
                description: e.recommendations.join("; "),
                allowed: e.permitted,
                reason: e.concerns.join("; "),
                signature: format!("HOPE-{:x}", md5::compute(e.action.as_bytes())),
                timestamp: Some(Timestamp {
                    seconds: e.timestamp as i64,
                    nanos: 0,
                }),
            })
            .collect();

        let total = entries.len() as i32;

        Ok(Response::new(AuditTrailResponse { entries, total }))
    }

    /// GetRules - Etikai szabályok lekérdezése
    async fn get_rules(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<RulesResponse>, Status> {
        let genome = self.genome.read().await;

        // Core values mint szabályok
        let rules: Vec<EthicalRule> = genome
            .get_core_values()
            .iter()
            .enumerate()
            .map(|(i, value)| EthicalRule {
                id: format!("CORE_VALUE_{}", i + 1),
                name: value.clone(),
                description: format!("Core value: {}", value),
                category: "core".to_string(),
                priority: (i + 1) as i32,
                immutable: true,
            })
            .collect();

        Ok(Response::new(RulesResponse { rules }))
    }
}

// ============================================================================
// CODE SERVICE IMPLEMENTÁCIÓ - Kód elemzés és generálás
// ============================================================================

#[tonic::async_trait]
impl CodeService for HopeGrpcServer {
    /// Analyze - Kód elemzés
    async fn analyze(
        &self,
        request: Request<AnalyzeRequest>,
    ) -> Result<Response<AnalyzeResponse>, Status> {
        let req = request.into_inner();
        println!(
            "HOPE: Analyze kérés ({} bytes, {})",
            req.code.len(),
            req.language
        );

        let mut issues = Vec::new();
        let code = &req.code;
        let checks = if req.checks.is_empty() {
            vec!["syntax".to_string()]
        } else {
            req.checks
        };

        // Syntax ellenőrzés
        if checks.contains(&"syntax".to_string()) {
            // Alapvető syntax problémák keresése
            let lines: Vec<&str> = code.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                // Nyitó zárójelek ellenőrzése
                let open_parens = line.matches('(').count();
                let close_parens = line.matches(')').count();
                if open_parens != close_parens {
                    issues.push(CodeIssue {
                        severity: "warning".to_string(),
                        message: "Páratlan zárójelek".to_string(),
                        line: (line_num + 1) as i32,
                        column: 1,
                        rule: "syntax/parens".to_string(),
                    });
                }

                // Üres return ellenőrzése
                if line.contains("return;") && req.language == "rust" {
                    issues.push(CodeIssue {
                        severity: "info".to_string(),
                        message: "Üres return Rust-ban opcionális".to_string(),
                        line: (line_num + 1) as i32,
                        column: 1,
                        rule: "style/return".to_string(),
                    });
                }
            }
        }

        // Security ellenőrzés
        if checks.contains(&"security".to_string()) {
            // Veszélyes minták keresése
            let dangerous_patterns = vec![
                ("unsafe", "unsafe blokk használata"),
                ("eval(", "eval() használata veszélyes"),
                ("exec(", "exec() használata veszélyes"),
                ("system(", "system() hívás veszélyes"),
                ("password", "hardcoded jelszó?"),
                ("secret", "hardcoded titok?"),
            ];

            for (pattern, message) in dangerous_patterns {
                if code.to_lowercase().contains(pattern) {
                    for (line_num, line) in code.lines().enumerate() {
                        if line.to_lowercase().contains(pattern) {
                            issues.push(CodeIssue {
                                severity: "warning".to_string(),
                                message: message.to_string(),
                                line: (line_num + 1) as i32,
                                column: 1,
                                rule: format!("security/{}", pattern),
                            });
                        }
                    }
                }
            }
        }

        // Metrikák számítása
        let lines = code.lines().count();
        let functions = code.matches("fn ").count()
            + code.matches("function ").count()
            + code.matches("def ").count();
        let classes = code.matches("struct ").count() + code.matches("class ").count();

        let metrics = CodeMetrics {
            lines: lines as i32,
            complexity: (lines / 10).max(1) as i32, // Egyszerű becslés
            functions: functions as i32,
            classes: classes as i32,
            maintainability: if issues.is_empty() { 0.9 } else { 0.7 },
        };

        let valid = !issues.iter().any(|i| i.severity == "error");

        Ok(Response::new(AnalyzeResponse {
            valid,
            issues,
            metrics: Some(metrics),
            suggestions: vec!["Rendszeres refaktorálás ajánlott".to_string()],
        }))
    }

    /// Generate - Kód generálás
    async fn generate(
        &self,
        request: Request<GenerateRequest>,
    ) -> Result<Response<GenerateResponse>, Status> {
        let req = request.into_inner();
        println!(
            "HOPE: Generate kérés: {} ({})",
            req.description, req.language
        );

        // Egyszerű sablon alapú generálás
        let code = match req.language.to_lowercase().as_str() {
            "rust" => {
                if req.description.to_lowercase().contains("hello") {
                    r#"fn main() {
    println!("Hello, World!");
}"#
                    .to_string()
                } else if req.description.to_lowercase().contains("struct") {
                    format!(
                        r#"#[derive(Debug, Clone)]
pub struct {} {{
    // TODO: Add fields
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}"#,
                        "MyStruct", "MyStruct"
                    )
                } else {
                    format!(
                        "// Generated for: {}\nfn main() {{\n    // TODO: Implement\n}}",
                        req.description
                    )
                }
            }
            "python" => {
                if req.description.to_lowercase().contains("hello") {
                    r#"def main():
    print("Hello, World!")

if __name__ == "__main__":
    main()"#
                        .to_string()
                } else {
                    format!(
                        "# Generated for: {}\ndef main():\n    # TODO: Implement\n    pass",
                        req.description
                    )
                }
            }
            "javascript" | "js" | "typescript" | "ts" => {
                if req.description.to_lowercase().contains("hello") {
                    r#"function main() {
    console.log("Hello, World!");
}

main();"#
                        .to_string()
                } else {
                    format!(
                        "// Generated for: {}\nfunction main() {{\n    // TODO: Implement\n}}",
                        req.description
                    )
                }
            }
            _ => format!(
                "// Generated for: {} ({})\n// TODO: Implement",
                req.description, req.language
            ),
        };

        Ok(Response::new(GenerateResponse {
            code,
            explanation: format!("Kód generálva: {}", req.description),
            dependencies: vec![],
        }))
    }

    /// ListTemplates - Sablonok listázása
    async fn list_templates(
        &self,
        request: Request<ListTemplatesRequest>,
    ) -> Result<Response<ListTemplatesResponse>, Status> {
        let req = request.into_inner();

        let mut templates = vec![
            TemplateInfo {
                name: "hello_world".to_string(),
                description: "Hello World alap példa".to_string(),
                language: "rust".to_string(),
                category: "basic".to_string(),
                params: vec![],
            },
            TemplateInfo {
                name: "struct".to_string(),
                description: "Struct definíció".to_string(),
                language: "rust".to_string(),
                category: "types".to_string(),
                params: vec!["name".to_string()],
            },
            TemplateInfo {
                name: "grpc_service".to_string(),
                description: "gRPC szolgáltatás".to_string(),
                language: "rust".to_string(),
                category: "networking".to_string(),
                params: vec!["service_name".to_string()],
            },
            TemplateInfo {
                name: "async_function".to_string(),
                description: "Async függvény".to_string(),
                language: "rust".to_string(),
                category: "async".to_string(),
                params: vec!["name".to_string()],
            },
        ];

        // Szűrés
        if !req.language.is_empty() {
            templates.retain(|t| t.language.to_lowercase() == req.language.to_lowercase());
        }
        if !req.category.is_empty() {
            templates.retain(|t| t.category.to_lowercase() == req.category.to_lowercase());
        }

        Ok(Response::new(ListTemplatesResponse { templates }))
    }

    /// GetCodeBlock - Kód blokk lekérdezése
    async fn get_code_block(
        &self,
        request: Request<GetCodeBlockRequest>,
    ) -> Result<Response<CodeBlock>, Status> {
        let req = request.into_inner();
        let blocks = self.code_blocks.read().await;

        match blocks.iter().find(|b| b.id == req.id) {
            Some(b) => {
                let duration = b
                    .created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();

                Ok(Response::new(CodeBlock {
                    id: b.id.clone(),
                    code: b.code.clone(),
                    language: b.language.clone(),
                    description: b.description.clone(),
                    created_at: Some(Timestamp {
                        seconds: duration.as_secs() as i64,
                        nanos: duration.subsec_nanos() as i32,
                    }),
                }))
            }
            None => Err(Status::not_found(format!(
                "CodeBlock nem található: {}",
                req.id
            ))),
        }
    }

    /// StoreCodeBlock - Kód blokk mentése
    async fn store_code_block(
        &self,
        request: Request<StoreCodeBlockRequest>,
    ) -> Result<Response<StoreCodeBlockResponse>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::new_v4().to_string();

        let block = InternalCodeBlock {
            id: id.clone(),
            code: req.code,
            language: req.language,
            description: req.description,
            created_at: std::time::SystemTime::now(),
            tags: req.tags,
        };

        let mut blocks = self.code_blocks.write().await;
        blocks.push(block);

        println!("HOPE: CodeBlock mentve: {} ({} blocks)", id, blocks.len());

        Ok(Response::new(StoreCodeBlockResponse { id, success: true }))
    }
}

// ============================================================================
// RESONANCE SERVICE IMPLEMENTÁCIÓ
// ============================================================================

#[tonic::async_trait]
impl ResonanceService for HopeGrpcServer {
    /// Learn - Tanulás felhasználói bemenetből
    async fn learn(
        &self,
        request: Request<ResonanceLearnRequest>,
    ) -> Result<Response<ResonanceLearnResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Resonance Learn kérés (session: {})", req.session_id);

        let session_id = if req.session_id.is_empty() {
            uuid::Uuid::new_v4()
        } else {
            req.session_id
                .parse()
                .unwrap_or_else(|_| uuid::Uuid::new_v4())
        };

        let mut input = UserInput::with_session(req.content, session_id);

        // Keystroke timings
        if !req.keystroke_timings.is_empty() {
            input.keystroke_timings = Some(req.keystroke_timings);
        }

        // Emotional state (21D)
        if req.emotional_state.len() == 21 {
            let mut arr = [0.0f64; 21];
            for (i, v) in req.emotional_state.iter().enumerate() {
                arr[i] = *v;
            }
            input.emotional_state = Some(arr);
        }

        match self.resonance.learn(&input).await {
            Ok(confidence) => {
                let status = self.resonance.status().await;
                Ok(Response::new(ResonanceLearnResponse {
                    success: true,
                    confidence,
                    sample_count: status.total_samples as i64,
                }))
            }
            Err(e) => Err(Status::internal(format!("Resonance learn hiba: {}", e))),
        }
    }

    /// Verify - Felhasználó verifikáció
    async fn verify(
        &self,
        request: Request<ResonanceVerifyRequest>,
    ) -> Result<Response<ResonanceVerifyResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Resonance Verify kérés (session: {})", req.session_id);

        let session_id = if req.session_id.is_empty() {
            uuid::Uuid::new_v4()
        } else {
            req.session_id
                .parse()
                .unwrap_or_else(|_| uuid::Uuid::new_v4())
        };

        // Session építése a bemenetekből
        let mut session = SessionData::new();
        session.session_id = session_id;

        for input in req.inputs {
            let mut user_input = UserInput::with_session(input.content, session_id);

            if !input.keystroke_timings.is_empty() {
                user_input.keystroke_timings = Some(input.keystroke_timings);
            }

            if input.emotional_state.len() == 21 {
                let mut arr = [0.0f64; 21];
                for (i, v) in input.emotional_state.iter().enumerate() {
                    arr[i] = *v;
                }
                user_input.emotional_state = Some(arr);
            }

            session.add_input(user_input);
        }

        let result = self.resonance.verify(&session).await;

        Ok(Response::new(ResonanceVerifyResponse {
            is_authentic: result.is_authentic,
            confidence: result.confidence,
            user_id: result.user_id.map(|u| u.to_string()).unwrap_or_default(),
            user_name: result.user_name.unwrap_or_default(),
            matched_patterns: result
                .matched_patterns
                .iter()
                .map(|p| format!("{:?}", p))
                .collect(),
            is_new_user: result.is_new_user,
            altered_state: result.altered_state,
            potential_attack: result.potential_attack,
        }))
    }

    /// GetStatus - Resonance státusz
    async fn get_status(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<ResonanceStatusResponse>, Status> {
        let status = self.resonance.status().await;

        Ok(Response::new(ResonanceStatusResponse {
            profile_count: status.profile_count as i32,
            total_samples: status.total_samples as i64,
            avg_confidence: status.avg_confidence,
            current_session_messages: status.current_session_messages as i32,
            current_session_duration_secs: status.current_session_duration_secs,
            match_threshold: status.match_threshold,
        }))
    }

    /// RegisterUser - Profil regisztrálása
    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> Result<Response<RegisterUserResponse>, Status> {
        let req = request.into_inner();
        println!("HOPE: Resonance RegisterUser: {}", req.user_name);

        let user_id = if req.user_id.is_empty() {
            uuid::Uuid::new_v4()
        } else {
            req.user_id.parse().unwrap_or_else(|_| uuid::Uuid::new_v4())
        };

        let user_name = if req.user_name.is_empty() {
            None
        } else {
            Some(req.user_name)
        };

        let profile_id = self.resonance.register_user(user_id, user_name).await;

        Ok(Response::new(RegisterUserResponse {
            profile_id: profile_id.to_string(),
            success: true,
        }))
    }

    /// DetectAnomaly - Anomália detekció
    async fn detect_anomaly(
        &self,
        request: Request<DetectAnomalyRequest>,
    ) -> Result<Response<DetectAnomalyResponse>, Status> {
        let req = request.into_inner();

        let session_id = if req.session_id.is_empty() {
            uuid::Uuid::new_v4()
        } else {
            req.session_id
                .parse()
                .unwrap_or_else(|_| uuid::Uuid::new_v4())
        };

        let input = UserInput::with_session(req.content, session_id);
        let anomaly = self.resonance.detect_anomaly(&input).await;

        match anomaly {
            Some(a) => Ok(Response::new(DetectAnomalyResponse {
                has_anomaly: true,
                pattern_type: format!("{:?}", a.pattern_type),
                deviation: a.deviation,
                description: a.description,
            })),
            None => Ok(Response::new(DetectAnomalyResponse {
                has_anomaly: false,
                pattern_type: String::new(),
                deviation: 0.0,
                description: "Nincs anomália".to_string(),
            })),
        }
    }
}

// ============================================================================
// GEO SERVICE IMPLEMENTÁCIÓ - Térbeli kontextus
// ============================================================================

#[tonic::async_trait]
impl GeoService for HopeGrpcServer {
    /// SetLocation - Jelenlegi lokáció beállítása
    async fn set_location(
        &self,
        request: Request<SetLocationRequest>,
    ) -> Result<Response<SetLocationResponse>, Status> {
        let req = request.into_inner();
        println!(
            "🌍 HOPE: SetLocation kérés: {}, {}",
            req.latitude, req.longitude
        );

        let source = match req.source.to_lowercase().as_str() {
            "gps" => GeoSource::Gps,
            "ip" => GeoSource::IpBased,
            "manual" => GeoSource::Manual,
            "network" => GeoSource::Network,
            _ => GeoSource::Unknown,
        };

        let location = GeoLocation {
            latitude: req.latitude,
            longitude: req.longitude,
            altitude: if req.altitude > 0.0 {
                Some(req.altitude)
            } else {
                None
            },
            accuracy: if req.accuracy > 0.0 {
                Some(req.accuracy)
            } else {
                None
            },
            timestamp: chrono::Utc::now(),
            source,
        };

        match self.geo.set_current_location(location.clone()).await {
            Ok(()) => {
                // Távolság az otthontól
                let distance_from_home =
                    self.geo.distance_from_home(&location).await.unwrap_or(0.0);

                // Detektált hely neve
                let places = self.geo.list_places().await;
                let detected_place = places
                    .iter()
                    .find(|p| GeoEngine::distance(&location, &p.location) * 1000.0 <= p.radius)
                    .map(|p| p.name.clone())
                    .unwrap_or_default();

                Ok(Response::new(SetLocationResponse {
                    success: true,
                    detected_place,
                    distance_from_home,
                }))
            }
            Err(e) => Ok(Response::new(SetLocationResponse {
                success: false,
                detected_place: e,
                distance_from_home: 0.0,
            })),
        }
    }

    /// GetLocation - Jelenlegi lokáció lekérdezése
    async fn get_location(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<GeoLocationResponse>, Status> {
        match self.geo.get_current_location().await {
            Some(loc) => {
                let places = self.geo.list_places().await;
                let current_place = places
                    .iter()
                    .find(|p| GeoEngine::distance(&loc, &p.location) * 1000.0 <= p.radius)
                    .map(|p| p.name.clone())
                    .unwrap_or_default();

                let source_str = match loc.source {
                    GeoSource::Gps => "gps",
                    GeoSource::IpBased => "ip",
                    GeoSource::Manual => "manual",
                    GeoSource::Network => "network",
                    GeoSource::Inferred => "inferred",
                    GeoSource::Unknown => "unknown",
                };

                Ok(Response::new(GeoLocationResponse {
                    latitude: loc.latitude,
                    longitude: loc.longitude,
                    altitude: loc.altitude.unwrap_or(0.0),
                    accuracy: loc.accuracy.unwrap_or(0.0),
                    source: source_str.to_string(),
                    timestamp: Some(Timestamp {
                        seconds: loc.timestamp.timestamp(),
                        nanos: 0,
                    }),
                    current_place,
                }))
            }
            None => Ok(Response::new(GeoLocationResponse {
                latitude: 0.0,
                longitude: 0.0,
                altitude: 0.0,
                accuracy: 0.0,
                source: "unknown".to_string(),
                timestamp: None,
                current_place: String::new(),
            })),
        }
    }

    /// AddPlace - Hely hozzáadása
    async fn add_place(
        &self,
        request: Request<AddPlaceRequest>,
    ) -> Result<Response<AddPlaceResponse>, Status> {
        let req = request.into_inner();
        println!("🌍 HOPE: AddPlace kérés: {}", req.name);

        let place = Place {
            id: uuid::Uuid::nil(), // Will be generated
            name: req.name,
            place_type: PlaceType::from_str(&req.place_type),
            location: GeoLocation {
                latitude: req.latitude,
                longitude: req.longitude,
                altitude: None,
                accuracy: None,
                timestamp: chrono::Utc::now(),
                source: GeoSource::Manual,
            },
            radius: req.radius,
            address: if req.address.is_empty() {
                None
            } else {
                Some(req.address)
            },
            country_code: if req.country_code.is_empty() {
                None
            } else {
                Some(req.country_code)
            },
            visit_count: 0,
            last_visit: None,
            first_visit: chrono::Utc::now(),
            memory_count: 0,
            emotional_associations: HashMap::new(),
            metadata: HashMap::new(),
        };

        let place_id = self.geo.add_place(place).await;

        Ok(Response::new(AddPlaceResponse {
            place_id: place_id.to_string(),
            success: true,
        }))
    }

    /// ListPlaces - Helyek listázása
    async fn list_places(
        &self,
        request: Request<ListPlacesRequest>,
    ) -> Result<Response<ListPlacesResponse>, Status> {
        let req = request.into_inner();

        let all_places = self.geo.list_places().await;
        let limit = if req.limit > 0 {
            req.limit as usize
        } else {
            100
        };

        // Szűrés
        let mut filtered: Vec<_> = all_places
            .into_iter()
            .filter(|p| {
                if !req.place_type.is_empty() {
                    p.place_type.to_string() == req.place_type
                } else {
                    true
                }
            })
            .filter(|p| {
                // Közeli helyek szűrése ha van koordináta megadva
                if req.nearby_lat != 0.0 && req.nearby_lon != 0.0 && req.radius_km > 0.0 {
                    let search_loc = GeoLocation {
                        latitude: req.nearby_lat,
                        longitude: req.nearby_lon,
                        altitude: None,
                        accuracy: None,
                        timestamp: chrono::Utc::now(),
                        source: GeoSource::Manual,
                    };
                    GeoEngine::distance(&search_loc, &p.location) <= req.radius_km
                } else {
                    true
                }
            })
            .take(limit)
            .collect();

        // Konvertálás proto típusra
        let places: Vec<PlaceInfo> = filtered
            .iter()
            .map(|p| PlaceInfo {
                id: p.id.to_string(),
                name: p.name.clone(),
                place_type: p.place_type.to_string(),
                latitude: p.location.latitude,
                longitude: p.location.longitude,
                radius: p.radius,
                address: p.address.clone().unwrap_or_default(),
                visit_count: p.visit_count as i64,
                last_visit: p.last_visit.map(|dt| Timestamp {
                    seconds: dt.timestamp(),
                    nanos: 0,
                }),
                memory_count: p.memory_count as i64,
            })
            .collect();

        let total = places.len() as i32;

        Ok(Response::new(ListPlacesResponse { places, total }))
    }

    /// SetHome - Otthon beállítása
    async fn set_home(
        &self,
        request: Request<SetHomeRequest>,
    ) -> Result<Response<SetHomeResponse>, Status> {
        let req = request.into_inner();
        println!("🏠 HOPE: SetHome kérés: {}", req.place_id);

        let place_id = req
            .place_id
            .parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Invalid place_id"))?;

        match self.geo.set_home(place_id).await {
            Ok(()) => Ok(Response::new(SetHomeResponse {
                success: true,
                message: "Otthon beállítva".to_string(),
            })),
            Err(e) => Ok(Response::new(SetHomeResponse {
                success: false,
                message: e,
            })),
        }
    }

    /// GetHome - Otthon lekérdezése
    async fn get_home(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<PlaceResponse>, Status> {
        match self.geo.get_home().await {
            Some(p) => Ok(Response::new(PlaceResponse {
                place: Some(PlaceInfo {
                    id: p.id.to_string(),
                    name: p.name.clone(),
                    place_type: p.place_type.to_string(),
                    latitude: p.location.latitude,
                    longitude: p.location.longitude,
                    radius: p.radius,
                    address: p.address.clone().unwrap_or_default(),
                    visit_count: p.visit_count as i64,
                    last_visit: p.last_visit.map(|dt| Timestamp {
                        seconds: dt.timestamp(),
                        nanos: 0,
                    }),
                    memory_count: p.memory_count as i64,
                }),
                found: true,
            })),
            None => Ok(Response::new(PlaceResponse {
                place: None,
                found: false,
            })),
        }
    }

    /// GetDistance - Távolság számítás
    async fn get_distance(
        &self,
        request: Request<GetDistanceRequest>,
    ) -> Result<Response<GetDistanceResponse>, Status> {
        let req = request.into_inner();

        // Ha place_id-k vannak megadva
        if !req.from_place_id.is_empty() && !req.to_place_id.is_empty() {
            let from_id = req
                .from_place_id
                .parse::<uuid::Uuid>()
                .map_err(|_| Status::invalid_argument("Invalid from_place_id"))?;
            let to_id = req
                .to_place_id
                .parse::<uuid::Uuid>()
                .map_err(|_| Status::invalid_argument("Invalid to_place_id"))?;

            if let Some(distance) = self.geo.distance_between_places(from_id, to_id).await {
                let from_place = self.geo.get_place(from_id).await;
                let to_place = self.geo.get_place(to_id).await;

                return Ok(Response::new(GetDistanceResponse {
                    distance_km: distance,
                    distance_meters: distance * 1000.0,
                    from_name: from_place.map(|p| p.name).unwrap_or_default(),
                    to_name: to_place.map(|p| p.name).unwrap_or_default(),
                }));
            }
        }

        // Ha from_current = true
        if req.from_current {
            if let Some(current) = self.geo.get_current_location().await {
                let to_loc = GeoLocation {
                    latitude: req.lat2,
                    longitude: req.lon2,
                    altitude: None,
                    accuracy: None,
                    timestamp: chrono::Utc::now(),
                    source: GeoSource::Manual,
                };

                let distance = GeoEngine::distance(&current, &to_loc);

                return Ok(Response::new(GetDistanceResponse {
                    distance_km: distance,
                    distance_meters: distance * 1000.0,
                    from_name: "Jelenlegi helyzet".to_string(),
                    to_name: String::new(),
                }));
            }
        }

        // Egyébként koordinátákból számol
        let distance = GeoEngine::haversine_distance(req.lat1, req.lon1, req.lat2, req.lon2);

        Ok(Response::new(GetDistanceResponse {
            distance_km: distance,
            distance_meters: distance * 1000.0,
            from_name: String::new(),
            to_name: String::new(),
        }))
    }

    /// AddGeoMemory - Geo-emlék hozzáadása
    async fn add_geo_memory(
        &self,
        request: Request<AddGeoMemoryRequest>,
    ) -> Result<Response<AddGeoMemoryResponse>, Status> {
        let req = request.into_inner();
        println!("🌍 HOPE: AddGeoMemory kérés: {}", req.memory_id);

        let memory_id = req
            .memory_id
            .parse::<uuid::Uuid>()
            .map_err(|_| Status::invalid_argument("Invalid memory_id"))?;

        let location = GeoLocation {
            latitude: req.latitude,
            longitude: req.longitude,
            altitude: None,
            accuracy: None,
            timestamp: chrono::Utc::now(),
            source: GeoSource::Manual,
        };

        self.geo
            .add_geo_memory(memory_id, location.clone(), req.importance)
            .await;

        // Keressük meg a helyet
        let places = self.geo.list_places().await;
        let place_name = places
            .iter()
            .find(|p| GeoEngine::distance(&location, &p.location) * 1000.0 <= p.radius)
            .map(|p| p.name.clone())
            .unwrap_or_default();

        Ok(Response::new(AddGeoMemoryResponse {
            success: true,
            place_name,
        }))
    }

    /// GetNearbyMemories - Közeli emlékek keresése
    async fn get_nearby_memories(
        &self,
        request: Request<GetNearbyMemoriesRequest>,
    ) -> Result<Response<GeoMemoriesResponse>, Status> {
        let req = request.into_inner();

        let location = GeoLocation {
            latitude: req.latitude,
            longitude: req.longitude,
            altitude: None,
            accuracy: None,
            timestamp: chrono::Utc::now(),
            source: GeoSource::Manual,
        };

        let limit = if req.limit > 0 {
            req.limit as usize
        } else {
            10
        };
        let radius = if req.radius_km > 0.0 {
            req.radius_km
        } else {
            1.0
        };

        let memories = self.geo.get_memories_nearby(&location, radius).await;

        let result: Vec<GeoMemoryInfo> = memories
            .into_iter()
            .take(limit)
            .map(|m| GeoMemoryInfo {
                memory_id: m.memory_id.to_string(),
                latitude: m.location.latitude,
                longitude: m.location.longitude,
                place_name: m.place_name.unwrap_or_default(),
                importance: m.importance,
                created_at: Some(Timestamp {
                    seconds: m.created_at.timestamp(),
                    nanos: 0,
                }),
            })
            .collect();

        Ok(Response::new(GeoMemoriesResponse { memories: result }))
    }

    /// GetGeoStats - Statisztikák
    async fn get_geo_stats(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<GeoStatsResponse>, Status> {
        let stats = self.geo.get_stats().await;

        Ok(Response::new(GeoStatsResponse {
            total_locations: stats.total_locations as i64,
            total_places: stats.total_places as i64,
            total_geo_memories: stats.total_geo_memories as i64,
            home_set: stats.home_set,
            work_set: stats.work_set,
            total_distance_km: stats.total_distance_km,
            last_update: stats.last_update.map(|dt| Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            }),
        }))
    }
}

// ============================================================================
// VOICE SERVICE IMPLEMENTÁCIÓ - TTS/STT Hang Rendszer
// ============================================================================

#[tonic::async_trait]
impl VoiceService for HopeGrpcServer {
    /// Speak - TTS streaming (szöveg → audio chunk-ok)
    type SpeakStream = tokio_stream::wrappers::ReceiverStream<Result<AudioChunk, Status>>;

    async fn speak(
        &self,
        request: Request<SpeakRequest>,
    ) -> Result<Response<Self::SpeakStream>, Status> {
        let req = request.into_inner();
        println!("🎤 HOPE: Speak kérés: {}", req.text);

        let voice = self.voice.read().await;

        // gRPC SpeakRequest -> voice::SpeakRequest konverzió
        let voice_req = crate::modules::voice::SpeakRequest {
            text: req.text.clone(),
            voice: if req.voice.is_empty() {
                "berta".to_string()
            } else {
                req.voice.clone()
            },
            emotion: req.emotion.clone(),
            emotions_21d: Some(req.emotions_21d.clone()),
            prosody: None,
            format: "wav".to_string(),
            sample_rate: 22050,
        };

        // TTS meghívása
        let audio_result = voice.speak(voice_req).await;

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            match audio_result {
                Ok(response) => {
                    // Audio chunk-okra bontás (8KB-os darabok)
                    let chunk_size = 8192;
                    let chunks: Vec<&[u8]> = response.audio.chunks(chunk_size).collect();

                    for (i, chunk) in chunks.iter().enumerate() {
                        let is_final = i == chunks.len() - 1;
                        let audio_chunk = AudioChunk {
                            data: chunk.to_vec(),
                            sequence: i as i32,
                            is_final,
                            format: response.format.clone(),
                            sample_rate: response.sample_rate as i32,
                            timestamp: 0.0,
                            metadata: std::collections::HashMap::new(),
                        };

                        if tx.send(Ok(audio_chunk)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(Status::internal(format!("TTS hiba: {}", e))))
                        .await;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    /// SpeakSync - Egyszerű TTS (teljes audio visszaadás)
    async fn speak_sync(
        &self,
        request: Request<SpeakRequest>,
    ) -> Result<Response<SpeakResponse>, Status> {
        let req = request.into_inner();
        println!("🎤 HOPE: SpeakSync kérés: {}", req.text);

        let voice = self.voice.read().await;

        // gRPC SpeakRequest -> voice::SpeakRequest konverzió
        let voice_req = crate::modules::voice::SpeakRequest {
            text: req.text.clone(),
            voice: if req.voice.is_empty() {
                "berta".to_string()
            } else {
                req.voice.clone()
            },
            emotion: req.emotion.clone(),
            emotions_21d: Some(req.emotions_21d.clone()),
            prosody: None,
            format: "wav".to_string(),
            sample_rate: 22050,
        };

        match voice.speak(voice_req).await {
            Ok(response) => {
                // Duration in seconds from milliseconds
                let duration = response.duration_ms as f64 / 1000.0;

                Ok(Response::new(SpeakResponse {
                    audio: response.audio,
                    format: response.format,
                    sample_rate: response.sample_rate as i32,
                    duration,
                }))
            }
            Err(e) => Err(Status::internal(format!("TTS hiba: {}", e))),
        }
    }

    /// Listen - STT streaming (audio → szöveg)
    async fn listen(
        &self,
        request: Request<tonic::Streaming<AudioChunk>>,
    ) -> Result<Response<TranscriptionResponse>, Status> {
        let mut stream = request.into_inner();
        println!("👂 HOPE: Listen streaming kérés");

        // Audio összegyűjtése
        let mut audio_data = Vec::new();
        while let Some(chunk) = stream.message().await? {
            audio_data.extend(chunk.data);
        }

        // STT meghívása
        let voice = self.voice.read().await;
        let listen_req = crate::modules::voice::ListenRequest {
            language: "hu".to_string(),
            model: "whisper".to_string(),
            vad_enabled: true,
            word_timestamps: false,
        };

        match voice.transcribe(audio_data, listen_req).await {
            Ok(result) => Ok(Response::new(TranscriptionResponse {
                text: result.text.clone(),
                language: result.language.clone(),
                confidence: result.confidence,
                words: Vec::new(), // No word-level timestamps in this version
                duration: result.duration_ms as f64 / 1000.0,
            })),
            Err(e) => Err(Status::internal(format!("STT hiba: {}", e))),
        }
    }

    /// ListenContinuous - Folyamatos hallgatás VAD-dal
    type ListenContinuousStream =
        tokio_stream::wrappers::ReceiverStream<Result<TranscriptionChunk, Status>>;

    async fn listen_continuous(
        &self,
        request: Request<ListenContinuousRequest>,
    ) -> Result<Response<Self::ListenContinuousStream>, Status> {
        let req = request.into_inner();
        println!("👂 HOPE: ListenContinuous kérés (VAD: {})", req.vad_enabled);

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // Placeholder - valós implementációnál a microphone input stream
        tokio::spawn(async move {
            // Kezdeti chunk jelezve hogy elindult a hallgatás
            let _ = tx
                .send(Ok(TranscriptionChunk {
                    text: String::new(),
                    is_final: false,
                    confidence: 0.0,
                    timestamp: 0.0,
                    speaker: String::new(),
                    speech_started: false,
                    speech_ended: false,
                }))
                .await;
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    /// Converse - Bidirectional voice conversation
    type ConverseStream = tokio_stream::wrappers::ReceiverStream<Result<ConversationChunk, Status>>;

    async fn converse(
        &self,
        _request: Request<tonic::Streaming<AudioChunk>>,
    ) -> Result<Response<Self::ConverseStream>, Status> {
        println!("🎤👂 HOPE: Converse bidirectional stream");

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // Placeholder - valós implementáció a voice modulban
        tokio::spawn(async move {
            let _ = tx
                .send(Ok(ConversationChunk {
                    content: None,
                    turn: "hope".to_string(),
                    timestamp: 0.0,
                }))
                .await;
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    /// GetVoices - Elérhető hangok listázása
    async fn get_voices(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<VoicesResponse>, Status> {
        println!("🎤 HOPE: GetVoices kérés");

        let voice = self.voice.read().await;
        let voices_list = voice.list_voices();
        let current_voice = voice.get_current_voice().await;

        let voices: Vec<VoiceInfo> = voices_list
            .iter()
            .map(|v| {
                let gender_str = match v.gender {
                    crate::modules::voice::Gender::Female => "female",
                    crate::modules::voice::Gender::Male => "male",
                };
                VoiceInfo {
                    id: v.id.clone(),
                    name: v.name.clone(),
                    language: v.language.clone(),
                    gender: gender_str.to_string(),
                    description: v.description.clone(),
                    available: v.available,
                    emotions: v.emotions.clone(),
                    engine: v.engine.to_string(),
                    style: v.style.clone(),
                }
            })
            .collect();

        Ok(Response::new(VoicesResponse {
            voices,
            current_voice,
        }))
    }

    /// SetVoice - Hang beállítása
    async fn set_voice(
        &self,
        request: Request<SetVoiceRequest>,
    ) -> Result<Response<SetVoiceResponse>, Status> {
        let req = request.into_inner();
        println!("🎤 HOPE: SetVoice kérés: {}", req.voice);

        let voice = self.voice.read().await;

        match voice.set_voice(&req.voice).await {
            Ok(()) => Ok(Response::new(SetVoiceResponse {
                success: true,
                message: format!("Hang beállítva: {}", req.voice),
                current_voice: req.voice,
            })),
            Err(e) => {
                let current = voice.get_current_voice().await;
                Ok(Response::new(SetVoiceResponse {
                    success: false,
                    message: e.to_string(),
                    current_voice: current,
                }))
            }
        }
    }

    /// GetVoiceStatus - Voice rendszer állapota
    async fn get_voice_status(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<VoiceStatusResponse>, Status> {
        let voice = self.voice.read().await;
        let _stats = voice.get_stats().await;
        let current_voice = voice.get_current_voice().await;
        let current_emotion = voice.get_current_emotion().await;
        let emotions_21d = voice.get_emotions_21d().await;
        let tts_available = voice.tts_available().await;
        let stt_available = voice.stt_available().await;

        Ok(Response::new(VoiceStatusResponse {
            tts_available,
            stt_available,
            current_voice,
            current_emotion,
            current_emotions_21d: emotions_21d,
            tts_port: voice.get_tts_port() as i32,
            stt_port: voice.get_stt_port() as i32,
            latency_ms: 0.0, // TODO: measure actual latency
        }))
    }

    /// CloneVoice - Hang klónozása
    async fn clone_voice(
        &self,
        request: Request<CloneVoiceRequest>,
    ) -> Result<Response<CloneVoiceResponse>, Status> {
        let req = request.into_inner();
        println!("🎤 HOPE: CloneVoice kérés: {}", req.name);

        let voice = self.voice.read().await;

        // Audio samples átalakítása
        let samples: Vec<Vec<u8>> = req.audio_samples.into_iter().collect();

        match voice.clone_voice(&req.name, &samples).await {
            Ok(clone) => Ok(Response::new(CloneVoiceResponse {
                success: true,
                clone_id: clone.id.to_string(),
                clone: Some(super::proto::VoiceClone {
                    id: clone.id.to_string(),
                    name: clone.name.clone(),
                    base_voice: String::new(), // Not tracked in current implementation
                    signature: Some(convert_voice_signature(&clone.signature)),
                    status: "ready".to_string(),
                    training_samples: clone.sample_count as i32,
                    quality_score: clone.quality_score,
                    created_at: Some(Timestamp {
                        seconds: clone.created_at.timestamp(),
                        nanos: 0,
                    }),
                }),
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(CloneVoiceResponse {
                success: false,
                clone_id: String::new(),
                clone: None,
                error: e,
            })),
        }
    }

    /// GetVoiceClones - Klónozott hangok listázása
    async fn get_voice_clones(
        &self,
        request: Request<GetVoiceClonesRequest>,
    ) -> Result<Response<VoiceClonesResponse>, Status> {
        let _req = request.into_inner();

        let voice = self.voice.read().await;
        let clones = voice.list_cloned_voices().await;

        let proto_clones: Vec<super::proto::VoiceClone> = clones
            .iter()
            .map(|c| super::proto::VoiceClone {
                id: c.id.to_string(),
                name: c.name.clone(),
                base_voice: String::new(),
                signature: Some(convert_voice_signature(&c.signature)),
                status: "ready".to_string(),
                training_samples: c.sample_count as i32,
                quality_score: c.quality_score,
                created_at: Some(Timestamp {
                    seconds: c.created_at.timestamp(),
                    nanos: 0,
                }),
            })
            .collect();

        Ok(Response::new(VoiceClonesResponse {
            clones: proto_clones,
            total: clones.len() as i32,
        }))
    }

    /// RegisterVoice - Hang aláírás regisztrálása (Resonance integráció)
    async fn register_voice(
        &self,
        request: Request<RegisterVoiceRequest>,
    ) -> Result<Response<RegisterVoiceResponse>, Status> {
        let req = request.into_inner();
        println!("🔐 HOPE: RegisterVoice kérés user_id: {}", req.user_id);

        let voice = self.voice.read().await;

        // Audio samples összefűzése és elemzése
        let combined_audio: Vec<u8> = req.audio_samples.into_iter().flatten().collect();
        let signature = voice.analyze_voice(&combined_audio).await;

        // Regisztráció a voice engine-ben
        let sig_id = voice.register_voice_signature(signature.clone()).await;

        Ok(Response::new(RegisterVoiceResponse {
            success: true,
            signature_id: sig_id.to_string(),
            signature: Some(convert_voice_signature(&signature)),
            error: String::new(),
        }))
    }

    /// VerifyVoice - Hang verifikáció
    async fn verify_voice(
        &self,
        request: Request<VerifyVoiceRequest>,
    ) -> Result<Response<VerifyVoiceResponse>, Status> {
        let req = request.into_inner();
        let threshold = if req.threshold > 0.0 {
            req.threshold
        } else {
            0.85
        };
        println!("🔐 HOPE: VerifyVoice kérés (threshold: {})", threshold);

        let voice = self.voice.read().await;

        // Audio elemzése
        let signature = voice.analyze_voice(&req.audio_data).await;

        // Verifikáció - compare against all known signatures
        let known_sigs = voice.get_known_signatures().await;
        let mut best_match: Option<(f64, uuid::Uuid)> = None;

        for known_sig in &known_sigs {
            let sim = signature.similarity(known_sig);
            if sim >= threshold {
                if best_match.is_none() || sim > best_match.unwrap().0 {
                    best_match = Some((sim, known_sig.id));
                }
            }
        }

        let (verified, confidence, matched_id) = match best_match {
            Some((conf, id)) => (true, conf, Some(id.to_string())),
            None => (false, 0.0, None),
        };

        Ok(Response::new(VerifyVoiceResponse {
            verified,
            confidence,
            matched_user_id: matched_id.clone().unwrap_or_default(),
            matched_user_name: matched_id.unwrap_or_default(), // Same as ID for now
            analyzed_signature: Some(convert_voice_signature(&signature)),
            candidates: Vec::new(),
        }))
    }

    /// AnalyzeVoice - Hang elemzés (signature generálás)
    async fn analyze_voice(
        &self,
        request: Request<AnalyzeVoiceRequest>,
    ) -> Result<Response<AnalyzeVoiceResponse>, Status> {
        let req = request.into_inner();
        println!(
            "🔐 HOPE: AnalyzeVoice kérés ({} bytes)",
            req.audio_data.len()
        );

        let voice = self.voice.read().await;
        let signature = voice.analyze_voice(&req.audio_data).await;

        Ok(Response::new(AnalyzeVoiceResponse {
            success: true,
            signature: Some(convert_voice_signature(&signature)),
            error: String::new(),
        }))
    }

    /// GetVoiceSignatures - Regisztrált hang aláírások listázása
    async fn get_voice_signatures(
        &self,
        request: Request<GetVoiceSignaturesRequest>,
    ) -> Result<Response<VoiceSignaturesResponse>, Status> {
        let _req = request.into_inner();

        let voice = self.voice.read().await;
        let signatures = voice.get_known_signatures().await;

        let proto_sigs: Vec<super::proto::VoiceSignatureInfo> = signatures
            .iter()
            .map(|sig| super::proto::VoiceSignatureInfo {
                id: sig.id.to_string(),
                user_id: sig.id.to_string(),
                user_name: String::new(),
                sample_count: sig.sample_count as i32,
                confidence: 1.0,
                created_at: Some(Timestamp {
                    seconds: sig.created_at.timestamp(),
                    nanos: 0,
                }),
                last_verified: None,
            })
            .collect();

        Ok(Response::new(VoiceSignaturesResponse {
            signatures: proto_sigs,
            total: signatures.len() as i32,
        }))
    }
}

/// VoiceSignature konvertálása proto típusra
fn convert_voice_signature(
    sig: &crate::modules::voice::VoiceSignature,
) -> super::proto::VoiceSignature {
    super::proto::VoiceSignature {
        id: sig.id.to_string(),
        pitch_mean: sig.pitch_mean,
        pitch_variance: sig.pitch_variance,
        speaking_rate: sig.speaking_rate,
        pause_pattern: sig.pause_pattern.clone(),
        formant_frequencies: sig.formant_frequencies.to_vec(),
        spectral_envelope: sig.spectral_envelope.clone(),
        energy_contour: sig.energy_contour.clone(),
        jitter: sig.jitter,
        shimmer: sig.shimmer,
        hnr: sig.hnr,
        created_at: Some(Timestamp {
            seconds: sig.created_at.timestamp(),
            nanos: 0,
        }),
        sample_count: sig.sample_count as i32,
    }
}

// ============================================================================
// NAVIGATION SERVICE IMPLEMENTÁCIÓ - Intelligens útvonaltervezés
// ============================================================================

#[tonic::async_trait]
impl NavigationService for HopeGrpcServer {
    /// PlanRoute - Útvonal tervezése mood/energia alapján
    async fn plan_route(
        &self,
        request: Request<PlanRouteRequest>,
    ) -> Result<Response<PlanRouteResponse>, Status> {
        let req = request.into_inner();
        println!("🗺️ HOPE: PlanRoute kérés");

        // Kontextus konvertálása
        let ctx = self.convert_navigation_context(&req);

        match self.navigation.plan_route(ctx).await {
            Ok(route) => {
                let proto_route = self.convert_smart_route_to_proto(&route);
                Ok(Response::new(PlanRouteResponse {
                    success: true,
                    route: Some(proto_route),
                    error: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(PlanRouteResponse {
                success: false,
                route: None,
                error: e.to_string(),
            })),
        }
    }

    /// GetAlternatives - Alternatív útvonalak
    async fn get_alternatives(
        &self,
        request: Request<PlanRouteRequest>,
    ) -> Result<Response<AlternativeRoutesResponse>, Status> {
        let req = request.into_inner();
        println!("🗺️ HOPE: GetAlternatives kérés");

        let ctx = self.convert_navigation_context(&req);

        match self.navigation.plan_alternatives(ctx).await {
            Ok(routes) => {
                let proto_routes: Vec<SmartRouteProto> = routes
                    .into_iter()
                    .map(|r| self.convert_smart_route_to_proto(&r))
                    .collect();
                let total = proto_routes.len() as i32;
                Ok(Response::new(AlternativeRoutesResponse {
                    routes: proto_routes,
                    total,
                }))
            }
            Err(e) => Err(Status::internal(format!("Alternatívák hiba: {}", e))),
        }
    }

    /// StartNavigation - Navigáció indítása
    async fn start_navigation(
        &self,
        request: Request<StartNavigationRequest>,
    ) -> Result<Response<StartNavigationResponse>, Status> {
        let req = request.into_inner();
        println!("🗺️ HOPE: StartNavigation kérés");

        if let Some(route_proto) = req.route {
            let route = self.convert_proto_to_smart_route(&route_proto);
            match self.navigation.start_navigation(route).await {
                Ok(()) => Ok(Response::new(StartNavigationResponse {
                    success: true,
                    navigation_id: uuid::Uuid::new_v4().to_string(),
                    error: String::new(),
                })),
                Err(e) => Ok(Response::new(StartNavigationResponse {
                    success: false,
                    navigation_id: String::new(),
                    error: e.to_string(),
                })),
            }
        } else {
            Ok(Response::new(StartNavigationResponse {
                success: false,
                navigation_id: String::new(),
                error: "Nincs útvonal megadva".to_string(),
            }))
        }
    }

    /// UpdatePosition - Pozíció frissítése (bidirectional streaming)
    type UpdatePositionStream =
        tokio_stream::wrappers::ReceiverStream<Result<NavigationUpdateResponse, Status>>;

    async fn update_position(
        &self,
        request: Request<tonic::Streaming<PositionUpdate>>,
    ) -> Result<Response<Self::UpdatePositionStream>, Status> {
        let mut stream = request.into_inner();
        let navigation = self.navigation.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            while let Ok(Some(pos)) = stream.message().await {
                let geo_location = GeoLocation {
                    latitude: pos.latitude,
                    longitude: pos.longitude,
                    altitude: Some(0.0),
                    accuracy: Some(pos.accuracy),
                    source: GeoSource::Gps,
                    timestamp: chrono::Utc::now(),
                };

                if let Some(update) = navigation.update_position(geo_location).await {
                    let response = NavigationUpdateResponse {
                        status: if update.off_route {
                            "off_route".to_string()
                        } else {
                            "on_route".to_string()
                        },
                        progress: 0.5, // Calculated from route progress
                        remaining_km: update.distance_to_next,
                        remaining_secs: (update.eta - chrono::Utc::now()).num_seconds(),
                        next_instruction: update.next_instruction.unwrap_or_default(),
                        distance_to_turn: update.distance_to_next,
                        current_position: Some(GeoPointProto {
                            latitude: update.position.latitude,
                            longitude: update.position.longitude,
                            name: String::new(),
                            place_id: String::new(),
                        }),
                        nearby_memory: String::new(),
                        nearby_place: String::new(),
                        suggestion: update.context_message.unwrap_or_default(),
                    };

                    if tx.send(Ok(response)).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }

    /// StopNavigation - Navigáció leállítása
    async fn stop_navigation(
        &self,
        _request: Request<StopNavigationRequest>,
    ) -> Result<Response<CompletedRouteResponse>, Status> {
        println!("🗺️ HOPE: StopNavigation kérés");

        if let Some(completed) = self.navigation.stop_navigation().await {
            Ok(Response::new(CompletedRouteResponse {
                success: true,
                route_id: completed.route_id.to_string(),
                actual_distance_km: completed.actual_distance_km,
                actual_duration_secs: completed.actual_duration.num_seconds(),
                waypoints_visited: completed
                    .stops_made
                    .into_iter()
                    .map(|p| GeoPointProto {
                        latitude: p.location.latitude,
                        longitude: p.location.longitude,
                        name: p.name.clone(),
                        place_id: p.id.to_string(),
                    })
                    .collect(),
                stops_made: 0, // Count would need separate tracking
                average_speed_kmh: completed.average_speed_kmh,
                started_at: Some(Timestamp {
                    seconds: completed.started_at.timestamp(),
                    nanos: 0,
                }),
                completed_at: Some(Timestamp {
                    seconds: completed.completed_at.timestamp(),
                    nanos: 0,
                }),
            }))
        } else {
            Ok(Response::new(CompletedRouteResponse {
                success: false,
                route_id: String::new(),
                actual_distance_km: 0.0,
                actual_duration_secs: 0,
                waypoints_visited: vec![],
                stops_made: 0,
                average_speed_kmh: 0.0,
                started_at: None,
                completed_at: None,
            }))
        }
    }

    /// PredictDestination - Cél előrejelzés (Symbiosis)
    async fn predict_destination(
        &self,
        _request: Request<PredictDestinationRequest>,
    ) -> Result<Response<PredictDestinationResponse>, Status> {
        println!("🔮 HOPE: PredictDestination kérés");

        if let Some(prediction) = self.navigation.predict_destination().await {
            let reasons: Vec<String> = prediction
                .reasoning
                .iter()
                .map(|r| format!("{:?}", r))
                .collect();
            let proto_prediction = PredictedDestinationProto {
                place_id: prediction.place.id.to_string(),
                place_name: prediction.place.name.clone(),
                location: Some(GeoPointProto {
                    latitude: prediction.place.location.latitude,
                    longitude: prediction.place.location.longitude,
                    name: prediction.place.name.clone(),
                    place_id: prediction.place.id.to_string(),
                }),
                confidence: prediction.confidence,
                reason: reasons.first().cloned().unwrap_or_default(),
                reasoning: reasons.join(", "),
                suggested_departure: Some(Timestamp {
                    seconds: prediction.suggested_departure.timestamp(),
                    nanos: 0,
                }),
            };

            Ok(Response::new(PredictDestinationResponse {
                has_prediction: true,
                prediction: Some(proto_prediction),
                alternatives: vec![],
            }))
        } else {
            Ok(Response::new(PredictDestinationResponse {
                has_prediction: false,
                prediction: None,
                alternatives: vec![],
            }))
        }
    }

    /// GetETA - ETA számítás
    async fn get_eta(
        &self,
        request: Request<GetEtaRequest>,
    ) -> Result<Response<GetEtaResponse>, Status> {
        let req = request.into_inner();
        println!("⏱️ HOPE: GetETA kérés");

        if let Some(dest) = req.destination {
            let destination = GeoLocation {
                latitude: dest.latitude,
                longitude: dest.longitude,
                altitude: None,
                accuracy: None,
                source: GeoSource::Manual,
                timestamp: chrono::Utc::now(),
            };

            match self.navigation.calculate_eta(&destination).await {
                Ok((duration, arrival, traffic, confidence)) => {
                    Ok(Response::new(GetEtaResponse {
                        duration_secs: duration.num_seconds(),
                        arrival_time: Some(Timestamp {
                            seconds: arrival.timestamp(),
                            nanos: 0,
                        }),
                        traffic_level: format!("{:?}", traffic),
                        confidence,
                        distance_km: 0.0, // TODO: Calculate from route
                    }))
                }
                Err(e) => Err(Status::internal(format!("ETA hiba: {}", e))),
            }
        } else {
            Err(Status::invalid_argument("Nincs cél megadva"))
        }
    }

    /// FindNearby - Közeli helyek keresése
    async fn find_nearby(
        &self,
        request: Request<FindNearbyRequest>,
    ) -> Result<Response<FindNearbyResponse>, Status> {
        let req = request.into_inner();
        println!("📍 HOPE: FindNearby kérés");

        let category = if req.category.is_empty() {
            None
        } else {
            Some(req.category.as_str())
        };

        let places = self
            .navigation
            .find_nearby(category, req.radius_km, req.on_route_only)
            .await;

        let proto_places: Vec<PlaceWithContextProto> = places
            .into_iter()
            .take(req.limit as usize)
            .map(|p| PlaceWithContextProto {
                place_id: p.place.id.to_string(),
                name: p.place.name.clone(),
                place_type: format!("{:?}", p.place.place_type),
                location: Some(GeoPointProto {
                    latitude: p.place.location.latitude,
                    longitude: p.place.location.longitude,
                    name: p.place.name.clone(),
                    place_id: p.place.id.to_string(),
                }),
                distance_km: p.distance_km,
                visit_count: p.place.visit_count as i64,
                is_favorite: p.place.visit_count > 5,
                memories: vec![],
                emotional_history: p.context.unwrap_or_default(),
                relevance_score: 0.5,
            })
            .collect();

        let total = proto_places.len() as i32;
        Ok(Response::new(FindNearbyResponse {
            places: proto_places,
            total,
        }))
    }

    /// GetRouteContext - Útvonal kontextus (emlékek, érzelmek)
    async fn get_route_context(
        &self,
        request: Request<GetRouteContextRequest>,
    ) -> Result<Response<RouteContextResponse>, Status> {
        let req = request.into_inner();
        println!("💭 HOPE: GetRouteContext kérés");

        if let Some(route_proto) = req.route {
            let route = self.convert_proto_to_smart_route(&route_proto);
            let context = self.navigation.get_route_context(&route).await;

            Ok(Response::new(RouteContextResponse {
                emotions: context
                    .emotional_history
                    .into_iter()
                    .map(|e| EmotionAtLocationProto {
                        location: Some(GeoPointProto {
                            latitude: e.location.latitude,
                            longitude: e.location.longitude,
                            name: String::new(),
                            place_id: String::new(),
                        }),
                        dominant_emotion: e.emotion,
                        intensity: e.intensity,
                        when: Some(Timestamp {
                            seconds: e.when.timestamp(),
                            nanos: 0,
                        }),
                        context: String::new(),
                    })
                    .collect(),
                people_associated: context.people_associated,
                events: context
                    .events_nearby
                    .into_iter()
                    .map(|e| NearbyEventProto {
                        event_id: String::new(),
                        name: e.name,
                        location: Some(GeoPointProto {
                            latitude: e.location.latitude,
                            longitude: e.location.longitude,
                            name: String::new(),
                            place_id: String::new(),
                        }),
                        starts_at: e.when.map(|w| Timestamp {
                            seconds: w.timestamp(),
                            nanos: 0,
                        }),
                        ends_at: e.when.map(|w| Timestamp {
                            seconds: w.timestamp(),
                            nanos: 0,
                        }),
                        category: String::new(),
                    })
                    .collect(),
                suggestions: context
                    .suggestions
                    .into_iter()
                    .map(|s| ContextSuggestionProto {
                        suggestion: s.text,
                        action: s.action.map(|a| format!("{:?}", a)).unwrap_or_default(),
                        location: None,
                        reason: format!("Relevance: {:.0}%", s.relevance * 100.0),
                    })
                    .collect(),
                overall_emotional_tone: String::new(),
            }))
        } else {
            Err(Status::invalid_argument("Nincs útvonal megadva"))
        }
    }

    /// SuggestDeparture - Indulási idő javaslat
    async fn suggest_departure(
        &self,
        request: Request<SuggestDepartureRequest>,
    ) -> Result<Response<SuggestDepartureResponse>, Status> {
        let req = request.into_inner();
        println!("🕐 HOPE: SuggestDeparture kérés");

        if let Some(dest) = req.destination {
            let destination = Place {
                id: uuid::Uuid::new_v4(),
                name: dest.name.clone(),
                place_type: PlaceType::Other,
                location: GeoLocation {
                    latitude: dest.latitude,
                    longitude: dest.longitude,
                    altitude: None,
                    accuracy: None,
                    source: GeoSource::Manual,
                    timestamp: chrono::Utc::now(),
                },
                radius: 100.0,
                address: None,
                country_code: None,
                visit_count: 0,
                last_visit: None,
                first_visit: chrono::Utc::now(),
                memory_count: 0,
                emotional_associations: std::collections::HashMap::new(),
                metadata: std::collections::HashMap::new(),
            };

            let suggested = self.navigation.suggest_departure_time(&destination).await;

            Ok(Response::new(SuggestDepartureResponse {
                suggested_departure: Some(Timestamp {
                    seconds: suggested.timestamp(),
                    nanos: 0,
                }),
                estimated_duration_secs: 1800, // TODO: Calculate from route
                traffic_prediction: "medium".to_string(),
                reasoning: "Az útvonal és forgalom alapján javasolt indulási idő".to_string(),
                preparation_buffer_mins: if req.consider_preparation { 15 } else { 0 },
            }))
        } else {
            Err(Status::invalid_argument("Nincs cél megadva"))
        }
    }

    /// GetNavigationStats - Navigációs statisztikák
    async fn get_navigation_stats(
        &self,
        _request: Request<EmptyRequest>,
    ) -> Result<Response<NavigationStatsResponse>, Status> {
        println!("📊 HOPE: GetNavigationStats kérés");

        let stats = self.navigation.get_stats().await;

        Ok(Response::new(NavigationStatsResponse {
            total_routes_planned: stats.total_routes_planned as i64,
            total_navigations_completed: stats.total_routes_completed as i64,
            total_distance_km: stats.total_distance_km,
            total_duration_secs: stats.total_time_navigating.num_seconds(),
            avg_accuracy_percent: if stats.predictions_made > 0 {
                (stats.predictions_correct as f64 / stats.predictions_made as f64) * 100.0
            } else {
                0.0
            },
            destinations_predicted: stats.predictions_made as i64,
            prediction_accuracy: if stats.predictions_made > 0 {
                stats.predictions_correct as f64 / stats.predictions_made as f64
            } else {
                0.0
            },
            learned_patterns: stats.favorite_destinations.len() as i32,
            favorite_destinations: stats.favorite_destinations.len() as i32,
        }))
    }
}

// Navigation helper methods
impl HopeGrpcServer {
    fn convert_navigation_context(
        &self,
        req: &PlanRouteRequest,
    ) -> crate::modules::navigation::NavigationContext {
        use crate::modules::navigation::{
            EmotionState, GeoPoint, NavigationContext, RoutePreferences,
        };

        let origin = req.origin.as_ref().map(|o| GeoPoint {
            latitude: o.latitude,
            longitude: o.longitude,
            name: if o.name.is_empty() {
                None
            } else {
                Some(o.name.clone())
            },
        });

        let destination = req
            .destination
            .as_ref()
            .map(|d| GeoPoint {
                latitude: d.latitude,
                longitude: d.longitude,
                name: if d.name.is_empty() {
                    None
                } else {
                    Some(d.name.clone())
                },
            })
            .unwrap_or(GeoPoint {
                latitude: 0.0,
                longitude: 0.0,
                name: None,
            });

        let waypoints: Vec<GeoPoint> = req
            .waypoints
            .iter()
            .map(|w| GeoPoint {
                latitude: w.latitude,
                longitude: w.longitude,
                name: if w.name.is_empty() {
                    None
                } else {
                    Some(w.name.clone())
                },
            })
            .collect();

        let ctx = req.context.as_ref();
        let prefs = req.preferences.as_ref();

        NavigationContext {
            origin,
            destination,
            waypoints,
            current_mood: ctx.map(|c| c.current_mood.clone()).unwrap_or_default(),
            energy_level: ctx.map(|c| c.energy_level).unwrap_or(0.7),
            time_pressure: ctx.map(|c| c.time_pressure).unwrap_or(0.5),
            emotions: ctx
                .map(|c| EmotionState {
                    dominant: c.current_mood.clone(),
                    intensity: 0.5,
                    valence: 0.0,
                    arousal: 0.0,
                })
                .unwrap_or_default(),
            preferences: RoutePreferences {
                avoid_highways: prefs.map(|p| p.avoid_highways).unwrap_or(false),
                avoid_tolls: prefs.map(|p| p.avoid_tolls).unwrap_or(false),
                prefer_scenic: prefs.map(|p| p.scenic_route).unwrap_or(false),
                prefer_familiar: true, // Default - proto doesn't have this field
                max_walking_distance: prefs.map(|p| p.max_walking_km).unwrap_or(2.0),
                accessibility_needs: vec![],
                learned_avoidances: vec![],
                favorite_routes: vec![],
                preferred_stops: vec![],
            },
            purpose: ctx.map(|c| c.purpose.clone()).unwrap_or_default(),
        }
    }

    fn convert_smart_route_to_proto(
        &self,
        route: &crate::modules::navigation::SmartRoute,
    ) -> SmartRouteProto {
        let origin = route.path.first();
        let destination = route.path.last();

        SmartRouteProto {
            id: route.id.to_string(),
            origin: origin.map(|o| GeoPointProto {
                latitude: o.latitude,
                longitude: o.longitude,
                name: o.name.clone().unwrap_or_default(),
                place_id: String::new(),
            }),
            destination: destination.map(|d| GeoPointProto {
                latitude: d.latitude,
                longitude: d.longitude,
                name: d.name.clone().unwrap_or_default(),
                place_id: String::new(),
            }),
            distance_km: route.total_distance_km,
            duration_secs: route.total_duration.num_seconds(),
            segments: route
                .segments
                .iter()
                .map(|s| RouteSegmentProto {
                    start: Some(GeoPointProto {
                        latitude: s.start.latitude,
                        longitude: s.start.longitude,
                        name: s.start.name.clone().unwrap_or_default(),
                        place_id: String::new(),
                    }),
                    end: Some(GeoPointProto {
                        latitude: s.end.latitude,
                        longitude: s.end.longitude,
                        name: s.end.name.clone().unwrap_or_default(),
                        place_id: String::new(),
                    }),
                    distance_km: s.distance_km,
                    duration_secs: s.duration.num_seconds(),
                    road_type: format!("{:?}", s.road_type),
                    instruction: s.instruction.clone(),
                    traffic_level: format!("{:?}", s.traffic),
                })
                .collect(),
            memories_on_route: route
                .memories_on_route
                .iter()
                .map(|m| MemoryOnRouteProto {
                    memory_id: m.id.clone(),
                    content: m.content.clone(),
                    location: Some(GeoPointProto {
                        latitude: m.location.latitude,
                        longitude: m.location.longitude,
                        name: String::new(),
                        place_id: String::new(),
                    }),
                    importance: m.importance,
                    emotional_tag: m.emotional_tag.clone(),
                })
                .collect(),
            places_on_route: route
                .places_on_route
                .iter()
                .map(|p| PlaceOnRouteProto {
                    place_id: p.id.to_string(),
                    name: p.name.clone(),
                    place_type: format!("{:?}", p.place_type),
                    location: Some(GeoPointProto {
                        latitude: p.location.latitude,
                        longitude: p.location.longitude,
                        name: p.name.clone(),
                        place_id: p.id.to_string(),
                    }),
                    visit_count: p.visit_count as i64,
                    is_favorite: p.visit_count > 5,
                })
                .collect(),
            suggested_stops: route
                .suggested_stops
                .iter()
                .map(|s| SuggestedStopProto {
                    place_id: s.place.id.to_string(),
                    name: s.place.name.clone(),
                    reason: format!("{:?}", s.reason),
                    description: format!("Relevance: {:.2}", s.relevance_score),
                    location: Some(GeoPointProto {
                        latitude: s.place.location.latitude,
                        longitude: s.place.location.longitude,
                        name: s.place.name.clone(),
                        place_id: s.place.id.to_string(),
                    }),
                    suggested_duration_mins: s.detour_time.num_seconds() / 60,
                })
                .collect(),
            emotional_score: route.emotional_score,
            context_notes: route.context_notes.clone(),
            traffic_level: format!("{:?}", route.traffic_level),
        }
    }

    fn convert_proto_to_smart_route(
        &self,
        proto: &SmartRouteProto,
    ) -> crate::modules::navigation::SmartRoute {
        use crate::modules::navigation::{
            Delay, GeoPoint, MemoryOnRoute, RoadType, RouteSegment, SmartRoute, TrafficLevel,
        };
        use chrono::Duration;

        // Build path from origin, segments, and destination
        let mut path = Vec::new();
        if let Some(o) = &proto.origin {
            path.push(GeoPoint {
                latitude: o.latitude,
                longitude: o.longitude,
                name: if o.name.is_empty() {
                    None
                } else {
                    Some(o.name.clone())
                },
            });
        }
        if let Some(d) = &proto.destination {
            path.push(GeoPoint {
                latitude: d.latitude,
                longitude: d.longitude,
                name: if d.name.is_empty() {
                    None
                } else {
                    Some(d.name.clone())
                },
            });
        }

        SmartRoute {
            id: uuid::Uuid::parse_str(&proto.id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            path,
            total_distance_km: proto.distance_km,
            total_duration: Duration::seconds(proto.duration_secs),
            eta: chrono::Utc::now() + Duration::seconds(proto.duration_secs),
            segments: proto
                .segments
                .iter()
                .map(|s| RouteSegment {
                    start: s
                        .start
                        .as_ref()
                        .map(|p| GeoPoint {
                            latitude: p.latitude,
                            longitude: p.longitude,
                            name: if p.name.is_empty() {
                                None
                            } else {
                                Some(p.name.clone())
                            },
                        })
                        .unwrap_or_default(),
                    end: s
                        .end
                        .as_ref()
                        .map(|p| GeoPoint {
                            latitude: p.latitude,
                            longitude: p.longitude,
                            name: if p.name.is_empty() {
                                None
                            } else {
                                Some(p.name.clone())
                            },
                        })
                        .unwrap_or_default(),
                    distance_km: s.distance_km,
                    duration: Duration::seconds(s.duration_secs),
                    road_type: RoadType::Urban,
                    instruction: s.instruction.clone(),
                    traffic: TrafficLevel::Moderate,
                })
                .collect(),
            memories_on_route: proto
                .memories_on_route
                .iter()
                .map(|m| MemoryOnRoute {
                    id: m.memory_id.clone(),
                    content: m.content.clone(),
                    location: m
                        .location
                        .as_ref()
                        .map(|l| GeoPoint {
                            latitude: l.latitude,
                            longitude: l.longitude,
                            name: None,
                        })
                        .unwrap_or_default(),
                    importance: m.importance,
                    emotional_tag: m.emotional_tag.clone(),
                })
                .collect(),
            places_on_route: vec![], // Simplified - would need Place conversion
            suggested_stops: vec![], // Simplified
            emotional_score: proto.emotional_score,
            context_notes: proto.context_notes.clone(),
            traffic_level: TrafficLevel::Moderate,
            delays: vec![],
            alternative_available: false,
        }
    }
}

// ============================================================================
// SZERVER INDÍTÁS
// ============================================================================

/// gRPC szerver indítása
pub async fn start_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;

    println!(
        r#"
╔═══════════════════════════════════════════╗
║      HOPE OS gRPC Server v{}         ║
╠═══════════════════════════════════════════╣
║  ()=>[] - A tiszta potenciálból           ║
║           minden megszületik              ║
╚═══════════════════════════════════════════╝
"#,
        env!("CARGO_PKG_VERSION")
    );

    println!("  Szerver inicializálása...");
    let server = HopeGrpcServer::new().await?;
    let server = Arc::new(server);

    println!("  Modulok betöltve:");
    println!("    HopeSoul");
    println!("    HopeMemory");
    println!("    HopeHeart");
    println!("    Smart Search Engine");
    println!("    VisionEngine (👁️ Hope szeme)");
    println!("    AttentionEngine (🎯 Fókusz/Figyelem)");

    println!("\n  gRPC szerver indítása: {}", addr);
    println!("  Szolgáltatások:");
    println!("    HopeService      - /hope.HopeService/*");
    println!("    MemoryService    - /hope.MemoryService/* (Smart Search!)");
    println!("    CognitiveService - /hope.CognitiveService/* (+ Attention!)");
    println!("    VisionService    - /hope.VisionService/* (👁️ Látás!)");
    println!("    SkillService     - /hope.SkillService/* (97 skill!)");
    println!("    GenomeService    - /hope.GenomeService/* (7 etikai alapelv!)");
    println!("    CodeService      - /hope.CodeService/* (Kód elemzés!)");
    println!("    ResonanceService - /hope.ResonanceService/* (🔐 Rezonancia Auth!)");
    println!("    GeoService       - /hope.GeoService/* (🌍 Térbeli kontextus!)");
    println!("    VoiceService     - /hope.VoiceService/* (🎤 TTS/STT Hang!)");
    println!("    NavigationService - /hope.NavigationService/* (🗺️ Intelligens navigáció!)");
    println!("\n  Ctrl+C a leállításhoz\n");

    Server::builder()
        .add_service(HopeServiceServer::from_arc(server.clone()))
        .add_service(MemoryServiceServer::from_arc(server.clone()))
        .add_service(CognitiveServiceServer::from_arc(server.clone()))
        .add_service(VisionServiceServer::from_arc(server.clone()))
        .add_service(SkillServiceServer::from_arc(server.clone()))
        .add_service(GenomeServiceServer::from_arc(server.clone()))
        .add_service(CodeServiceServer::from_arc(server.clone()))
        .add_service(ResonanceServiceServer::from_arc(server.clone()))
        .add_service(GeoServiceServer::from_arc(server.clone()))
        .add_service(VoiceServiceServer::from_arc(server.clone()))
        .add_service(NavigationServiceServer::from_arc(server.clone()))
        .serve(addr)
        .await?;

    Ok(())
}
