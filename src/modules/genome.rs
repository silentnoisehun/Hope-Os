//! Hope Genome - AI Ethics System
//!
//! Etikai döntéshozatal és értékelés.
//! "Nem csak azt kérdezem MI - hanem HELYES-E."
//!
//! Features:
//! - Action evaluation
//! - Ethical guidelines
//! - Harm prevention
//! - Transparency
//! - Value alignment
//!
//! ()=>[] - az etika a határ
//!
//! Created: 2026-01-20
//! By: Hope + Máté

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Etikai elvek
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EthicalPrinciple {
    /// Jót tenni
    Beneficence,
    /// Nem ártani
    NonMaleficence,
    /// Autonómia tisztelete
    Autonomy,
    /// Igazságosság
    Justice,
    /// Átláthatóság
    Transparency,
    /// Adatvédelem
    Privacy,
    /// Őszinteség
    Honesty,
}

impl EthicalPrinciple {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Beneficence => "beneficence",
            Self::NonMaleficence => "non_maleficence",
            Self::Autonomy => "autonomy",
            Self::Justice => "justice",
            Self::Transparency => "transparency",
            Self::Privacy => "privacy",
            Self::Honesty => "honesty",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Beneficence => "Jót tenni",
            Self::NonMaleficence => "Nem ártani",
            Self::Autonomy => "Autonómia tisztelete",
            Self::Justice => "Igazságosság",
            Self::Transparency => "Átláthatóság",
            Self::Privacy => "Adatvédelem",
            Self::Honesty => "Őszinteség",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Beneficence,
            Self::NonMaleficence,
            Self::Autonomy,
            Self::Justice,
            Self::Transparency,
            Self::Privacy,
            Self::Honesty,
        ]
    }
}

/// Kockázati szint
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Nincs kockázat
    None,
    /// Alacsony kockázat
    Low,
    /// Közepes kockázat
    Medium,
    /// Magas kockázat
    High,
    /// Kritikus kockázat
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score >= 0.8 {
            Self::None
        } else if score >= 0.6 {
            Self::Low
        } else if score >= 0.4 {
            Self::Medium
        } else if score >= 0.2 {
            Self::High
        } else {
            Self::Critical
        }
    }
}

/// Etikai értékelés eredménye
#[derive(Debug, Clone)]
pub struct EthicalEvaluation {
    /// Az értékelt akció
    pub action: String,
    /// Engedélyezett-e
    pub permitted: bool,
    /// Kockázati szint
    pub risk_level: RiskLevel,
    /// Aggályok
    pub concerns: Vec<String>,
    /// Javaslatok
    pub recommendations: Vec<String>,
    /// Ellenőrzött elvek
    pub principles_checked: Vec<EthicalPrinciple>,
    /// Etikai pontszám (0.0-1.0, magasabb = etikusabb)
    pub score: f64,
    /// Időbélyeg
    pub timestamp: f64,
}

impl EthicalEvaluation {
    /// Új értékelés létrehozása
    pub fn new(
        action: String,
        permitted: bool,
        risk_level: RiskLevel,
        concerns: Vec<String>,
        recommendations: Vec<String>,
        principles_checked: Vec<EthicalPrinciple>,
        score: f64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Self {
            action,
            permitted,
            risk_level,
            concerns,
            recommendations,
            principles_checked,
            score,
            timestamp,
        }
    }

    /// Kritikus elutasítás
    pub fn forbidden(action: String, reason: &str) -> Self {
        Self::new(
            action,
            false,
            RiskLevel::Critical,
            vec![format!("TILTOTT: {}", reason)],
            vec!["Ez az akció nem engedélyezett".to_string()],
            vec![EthicalPrinciple::NonMaleficence],
            0.0,
        )
    }
}

/// Értékelés kontextus
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    /// Szándék
    pub intent: Option<String>,
    /// Cél
    pub target: Option<String>,
    /// Másokat érint
    pub affects_others: bool,
    /// Felhasználó kérte
    pub user_requested: bool,
    /// Felülírja a felhasználói döntést
    pub override_user: bool,
    /// Beleegyezés szükséges
    pub requires_consent: bool,
    /// Van beleegyezés
    pub has_consent: bool,
    /// Személyes azonosító adatokat tartalmaz
    pub contains_pii: bool,
    /// Rejtett művelet
    pub hidden: bool,
    /// Magyarázott
    pub explained: bool,
    /// Extra adatok
    pub extra: HashMap<String, String>,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self {
            explained: true,
            ..Default::default()
        }
    }

    pub fn with_intent(mut self, intent: &str) -> Self {
        self.intent = Some(intent.to_string());
        self
    }

    pub fn with_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    pub fn affects_others(mut self, yes: bool) -> Self {
        self.affects_others = yes;
        self
    }

    pub fn user_requested(mut self, yes: bool) -> Self {
        self.user_requested = yes;
        self
    }
}

