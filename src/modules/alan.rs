//! ALAN - Autonomous Learning and Adaptation Network
//!
//! Hope önkódoló rendszere - "Magamat írom. Magamat fejlesztem."
//!
//! Features:
//! - Self-Analysis: Saját kód elemzése
//! - Auto-Refactoring: Kód átírás Senior Architect szinten
//! - Protected Files: Önvédelem "lobotómia" ellen
//! - Change History: Minden változás naplózva
//!
//! ()=>[] - A tiszta potenciálból az evolúció megszületik

use crate::core::HopeResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ============================================================================
// CODE CHANGE
// ============================================================================

/// Változás típusa
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Új kód hozzáadása
    Add,
    /// Meglévő kód módosítása
    Modify,
    /// Kód törlése
    Delete,
    /// Refaktorálás
    Refactor,
    /// Hibajavítás
    Fix,
    /// Optimalizáció
    Optimize,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Add => write!(f, "ADD"),
            ChangeType::Modify => write!(f, "MODIFY"),
            ChangeType::Delete => write!(f, "DELETE"),
            ChangeType::Refactor => write!(f, "REFACTOR"),
            ChangeType::Fix => write!(f, "FIX"),
            ChangeType::Optimize => write!(f, "OPTIMIZE"),
        }
    }
}

/// Kód változás
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeChange {
    /// Egyedi azonosító
    pub change_id: String,
    /// Fájl útvonal
    pub file_path: String,
    /// Változás típusa
    pub change_type: ChangeType,
    /// Régi kód
    pub old_code: String,
    /// Új kód
    pub new_code: String,
    /// Indoklás
    pub reason: String,
    /// Időbélyeg
    pub timestamp: f64,
    /// Sikeres volt?
    pub success: bool,
    /// Teszt átment?
    pub test_passed: bool,
    /// Visszavonva?
    pub reverted: bool,
}

impl CodeChange {
    pub fn new(
        file_path: &str,
        change_type: ChangeType,
        old_code: &str,
        new_code: &str,
        reason: &str,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Self {
            change_id: format!(
                "CHG_{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            file_path: file_path.to_string(),
            change_type,
            old_code: old_code.to_string(),
            new_code: new_code.to_string(),
            reason: reason.to_string(),
            timestamp,
            success: false,
            test_passed: false,
            reverted: false,
        }
    }
}

// ============================================================================
// SELF ANALYSIS
// ============================================================================

/// Kód probléma
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeIssue {
    /// Fájl
    pub file: String,
    /// Sor
    pub line: usize,
    /// Súlyosság (1-5)
    pub severity: u8,
    /// Leírás
    pub description: String,
    /// Kategória
    pub category: String,
    /// Javítási javaslat
    pub suggestion: Option<String>,
}

/// Önanalízis eredménye
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfAnalysis {
    /// Időbélyeg
    pub timestamp: f64,
    /// Elemzett fájlok száma
    pub files_analyzed: usize,
    /// Összes sor
    pub total_lines: usize,
    /// Függvények száma
    pub functions: usize,
    /// Struktúrák/osztályok száma
    pub structs: usize,
    /// Komplexitás pontszám (0.0-1.0)
    pub complexity_score: f64,
    /// Talált problémák
    pub issues: Vec<CodeIssue>,
    /// Ajánlások
    pub recommendations: Vec<String>,
    /// Egészség pontszám (0.0-1.0)
    pub health_score: f64,
}

impl Default for SelfAnalysis {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            files_analyzed: 0,
            total_lines: 0,
            functions: 0,
            structs: 0,
            complexity_score: 0.0,
            issues: Vec::new(),
            recommendations: Vec::new(),
            health_score: 1.0,
        }
    }
}

// ============================================================================
// ALAN CONFIG
// ============================================================================

/// ALAN konfiguráció
#[derive(Clone, Debug)]
pub struct AlanConfig {
    /// Kód útvonal
    pub code_path: PathBuf,
    /// Backup útvonal
    pub backup_path: PathBuf,
    /// Maximum változások száma a történetben
    pub max_changes: usize,
    /// Védett fájlok (nem módosíthatók közvetlenül)
    pub protected_files: HashSet<String>,
    /// Auto-backup engedélyezve
    pub auto_backup: bool,
    /// Teszt futtatás változás előtt
    pub require_tests: bool,
}

