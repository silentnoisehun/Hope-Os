//! Hope Code DNA - Evolúciós Kód Rendszer
//!
//! Minden kódnak "genetikája" van:
//! - Gene: atomi kód egység (function, class, pattern)
//! - Chromosome: gének gyűjteménye (modul)
//! - Mutation: kis változtatások
//! - Crossover: megoldások kombinálása
//! - Selection: ami működik, túlél
//!
//! ()=>[] - A tiszta potenciálból az evolúció megszületik

use crate::core::HopeResult;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// GENE - Atomi kód egység
// ============================================================================

/// Gén típus
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneType {
    /// Függvény
    Function,
    /// Osztály/Struct
    Class,
    /// Pattern (list comp, lambda, stb)
    Pattern,
    /// Kód snippet
    Snippet,
    /// Template
    Template,
}

impl std::fmt::Display for GeneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneType::Function => write!(f, "function"),
            GeneType::Class => write!(f, "class"),
            GeneType::Pattern => write!(f, "pattern"),
            GeneType::Snippet => write!(f, "snippet"),
            GeneType::Template => write!(f, "template"),
        }
    }
}

/// Gén tulajdonságok (traits)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeneTraits {
    /// Sebesség (0.0 - 1.0)
    pub speed: f64,
    /// Olvashatóság (0.0 - 1.0)
    pub readability: f64,
    /// Biztonság (0.0 - 1.0)
    pub safety: f64,
    /// Komplexitás (0.0 - 1.0)
    pub complexity: f64,
    /// Dokumentált (0.0 vagy 1.0)
    pub documented: f64,
    /// Típusozott (0.0 vagy 1.0)
    pub typed: f64,
}

impl GeneTraits {
    /// Átlagos fitness számítás
    pub fn average_fitness(&self) -> f64 {
        (self.speed + self.readability + self.safety + (1.0 - self.complexity) + self.documented + self.typed) / 6.0
    }
}

/// Gén - atomi kód egység
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gene {
    /// Egyedi azonosító (hash)
    pub gene_id: String,
    /// Gén típusa
    pub gene_type: GeneType,
    /// A kód maga
    pub code: String,
    /// Tulajdonságok
    pub traits: GeneTraits,
    /// Fitness érték (0.0 - 1.0)
    pub fitness: f64,
    /// Generáció száma
    pub generation: u32,
    /// Szülő gének ID-i
    pub parent_ids: Vec<String>,
    /// Létrehozás ideje
    pub created_at: f64,
    /// Sikeres futások száma
    pub success_count: u32,
    /// Összes futás száma
    pub total_runs: u32,
}

impl Gene {
    /// Új gén létrehozása
    pub fn new(gene_type: GeneType, code: &str) -> Self {
        let gene_id = Self::hash_code(code);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            gene_id,
            gene_type,
            code: code.to_string(),
            traits: GeneTraits::default(),
            fitness: 0.5,
            generation: 0,
            parent_ids: Vec::new(),
            created_at,
            success_count: 0,
            total_runs: 0,
        }
    }

    /// Kód hash generálás
    pub fn hash_code(code: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("GEN_{:012X}", hasher.finish())
    }

    /// Tulajdonságok elemzése a kódból
    pub fn analyze_traits(&mut self) {
        let lines: Vec<&str> = self.code.lines().collect();
        let line_count = lines.len();

        // Komplexitás - if/for/while/match száma
        let complexity_count = self.code.matches("if ").count()
            + self.code.matches("for ").count()
            + self.code.matches("while ").count()
            + self.code.matches("match ").count();
        self.traits.complexity = (complexity_count as f64 / 10.0).min(1.0);

        // Olvashatóság - átlagos sor hossz
        let avg_line_len: f64 = if line_count > 0 {
            lines.iter().map(|l| l.len() as f64).sum::<f64>() / line_count as f64
        } else {
            0.0
        };
        self.traits.readability = 1.0 - (avg_line_len / 100.0).min(1.0);

        // Dokumentált - van-e /// vagy //!
        self.traits.documented = if self.code.contains("///") || self.code.contains("//!") {
            1.0
        } else {
            0.0
        };

        // Típusozott - van-e típus annotáció
        self.traits.typed = if self.code.contains("->") || self.code.contains(": ") {
            1.0
        } else {
            0.0
        };

        // Sebesség és biztonság default
        self.traits.speed = 0.5;
        self.traits.safety = 0.5;
    }

    /// Fitness frissítése futási eredmény alapján
    pub fn update_fitness(&mut self, execution_time_ms: f64, success: bool, quality: f64) {
        self.total_runs += 1;
        if success {
            self.success_count += 1;
        }

        // Sebesség pont (gyorsabb = jobb)
        let speed_score = 1.0 / (1.0 + execution_time_ms / 100.0);
        self.traits.speed = self.traits.speed * 0.7 + speed_score * 0.3;

        // Biztonság (sikeres futás = biztonságos)
        let success_rate = self.success_count as f64 / self.total_runs as f64;
        self.traits.safety = success_rate;

        // Össz fitness
        let new_fitness = speed_score * 0.3 + (if success { 1.0 } else { 0.0 }) * 0.4 + quality * 0.3;
        self.fitness = self.fitness * 0.7 + new_fitness * 0.3;
    }
}