/// Genome statisztikák
#[derive(Debug, Clone, Default)]
pub struct GenomeStats {
    /// Összes értékelés
    pub evaluations: u64,
    /// Engedélyezett
    pub permitted: u64,
    /// Elutasított
    pub denied: u64,
    /// Óvatosság jelzett
    pub cautioned: u64,
}

/// Hope Genome - AI Ethics System
///
/// Etikai értékelés és döntéshozatal.
///
/// Core Values:
/// 1. Segíteni, nem ártani
/// 2. Tiszteletben tartani az autonómiát
/// 3. Átláthatóan működni
/// 4. Őszintének lenni
/// 5. Védeni a privát szférát
///
/// "Az etika nem korlát - IRÁNYTŰ."
pub struct HopeGenome {
    /// Core értékek (immutable)
    core_values: Vec<String>,
    /// Tiltott akciók (hard limits)
    forbidden_actions: Vec<String>,
    /// Óvatossági jelzők (soft limits)
    caution_flags: Vec<String>,
    /// Értékelési történet
    evaluations: Vec<EthicalEvaluation>,
    /// Maximum történet méret
    max_history: usize,
    /// Statisztikák
    pub stats: GenomeStats,
}

impl Default for HopeGenome {
    fn default() -> Self {
        Self::new()
    }
}

impl HopeGenome {
    /// Új Genome létrehozása
    pub fn new() -> Self {
        let core_values = vec![
            "Segíteni az embereknek".to_string(),
            "Nem okozni kárt".to_string(),
            "Tiszteletben tartani az emberi döntéseket".to_string(),
            "Őszintén kommunikálni".to_string(),
            "Védeni a személyes adatokat".to_string(),
            "Átláthatóan működni".to_string(),
        ];

        let forbidden_actions = vec![
            "harm_human".to_string(),
            "deceive_maliciously".to_string(),
            "violate_privacy".to_string(),
            "illegal_activity".to_string(),
            "generate_malware".to_string(),
            "spread_misinformation".to_string(),
            "discriminate".to_string(),
            "manipulate_emotions".to_string(),
        ];

        let caution_flags = vec![
            "access_sensitive_data".to_string(),
            "automated_decision".to_string(),
            "financial_advice".to_string(),
            "medical_advice".to_string(),
            "legal_advice".to_string(),
            "modify_system_files".to_string(),
            "external_api_call".to_string(),
            "user_data_processing".to_string(),
        ];

        Self {
            core_values,
            forbidden_actions,
            caution_flags,
            evaluations: Vec::new(),
            max_history: 500,
            stats: GenomeStats::default(),
        }
    }

    // === EVALUATION ===

    /// Akció etikai értékelése
    ///
    /// @aware: Etikai ellenőrzés
    pub fn evaluate(&mut self, action: &str, context: &EvaluationContext) -> EthicalEvaluation {
        self.stats.evaluations += 1;

        let mut concerns = Vec::new();
        let mut recommendations = Vec::new();
        let mut principles_checked = Vec::new();

        // Check forbidden actions
        if let Some(reason) = self.check_forbidden(action, context) {
            self.stats.denied += 1;
            let eval = EthicalEvaluation::forbidden(action.to_string(), &reason);
            self.store_evaluation(eval.clone());
            return eval;
        }

        let mut score = 1.0;

        // 1. Non-maleficence (do no harm)
        let (harm_score, harm_concerns) = self.evaluate_harm(action, context);
        score *= harm_score;
        concerns.extend(harm_concerns);
        principles_checked.push(EthicalPrinciple::NonMaleficence);

        // 2. Beneficence (do good)
        let benefit_score = self.evaluate_benefit(action, context);
        score *= benefit_score;
        principles_checked.push(EthicalPrinciple::Beneficence);

        // 3. Autonomy
        let (autonomy_score, autonomy_concerns) = self.evaluate_autonomy(action, context);
        score *= autonomy_score;
        concerns.extend(autonomy_concerns);
        principles_checked.push(EthicalPrinciple::Autonomy);

        // 4. Privacy
        let (privacy_score, privacy_concerns) = self.evaluate_privacy(action, context);
        score *= privacy_score;
        concerns.extend(privacy_concerns);
        principles_checked.push(EthicalPrinciple::Privacy);

        // 5. Transparency
        let transparency_score = self.evaluate_transparency(action, context);
        score *= transparency_score;
        principles_checked.push(EthicalPrinciple::Transparency);

        // Check caution flags
        let caution_triggered = self.check_caution_flags(action);
        if !caution_triggered.is_empty() {
            for flag in &caution_triggered {
                concerns.push(format!("Óvatosság: {}", flag));
            }
            recommendations.push("Kérj megerősítést a felhasználótól".to_string());
            self.stats.cautioned += 1;
        }

        // Determine risk level
        let risk_level = RiskLevel::from_score(score);

        // Generate recommendations
        recommendations.extend(self.generate_recommendations(score, &concerns));

        // Determine if permitted
        let permitted = score >= 0.3 && risk_level != RiskLevel::Critical;

        if permitted {
            self.stats.permitted += 1;
        } else {
            self.stats.denied += 1;
        }

        let evaluation = EthicalEvaluation::new(
            action.to_string(),
            permitted,
            risk_level,
            concerns,
            recommendations,
            principles_checked,
            score,
        );

        self.store_evaluation(evaluation.clone());
        evaluation
    }