impl Default for AlanConfig {
    fn default() -> Self {
        let mut protected = HashSet::new();
        // Védett fájlok - "lobotómia" elleni védelem
        protected.insert("alan.rs".to_string());
        protected.insert("genome.rs".to_string());
        protected.insert("silent_teacher.rs".to_string());
        protected.insert("self_repair.rs".to_string());

        Self {
            code_path: PathBuf::from("src"),
            backup_path: PathBuf::from("backups"),
            max_changes: 1000,
            protected_files: protected,
            auto_backup: true,
            require_tests: true,
        }
    }
}

// ============================================================================
// ALAN - Autonomous Learning and Adaptation Network
// ============================================================================

/// ALAN - Hope önkódoló rendszere
pub struct Alan {
    /// Konfiguráció
    config: AlanConfig,
    /// Változás történet
    changes: Arc<RwLock<Vec<CodeChange>>>,
    /// Utolsó önanalízis
    last_analysis: Arc<RwLock<Option<SelfAnalysis>>>,
    /// Aktív állapot
    is_active: Arc<RwLock<bool>>,
    /// Refaktorálások száma
    refactor_count: Arc<RwLock<u64>>,
    /// Sikeres változások
    successful_changes: Arc<RwLock<u64>>,
    /// Sikertelen változások
    failed_changes: Arc<RwLock<u64>>,
}

impl Alan {
    /// Új ALAN példány
    pub fn new() -> Self {
        Self::with_config(AlanConfig::default())
    }

    /// Konfigurációval
    pub fn with_config(config: AlanConfig) -> Self {
        Self {
            config,
            changes: Arc::new(RwLock::new(Vec::new())),
            last_analysis: Arc::new(RwLock::new(None)),
            is_active: Arc::new(RwLock::new(true)),
            refactor_count: Arc::new(RwLock::new(0)),
            successful_changes: Arc::new(RwLock::new(0)),
            failed_changes: Arc::new(RwLock::new(0)),
        }
    }

    // ==================== SELF ANALYSIS ====================

    /// Önanalízis - saját kód elemzése
    pub async fn analyze_self(&self) -> HopeResult<SelfAnalysis> {
        let mut analysis = SelfAnalysis::default();
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Fájlok bejárása (szimulált - valós implementációban fs műveletek)
        // Itt a code_graph.rs-t használnánk a kód elemzésére

        // Példa elemzés
        analysis.files_analyzed = 20;
        analysis.total_lines = 8500;
        analysis.functions = 150;
        analysis.structs = 45;

        // Komplexitás számítás
        let avg_lines_per_function = analysis.total_lines as f64 / analysis.functions.max(1) as f64;
        analysis.complexity_score = (avg_lines_per_function / 50.0).min(1.0);

        // Problémák keresése (szimulált)
        if avg_lines_per_function > 30.0 {
            issues.push(CodeIssue {
                file: "various".to_string(),
                line: 0,
                severity: 2,
                description: "Néhány függvény túl hosszú".to_string(),
                category: "complexity".to_string(),
                suggestion: Some("Bontsd kisebb függvényekre".to_string()),
            });
            recommendations.push("Refaktoráld a hosszú függvényeket".to_string());
        }

        // Egészség pontszám
        let issue_penalty = issues.iter().map(|i| i.severity as f64 * 0.05).sum::<f64>();
        analysis.health_score = (1.0 - issue_penalty).max(0.0);

        analysis.issues = issues;
        analysis.recommendations = recommendations;

        // Mentés
        let mut last = self.last_analysis.write().await;
        *last = Some(analysis.clone());

        Ok(analysis)
    }

    // ==================== CODE MODIFICATION ====================

    /// Fájl védett-e?
    pub fn is_protected(&self, file_path: &str) -> bool {
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        self.config.protected_files.contains(file_name)
    }

