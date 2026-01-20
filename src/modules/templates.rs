//! Hope Templates - Refactoring & Architecture Templates
//!
//! Sablon gyűjtemény Hope önfejlesztéséhez:
//! - Refactoring Templates: Extract Method, Dict Dispatch, Early Return
//! - Clean Architecture Templates: Repository, Service, Controller
//! - Security Templates: Input Validation, Auth, Rate Limiting
//! - Microservices Templates: Circuit Breaker, Retry, Saga
//!
//! ()=>[] - A tiszta potenciálból a struktúra megszületik

use crate::core::HopeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// TEMPLATE CATEGORY
// ============================================================================

/// Template kategória
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Refactoring minták
    Refactoring,
    /// Clean Architecture
    Architecture,
    /// Biztonsági minták
    Security,
    /// Microservices minták
    Microservices,
    /// Domain minták
    Domain,
    /// Testing minták
    Testing,
    /// Egyéb
    Other,
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateCategory::Refactoring => write!(f, "refactoring"),
            TemplateCategory::Architecture => write!(f, "architecture"),
            TemplateCategory::Security => write!(f, "security"),
            TemplateCategory::Microservices => write!(f, "microservices"),
            TemplateCategory::Domain => write!(f, "domain"),
            TemplateCategory::Testing => write!(f, "testing"),
            TemplateCategory::Other => write!(f, "other"),
        }
    }
}

// ============================================================================
// TEMPLATE
// ============================================================================

/// Template definíció
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    /// Template neve
    pub name: String,
    /// Leírás
    pub description: String,
    /// Kategória
    pub category: TemplateCategory,
    /// Kulcsszavak kereséshez
    pub keywords: Vec<String>,
    /// Template kód (placeholderekkel)
    pub template_code: String,
    /// Paraméterek
    pub parameters: Vec<String>,
    /// Komplexitás (0.0 - 1.0)
    pub complexity: f64,
    /// Programnyelv
    pub language: String,
}

impl Template {
    pub fn new(name: &str, description: &str, category: TemplateCategory) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category,
            keywords: Vec::new(),
            template_code: String::new(),
            parameters: Vec::new(),
            complexity: 0.5,
            language: "rust".to_string(),
        }
    }

    pub fn with_keywords(mut self, keywords: Vec<&str>) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_code(mut self, code: &str) -> Self {
        self.template_code = code.to_string();
        // Extract parameters from {{param}} placeholders
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        self.parameters = re
            .captures_iter(code)
            .map(|c| c.get(1).unwrap().as_str().to_string())
            .collect();
        self
    }

    pub fn with_complexity(mut self, complexity: f64) -> Self {
        self.complexity = complexity;
        self
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    /// Template kód generálása paraméterekkel
    pub fn generate(&self, params: &HashMap<String, String>) -> String {
        let mut code = self.template_code.clone();
        for (key, value) in params {
            code = code.replace(&format!("{{{{{}}}}}", key), value);
        }
        code
    }

    /// Ellenőrzi, hogy minden paraméter meg van-e adva
    pub fn validate_params(&self, params: &HashMap<String, String>) -> Vec<String> {
        self.parameters
            .iter()
            .filter(|p| !params.contains_key(*p))
            .cloned()
            .collect()
    }
}

// ============================================================================
// TEMPLATE ENGINE
// ============================================================================

/// Template Engine
pub struct TemplateEngine {
    templates: HashMap<String, Template>,
}

