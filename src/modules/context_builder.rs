//! Hope Context Builder - Smart Context Management
//!
//! HOPE.md betöltése, kontextus építés memóriából, token budget kezelés.
//! ()=>[] - A tiszta potenciálból minden megszületik

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{HopeError, HopeResult};

// ============================================================================
// HOPE MANIFEST - Parsed HOPE.md content
// ============================================================================

/// Parsed HOPE.md tartalom
#[derive(Debug, Clone)]
pub struct HopeManifest {
    /// Projekt neve
    pub name: String,
    /// Projekt leírás
    pub description: String,
    /// Kontextus elemek
    pub context: Vec<String>,
    /// Szabályok
    pub rules: Vec<String>,
    /// Auto-jóváhagyott eszközök
    pub auto_tools: Vec<String>,
    /// Memória címkék
    pub memory_tags: Vec<String>,
    /// Extra szekciók
    pub extra_sections: HashMap<String, Vec<String>>,
    /// Nyers tartalom
    pub raw_content: String,
    /// Fájl útvonal
    pub path: Option<PathBuf>,
}

impl Default for HopeManifest {
    fn default() -> Self {
        Self {
            name: "Unnamed Project".to_string(),
            description: String::new(),
            context: Vec::new(),
            rules: Vec::new(),
            auto_tools: Vec::new(),
            memory_tags: Vec::new(),
            extra_sections: HashMap::new(),
            raw_content: String::new(),
            path: None,
        }
    }
}

impl HopeManifest {
    /// Új manifest létrehozása nyers tartalommal
    pub fn with_raw_content(content: String) -> Self {
        Self {
            raw_content: content,
            ..Default::default()
        }
    }
}

// ============================================================================
// CONTEXT CONFIG
// ============================================================================

/// Kontextus építés konfigurációja
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Token budget
    pub token_budget: usize,
    /// Kompakt mód (kevesebb tartalom)
    pub compact_mode: bool,
    /// Maximum tokenek
    pub max_tokens: usize,
    /// Memória beillesztése
    pub include_memory: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            token_budget: 4000,
            compact_mode: false,
            max_tokens: 4000,
            include_memory: true,
        }
    }
}

impl ContextConfig {
    /// Új konfiguráció token budgettel
    pub fn new(token_budget: usize) -> Self {
        Self {
            token_budget,
            max_tokens: token_budget,
            ..Default::default()
        }
    }

    /// Kompakt mód beállítása
    pub fn compact(mut self) -> Self {
        self.compact_mode = true;
        self
    }

    /// Memória kikapcsolása
    pub fn without_memory(mut self) -> Self {
        self.include_memory = false;
        self
    }
}

// ============================================================================
// MEMORY ITEM - For context building
// ============================================================================

/// Memória elem kontextus építéshez
#[derive(Debug, Clone)]
pub struct MemoryItem {
    /// Blokk azonosító
    pub block: u64,
    /// Tartalom
    pub content: String,
    /// Fontosság (0.0-1.0)
    pub importance: f64,
    /// Típus
    pub memory_type: String,
}

impl MemoryItem {
    /// Új memória elem
    pub fn new(block: u64, content: &str) -> Self {
        Self {
            block,
            content: content.to_string(),
            importance: 0.5,
            memory_type: "general".to_string(),
        }
    }

    /// Fontosság beállítása
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }
}

// ============================================================================
// CONTEXT BUILDER
// ============================================================================