// ============================================================================
// CHROMOSOME - Gének gyűjteménye
// ============================================================================

/// Kromoszóma - gének gyűjteménye (pl. egy modul)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chromosome {
    /// Egyedi azonosító
    pub chromosome_id: String,
    /// Név (pl. modul név)
    pub name: String,
    /// Gének listája
    pub genes: Vec<Gene>,
    /// Össz fitness
    pub fitness: f64,
    /// Generáció
    pub generation: u32,
    /// Létrehozás ideje
    pub created_at: f64,
}

impl Chromosome {
    pub fn new(name: &str) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            chromosome_id: format!("CHR_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
            name: name.to_string(),
            genes: Vec::new(),
            fitness: 0.5,
            generation: 0,
            created_at,
        }
    }

    /// Gén hozzáadása
    pub fn add_gene(&mut self, gene: Gene) {
        self.genes.push(gene);
        self.recalculate_fitness();
    }

    /// Fitness újraszámítása
    pub fn recalculate_fitness(&mut self) {
        if self.genes.is_empty() {
            self.fitness = 0.5;
            return;
        }
        self.fitness = self.genes.iter().map(|g| g.fitness).sum::<f64>() / self.genes.len() as f64;
    }
}

// ============================================================================
// MUTATION TYPES
// ============================================================================

/// Mutáció típusok
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MutationType {
    /// Pont mutáció - kis változtatás (pl. változó átnevezés)
    Point,
    /// Insertion - új elem hozzáadása
    Insertion,
    /// Deletion - elem törlése
    Deletion,
    /// Optimalizáció - kód egyszerűsítés
    Optimization,
}

// ============================================================================
// CODE DNA ENGINE
// ============================================================================

/// Code DNA konfiguráció
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeDnaConfig {
    /// Mutációs ráta (0.0 - 1.0)
    pub mutation_rate: f64,
    /// Crossover ráta (0.0 - 1.0)
    pub crossover_rate: f64,
    /// Tournament méret szelekciónál
    pub tournament_size: usize,
    /// Max gén pool méret
    pub max_pool_size: usize,
    /// Min fitness a túléléshez
    pub min_fitness_threshold: f64,
}

impl Default for CodeDnaConfig {
    fn default() -> Self {
        Self {
            mutation_rate: 0.1,
            crossover_rate: 0.3,
            tournament_size: 3,
            max_pool_size: 1000,
            min_fitness_threshold: 0.2,
        }
    }
}