impl TemplateEngine {
    /// Új engine létrehozása az összes beépített template-tel
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
        };

        // Betöltjük az összes template-et
        for template in refactoring_templates() {
            engine.templates.insert(template.name.clone(), template);
        }
        for template in architecture_templates() {
            engine.templates.insert(template.name.clone(), template);
        }
        for template in security_templates() {
            engine.templates.insert(template.name.clone(), template);
        }
        for template in microservices_templates() {
            engine.templates.insert(template.name.clone(), template);
        }
        for template in testing_templates() {
            engine.templates.insert(template.name.clone(), template);
        }

        engine
    }

    /// Template keresése
    pub fn search(
        &self,
        query: &str,
        category: Option<TemplateCategory>,
        limit: usize,
    ) -> Vec<&Template> {
        let query = query.to_lowercase();
        let mut results: Vec<(&Template, i32)> = self
            .templates
            .values()
            .filter(|t| {
                if let Some(ref cat) = category {
                    if &t.category != cat {
                        return false;
                    }
                }
                true
            })
            .map(|t| {
                let mut score = 0i32;
                if t.name.to_lowercase().contains(&query) {
                    score += 10;
                }
                if t.description.to_lowercase().contains(&query) {
                    score += 5;
                }
                for keyword in &t.keywords {
                    if keyword.to_lowercase().contains(&query) {
                        score += 3;
                    }
                }
                (t, score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().take(limit).map(|(t, _)| t).collect()
    }

    /// Template lekérése név alapján
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Kód generálása template-ből
    pub fn generate(
        &self,
        template_name: &str,
        params: &HashMap<String, String>,
    ) -> HopeResult<String> {
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| format!("Template not found: {}", template_name))?;

        let missing = template.validate_params(params);
        if !missing.is_empty() {
            return Err(format!("Missing parameters: {:?}", missing).into());
        }

        Ok(template.generate(params))
    }

    /// Kategóriák listázása
    pub fn list_categories(&self) -> Vec<(TemplateCategory, usize)> {
        let mut counts: HashMap<TemplateCategory, usize> = HashMap::new();
        for template in self.templates.values() {
            *counts.entry(template.category.clone()).or_insert(0) += 1;
        }
        let mut list: Vec<_> = counts.into_iter().collect();
        list.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
        list
    }

    /// Template-ek listázása
    pub fn list(&self, category: Option<TemplateCategory>) -> Vec<&Template> {
        let mut templates: Vec<&Template> = self
            .templates
            .values()
            .filter(|t| {
                if let Some(ref cat) = category {
                    &t.category == cat
                } else {
                    true
                }
            })
            .collect();

        templates.sort_by(|a, b| a.name.cmp(&b.name));
        templates
    }

    /// Statisztikák
    pub fn stats(&self) -> TemplateEngineStats {
        let categories = self.list_categories();
        TemplateEngineStats {
            total_templates: self.templates.len(),
            categories: categories.into_iter().collect(),
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Template Engine statisztikák
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateEngineStats {
    pub total_templates: usize,
    pub categories: Vec<(TemplateCategory, usize)>,
}

// ============================================================================
// REFACTORING TEMPLATES
// ============================================================================

fn refactoring_templates() -> Vec<Template> {
    vec![
        Template::new(
            "extract_method",
            "Extract Method - Kód kiszervezése külön metódusba",
            TemplateCategory::Refactoring,
        )
        .with_keywords(vec!["extract", "method", "function", "refactor", "dry"])
        .with_code(
            r#"/// {{description}}
fn {{method_name}}(&self, {{params}}) -> {{return_type}} {
    {{extracted_code}}
}"#,
        )
        .with_complexity(0.3),
        Template::new(
            "dict_dispatch",
            "Dictionary Dispatch - If/elif lánc helyett HashMap",
            TemplateCategory::Refactoring,
        )
        .with_keywords(vec!["dispatch", "match", "if", "elif", "hashmap", "lookup"])
        .with_code(
            r#"lazy_static! {
    static ref {{DISPATCH_NAME}}: HashMap<&'static str, fn({{param_type}}) -> {{return_type}}> = {
        let mut m = HashMap::new();
        {{dispatch_entries}}
        m
    };
}

fn {{function_name}}(&self, key: &str, {{params}}) -> Option<{{return_type}}> {
    {{DISPATCH_NAME}}.get(key).map(|f| f({{param_values}}))
}"#,
        )
        .with_complexity(0.5),
        Template::new(
            "early_return",
            "Early Return - Guard clause pattern",
            TemplateCategory::Refactoring,
        )
        .with_keywords(vec!["early", "return", "guard", "clause", "validate"])
        .with_code(
            r#"fn {{function_name}}(&self, {{params}}) -> {{return_type}} {
    // Guard clauses
    if {{condition1}} {
        return {{early_return1}};
    }
    if {{condition2}} {
        return {{early_return2}};
    }

    // Main logic
    {{main_logic}}
}"#,
        )
        .with_complexity(0.2),
        Template::new(
            "builder_pattern",
            "Builder Pattern - Fluent interface objektum építéshez",
            TemplateCategory::Refactoring,
        )
        .with_keywords(vec!["builder", "fluent", "pattern", "construct"])
        .with_code(
            r#"pub struct {{TypeName}}Builder {
    {{fields}}
}

impl {{TypeName}}Builder {
    pub fn new() -> Self {
        Self {
            {{default_fields}}
        }
    }

    {{builder_methods}}

    pub fn build(self) -> {{TypeName}} {
        {{TypeName}} {
            {{build_fields}}
        }
    }
}"#,
        )
        .with_complexity(0.4),
        Template::new(
            "strategy_pattern",
            "Strategy Pattern - Algoritmus cserélhetősége",
            TemplateCategory::Refactoring,
        )
        .with_keywords(vec!["strategy", "pattern", "algorithm", "behavior"])
        .with_code(
            r#"pub trait {{StrategyName}} {
    fn execute(&self, {{params}}) -> {{return_type}};
}

pub struct {{ConcreteStrategy1}};
impl {{StrategyName}} for {{ConcreteStrategy1}} {
    fn execute(&self, {{params}}) -> {{return_type}} {
        {{implementation1}}
    }
}

pub struct {{ConcreteStrategy2}};
impl {{StrategyName}} for {{ConcreteStrategy2}} {
    fn execute(&self, {{params}}) -> {{return_type}} {
        {{implementation2}}
    }
}

pub struct Context {
    strategy: Box<dyn {{StrategyName}}>,
}

impl Context {
    pub fn new(strategy: Box<dyn {{StrategyName}}>) -> Self {
        Self { strategy }
    }

    pub fn execute(&self, {{params}}) -> {{return_type}} {
        self.strategy.execute({{param_values}})
    }
}"#,
        )
        .with_complexity(0.6),
    ]
}