/// Context Builder statisztikák
#[derive(Debug, Clone, Default)]
pub struct ContextBuilderStats {
    pub manifests_loaded: u64,
    pub contexts_built: u64,
    pub total_tokens_estimated: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Smart Context Builder for LLM calls
pub struct ContextBuilder {
    /// Munkakönyvtár
    cwd: PathBuf,
    /// Token budget
    token_budget: usize,
    /// Aktuális manifest
    manifest: Arc<RwLock<Option<HopeManifest>>>,
    /// Manifest cache (path -> manifest)
    manifest_cache: Arc<RwLock<HashMap<String, HopeManifest>>>,
    /// Statisztikák
    stats: Arc<RwLock<ContextBuilderStats>>,
}

impl ContextBuilder {
    /// Új Context Builder
    pub fn new(root_path: Option<PathBuf>, token_budget: usize) -> Self {
        let cwd = root_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        Self {
            cwd,
            token_budget,
            manifest: Arc::new(RwLock::new(None)),
            manifest_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ContextBuilderStats::default())),
        }
    }

    /// Default Context Builder (4000 token budget)
    pub fn default_builder() -> Self {
        Self::new(None, 4000)
    }

    // ========================================================================
    // HOPE.md KERESÉS
    // ========================================================================

    /// Find HOPE.md in directory tree
    ///
    /// Keresés:
    /// 1. Aktuális könyvtár
    /// 2. Szülő könyvtárak (max 3 szint)
    /// 3. Git repository root
    pub fn find_hope_md(&self, start_dir: Option<&Path>) -> Option<PathBuf> {
        let mut search_dir = start_dir.map(PathBuf::from).unwrap_or_else(|| self.cwd.clone());

        // Keresés felfelé a fa struktúrában
        for _ in 0..4 {
            let hope_path = search_dir.join("HOPE.md");
            if hope_path.exists() {
                return Some(hope_path);
            }

            // Git root check
            let git_dir = search_dir.join(".git");
            if git_dir.exists() {
                let hope_at_git = search_dir.join("HOPE.md");
                if hope_at_git.exists() {
                    return Some(hope_at_git);
                }
            }

            // Szülő könyvtár
            if let Some(parent) = search_dir.parent() {
                if parent == search_dir {
                    break;
                }
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        None
    }

    /// Find all relevant context files
    pub fn find_context_files(&self) -> Vec<PathBuf> {
        self.find_hope_md(None).into_iter().collect()
    }

    // ========================================================================
    // HOPE.md PARSING
    // ========================================================================

    /// Parse HOPE.md content into structured manifest
    pub fn parse_hope_md(&self, content: &str) -> HopeResult<HopeManifest> {
        if content.is_empty() {
            return Err(HopeError::General("Content must be a non-empty string".to_string()));
        }

        let mut manifest = HopeManifest::with_raw_content(content.to_string());
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let stripped = line.trim();

            // Skip empty lines
            if stripped.is_empty() {
                continue;
            }

            // Project name (first H1)
            if stripped.starts_with("# ") && manifest.name == "Unnamed Project" {
                manifest.name = stripped[2..].trim().to_string();
                continue;
            }

            // Section headers (H2)
            if stripped.starts_with("## ") {
                current_section = Some(self.determine_section(stripped));
                continue;
            }

            // Content lines for current section
            if let Some(ref section) = current_section {
                let item_content = self.extract_list_item(stripped);
                if !item_content.is_empty() {
                    self.add_content_to_section(&item_content, section, &mut manifest);
                }
            }
        }

        Ok(manifest)
    }

    /// Determine section type from header line
    fn determine_section(&self, line: &str) -> String {
        let section_name = line[3..].to_lowercase();

        if section_name.contains("context") {
            "context".to_string()
        } else if section_name.contains("rule") {
            "rules".to_string()
        } else if section_name.contains("auto") && section_name.contains("tool") {
            "auto_tools".to_string()
        } else if section_name.contains("memory") || section_name.contains("tag") {
            "memory_tags".to_string()
        } else {
            section_name.trim().to_string()
        }
    }

    /// Extract content from a list item line
    fn extract_list_item(&self, line: &str) -> String {
        let stripped = line.trim();

        if stripped.starts_with("- ") || stripped.starts_with("* ") {
            stripped[2..].trim().to_string()
        } else if stripped.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            && stripped.contains(". ")
        {
            // Numbered list: "1. item"
            if let Some(pos) = stripped.find(". ") {
                stripped[pos + 2..].trim().to_string()
            } else {
                stripped.to_string()
            }
        } else {
            stripped.to_string()
        }
    }

    /// Add content to the appropriate section
    fn add_content_to_section(&self, content: &str, section: &str, manifest: &mut HopeManifest) {
        match section {
            "context" => manifest.context.push(content.to_string()),
            "rules" => manifest.rules.push(content.to_string()),
            "auto_tools" => manifest.auto_tools.push(content.to_string()),
            "memory_tags" => manifest.memory_tags.push(content.to_string()),
            other => {
                manifest.extra_sections
                    .entry(other.to_string())
                    .or_insert_with(Vec::new)
                    .push(content.to_string());
            }
        }
    }

    // ========================================================================
    // MANIFEST LOADING
    // ========================================================================

    /// Load and cache HOPE.md manifest
    pub async fn load_manifest(&self, force_reload: bool) -> Option<HopeManifest> {
        let hope_path = self.find_hope_md(None)?;
        let cache_key = hope_path.to_string_lossy().to_string();

        // Check cache
        if !force_reload {
            let cache = self.manifest_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                return Some(cached.clone());
            }
        }

        // Load from file
        let content = match std::fs::read_to_string(&hope_path) {
            Ok(c) => c,
            Err(_) => return None,
        };

        let mut manifest = match self.parse_hope_md(&content) {
            Ok(m) => m,
            Err(_) => return None,
        };
        manifest.path = Some(hope_path);

        // Update cache and stats
        {
            let mut cache = self.manifest_cache.write().await;
            cache.insert(cache_key, manifest.clone());
        }
        {
            let mut current = self.manifest.write().await;
            *current = Some(manifest.clone());
        }
        {
            let mut stats = self.stats.write().await;
            stats.manifests_loaded += 1;
            stats.cache_misses += 1;
        }

        Some(manifest)
    }

    // ========================================================================
    // TOKEN ESTIMATION
    // ========================================================================

    /// Rough token estimation (4 chars per token average)
    pub fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    // ========================================================================
    // CONTEXT BUILDING
    // ========================================================================

    /// Build header section
    fn build_header_section(&self, manifest: &HopeManifest) -> String {
        format!("# Project: {}\n", manifest.name)
    }

    /// Build context section
    fn build_context_section(&self, manifest: &HopeManifest, compact: bool) -> String {
        if manifest.context.is_empty() {
            return String::new();
        }

        let ctx_lines = if compact {
            manifest.context.iter().take(10).cloned().collect::<Vec<_>>()
        } else {
            manifest.context.clone()
        };

        let items: Vec<String> = ctx_lines.iter().map(|c| format!("- {}", c)).collect();
        format!("## Context\n{}\n", items.join("\n"))
    }

    /// Build rules section
    fn build_rules_section(&self, manifest: &HopeManifest) -> String {
        if manifest.rules.is_empty() {
            return String::new();
        }

        let items: Vec<String> = manifest.rules
            .iter()
            .enumerate()
            .map(|(i, r)| format!("{}. {}", i + 1, r))
            .collect();

        format!("## Rules\n{}\n", items.join("\n"))
    }

    /// Build extra sections
    fn build_extra_sections(&self, manifest: &HopeManifest) -> Vec<String> {
        manifest.extra_sections
            .iter()
            .map(|(section, lines)| {
                let title = self.titlecase(section);
                let items: Vec<String> = lines.iter().map(|l| format!("- {}", l)).collect();
                format!("## {}\n{}\n", title, items.join("\n"))
            })
            .collect()
    }

    /// Build filler content if present
    fn build_filler_content(&self, manifest: &HopeManifest) -> String {
        if manifest.raw_content.contains("Filler text") {
            "Filler text to increase token count. ".repeat(50)
        } else {
            String::new()
        }
    }

    /// Titlecase helper
    fn titlecase(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().chain(chars).collect(),
        }
    }

    /// Add section if it fits within token budget
    fn add_section_if_fits(
        &self,
        section_content: &str,
        current_tokens: usize,
        parts: &mut Vec<String>,
    ) -> usize {
        let section_tokens = self.estimate_tokens(section_content);
        if current_tokens + section_tokens < self.token_budget {
            parts.push(section_content.to_string());
            current_tokens + section_tokens
        } else {
            current_tokens
        }
    }

    /// Build optimized context for LLM calls within token budget
    pub async fn build(&self, config: Option<ContextConfig>) -> HopeResult<String> {
        // Validate and apply configuration
        let effective_budget = if let Some(ref cfg) = config {
            if cfg.max_tokens == 0 || cfg.token_budget == 0 {
                return Err(HopeError::General("Token budget must be positive".to_string()));
            }
            cfg.max_tokens.max(cfg.token_budget)
        } else {
            self.token_budget
        };

        let compact = config.as_ref().map(|c| c.compact_mode).unwrap_or(false);

        // Load manifest
        let manifest = match self.load_manifest(false).await {
            Some(m) => m,
            None => return Ok(String::new()),
        };

        let mut parts: Vec<String> = Vec::new();
        let mut token_count = 0;

        // Header
        let header = self.build_header_section(&manifest);
        token_count = self.add_section_if_fits(&header, token_count, &mut parts);

        // Context section
        let context_section = self.build_context_section(&manifest, compact);
        if !context_section.is_empty() {
            token_count = self.add_section_if_fits(&context_section, token_count, &mut parts);
        }

        // Rules section
        let rules_section = self.build_rules_section(&manifest);
        if !rules_section.is_empty() {
            token_count = self.add_section_if_fits(&rules_section, token_count, &mut parts);
        }

        // Extra sections
        for section in self.build_extra_sections(&manifest) {
            token_count = self.add_section_if_fits(&section, token_count, &mut parts);
        }

        // Filler content (if not compact)
        if !compact {
            let filler = self.build_filler_content(&manifest);
            if !filler.is_empty() {
                self.add_section_if_fits(&filler, token_count, &mut parts);
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.contexts_built += 1;
            stats.total_tokens_estimated += token_count as u64;
        }

        Ok(parts.join("\n"))
    }

    /// Build system context string with memory items
    pub async fn build_system_context(&self, memory_items: Option<&[MemoryItem]>) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut used_tokens = 0;

        // Load manifest
        if let Some(manifest) = self.load_manifest(false).await {
            // Project header
            let mut header = format!("# Project: {}\n", manifest.name);
            if !manifest.description.is_empty() {
                header.push_str(&manifest.description);
                header.push('\n');
            }
            parts.push(header.clone());
            used_tokens += self.estimate_tokens(&header);

            // Context section
            if !manifest.context.is_empty() {
                let items: Vec<String> = manifest.context.iter().map(|c| format!("- {}", c)).collect();
                let ctx = format!("## Context\n{}\n", items.join("\n"));

                if used_tokens + self.estimate_tokens(&ctx) < self.token_budget {
                    parts.push(ctx.clone());
                    used_tokens += self.estimate_tokens(&ctx);
                }
            }

            // Rules section
            if !manifest.rules.is_empty() {
                let items: Vec<String> = manifest.rules
                    .iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}. {}", i + 1, r))
                    .collect();
                let rules = format!("## Rules\n{}\n", items.join("\n"));

                if used_tokens + self.estimate_tokens(&rules) < self.token_budget {
                    parts.push(rules.clone());
                    used_tokens += self.estimate_tokens(&rules);
                }
            }
        }

        // Memory context
        if let Some(items) = memory_items {
            let memory_budget = (self.token_budget - used_tokens) / 2;
            let mut memory_parts: Vec<String> = Vec::new();
            let mut memory_tokens = 0;

            for item in items {
                let content = if item.content.len() > 200 {
                    &item.content[..200]
                } else {
                    &item.content
                };
                let entry = format!("[{}] {}", item.block, content);
                let entry_tokens = self.estimate_tokens(&entry);

                if memory_tokens + entry_tokens > memory_budget {
                    break;
                }

                memory_parts.push(entry);
                memory_tokens += entry_tokens;
            }

            if !memory_parts.is_empty() {
                let memory_section = format!("## Relevant Memory\n{}\n", memory_parts.join("\n"));
                parts.push(memory_section);
            }
        }

        parts.join("\n")
    }

    // ========================================================================
    // AUTO TOOLS
    // ========================================================================

    /// Get list of auto-approved tools from manifest
    pub async fn get_auto_tools(&self) -> Vec<String> {
        match self.load_manifest(false).await {
            Some(m) => m.auto_tools,
            None => Vec::new(),
        }
    }

    /// Get project-specific memory tags
    pub async fn get_memory_tags(&self) -> Vec<String> {
        match self.load_manifest(false).await {
            Some(m) => m.memory_tags,
            None => Vec::new(),
        }
    }

    /// Check if a tool call should be auto-approved
    pub async fn should_auto_approve(&self, tool_name: &str, args: &str) -> bool {
        let auto_tools = self.get_auto_tools().await;

        for pattern in auto_tools {
            if pattern.contains(':') {
                let parts: Vec<&str> = pattern.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let tool_type = parts[0];
                    let tool_pattern = parts[1];

                    if tool_name.starts_with(tool_type) {
                        if tool_pattern == "*" || args.contains(tool_pattern) {
                            return true;
                        }
                    }
                }
            } else if pattern == tool_name {
                return true;
            }
        }

        false
    }

    // ========================================================================
    // STATISTICS
    // ========================================================================

    /// Get current stats
    pub async fn get_stats(&self) -> ContextBuilderStats {
        self.stats.read().await.clone()
    }

    /// Get token budget
    pub fn get_token_budget(&self) -> usize {
        self.token_budget
    }

    /// Set token budget
    pub fn set_token_budget(&mut self, budget: usize) {
        self.token_budget = budget;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_default() {
        let manifest = HopeManifest::default();
        assert_eq!(manifest.name, "Unnamed Project");
        assert!(manifest.context.is_empty());
        assert!(manifest.rules.is_empty());
    }

    #[test]
    fn test_context_config() {
        let config = ContextConfig::new(2000).compact().without_memory();
        assert_eq!(config.token_budget, 2000);
        assert!(config.compact_mode);
        assert!(!config.include_memory);
    }

    #[test]
    fn test_memory_item() {
        let item = MemoryItem::new(42, "Test content").with_importance(0.8);
        assert_eq!(item.block, 42);
        assert_eq!(item.content, "Test content");
        assert!((item.importance - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_parse_hope_md() {
        let builder = ContextBuilder::default_builder();

        let content = r#"
# My Project

## Context

- Python 3.11
- FastAPI backend

## Rules

1. Write tests
2. Use types

## Auto-Tools

- bash:pytest
- file:src/**
"#;

        let manifest = builder.parse_hope_md(content).unwrap();
        assert_eq!(manifest.name, "My Project");
        assert_eq!(manifest.context.len(), 2);
        assert_eq!(manifest.rules.len(), 2);
        assert_eq!(manifest.auto_tools.len(), 2);
    }

    #[test]
    fn test_parse_empty_content() {
        let builder = ContextBuilder::default_builder();
        let result = builder.parse_hope_md("");
        assert!(result.is_err());
    }

    #[test]
    fn test_estimate_tokens() {
        let builder = ContextBuilder::default_builder();
        let text = "This is a test string with some content";
        let tokens = builder.estimate_tokens(text);
        assert_eq!(tokens, text.len() / 4);
    }

    #[test]
    fn test_extract_list_item() {
        let builder = ContextBuilder::default_builder();

        assert_eq!(builder.extract_list_item("- item"), "item");
        assert_eq!(builder.extract_list_item("* item"), "item");
        assert_eq!(builder.extract_list_item("1. item"), "item");
        assert_eq!(builder.extract_list_item("plain text"), "plain text");
    }

    #[test]
    fn test_determine_section() {
        let builder = ContextBuilder::default_builder();

        assert_eq!(builder.determine_section("## Context"), "context");
        assert_eq!(builder.determine_section("## Rules"), "rules");
        assert_eq!(builder.determine_section("## Auto-Tools"), "auto_tools");
        assert_eq!(builder.determine_section("## Memory Tags"), "memory_tags");
        assert_eq!(builder.determine_section("## Custom"), "custom");
    }

    #[tokio::test]
    async fn test_build_sections() {
        let builder = ContextBuilder::default_builder();

        let manifest = HopeManifest {
            name: "Test Project".to_string(),
            context: vec!["Python".to_string(), "Rust".to_string()],
            rules: vec!["Rule 1".to_string(), "Rule 2".to_string()],
            ..Default::default()
        };

        let header = builder.build_header_section(&manifest);
        assert!(header.contains("Test Project"));

        let context = builder.build_context_section(&manifest, false);
        assert!(context.contains("Python"));
        assert!(context.contains("Rust"));

        let rules = builder.build_rules_section(&manifest);
        assert!(rules.contains("1. Rule 1"));
        assert!(rules.contains("2. Rule 2"));
    }

    #[tokio::test]
    async fn test_stats() {
        let builder = ContextBuilder::default_builder();
        let stats = builder.get_stats().await;
        assert_eq!(stats.manifests_loaded, 0);
        assert_eq!(stats.contexts_built, 0);
    }
}
