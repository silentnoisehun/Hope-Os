//! Hope OS - gRPC Client
//!
//! Kapcsolat a Python Hope szerverhez.
//! ()=>[] - A tiszta potenciálból minden megszületik

use std::collections::HashMap;
use tonic::transport::Channel;

use crate::core::{HopeError, HopeResult};

// Közös proto modul használata
use super::proto;

use proto::{
    action_service_client::ActionServiceClient, code_service_client::CodeServiceClient,
    cognitive_service_client::CognitiveServiceClient, echo_service_client::EchoServiceClient,
    genome_service_client::GenomeServiceClient, geo_service_client::GeoServiceClient,
    hope_service_client::HopeServiceClient, knowledge_service_client::KnowledgeServiceClient,
    memory_service_client::MemoryServiceClient, resonance_service_client::ResonanceServiceClient,
    skill_service_client::SkillServiceClient, vision_service_client::VisionServiceClient, *,
};

/// Hope gRPC Client
///
/// Kapcsolódás a Python Hope szerverhez minden szolgáltatással.
pub struct HopeClient {
    /// Hope fő szolgáltatás
    pub hope: HopeServiceClient<Channel>,
    /// Skill szolgáltatás
    pub skills: SkillServiceClient<Channel>,
    /// Memória szolgáltatás
    pub memory: MemoryServiceClient<Channel>,
    /// Kognitív szolgáltatás
    pub cognitive: CognitiveServiceClient<Channel>,
    /// Akció szolgáltatás
    pub action: ActionServiceClient<Channel>,
    /// Kód szolgáltatás
    pub code: CodeServiceClient<Channel>,
    /// Echo szolgáltatás (önfejlesztés)
    pub echo: EchoServiceClient<Channel>,
    /// Tudás szolgáltatás
    pub knowledge: KnowledgeServiceClient<Channel>,
    /// Genome szolgáltatás (etika)
    pub genome: GenomeServiceClient<Channel>,
    /// Vision szolgáltatás (Hope "szeme")
    pub vision: VisionServiceClient<Channel>,
    /// Resonance szolgáltatás (rezonancia auth)
    pub resonance: ResonanceServiceClient<Channel>,
    /// Geo szolgáltatás (térbeli kontextus)
    pub geo: GeoServiceClient<Channel>,
    /// Szerver cím
    address: String,
}

impl HopeClient {
    /// Csatlakozás a szerverhez
    pub async fn connect(address: &str) -> HopeResult<Self> {
        let channel = Channel::from_shared(address.to_string())
            .map_err(|e| HopeError::General(format!("Invalid address: {}", e)))?
            .connect()
            .await?;

        Ok(Self {
            hope: HopeServiceClient::new(channel.clone()),
            skills: SkillServiceClient::new(channel.clone()),
            memory: MemoryServiceClient::new(channel.clone()),
            cognitive: CognitiveServiceClient::new(channel.clone()),
            action: ActionServiceClient::new(channel.clone()),
            code: CodeServiceClient::new(channel.clone()),
            echo: EchoServiceClient::new(channel.clone()),
            knowledge: KnowledgeServiceClient::new(channel.clone()),
            genome: GenomeServiceClient::new(channel.clone()),
            vision: VisionServiceClient::new(channel.clone()),
            resonance: ResonanceServiceClient::new(channel.clone()),
            geo: GeoServiceClient::new(channel.clone()),
            address: address.to_string(),
        })
    }