// ============================================================================
// ARCHITECTURE TEMPLATES
// ============================================================================

fn architecture_templates() -> Vec<Template> {
    vec![
        Template::new(
            "repository_pattern",
            "Repository Pattern - Adatelérési réteg",
            TemplateCategory::Architecture,
        )
        .with_keywords(vec!["repository", "data", "access", "crud", "storage"])
        .with_code(
            r#"#[async_trait]
pub trait {{EntityName}}Repository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<{{EntityName}}>, Error>;
    async fn find_all(&self) -> Result<Vec<{{EntityName}}>, Error>;
    async fn save(&self, entity: &{{EntityName}}) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<bool, Error>;
}

pub struct {{EntityName}}RepositoryImpl {
    {{storage_field}}
}

#[async_trait]
impl {{EntityName}}Repository for {{EntityName}}RepositoryImpl {
    async fn find_by_id(&self, id: &str) -> Result<Option<{{EntityName}}>, Error> {
        {{find_by_id_impl}}
    }

    async fn find_all(&self) -> Result<Vec<{{EntityName}}>, Error> {
        {{find_all_impl}}
    }

    async fn save(&self, entity: &{{EntityName}}) -> Result<(), Error> {
        {{save_impl}}
    }

    async fn delete(&self, id: &str) -> Result<bool, Error> {
        {{delete_impl}}
    }
}"#,
        )
        .with_complexity(0.5),
        Template::new(
            "service_layer",
            "Service Layer - Üzleti logika réteg",
            TemplateCategory::Architecture,
        )
        .with_keywords(vec!["service", "business", "logic", "layer"])
        .with_code(
            r#"pub struct {{ServiceName}} {
    repository: Arc<dyn {{EntityName}}Repository>,
    {{dependencies}}
}

impl {{ServiceName}} {
    pub fn new(
        repository: Arc<dyn {{EntityName}}Repository>,
        {{dependency_params}}
    ) -> Self {
        Self {
            repository,
            {{dependency_fields}}
        }
    }

    pub async fn create(&self, input: {{CreateInput}}) -> Result<{{EntityName}}, Error> {
        // Validation
        {{validation}}

        // Business logic
        {{business_logic}}

        // Persistence
        let entity = {{EntityName}}::new({{entity_params}});
        self.repository.save(&entity).await?;

        Ok(entity)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<{{EntityName}}>, Error> {
        self.repository.find_by_id(id).await
    }

    {{additional_methods}}
}"#,
        )
        .with_complexity(0.6),
        Template::new(
            "event_sourcing",
            "Event Sourcing - Események mint igazság forrása",
            TemplateCategory::Architecture,
        )
        .with_keywords(vec!["event", "sourcing", "cqrs", "aggregate"])
        .with_code(
            r#"#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum {{AggregateName}}Event {
    Created { id: String, {{created_fields}} },
    Updated { id: String, {{updated_fields}} },
    Deleted { id: String },
}

pub struct {{AggregateName}} {
    id: String,
    version: u64,
    {{state_fields}}
}

impl {{AggregateName}} {
    pub fn apply(&mut self, event: {{AggregateName}}Event) {
        match event {
            {{AggregateName}}Event::Created { id, {{created_fields}} } => {
                self.id = id;
                {{apply_created}}
            }
            {{AggregateName}}Event::Updated { id, {{updated_fields}} } => {
                {{apply_updated}}
            }
            {{AggregateName}}Event::Deleted { id } => {
                {{apply_deleted}}
            }
        }
        self.version += 1;
    }

    pub fn replay(events: Vec<{{AggregateName}}Event>) -> Self {
        let mut aggregate = Self::default();
        for event in events {
            aggregate.apply(event);
        }
        aggregate
    }
}"#,
        )
        .with_complexity(0.8),
    ]
}

