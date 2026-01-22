//! Hope OS - gRPC Server
//!
//! A Hope OS natív Rust gRPC szervere.
//! ()=>[] - A tiszta potenciálból minden megszületik

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use super::proto::{
    // CognitiveService
    cognitive_service_server::{CognitiveService, CognitiveServiceServer},
    // HopeService
    hope_service_server::{HopeService, HopeServiceServer},
    // MemoryService
    memory_service_server::{MemoryService, MemoryServiceServer},
    // VisionService
    vision_service_server::{VisionService, VisionServiceServer},
    ChatRequest,
    ChatResponse,
    CognitiveStateResponse,
    CompareImagesRequest,
    CompareImagesResponse,
    EmptyRequest,
    FeelRequest,
    FeelResponse,
    GetVisualMemoriesRequest,
    HeartbeatResponse,
    ImageAnalysis,
    MemoryItem,
    RecallRequest,
    RecallResponse,
    RememberRequest,
    RememberResponse,
    SeeRequest,
    SeeResponse,
    StatusResponse,
    ThinkRequest,
    ThinkResponse,
    // Timestamp
    Timestamp,
    VisionStatusResponse,
    VisualMemoriesResponse,
    VisualMemoryInfo,
};

use crate::core::HopeRegistry;
use crate::data::CodeGraph;
use crate::modules::{HopeHeart, HopeMemory, HopeSoul, VisionEngine};

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

    println!("\n  gRPC szerver indítása: {}", addr);
    println!("  Szolgáltatások:");
    println!("    HopeService      - /hope.HopeService/*");
    println!("    MemoryService    - /hope.MemoryService/* (Smart Search!)");
    println!("    CognitiveService - /hope.CognitiveService/*");
    println!("    VisionService    - /hope.VisionService/* (👁️ Látás!)");
    println!("\n  Ctrl+C a leállításhoz\n");

    Server::builder()
        .add_service(HopeServiceServer::from_arc(server.clone()))
        .add_service(MemoryServiceServer::from_arc(server.clone()))
        .add_service(CognitiveServiceServer::from_arc(server.clone()))
        .add_service(VisionServiceServer::from_arc(server.clone()))
        .serve(addr)
        .await?;

    Ok(())
}