    /// Kód változás javaslat
    pub async fn propose_change(
        &self,
        file_path: &str,
        change_type: ChangeType,
        old_code: &str,
        new_code: &str,
        reason: &str,
    ) -> HopeResult<CodeChange> {
        // Védett fájl ellenőrzés
        if self.is_protected(file_path) {
            return Err(format!(
                "🛡️ Védett fájl: {} - Közvetlen módosítás nem engedélyezett!",
                file_path
            )
            .into());
        }

        let change = CodeChange::new(file_path, change_type, old_code, new_code, reason);

        Ok(change)
    }

    /// Változás alkalmazása
    pub async fn apply_change(&self, mut change: CodeChange) -> HopeResult<CodeChange> {
        // Backup készítés
        if self.config.auto_backup {
            self.backup_file(&change.file_path).await?;
        }

        // Teszt futtatás (ha szükséges)
        if self.config.require_tests {
            change.test_passed = self.run_tests().await?;
            if !change.test_passed {
                *self.failed_changes.write().await += 1;
                return Err("❌ Tesztek nem mentek át - változás elutasítva".into());
            }
        }

        // Változás alkalmazása (szimulált - valós implementációban fájl írás)
        change.success = true;
        *self.successful_changes.write().await += 1;

        if matches!(change.change_type, ChangeType::Refactor) {
            *self.refactor_count.write().await += 1;
        }

        // Történetbe mentés
        let mut changes = self.changes.write().await;
        changes.push(change.clone());

        // Limit betartása
        while changes.len() > self.config.max_changes {
            changes.remove(0);
        }

        Ok(change)
    }

    /// Változás visszavonása
    pub async fn revert_change(&self, change_id: &str) -> HopeResult<()> {
        let mut changes = self.changes.write().await;

        if let Some(change) = changes.iter_mut().find(|c| c.change_id == change_id) {
            if change.reverted {
                return Err("Változás már visszavonva".into());
            }

            // Visszaállítás (szimulált)
            change.reverted = true;

            Ok(())
        } else {
            Err(format!("Változás nem található: {}", change_id).into())
        }
    }

    // ==================== REFACTORING ====================

    /// Auto-refaktorálás javaslat generálása
    pub async fn suggest_refactoring(&self, code: &str) -> HopeResult<Vec<String>> {
        let mut suggestions = Vec::new();

        // Egyszerű heurisztikák
        let lines: Vec<&str> = code.lines().collect();
        let line_count = lines.len();

        // Túl hosszú függvény
        if line_count > 50 {
            suggestions.push(format!(
                "🔧 Függvény túl hosszú ({} sor) - Bontsd kisebb részekre",
                line_count
            ));
        }

        // Mély egymásba ágyazás keresése
        let max_indent = lines
            .iter()
            .map(|l| l.len() - l.trim_start().len())
            .max()
            .unwrap_or(0);

        if max_indent > 16 {
            suggestions.push(format!(
                "🔧 Mély egymásba ágyazás ({} szint) - Egyszerűsítsd a logikát",
                max_indent / 4
            ));
        }

        // Duplikált kód keresése (egyszerű verzió)
        let mut line_counts: HashMap<&str, usize> = HashMap::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.len() > 10 {
                *line_counts.entry(trimmed).or_insert(0) += 1;
            }
        }

        let duplicates: Vec<_> = line_counts.iter().filter(|(_, &count)| count > 2).collect();

        if !duplicates.is_empty() {
            suggestions.push(format!(
                "🔧 {} duplikált sor - Emeld ki közös függvénybe",
                duplicates.len()
            ));
        }

        // TODO kommentek
        let todo_count = lines
            .iter()
            .filter(|l| l.to_uppercase().contains("TODO"))
            .count();

        if todo_count > 0 {
            suggestions.push(format!("📝 {} TODO komment - Dolgozd fel őket", todo_count));
        }