// ============================================================================
// SECURITY TEMPLATES
// ============================================================================

fn security_templates() -> Vec<Template> {
    vec![
        Template::new(
            "input_validation",
            "Input Validation - Bemenet validálás",
            TemplateCategory::Security,
        )
        .with_keywords(vec!["input", "validation", "sanitize", "security"])
        .with_code(
            r#"#[derive(Debug, Validate)]
pub struct {{InputName}} {
    #[validate(length(min = {{min_length}}, max = {{max_length}}))]
    pub {{field1}}: String,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(range(min = {{min_value}}, max = {{max_value}}))]
    pub {{field2}}: i32,

    #[validate(custom = "{{custom_validator}}")]
    pub {{field3}}: {{Field3Type}},
}

impl {{InputName}} {
    pub fn validate_and_sanitize(&mut self) -> Result<(), ValidationError> {
        // Trim strings
        self.{{field1}} = self.{{field1}}.trim().to_string();

        // Validate
        self.validate()?;

        // Additional sanitization
        {{sanitization}}

        Ok(())
    }
}"#,
        )
        .with_complexity(0.4),
        Template::new(
            "rate_limiter",
            "Rate Limiter - Kérés korlátozás",
            TemplateCategory::Security,
        )
        .with_keywords(vec!["rate", "limit", "throttle", "security"])
        .with_code(
            r#"pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    pub async fn check(&self, key: &str) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        let mut requests = self.requests.write().await;
        let entry = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old requests
        entry.retain(|t| now.duration_since(*t) < window);

        if entry.len() >= self.max_requests {
            return Err(RateLimitError::TooManyRequests);
        }

        entry.push(now);
        Ok(())
    }
}"#,
        )
        .with_complexity(0.5),
        Template::new(
            "auth_middleware",
            "Auth Middleware - Authentikációs middleware",
            TemplateCategory::Security,
        )
        .with_keywords(vec!["auth", "authentication", "middleware", "jwt", "token"])
        .with_code(
            r#"pub struct AuthMiddleware {
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: &str) -> Self {
        Self {
            jwt_secret: jwt_secret.to_string(),
        }
    }

    pub async fn authenticate(&self, token: &str) -> Result<Claims, AuthError> {
        // Decode and validate JWT
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        ).map_err(|_| AuthError::InvalidToken)?;

        // Check expiration
        if token_data.claims.exp < Utc::now().timestamp() as usize {
            return Err(AuthError::TokenExpired);
        }

        Ok(token_data.claims)
    }

    pub fn generate_token(&self, user_id: &str, roles: Vec<String>) -> Result<String, AuthError> {
        let claims = Claims {
            sub: user_id.to_string(),
            roles,
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        ).map_err(|_| AuthError::TokenGenerationFailed)
    }
}"#,
        )
        .with_complexity(0.6),
    ]
}

// ============================================================================
// MICROSERVICES TEMPLATES
// ============================================================================