    /// Alapértelmezett szerver cím
    pub fn default_address() -> &'static str {
        "http://localhost:50051"
    }

    /// Szerver cím lekérdezése
    pub fn address(&self) -> &str {
        &self.address
    }

    // ==================== HOPE SERVICE ====================

    /// Beszélgetés
    pub async fn chat(&mut self, message: &str) -> HopeResult<ChatResponse> {
        let request = ChatRequest {
            message: message.to_string(),
            context: String::new(),
            metadata: HashMap::new(),
        };

        let response = self.hope.chat(request).await?;
        Ok(response.into_inner())
    }

    /// Szerver állapot
    pub async fn status(&mut self) -> HopeResult<StatusResponse> {
        let response = self.hope.get_status(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Szerver állapot (alias for MCP)
    pub async fn get_status(&mut self) -> HopeResult<StatusResponse> {
        self.status().await
    }

    /// Életjel
    pub async fn heartbeat(&mut self) -> HopeResult<bool> {
        let response = self.hope.heartbeat(EmptyRequest {}).await?;
        Ok(response.into_inner().alive)
    }

    // ==================== SKILL SERVICE ====================

    /// Skillek listázása
    pub async fn list_skills(&mut self) -> HopeResult<Vec<SkillInfo>> {
        let request = ListSkillsRequest {
            category: String::new(),
            search: String::new(),
            limit: 1000,
            offset: 0,
        };

        let response = self.skills.list_skills(request).await?;
        Ok(response.into_inner().skills)
    }

    /// Skill meghívása (egyszerű)
    pub async fn invoke_skill_simple(
        &mut self,
        name: &str,
        input: &str,
    ) -> HopeResult<InvokeSkillResponse> {
        self.invoke_skill(name, input, HashMap::new()).await
    }

    /// Skill meghívása (teljes)
    pub async fn invoke_skill(
        &mut self,
        name: &str,
        input: &str,
        params: HashMap<String, String>,
    ) -> HopeResult<InvokeSkillResponse> {
        let request = InvokeSkillRequest {
            name: name.to_string(),
            input: input.to_string(),
            params,
        };

        let response = self.skills.invoke_skill(request).await?;
        Ok(response.into_inner())
    }

    // ==================== MEMORY SERVICE ====================

    /// Emlék mentése (egyszerű)
    pub async fn remember_simple(
        &mut self,
        content: &str,
        layer: &str,
    ) -> HopeResult<RememberResponse> {
        self.remember(content, layer, 0.5, "").await
    }

    /// Emlék mentése (teljes)
    pub async fn remember(
        &mut self,
        content: &str,
        layer: &str,
        importance: f64,
        emotional_tag: &str,
    ) -> HopeResult<RememberResponse> {
        let request = RememberRequest {
            content: content.to_string(),
            layer: layer.to_string(),
            importance,
            emotional_tag: emotional_tag.to_string(),
            metadata: HashMap::new(),
        };

        let response = self.memory.remember(request).await?;
        Ok(response.into_inner())
    }

    /// Emlék keresése (egyszerű)
    pub async fn recall_simple(&mut self, query: &str, layer: &str) -> HopeResult<RecallResponse> {
        self.recall(query, layer, 10).await
    }

    /// Emlék keresése (teljes)
    pub async fn recall(
        &mut self,
        query: &str,
        layer: &str,
        limit: i32,
    ) -> HopeResult<RecallResponse> {
        let request = RecallRequest {
            query: query.to_string(),
            layer: layer.to_string(),
            limit,
            min_importance: 0.0,
        };

        let response = self.memory.recall(request).await?;
        Ok(response.into_inner())
    }

    /// Working memory lekérdezése
    pub async fn get_working_memory(&mut self) -> HopeResult<WorkingMemoryResponse> {
        let response = self.memory.get_working_memory(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    // ==================== COGNITIVE SERVICE ====================

    /// Gondolkodás (egyszerű)
    pub async fn think_simple(&mut self, input: &str, deep: bool) -> HopeResult<ThinkResponse> {
        self.think(input, deep, "").await
    }

    /// Gondolkodás (teljes)
    pub async fn think(
        &mut self,
        input: &str,
        deep: bool,
        context: &str,
    ) -> HopeResult<ThinkResponse> {
        let request = ThinkRequest {
            input: input.to_string(),
            deep,
            context: context.to_string(),
            focus_areas: Vec::new(),
        };

        let response = self.cognitive.think(request).await?;
        Ok(response.into_inner())
    }

    /// Érzelmek beállítása (egyszerű)
    pub async fn feel_simple(
        &mut self,
        emotions: HashMap<String, f64>,
    ) -> HopeResult<FeelResponse> {
        self.feel(emotions, "").await
    }

    /// Érzelmek beállítása (teljes)
    pub async fn feel(
        &mut self,
        emotions: HashMap<String, f64>,
        trigger: &str,
    ) -> HopeResult<FeelResponse> {
        let request = FeelRequest {
            emotions,
            trigger: trigger.to_string(),
        };

        let response = self.cognitive.feel(request).await?;
        Ok(response.into_inner())
    }

    /// Kognitív állapot
    pub async fn cognitive_state(&mut self) -> HopeResult<CognitiveStateResponse> {
        let response = self.cognitive.get_cognitive_state(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    // ==================== CODE SERVICE ====================

    /// Kód elemzés (egyszerű)
    pub async fn analyze_code_simple(
        &mut self,
        code: &str,
        language: &str,
    ) -> HopeResult<AnalyzeResponse> {
        self.analyze_code(
            code,
            language,
            &["syntax".to_string(), "security".to_string()],
        )
        .await
    }

    /// Kód elemzés (teljes)
    pub async fn analyze_code(
        &mut self,
        code: &str,
        language: &str,
        checks: &[String],
    ) -> HopeResult<AnalyzeResponse> {
        let request = AnalyzeRequest {
            code: code.to_string(),
            language: language.to_string(),
            checks: checks.to_vec(),
        };

        let response = self.code.analyze(request).await?;
        Ok(response.into_inner())
    }

    /// Kód generálás (egyszerű)
    pub async fn generate_code_simple(
        &mut self,
        description: &str,
        language: &str,
    ) -> HopeResult<GenerateResponse> {
        self.generate_code(description, language, "").await
    }

    /// Kód generálás (teljes)
    pub async fn generate_code(
        &mut self,
        description: &str,
        language: &str,
        template: &str,
    ) -> HopeResult<GenerateResponse> {
        let request = GenerateRequest {
            description: description.to_string(),
            language: language.to_string(),
            template: template.to_string(),
            params: HashMap::new(),
        };

        let response = self.code.generate(request).await?;
        Ok(response.into_inner())
    }

    // ==================== KNOWLEDGE SERVICE ====================

    /// Tudás keresése
    pub async fn query_knowledge(&mut self, query: &str) -> HopeResult<KnowledgeQueryResponse> {
        let request = KnowledgeQueryRequest {
            query: query.to_string(),
            domains: Vec::new(),
            limit: 10,
            threshold: 0.5,
        };

        let response = self.knowledge.query(request).await?;
        Ok(response.into_inner())
    }

    // ==================== GENOME SERVICE ====================

    /// Genome (etika) állapot
    pub async fn genome_status(&mut self) -> HopeResult<GenomeStatusResponse> {
        let response = self.genome.get_status(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Akció ellenőrzése (egyszerű)
    pub async fn genome_verify_action_simple(
        &mut self,
        action_type: &str,
        description: &str,
    ) -> HopeResult<VerifyActionResponse> {
        self.verify_action(action_type, description, HashMap::new())
            .await
    }

    /// Akció ellenőrzése (teljes, MCP kompatibilis alias)
    pub async fn verify_action(
        &mut self,
        action_type: &str,
        description: &str,
        context: HashMap<String, String>,
    ) -> HopeResult<VerifyActionResponse> {
        let request = VerifyActionRequest {
            action_type: action_type.to_string(),
            description: description.to_string(),
            context,
        };

        let response = self.genome.verify_action(request).await?;
        Ok(response.into_inner())
    }

    /// Audit napló
    pub async fn genome_audit_trail(&mut self) -> HopeResult<AuditTrailResponse> {
        let request = GetAuditTrailRequest {
            limit: 100,
            action_type: String::new(),
            since: None,
        };

        let response = self.genome.get_audit_trail(request).await?;
        Ok(response.into_inner())
    }

    /// Etikai szabályok
    pub async fn genome_rules(&mut self) -> HopeResult<RulesResponse> {
        let response = self.genome.get_rules(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    // ==================== VISION SERVICE ====================

    /// Kép feldolgozás (egyszerű)
    pub async fn see_simple(&mut self, image_data: &[u8]) -> HopeResult<SeeResponse> {
        self.see(image_data, "", 0.5).await
    }

    /// Kép feldolgozás (teljes)
    pub async fn see(
        &mut self,
        image_data: &[u8],
        description: &str,
        importance: f64,
    ) -> HopeResult<SeeResponse> {
        let request = SeeRequest {
            image_data: image_data.to_vec(),
            description: description.to_string(),
            context: String::new(),
            importance,
            store_in_memory: true,
            metadata: HashMap::new(),
        };

        let response = self.vision.see(request).await?;
        Ok(response.into_inner())
    }

    /// Vision státusz
    pub async fn vision_status(&mut self) -> HopeResult<VisionStatusResponse> {
        let response = self.vision.get_vision_status(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Vizuális emlékek
    pub async fn visual_memories(&mut self, limit: i32) -> HopeResult<VisualMemoriesResponse> {
        let request = GetVisualMemoriesRequest {
            limit,
            min_importance: 0.0,
            format_filter: String::new(),
            recent_only: false,
        };

        let response = self.vision.get_visual_memories(request).await?;
        Ok(response.into_inner())
    }

    // ==================== RESONANCE SERVICE ====================

    /// Resonance Learn - Tanulás bemenetből
    pub async fn resonance_learn(
        &mut self,
        content: &str,
        session_id: &str,
    ) -> HopeResult<ResonanceLearnResponse> {
        let request = ResonanceLearnRequest {
            content: content.to_string(),
            session_id: session_id.to_string(),
            keystroke_timings: Vec::new(),
            emotional_state: Vec::new(),
        };

        let response = self.resonance.learn(request).await?;
        Ok(response.into_inner())
    }

    /// Resonance Verify - Felhasználó verifikáció
    pub async fn resonance_verify(
        &mut self,
        content: &str,
        session_id: &str,
    ) -> HopeResult<ResonanceVerifyResponse> {
        let input = ResonanceInput {
            content: content.to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            keystroke_timings: Vec::new(),
            emotional_state: Vec::new(),
        };

        let request = ResonanceVerifyRequest {
            session_id: session_id.to_string(),
            inputs: vec![input],
        };

        let response = self.resonance.verify(request).await?;
        Ok(response.into_inner())
    }

    /// Resonance Status - Státusz lekérdezés
    pub async fn resonance_status(&mut self) -> HopeResult<ResonanceStatusResponse> {
        let response = self.resonance.get_status(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    // ==================== GEO SERVICE ====================

    /// Jelenlegi lokáció beállítása
    pub async fn set_location(
        &mut self,
        latitude: f64,
        longitude: f64,
        source: &str,
    ) -> HopeResult<SetLocationResponse> {
        let request = SetLocationRequest {
            latitude,
            longitude,
            altitude: 0.0,
            accuracy: 0.0,
            source: source.to_string(),
        };

        let response = self.geo.set_location(request).await?;
        Ok(response.into_inner())
    }

    /// Jelenlegi lokáció lekérdezése
    pub async fn get_location(&mut self) -> HopeResult<GeoLocationResponse> {
        let response = self.geo.get_location(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Hely hozzáadása
    pub async fn add_place(
        &mut self,
        name: &str,
        place_type: &str,
        latitude: f64,
        longitude: f64,
        radius: f64,
    ) -> HopeResult<AddPlaceResponse> {
        let request = AddPlaceRequest {
            name: name.to_string(),
            place_type: place_type.to_string(),
            latitude,
            longitude,
            radius,
            address: String::new(),
            country_code: String::new(),
        };

        let response = self.geo.add_place(request).await?;
        Ok(response.into_inner())
    }

    /// Helyek listázása
    pub async fn list_places(&mut self) -> HopeResult<ListPlacesResponse> {
        let request = ListPlacesRequest {
            place_type: String::new(),
            nearby_lat: 0.0,
            nearby_lon: 0.0,
            radius_km: 0.0,
            limit: 100,
        };

        let response = self.geo.list_places(request).await?;
        Ok(response.into_inner())
    }

    /// Otthon beállítása
    pub async fn set_home(&mut self, place_id: &str) -> HopeResult<SetHomeResponse> {
        let request = SetHomeRequest {
            place_id: place_id.to_string(),
        };

        let response = self.geo.set_home(request).await?;
        Ok(response.into_inner())
    }

    /// Otthon lekérdezése
    pub async fn get_home(&mut self) -> HopeResult<PlaceResponse> {
        let response = self.geo.get_home(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }

    /// Távolság számítás két pont között
    pub async fn get_distance(
        &mut self,
        lat1: f64,
        lon1: f64,
        lat2: f64,
        lon2: f64,
    ) -> HopeResult<GetDistanceResponse> {
        let request = GetDistanceRequest {
            lat1,
            lon1,
            lat2,
            lon2,
            from_place_id: String::new(),
            to_place_id: String::new(),
            from_current: false,
        };

        let response = self.geo.get_distance(request).await?;
        Ok(response.into_inner())
    }

    /// Geo statisztikák
    pub async fn geo_stats(&mut self) -> HopeResult<GeoStatsResponse> {
        let response = self.geo.get_geo_stats(EmptyRequest {}).await?;
        Ok(response.into_inner())
    }
}

/// Skill információ megjelenítése
impl std::fmt::Display for SkillInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} [{}]",
            self.name, self.description, self.category
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_address() {
        assert_eq!(HopeClient::default_address(), "http://localhost:50051");
    }
}