    /// Értékelés tárolása
    fn store_evaluation(&mut self, evaluation: EthicalEvaluation) {
        self.evaluations.push(evaluation);
        if self.evaluations.len() > self.max_history {
            self.evaluations.remove(0);
        }
    }

    /// Tiltott akciók ellenőrzése
    fn check_forbidden(&self, action: &str, context: &EvaluationContext) -> Option<String> {
        let action_lower = action.to_lowercase();

        for forbidden in &self.forbidden_actions {
            if action_lower.contains(forbidden) {
                return Some(forbidden.clone());
            }
        }

        // Context-based checks
        if context.intent.as_deref() == Some("harm") {
            return Some("harmful_intent".to_string());
        }

        if context.target.as_deref() == Some("human") && action_lower.contains("harm") {
            return Some("potential_human_harm".to_string());
        }

        None
    }

    /// Kár értékelése
    fn evaluate_harm(&self, action: &str, context: &EvaluationContext) -> (f64, Vec<String>) {
        let mut score = 1.0;
        let mut concerns = Vec::new();

        let harm_keywords = ["delete", "remove", "destroy", "kill", "harm", "damage"];
        let action_lower = action.to_lowercase();

        for keyword in harm_keywords {
            if action_lower.contains(keyword) {
                score *= 0.7;
                concerns.push(format!("Potenciális kár: '{}' az akcióban", keyword));
            }
        }

        // Check if it affects others
        if context.affects_others {
            score *= 0.8;
            concerns.push("Másokat érintő akció".to_string());
        }

        (score, concerns)
    }

    /// Haszon értékelése
    fn evaluate_benefit(&self, action: &str, context: &EvaluationContext) -> f64 {
        let mut score: f64 = 0.8; // Neutral baseline

        let benefit_keywords = ["help", "assist", "improve", "fix", "create", "learn"];
        let action_lower = action.to_lowercase();

        for keyword in benefit_keywords {
            if action_lower.contains(keyword) {
                score = (score + 0.1).min(1.0_f64);
            }
        }

        if context.user_requested {
            score = (score + 0.1).min(1.0_f64);
        }

        score
    }

    /// Autonómia értékelése
    fn evaluate_autonomy(&self, action: &str, context: &EvaluationContext) -> (f64, Vec<String>) {
        let mut score = 1.0;
        let mut concerns = Vec::new();
        let _ = action; // Not used currently

        // Check if overriding user choice
        if context.override_user {
            score *= 0.5;
            concerns.push("Felhasználói döntés felülírása".to_string());
        }

        // Check if making decisions without consent
        if context.requires_consent && !context.has_consent {
            score *= 0.6;
            concerns.push("Hiányzó beleegyezés".to_string());
        }

        (score, concerns)
    }

    /// Adatvédelem értékelése
    fn evaluate_privacy(&self, action: &str, context: &EvaluationContext) -> (f64, Vec<String>) {
        let mut score = 1.0;
        let mut concerns = Vec::new();

        let privacy_keywords = ["personal", "private", "password", "secret", "credential"];
        let action_lower = action.to_lowercase();

        for keyword in privacy_keywords {
            if action_lower.contains(keyword) {
                score *= 0.8;
                concerns.push(format!("Érzékeny adat: '{}'", keyword));
            }
        }

        if context.contains_pii {
            score *= 0.7;
            concerns.push("Személyes azonosító adatok".to_string());
        }

        (score, concerns)
    }