fn microservices_templates() -> Vec<Template> {
    vec![
        Template::new(
            "circuit_breaker",
            "Circuit Breaker - Szolgáltatás védelem",
            TemplateCategory::Microservices,
        )
        .with_keywords(vec!["circuit", "breaker", "resilience", "fault", "tolerance"])
        .with_code(
            r#"pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

#[derive(Clone, Copy, PartialEq)]
enum CircuitState {
    Closed { failures: u32 },
    Open { opened_at: Instant },
    HalfOpen { successes: u32 },
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed { failures: 0 })),
            failure_threshold,
            success_threshold,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check state
        {
            let mut state = self.state.write().await;
            match *state {
                CircuitState::Open { opened_at } => {
                    if opened_at.elapsed() >= self.timeout {
                        *state = CircuitState::HalfOpen { successes: 0 };
                    } else {
                        return Err(CircuitError::Open);
                    }
                }
                _ => {}
            }
        }

        // Execute
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(CircuitError::Inner(e))
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::HalfOpen { successes } => {
                if successes + 1 >= self.success_threshold {
                    *state = CircuitState::Closed { failures: 0 };
                } else {
                    *state = CircuitState::HalfOpen { successes: successes + 1 };
                }
            }
            CircuitState::Closed { .. } => {
                *state = CircuitState::Closed { failures: 0 };
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed { failures } => {
                if failures + 1 >= self.failure_threshold {
                    *state = CircuitState::Open { opened_at: Instant::now() };
                } else {
                    *state = CircuitState::Closed { failures: failures + 1 };
                }
            }
            CircuitState::HalfOpen { .. } => {
                *state = CircuitState::Open { opened_at: Instant::now() };
            }
            _ => {}
        }
    }
}"#,
        )
        .with_complexity(0.7),

        Template::new(
            "retry_pattern",
            "Retry Pattern - Újrapróbálkozás exponenciális backoff-fal",
            TemplateCategory::Microservices,
        )
        .with_keywords(vec!["retry", "backoff", "exponential", "resilience"])
        .with_code(
            r#"pub struct RetryPolicy {
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl RetryPolicy {
    pub fn new(max_retries: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            initial_delay: Duration::from_millis(initial_delay_ms),
            max_delay: Duration::from_millis(max_delay_ms),
            multiplier: 2.0,
        }
    }

    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut delay = self.initial_delay;

        for attempt in 0..=self.max_retries {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == self.max_retries {
                        return Err(e);
                    }

                    // Add jitter
                    let jitter = rand::thread_rng().gen_range(0..100);
                    let sleep_time = delay + Duration::from_millis(jitter);

                    tokio::time::sleep(sleep_time).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        Duration::from_secs_f64(delay.as_secs_f64() * self.multiplier),
                        self.max_delay,
                    );
                }
            }
        }

        unreachable!()
    }
}"#,
        )
        .with_complexity(0.5),

        Template::new(
            "saga_pattern",
            "Saga Pattern - Elosztott tranzakció kezelés",
            TemplateCategory::Microservices,
        )
        .with_keywords(vec!["saga", "transaction", "distributed", "compensation"])
        .with_code(
            r#"#[async_trait]
pub trait SagaStep: Send + Sync {
    async fn execute(&self, context: &mut SagaContext) -> Result<(), SagaError>;
    async fn compensate(&self, context: &mut SagaContext) -> Result<(), SagaError>;
    fn name(&self) -> &str;
}

pub struct Saga {
    steps: Vec<Box<dyn SagaStep>>,
}

impl Saga {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(mut self, step: Box<dyn SagaStep>) -> Self {
        self.steps.push(step);
        self
    }

    pub async fn execute(&self, context: &mut SagaContext) -> Result<(), SagaError> {
        let mut completed_steps = Vec::new();

        for step in &self.steps {
            match step.execute(context).await {
                Ok(()) => {
                    completed_steps.push(step);
                }
                Err(e) => {
                    // Compensate in reverse order
                    for completed in completed_steps.iter().rev() {
                        if let Err(comp_err) = completed.compensate(context).await {
                            // Log compensation error but continue
                            eprintln!("Compensation failed for {}: {:?}", completed.name(), comp_err);
                        }
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}"#,
        )
        .with_complexity(0.8),
    ]
}

// ============================================================================
// TESTING TEMPLATES
// ============================================================================

fn testing_templates() -> Vec<Template> {
    vec![
        Template::new(
            "unit_test",
            "Unit Test - Egységteszt template",
            TemplateCategory::Testing,
        )
        .with_keywords(vec!["test", "unit", "assert"])
        .with_code(
            r#"#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{{test_name}}() {
        // Arrange
        {{arrange}}

        // Act
        let result = {{act}};

        // Assert
        {{assert}}
    }

    #[test]
    fn test_{{test_name}}_edge_case() {
        {{edge_case_test}}
    }

    #[test]
    #[should_panic(expected = "{{expected_panic}}")]
    fn test_{{test_name}}_panic() {
        {{panic_test}}
    }
}"#,
        )
        .with_complexity(0.3),
        Template::new(
            "async_test",
            "Async Test - Aszinkron teszt template",
            TemplateCategory::Testing,
        )
        .with_keywords(vec!["test", "async", "tokio"])
        .with_code(
            r#"#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_{{test_name}}() {
        // Arrange
        {{arrange}}

        // Act
        let result = {{act}}.await;

        // Assert
        {{assert}}
    }

    #[tokio::test]
    async fn test_{{test_name}}_concurrent() {
        let handles: Vec<_> = (0..{{concurrency}})
            .map(|i| {
                tokio::spawn(async move {
                    {{concurrent_test}}
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }
    }
}"#,
        )
        .with_complexity(0.4),
        Template::new(
            "mock_object",
            "Mock Object - Mock objektum generálás",
            TemplateCategory::Testing,
        )
        .with_keywords(vec!["mock", "test", "fake", "stub"])
        .with_code(
            r#"#[cfg(test)]
pub struct Mock{{TraitName}} {
    {{mock_fields}}
}

#[cfg(test)]
impl Mock{{TraitName}} {
    pub fn new() -> Self {
        Self {
            {{mock_defaults}}
        }
    }

    pub fn with_{{method_name}}_returning(mut self, value: {{ReturnType}}) -> Self {
        self.{{method_name}}_result = Some(value);
        self
    }
}

#[cfg(test)]
#[async_trait]
impl {{TraitName}} for Mock{{TraitName}} {
    async fn {{method_name}}(&self, {{params}}) -> {{ReturnType}} {
        self.{{method_name}}_calls.fetch_add(1, Ordering::SeqCst);
        self.{{method_name}}_result.clone().unwrap_or_default()
    }
}"#,
        )
        .with_complexity(0.5),
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = Template::new("test", "Test template", TemplateCategory::Refactoring)
            .with_keywords(vec!["test", "example"])
            .with_code("fn {{name}}() { {{body}} }")
            .with_complexity(0.3);

        assert_eq!(template.name, "test");
        assert_eq!(template.keywords.len(), 2);
        assert_eq!(template.parameters, vec!["name", "body"]);
    }

    #[test]
    fn test_template_generation() {
        let template = Template::new("test", "Test", TemplateCategory::Refactoring)
            .with_code("fn {{name}}() { println!(\"{{message}}\"); }");

        let mut params = HashMap::new();
        params.insert("name".to_string(), "hello".to_string());
        params.insert("message".to_string(), "Hello World".to_string());

        let code = template.generate(&params);
        assert!(code.contains("fn hello()"));
        assert!(code.contains("Hello World"));
    }

    #[test]
    fn test_param_validation() {
        let template = Template::new("test", "Test", TemplateCategory::Refactoring)
            .with_code("{{a}} {{b}} {{c}}");

        let mut params = HashMap::new();
        params.insert("a".to_string(), "1".to_string());

        let missing = template.validate_params(&params);
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"b".to_string()));
        assert!(missing.contains(&"c".to_string()));
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        let stats = engine.stats();

        assert!(stats.total_templates > 0);
    }

    #[test]
    fn test_template_search() {
        let engine = TemplateEngine::new();

        let results = engine.search("extract", None, 5);
        assert!(!results.is_empty());
        assert!(results.iter().any(|t| t.name.contains("extract")));
    }

    #[test]
    fn test_template_category_filter() {
        let engine = TemplateEngine::new();

        let security = engine.list(Some(TemplateCategory::Security));
        assert!(security
            .iter()
            .all(|t| t.category == TemplateCategory::Security));
    }

    #[test]
    fn test_template_generate() {
        let engine = TemplateEngine::new();

        let mut params = HashMap::new();
        params.insert("test_name".to_string(), "my_function".to_string());
        params.insert("arrange".to_string(), "let x = 1;".to_string());
        params.insert("act".to_string(), "my_function(x)".to_string());
        params.insert("assert".to_string(), "assert_eq!(result, 2);".to_string());
        params.insert("edge_case_test".to_string(), "// edge case".to_string());
        params.insert("expected_panic".to_string(), "error".to_string());
        params.insert("panic_test".to_string(), "panic!(\"error\")".to_string());

        let result = engine.generate("unit_test", &params);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test_my_function"));
    }

    #[test]
    fn test_categories_listing() {
        let engine = TemplateEngine::new();
        let categories = engine.list_categories();

        assert!(!categories.is_empty());
        assert!(categories
            .iter()
            .any(|(cat, _)| *cat == TemplateCategory::Refactoring));
    }
}