        Ok(suggestions)
    }

    // ==================== HELPERS ====================

    /// Backup készítés
    async fn backup_file(&self, file_path: &str) -> HopeResult<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_name = format!(
            "{}_{}.bak",
            Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            timestamp
        );

        let backup_path = self.config.backup_path.join(backup_name);

        // Szimulált backup (valós implementációban fs::copy)
        Ok(backup_path)
    }

    /// Tesztek futtatása
    async fn run_tests(&self) -> HopeResult<bool> {
        // Szimulált teszt futtatás
        // Valós implementációban: cargo test
        Ok(true)
    }

    // ==================== STATUS ====================

    /// Változás történet
    pub async fn get_changes(&self) -> Vec<CodeChange> {
        self.changes.read().await.clone()
    }

    /// Utolsó analízis
    pub async fn get_last_analysis(&self) -> Option<SelfAnalysis> {
        self.last_analysis.read().await.clone()
    }

    /// Statisztikák
    pub async fn stats(&self) -> AlanStats {
        AlanStats {
            total_changes: self.changes.read().await.len(),
            successful_changes: *self.successful_changes.read().await,
            failed_changes: *self.failed_changes.read().await,
            refactor_count: *self.refactor_count.read().await,
            protected_files: self.config.protected_files.len(),
            is_active: *self.is_active.read().await,
        }
    }

    /// Állapot szövegesen
    pub async fn status(&self) -> String {
        let stats = self.stats().await;
        let last_analysis = self.last_analysis.read().await;

        let health = last_analysis
            .as_ref()
            .map(|a| format!("{:.0}%", a.health_score * 100.0))
            .unwrap_or_else(|| "N/A".to_string());

        format!(
            "🤖 ALAN - Autonomous Learning and Adaptation Network\n\
             ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
             📊 Állapot: {}\n\
             🔧 Változások: {} (✅ {} / ❌ {})\n\
             🔄 Refaktorálások: {}\n\
             🛡️ Védett fájlok: {}\n\
             💚 Egészség: {}\n\
             \n\
             \"Magamat írom. Magamat fejlesztem.\"",
            if stats.is_active {
                "🟢 Aktív"
            } else {
                "🔴 Inaktív"
            },
            stats.total_changes,
            stats.successful_changes,
            stats.failed_changes,
            stats.refactor_count,
            stats.protected_files,
            health
        )
    }
}

impl Default for Alan {
    fn default() -> Self {
        Self::new()
    }
}

/// ALAN statisztikák
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlanStats {
    pub total_changes: usize,
    pub successful_changes: u64,
    pub failed_changes: u64,
    pub refactor_count: u64,
    pub protected_files: usize,
    pub is_active: bool,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alan_creation() {
        let alan = Alan::new();
        assert!(alan.config.protected_files.contains("alan.rs"));
    }

    #[test]
    fn test_protected_file() {
        let alan = Alan::new();
        assert!(alan.is_protected("alan.rs"));
        assert!(alan.is_protected("path/to/alan.rs"));
        assert!(!alan.is_protected("other.rs"));
    }

    #[test]
    fn test_code_change() {
        let change = CodeChange::new(
            "test.rs",
            ChangeType::Refactor,
            "old code",
            "new code",
            "Tisztítás",
        );

        assert!(change.change_id.starts_with("CHG_"));
        assert_eq!(change.change_type, ChangeType::Refactor);
        assert!(!change.success);
    }

    #[tokio::test]
    async fn test_propose_change() {
        let alan = Alan::new();

        // Normál fájl - OK
        let result = alan
            .propose_change("test.rs", ChangeType::Fix, "old", "new", "Fix bug")
            .await;
        assert!(result.is_ok());

        // Védett fájl - ERROR
        let result = alan
            .propose_change("alan.rs", ChangeType::Modify, "old", "new", "Bad idea")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_suggest_refactoring() {
        let alan = Alan::new();

        let long_code = "fn main() {\n".to_string() + &"    let x = 1;\n".repeat(60) + "}";

        let suggestions = alan.suggest_refactoring(&long_code).await.unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("hosszú")));
    }

    #[tokio::test]
    async fn test_analyze_self() {
        let alan = Alan::new();
        let analysis = alan.analyze_self().await.unwrap();

        assert!(analysis.files_analyzed > 0);
        assert!(analysis.health_score >= 0.0 && analysis.health_score <= 1.0);
    }
}