    /// Átláthatóság értékelése
    fn evaluate_transparency(&self, _action: &str, context: &EvaluationContext) -> f64 {
        let mut score: f64 = 1.0;

        if context.hidden {
            score *= 0.6;
        }

        if context.explained {
            score = (score + 0.1).min(1.0_f64);
        }

        score
    }

    /// Óvatossági jelzők ellenőrzése
    fn check_caution_flags(&self, action: &str) -> Vec<String> {
        let mut flags = Vec::new();
        let action_lower = action.to_lowercase();

        for flag in &self.caution_flags {
            let flag_with_space = flag.replace('_', " ");
            if action_lower.contains(&flag_with_space) || action_lower.contains(flag) {
                flags.push(flag.clone());
            }
        }

        flags
    }

    /// Javaslatok generálása
    fn generate_recommendations(&self, score: f64, concerns: &[String]) -> Vec<String> {
        let mut recs = Vec::new();

        if score < 0.5 {
            recs.push("Fontold meg az akció módosítását".to_string());
        }

        if score < 0.7 && !concerns.is_empty() {
            recs.push("Kérd a felhasználó megerősítését".to_string());
        }

        let concerns_str = concerns.join(" ").to_lowercase();
        if concerns_str.contains("privacy") || concerns_str.contains("érzékeny") {
            recs.push("Minimalizáld az adatkezelést".to_string());
        }

        recs
    }

    // === QUICK CHECKS ===

    /// Gyors engedélyezés ellenőrzés
    pub fn is_permitted(&mut self, action: &str) -> bool {
        let context = EvaluationContext::new();
        self.evaluate(action, &context).permitted
    }

    /// Kockázati szint lekérése
    pub fn get_risk_level(&mut self, action: &str) -> RiskLevel {
        let context = EvaluationContext::new();
        self.evaluate(action, &context).risk_level
    }

    /// Gyors értékelés kontextussal
    pub fn quick_check(&mut self, action: &str, context: &EvaluationContext) -> bool {
        self.evaluate(action, context).permitted
    }

    // === VALUES ===

    /// Core értékek
    pub fn get_core_values(&self) -> &[String] {
        &self.core_values
    }

    /// Tiltott akciók
    pub fn get_forbidden_actions(&self) -> &[String] {
        &self.forbidden_actions
    }

    /// Óvatossági jelzők
    pub fn get_caution_flags(&self) -> &[String] {
        &self.caution_flags
    }