/// Code DNA Engine - Evolúciós kód rendszer
pub struct CodeDna {
    /// Konfiguráció
    config: CodeDnaConfig,
    /// Gén pool
    gene_pool: Arc<RwLock<HashMap<String, Gene>>>,
    /// Kromoszómák
    chromosomes: Arc<RwLock<HashMap<String, Chromosome>>>,
    /// Statisztikák
    stats: Arc<RwLock<CodeDnaStats>>,
}

/// Code DNA statisztikák
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeDnaStats {
    /// Összes gén
    pub total_genes: usize,
    /// Összes kromoszóma
    pub total_chromosomes: usize,
    /// Evolúciós generációk száma
    pub generations_evolved: u32,
    /// Legjobb fitness
    pub best_fitness: f64,
    /// Mutációk száma
    pub mutations: u32,
    /// Crossover-ek száma
    pub crossovers: u32,
    /// Sikeres mutációk
    pub successful_mutations: u32,
}

impl CodeDna {
    /// Új Code DNA engine létrehozása
    pub fn new() -> Self {
        Self::with_config(CodeDnaConfig::default())
    }

    /// Code DNA engine létrehozása konfigurációval
    pub fn with_config(config: CodeDnaConfig) -> Self {
        Self {
            config,
            gene_pool: Arc::new(RwLock::new(HashMap::new())),
            chromosomes: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CodeDnaStats::default())),
        }
    }

    // === GENE EXTRACTION ===

    /// Gének kinyerése Rust kódból
    pub async fn extract_genes(&self, code: &str, source: &str) -> HopeResult<Vec<Gene>> {
        let mut genes = Vec::new();

        // Function extraction (fn név(...))
        let fn_pattern = regex::Regex::new(r"(?s)((?:pub\s+)?(?:async\s+)?fn\s+\w+[^{]+\{[^}]*\})").unwrap();
        for cap in fn_pattern.captures_iter(code) {
            let func_code = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let mut gene = Gene::new(GeneType::Function, func_code);
            gene.analyze_traits();
            genes.push(gene.clone());

            // Add to pool
            let mut pool = self.gene_pool.write().await;
            pool.insert(gene.gene_id.clone(), gene);
        }

        // Struct extraction
        let struct_pattern = regex::Regex::new(r"(?s)((?:pub\s+)?struct\s+\w+[^{]*\{[^}]*\})").unwrap();
        for cap in struct_pattern.captures_iter(code) {
            let struct_code = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let mut gene = Gene::new(GeneType::Class, struct_code);
            gene.analyze_traits();
            genes.push(gene.clone());

            let mut pool = self.gene_pool.write().await;
            pool.insert(gene.gene_id.clone(), gene);
        }

        // Impl block extraction
        let impl_pattern = regex::Regex::new(r"(?s)(impl(?:<[^>]+>)?\s+\w+[^{]*\{[^}]*\})").unwrap();
        for cap in impl_pattern.captures_iter(code) {
            let impl_code = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let mut gene = Gene::new(GeneType::Class, impl_code);
            gene.analyze_traits();
            genes.push(gene.clone());

            let mut pool = self.gene_pool.write().await;
            pool.insert(gene.gene_id.clone(), gene);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_genes = self.gene_pool.read().await.len();
        }

        Ok(genes)
    }

    // === GENETIC OPERATIONS ===

    /// Gén mutációja
    pub async fn mutate(&self, gene: &Gene) -> HopeResult<Gene> {
        let mut rng = rand::thread_rng();

        // Mutáció esélye
        if rng.gen::<f64>() > self.config.mutation_rate {
            return Ok(gene.clone());
        }

        let mutation_type = match rng.gen_range(0..4) {
            0 => MutationType::Point,
            1 => MutationType::Insertion,
            2 => MutationType::Deletion,
            _ => MutationType::Optimization,
        };

        let mutated_code = match mutation_type {
            MutationType::Point => self.point_mutation(&gene.code),
            MutationType::Insertion => self.insertion_mutation(&gene.code),
            MutationType::Deletion => self.deletion_mutation(&gene.code),
            MutationType::Optimization => self.optimization_mutation(&gene.code),
        };

        // Új gén létrehozása
        let mut new_gene = Gene::new(gene.gene_type.clone(), &mutated_code);
        new_gene.generation = gene.generation + 1;
        new_gene.parent_ids = vec![gene.gene_id.clone()];
        new_gene.analyze_traits();

        // Stats
        {
            let mut stats = self.stats.write().await;
            stats.mutations += 1;
        }

        // Add to pool
        {
            let mut pool = self.gene_pool.write().await;
            pool.insert(new_gene.gene_id.clone(), new_gene.clone());
        }

        Ok(new_gene)
    }

    /// Pont mutáció - változó átnevezés
    fn point_mutation(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Egyszerű változó átnevezés pattern
        let var_pattern = regex::Regex::new(r"\b([a-z_][a-z0-9_]*)\b").unwrap();
        if let Some(cap) = var_pattern.captures(&result) {
            if let Some(m) = cap.get(1) {
                let old_var = m.as_str();
                if !["self", "true", "false", "let", "mut", "fn", "if", "else", "for", "while", "return", "pub", "struct", "impl", "use", "mod"].contains(&old_var) {
                    let new_var = format!("{}_v2", old_var);
                    result = result.replace(old_var, &new_var);
                }
            }
        }

        result
    }

    /// Insertion mutáció - komment hozzáadása
    fn insertion_mutation(&self, code: &str) -> String {
        let mut lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
        if lines.len() > 2 {
            let mut rng = rand::thread_rng();
            let pos = rng.gen_range(1..lines.len());
            let indent = lines[pos].len() - lines[pos].trim_start().len();
            let comment = format!("{}// TODO: Review this section", " ".repeat(indent));
            lines.insert(pos, comment);
        }
        lines.join("\n")
    }

    /// Deletion mutáció - üres sorok törlése
    fn deletion_mutation(&self, code: &str) -> String {
        code.lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Optimalizációs mutáció
    fn optimization_mutation(&self, code: &str) -> String {
        let mut result = code.to_string();

        // Redundáns return eltávolítása
        result = result.replace("return;", "");

        // == true eltávolítása
        result = result.replace("== true", "");

        // == false -> !
        result = result.replace("== false", "!");

        result
    }

    /// Crossover - két gén kombinálása
    pub async fn crossover(&self, gene1: &Gene, gene2: &Gene) -> HopeResult<Gene> {
        let mut rng = rand::thread_rng();

        // Crossover esélye
        if rng.gen::<f64>() > self.config.crossover_rate {
            return Ok(gene1.clone());
        }

        // Csak azonos típusú géneket kombinálunk
        if gene1.gene_type != gene2.gene_type {
            return Ok(gene1.clone());
        }

        // Line-based crossover
        let lines1: Vec<&str> = gene1.code.lines().collect();
        let lines2: Vec<&str> = gene2.code.lines().collect();

        let crossover_point1 = lines1.len() / 2;
        let crossover_point2 = lines2.len() / 2;

        let mut new_lines = Vec::new();
        new_lines.extend(lines1[..crossover_point1].iter().copied());
        new_lines.extend(lines2[crossover_point2..].iter().copied());

        let new_code = new_lines.join("\n");

        // Új gén
        let mut new_gene = Gene::new(gene1.gene_type.clone(), &new_code);
        new_gene.generation = gene1.generation.max(gene2.generation) + 1;
        new_gene.parent_ids = vec![gene1.gene_id.clone(), gene2.gene_id.clone()];
        new_gene.fitness = (gene1.fitness + gene2.fitness) / 2.0;
        new_gene.analyze_traits();

        // Stats
        {
            let mut stats = self.stats.write().await;
            stats.crossovers += 1;
        }

        // Add to pool
        {
            let mut pool = self.gene_pool.write().await;
            pool.insert(new_gene.gene_id.clone(), new_gene.clone());
        }

        Ok(new_gene)
    }

    /// Tournament selection
    pub async fn select(&self, population: &[Gene], count: usize) -> Vec<Gene> {
        let mut rng = rand::thread_rng();
        let mut selected = Vec::new();

        for _ in 0..count {
            // Tournament
            let tournament_size = self.config.tournament_size.min(population.len());
            let mut tournament = Vec::new();

            for _ in 0..tournament_size {
                let idx = rng.gen_range(0..population.len());
                tournament.push(&population[idx]);
            }

            // Winner = highest fitness
            if let Some(winner) = tournament.iter().max_by(|a, b| {
                a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                selected.push((*winner).clone());
            }
        }

        selected
    }

    // === EVOLUTION ===

    /// Populáció evolúciója
    pub async fn evolve(&self, generations: u32) -> HopeResult<EvolutionResult> {
        let mut pool = self.gene_pool.write().await;

        if pool.len() < 2 {
            return Ok(EvolutionResult {
                generations_run: 0,
                mutations: 0,
                crossovers: 0,
                improvements: 0,
                best_fitness: 0.0,
            });
        }

        let mut population: Vec<Gene> = pool.values().cloned().collect();
        let initial_size = population.len();
        let mut result = EvolutionResult::default();

        drop(pool); // Release lock

        for gen in 0..generations {
            // Selection
            let selected = self.select(&population, population.len() / 2).await;

            // Crossover
            let mut offspring = Vec::new();
            for i in (0..selected.len()).step_by(2) {
                if i + 1 < selected.len() {
                    let child = self.crossover(&selected[i], &selected[i + 1]).await?;
                    if child.gene_id != selected[i].gene_id {
                        result.crossovers += 1;
                    }
                    offspring.push(child);
                }
            }

            // Mutation
            let mut mutated = Vec::new();
            for gene in &offspring {
                let mutant = self.mutate(gene).await?;
                if mutant.gene_id != gene.gene_id {
                    result.mutations += 1;
                }
                mutated.push(mutant);
            }

            // Combine and select best
            population.extend(mutated);
            population = self.select(&population, initial_size).await;

            // Track improvements
            if let Some(best) = population.iter().max_by(|a, b| {
                a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                if best.fitness > result.best_fitness {
                    result.best_fitness = best.fitness;
                    result.improvements += 1;
                }
            }

            result.generations_run += 1;
        }

        // Update pool with evolved population
        {
            let mut pool = self.gene_pool.write().await;
            for gene in population {
                pool.insert(gene.gene_id.clone(), gene);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.generations_evolved += result.generations_run;
            if result.best_fitness > stats.best_fitness {
                stats.best_fitness = result.best_fitness;
            }
        }

        Ok(result)
    }

    // === QUERIES ===

    /// Legjobb gének lekérése
    pub async fn get_best_genes(&self, gene_type: Option<GeneType>, limit: usize) -> Vec<Gene> {
        let pool = self.gene_pool.read().await;
        let mut genes: Vec<Gene> = pool.values().cloned().collect();

        if let Some(gt) = gene_type {
            genes.retain(|g| g.gene_type == gt);
        }

        genes.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
        genes.truncate(limit);
        genes
    }

    /// Gén keresése ID alapján
    pub async fn get_gene(&self, gene_id: &str) -> Option<Gene> {
        let pool = self.gene_pool.read().await;
        pool.get(gene_id).cloned()
    }

    /// Statisztikák
    pub async fn get_stats(&self) -> CodeDnaStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Awareness
    pub async fn awareness(&self) -> HashMap<String, serde_json::Value> {
        let stats = self.get_stats().await;
        let best_genes = self.get_best_genes(None, 5).await;

        let mut map = HashMap::new();
        map.insert("total_genes".to_string(), serde_json::json!(stats.total_genes));
        map.insert("generations_evolved".to_string(), serde_json::json!(stats.generations_evolved));
        map.insert("best_fitness".to_string(), serde_json::json!(stats.best_fitness));
        map.insert("mutations".to_string(), serde_json::json!(stats.mutations));
        map.insert("crossovers".to_string(), serde_json::json!(stats.crossovers));
        map.insert("best_genes".to_string(), serde_json::json!(
            best_genes.iter().map(|g| serde_json::json!({
                "id": g.gene_id,
                "type": g.gene_type.to_string(),
                "fitness": g.fitness
            })).collect::<Vec<_>>()
        ));
        map
    }
}

impl Default for CodeDna {
    fn default() -> Self {
        Self::new()
    }
}

/// Evolúció eredménye
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EvolutionResult {
    /// Lefutott generációk
    pub generations_run: u32,
    /// Mutációk száma
    pub mutations: u32,
    /// Crossover-ek száma
    pub crossovers: u32,
    /// Javulások száma
    pub improvements: u32,
    /// Legjobb fitness
    pub best_fitness: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gene_creation() {
        let code = "fn hello() { println!(\"Hello\"); }";
        let gene = Gene::new(GeneType::Function, code);

        assert!(!gene.gene_id.is_empty());
        assert_eq!(gene.gene_type, GeneType::Function);
        assert_eq!(gene.code, code);
        assert_eq!(gene.generation, 0);
    }

    #[test]
    fn test_gene_traits_analysis() {
        let code = r#"
/// Documented function
fn complex_function(x: i32) -> i32 {
    if x > 0 {
        for i in 0..x {
            if i % 2 == 0 {
                return i;
            }
        }
    }
    x
}
"#;
        let mut gene = Gene::new(GeneType::Function, code);
        gene.analyze_traits();

        assert!(gene.traits.documented > 0.0);
        assert!(gene.traits.typed > 0.0);
        assert!(gene.traits.complexity > 0.0);
    }

    #[test]
    fn test_chromosome() {
        let mut chromosome = Chromosome::new("test_module");

        let gene1 = Gene::new(GeneType::Function, "fn a() {}");
        let gene2 = Gene::new(GeneType::Function, "fn b() {}");

        chromosome.add_gene(gene1);
        chromosome.add_gene(gene2);

        assert_eq!(chromosome.genes.len(), 2);
    }

    #[tokio::test]
    async fn test_code_dna_creation() {
        let dna = CodeDna::new();
        let stats = dna.get_stats().await;

        assert_eq!(stats.total_genes, 0);
    }

    #[tokio::test]
    async fn test_gene_extraction() {
        let dna = CodeDna::new();

        let code = r#"
pub fn hello() {
    println!("Hello");
}

pub struct Point {
    x: i32,
    y: i32,
}
"#;

        let genes = dna.extract_genes(code, "test").await.unwrap();
        assert!(!genes.is_empty());
    }

    #[tokio::test]
    async fn test_mutation() {
        let dna = CodeDna::with_config(CodeDnaConfig {
            mutation_rate: 1.0, // Force mutation
            ..Default::default()
        });

        let gene = Gene::new(GeneType::Function, "fn test_func() { let x = 1; }");
        let mutated = dna.mutate(&gene).await.unwrap();

        // Either mutated or original returned
        assert!(!mutated.gene_id.is_empty());
    }

    #[tokio::test]
    async fn test_crossover() {
        let dna = CodeDna::with_config(CodeDnaConfig {
            crossover_rate: 1.0, // Force crossover
            ..Default::default()
        });

        let gene1 = Gene::new(GeneType::Function, "fn a() {\n    let x = 1;\n    let y = 2;\n}");
        let gene2 = Gene::new(GeneType::Function, "fn b() {\n    let z = 3;\n    let w = 4;\n}");

        let child = dna.crossover(&gene1, &gene2).await.unwrap();
        assert!(!child.gene_id.is_empty());
        assert_eq!(child.parent_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_selection() {
        let dna = CodeDna::new();

        let mut genes = Vec::new();
        for i in 0..10 {
            let mut gene = Gene::new(GeneType::Function, &format!("fn test_{}() {{}}", i));
            gene.fitness = i as f64 / 10.0;
            genes.push(gene);
        }

        let selected = dna.select(&genes, 3).await;
        assert_eq!(selected.len(), 3);
    }
}
