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
    genome_service_client::GenomeServiceClient, hope_service_client::HopeServiceClient,
    knowledge_service_client::KnowledgeServiceClient, memory_service_client::MemoryServiceClient,
    skill_service_client::SkillServiceClient, *,
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

    /// Skill meghívása
    pub async fn invoke_skill(
        &mut self,
        name: &str,
        input: &str,
    ) -> HopeResult<InvokeSkillResponse> {
        let request = InvokeSkillRequest {
            name: name.to_string(),
            input: input.to_string(),
            params: HashMap::new(),
        };

        let response = self.skills.invoke_skill(request).await?;
        Ok(response.into_inner())
    }

    // ==================== MEMORY SERVICE ====================

    /// Emlék mentése
    pub async fn remember(&mut self, content: &str, layer: &str) -> HopeResult<RememberResponse> {
        let request = RememberRequest {
            content: content.to_string(),
            layer: layer.to_string(),
            importance: 0.5,
            emotional_tag: String::new(),
            metadata: HashMap::new(),
        };

        let response = self.memory.remember(request).await?;
        Ok(response.into_inner())
    }

    /// Emlék keresése
    pub async fn recall(&mut self, query: &str, layer: &str) -> HopeResult<RecallResponse> {
        let request = RecallRequest {
            query: query.to_string(),
            layer: layer.to_string(),
            limit: 10,
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

    /// Gondolkodás
    pub async fn think(&mut self, input: &str, deep: bool) -> HopeResult<ThinkResponse> {
        let request = ThinkRequest {
            input: input.to_string(),
            deep,
            context: String::new(),
            focus_areas: Vec::new(),
        };

        let response = self.cognitive.think(request).await?;
        Ok(response.into_inner())
    }

    /// Érzelmek beállítása
    pub async fn feel(&mut self, emotions: HashMap<String, f64>) -> HopeResult<FeelResponse> {
        let request = FeelRequest {
            emotions,
            trigger: String::new(),
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

    /// Kód elemzés
    pub async fn analyze_code(
        &mut self,
        code: &str,
        language: &str,
    ) -> HopeResult<AnalyzeResponse> {
        let request = AnalyzeRequest {
            code: code.to_string(),
            language: language.to_string(),
            checks: vec!["syntax".to_string(), "security".to_string()],
        };

        let response = self.code.analyze(request).await?;
        Ok(response.into_inner())
    }

    /// Kód generálás
    pub async fn generate_code(
        &mut self,
        description: &str,
        language: &str,
    ) -> HopeResult<GenerateResponse> {
        let request = GenerateRequest {
            description: description.to_string(),
            language: language.to_string(),
            template: String::new(),
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

    /// Akció ellenőrzése
    pub async fn genome_verify_action(
        &mut self,
        action_type: &str,
        description: &str,
    ) -> HopeResult<VerifyActionResponse> {
        let request = VerifyActionRequest {
            action_type: action_type.to_string(),
            description: description.to_string(),
            context: HashMap::new(),
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