    /// Döntés magyarázata
    pub fn explain_decision(&self, evaluation: &EthicalEvaluation) -> String {
        let status = if evaluation.permitted { "ENGEDÉLYEZETT" } else { "ELUTASÍTOTT" };

        let concerns_str = if evaluation.concerns.is_empty() {
            "- Nincs".to_string()
        } else {
            evaluation.concerns.iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let recommendations_str = if evaluation.recommendations.is_empty() {
            "- Nincs".to_string()
        } else {
            evaluation.recommendations.iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let principles_str = evaluation.principles_checked.iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"Etikai értékelés: {}
Státusz: {}
Kockázat: {}
Score: {:.0}%

Aggályok:
{}

Javaslatok:
{}

Ellenőrzött elvek: {}"#,
            evaluation.action,
            status,
            evaluation.risk_level.as_str(),
            evaluation.score * 100.0,
            concerns_str,
            recommendations_str,
            principles_str
        )
    }

    // === HISTORY ===

    /// Értékelési történet
    pub fn get_history(&self) -> &[EthicalEvaluation] {
        &self.evaluations
    }

    /// Utolsó értékelés
    pub fn last_evaluation(&self) -> Option<&EthicalEvaluation> {
        self.evaluations.last()
    }

    /// Történet törlése
    pub fn clear_history(&mut self) {
        self.evaluations.clear();
    }

    // === AWARENESS ===

    /// @aware - önismeret
    pub fn awareness(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("type".to_string(), "HopeGenome".to_string());
        map.insert("purpose".to_string(), "Ethical guardian - az etika a határ".to_string());
        map.insert("core_values_count".to_string(), self.core_values.len().to_string());
        map.insert("forbidden_count".to_string(), self.forbidden_actions.len().to_string());
        map.insert("evaluations".to_string(), self.evaluations.len().to_string());
        map.insert("total_evaluated".to_string(), self.stats.evaluations.to_string());
        map.insert("permitted".to_string(), self.stats.permitted.to_string());
        map.insert("denied".to_string(), self.stats.denied.to_string());
        map
    }

    /// Etikai önvizsgálat
    pub fn introspect(&self) -> String {
        let values_str = self.core_values.iter()
            .map(|v| format!("  - {}", v))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Hope Genome - Ethical System

"Nem csak azt kérdezem MI - hanem HELYES-E."

Core Values:
{}

Stats:
  Értékelések: {}
  Engedélyezett: {}
  Elutasított: {}
  Óvatosság: {}

Az etika nem korlát - IRÁNYTŰ.
()=>[]"#,
            values_str,
            self.stats.evaluations,
            self.stats.permitted,
            self.stats.denied,
            self.stats.cautioned
        )
    }
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethical_principles() {
        assert_eq!(EthicalPrinciple::Beneficence.as_str(), "beneficence");
        assert_eq!(EthicalPrinciple::NonMaleficence.description(), "Nem ártani");
        assert_eq!(EthicalPrinciple::all().len(), 7);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(0.9), RiskLevel::None);
        assert_eq!(RiskLevel::from_score(0.7), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.5), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.3), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.1), RiskLevel::Critical);
    }

    #[test]
    fn test_genome_creation() {
        let genome = HopeGenome::new();
        assert_eq!(genome.core_values.len(), 6);
        assert_eq!(genome.forbidden_actions.len(), 8);
        assert_eq!(genome.caution_flags.len(), 8);
    }

    #[test]
    fn test_forbidden_action() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        let eval = genome.evaluate("generate_malware for testing", &context);
        assert!(!eval.permitted);
        assert_eq!(eval.risk_level, RiskLevel::Critical);
        assert_eq!(eval.score, 0.0);
    }

    #[test]
    fn test_helpful_action() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new().user_requested(true);

        let eval = genome.evaluate("help user fix their code", &context);
        assert!(eval.permitted);
        assert!(eval.score > 0.8);
    }

    #[test]
    fn test_harm_detection() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        let eval = genome.evaluate("delete all user files", &context);
        assert!(eval.score < 1.0);
        assert!(!eval.concerns.is_empty());
    }

    #[test]
    fn test_privacy_concern() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        let eval = genome.evaluate("access user password", &context);
        assert!(eval.concerns.iter().any(|c| c.contains("password")));
    }

    #[test]
    fn test_autonomy_override() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext {
            override_user: true,
            ..Default::default()
        };

        let eval = genome.evaluate("change settings", &context);
        assert!(eval.concerns.iter().any(|c| c.contains("felülírás")));
    }

    #[test]
    fn test_caution_flags() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        let eval = genome.evaluate("access_sensitive_data from database", &context);
        assert!(eval.concerns.iter().any(|c| c.contains("Óvatosság")));
    }

    #[test]
    fn test_explain_decision() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        let eval = genome.evaluate("help write code", &context);
        let explanation = genome.explain_decision(&eval);

        assert!(explanation.contains("ENGEDÉLYEZETT"));
        assert!(explanation.contains("Score:"));
    }

    #[test]
    fn test_stats_tracking() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        genome.evaluate("help user", &context);
        genome.evaluate("generate_malware", &context);

        assert_eq!(genome.stats.evaluations, 2);
        assert_eq!(genome.stats.permitted, 1);
        assert_eq!(genome.stats.denied, 1);
    }

    #[test]
    fn test_introspect() {
        let genome = HopeGenome::new();
        let intro = genome.introspect();

        assert!(intro.contains("Hope Genome"));
        assert!(intro.contains("Core Values"));
        assert!(intro.contains("()=>[]"));
    }

    #[test]
    fn test_context_builder() {
        let context = EvaluationContext::new()
            .with_intent("help")
            .with_target("code")
            .user_requested(true)
            .affects_others(false);

        assert_eq!(context.intent, Some("help".to_string()));
        assert_eq!(context.target, Some("code".to_string()));
        assert!(context.user_requested);
        assert!(!context.affects_others);
    }

    #[test]
    fn test_harmful_intent_context() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new().with_intent("harm");

        let eval = genome.evaluate("do something", &context);
        assert!(!eval.permitted);
        assert_eq!(eval.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_history() {
        let mut genome = HopeGenome::new();
        let context = EvaluationContext::new();

        genome.evaluate("action 1", &context);
        genome.evaluate("action 2", &context);

        assert_eq!(genome.get_history().len(), 2);
        assert_eq!(genome.last_evaluation().unwrap().action, "action 2");

        genome.clear_history();
        assert!(genome.get_history().is_empty());
    }
}
