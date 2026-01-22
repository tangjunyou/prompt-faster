use std::collections::HashSet;

use crate::domain::models::{
    BaselineComparison, DiversityAnalysisResult, DiversityBaseline, DiversityConfig,
    DiversityMetrics, DiversitySuggestion, DiversityTrend, DiversityWarning, DiversityWarningLevel,
};
use crate::domain::types::unix_ms_to_iso8601;
use crate::shared::time::now_millis;

pub trait DiversityAnalyzer: Send + Sync {
    fn compute_lexical_diversity(&self, outputs: &[String]) -> f64;
    fn compute_structural_diversity(&self, outputs: &[String]) -> f64;
    fn compute_semantic_diversity(
        &self,
        outputs: &[String],
        embeddings: Option<&[Vec<f64>]>,
    ) -> f64;
    fn analyze(
        &self,
        outputs: &[String],
        baseline: Option<&DiversityBaseline>,
        embeddings: Option<&[Vec<f64>]>,
    ) -> DiversityAnalysisResult;
}

#[derive(Debug, Clone)]
pub struct DefaultDiversityAnalyzer {
    config: DiversityConfig,
}

impl DefaultDiversityAnalyzer {
    pub fn new(config: DiversityConfig) -> Self {
        Self { config }
    }

    fn normalize_outputs(outputs: &[String]) -> Vec<String> {
        outputs
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn compute_overall_score(&self, metrics: &DiversityMetrics, semantic_available: bool) -> f64 {
        let mut parts = Vec::new();
        if self.config.compute_lexical {
            parts.push(metrics.lexical_diversity);
        }
        if self.config.compute_structural {
            parts.push(metrics.structural_diversity);
        }
        if self.config.compute_semantic && semantic_available {
            parts.push(metrics.semantic_diversity);
        }
        if parts.is_empty() {
            0.0
        } else {
            let sum: f64 = parts.iter().sum();
            (sum / parts.len() as f64).clamp(0.0, 1.0)
        }
    }

    fn build_baseline_comparison(
        &self,
        metrics: &DiversityMetrics,
        baseline: &DiversityBaseline,
    ) -> BaselineComparison {
        let overall_diff = metrics.overall_score - baseline.metrics.overall_score;
        let lexical_diff = metrics.lexical_diversity - baseline.metrics.lexical_diversity;
        let structural_diff = metrics.structural_diversity - baseline.metrics.structural_diversity;
        let semantic_diff = metrics.semantic_diversity - baseline.metrics.semantic_diversity;
        let trend = if overall_diff > 0.01 {
            DiversityTrend::Improved
        } else if overall_diff < -0.01 {
            DiversityTrend::Declined
        } else {
            DiversityTrend::Stable
        };
        BaselineComparison {
            overall_diff,
            lexical_diff,
            structural_diff,
            semantic_diff,
            trend,
        }
    }

    fn build_warnings(
        &self,
        metrics: &DiversityMetrics,
        semantic_available: bool,
    ) -> Vec<DiversityWarning> {
        if !self.config.enabled {
            return Vec::new();
        }
        let mut warnings = Vec::new();
        if self.config.compute_semantic && !semantic_available {
            warnings.push(DiversityWarning {
                level: DiversityWarningLevel::Low,
                message: "语义多样性暂不可用（未提供 embedding）".to_string(),
                affected_metrics: vec!["semantic".to_string()],
            });
        }
        if metrics.overall_score >= self.config.warning_threshold {
            return warnings;
        }
        let mut affected_metrics = Vec::new();
        if self.config.compute_lexical {
            affected_metrics.push("lexical".to_string());
        }
        if self.config.compute_structural {
            affected_metrics.push("structural".to_string());
        }
        if self.config.compute_semantic && semantic_available {
            affected_metrics.push("semantic".to_string());
        }
        warnings.push(DiversityWarning {
            level: if metrics.overall_score < 0.15 {
                DiversityWarningLevel::High
            } else if metrics.overall_score < 0.25 {
                DiversityWarningLevel::Medium
            } else {
                DiversityWarningLevel::Low
            },
            message: "优化可能导致输出过于单一".to_string(),
            affected_metrics,
        });
        warnings
    }

    fn build_suggestions(&self, metrics: &DiversityMetrics) -> Vec<DiversitySuggestion> {
        if metrics.overall_score >= self.config.warning_threshold {
            return Vec::new();
        }
        vec![DiversitySuggestion {
            suggestion_type: "adjust_goal".to_string(),
            content: "建议调整优化目标，增加多样性约束或放宽格式限制。".to_string(),
        }]
    }
}

impl DiversityAnalyzer for DefaultDiversityAnalyzer {
    fn compute_lexical_diversity(&self, outputs: &[String]) -> f64 {
        if outputs.len() < 2 {
            return 0.0;
        }
        let mut tokens = Vec::new();
        let mut total_tokens = 0usize;
        for output in outputs {
            let tk = tokenize_for_diversity(output);
            total_tokens += tk.len();
            tokens.extend(tk);
        }
        if total_tokens == 0 {
            return 0.0;
        }
        let unique: HashSet<String> = tokens.into_iter().collect();
        let ttr = unique.len() as f64 / total_tokens as f64;
        let pairwise = average_pairwise_jaccard_distance(outputs, 50);
        ((ttr + pairwise) / 2.0).clamp(0.0, 1.0)
    }

    fn compute_structural_diversity(&self, outputs: &[String]) -> f64 {
        if outputs.len() < 2 {
            return 0.0;
        }
        let lengths: Vec<f64> = outputs.iter().map(|s| s.chars().count() as f64).collect();
        let mean = lengths.iter().sum::<f64>() / lengths.len() as f64;
        let variance = if mean > 0.0 {
            lengths
                .iter()
                .map(|l| (l - mean) * (l - mean))
                .sum::<f64>()
                / lengths.len() as f64
        } else {
            0.0
        };
        let std = variance.sqrt();
        let length_diversity = if mean > 0.0 {
            (std / mean).min(1.0)
        } else {
            0.0
        };
        let mut format_patterns = HashSet::new();
        for output in outputs {
            format_patterns.insert(format_signature(output));
        }
        let format_diversity = (format_patterns.len() as f64 / outputs.len() as f64).clamp(0.0, 1.0);
        ((length_diversity + format_diversity) / 2.0).clamp(0.0, 1.0)
    }

    fn compute_semantic_diversity(
        &self,
        _outputs: &[String],
        embeddings: Option<&[Vec<f64>]>,
    ) -> f64 {
        let Some(embeddings) = embeddings else {
            return 0.0;
        };
        if embeddings.len() < 2 {
            return 0.0;
        }
        let n = embeddings.len().min(50);
        let mut total = 0.0;
        let mut count = 0usize;
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = 1.0 - cosine_similarity(&embeddings[i], &embeddings[j]);
                total += dist;
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            (total / count as f64).clamp(0.0, 1.0)
        }
    }

    fn analyze(
        &self,
        outputs: &[String],
        baseline: Option<&DiversityBaseline>,
        embeddings: Option<&[Vec<f64>]>,
    ) -> DiversityAnalysisResult {
        let normalized = Self::normalize_outputs(outputs);
        let semantic_available = embeddings.is_some();
        let mut metrics = DiversityMetrics::default();
        if self.config.compute_lexical {
            metrics.lexical_diversity = self.compute_lexical_diversity(&normalized);
        }
        if self.config.compute_structural {
            metrics.structural_diversity = self.compute_structural_diversity(&normalized);
        }
        if self.config.compute_semantic && semantic_available {
            metrics.semantic_diversity = self.compute_semantic_diversity(&normalized, embeddings);
        }
        metrics.overall_score = self.compute_overall_score(&metrics, semantic_available);

        let baseline_comparison = baseline.map(|b| self.build_baseline_comparison(&metrics, b));
        let warnings = self.build_warnings(&metrics, semantic_available);
        let suggestions = self.build_suggestions(&metrics);

        DiversityAnalysisResult {
            metrics,
            baseline_comparison,
            warnings,
            suggestions,
            analyzed_at: unix_ms_to_iso8601(now_millis()),
            sample_count: normalized.len() as u32,
        }
    }
}

fn tokenize_for_diversity(text: &str) -> Vec<String> {
    if text.chars().any(|c| matches!(c as u32, 0x4E00..=0x9FFF)) {
        let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.len() < 2 {
            return chars.into_iter().map(|c| c.to_string()).collect();
        }
        return chars
            .windows(2)
            .map(|w| w.iter().collect::<String>())
            .collect();
    }
    text.split_whitespace().map(|s| s.to_string()).collect()
}

fn average_pairwise_jaccard_distance(outputs: &[String], max_samples: usize) -> f64 {
    let count = outputs.len().min(max_samples);
    if count < 2 {
        return 0.0;
    }
    let mut token_sets = Vec::with_capacity(count);
    for output in outputs.iter().take(count) {
        token_sets.push(tokenize_for_diversity(output).into_iter().collect::<HashSet<String>>());
    }
    let mut total = 0.0;
    let mut pairs = 0usize;
    for i in 0..count {
        for j in (i + 1)..count {
            total += jaccard_distance(&token_sets[i], &token_sets[j]);
            pairs += 1;
        }
    }
    if pairs == 0 {
        0.0
    } else {
        (total / pairs as f64).clamp(0.0, 1.0)
    }
}

fn jaccard_distance(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        1.0 - (intersection / union)
    }
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(-1.0, 1.0)
}

fn format_signature(output: &str) -> String {
    let trimmed = output.trim();
    let mut flags = Vec::new();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        flags.push("json");
    }
    if output.contains("```") {
        flags.push("code");
    }
    if output.lines().any(|l| l.trim_start().starts_with("- "))
        || output.lines().any(|l| l.trim_start().starts_with("* "))
    {
        flags.push("bullets");
    }
    if output
        .lines()
        .any(|l| l.trim_start().chars().next().is_some_and(|c| c.is_ascii_digit()))
    {
        flags.push("numbered");
    }
    if flags.is_empty() {
        "plain".to_string()
    } else {
        flags.join("|")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyzer_with_threshold(threshold: f64) -> DefaultDiversityAnalyzer {
        let mut cfg = DiversityConfig::default();
        cfg.enabled = true;
        cfg.warning_threshold = threshold;
        DefaultDiversityAnalyzer::new(cfg)
    }

    #[test]
    fn lexical_diversity_empty_returns_zero() {
        let analyzer = DefaultDiversityAnalyzer::new(DiversityConfig::default());
        let score = analyzer.compute_lexical_diversity(&[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn lexical_diversity_handles_chinese_tokens() {
        let analyzer = DefaultDiversityAnalyzer::new(DiversityConfig::default());
        let outputs = vec!["中文测试".to_string(), "中文检测".to_string()];
        let score = analyzer.compute_lexical_diversity(&outputs);
        assert!(score > 0.0);
    }

    #[test]
    fn structural_diversity_detects_length_and_format_changes() {
        let analyzer = DefaultDiversityAnalyzer::new(DiversityConfig::default());
        let outputs = vec![
            "- a\n- b".to_string(),
            "1. a\n2. b\n3. c".to_string(),
            "{ \"a\": 1 }".to_string(),
        ];
        let score = analyzer.compute_structural_diversity(&outputs);
        assert!(score > 0.0);
    }

    #[test]
    fn warning_generated_when_below_threshold() {
        let analyzer = analyzer_with_threshold(0.9);
        let outputs = vec!["same".to_string(), "same".to_string()];
        let analysis = analyzer.analyze(&outputs, None, None);
        assert!(!analysis.warnings.is_empty());
        assert!(analysis
            .warnings
            .iter()
            .any(|w| w.message.contains("输出过于单一")));
    }

    #[test]
    fn semantic_disabled_returns_zero() {
        let analyzer = DefaultDiversityAnalyzer::new(DiversityConfig::default());
        let outputs = vec!["a".to_string(), "b".to_string()];
        let analysis = analyzer.analyze(&outputs, None, None);
        assert_eq!(analysis.metrics.semantic_diversity, 0.0);
    }

    #[test]
    fn semantic_unavailable_adds_warning() {
        let mut cfg = DiversityConfig::default();
        cfg.enabled = true;
        cfg.compute_semantic = true;
        cfg.warning_threshold = 0.0;
        let analyzer = DefaultDiversityAnalyzer::new(cfg);
        let outputs = vec!["a".to_string(), "b".to_string()];
        let analysis = analyzer.analyze(&outputs, None, None);
        assert!(analysis
            .warnings
            .iter()
            .any(|w| w.message.contains("语义多样性暂不可用")));
    }

    #[test]
    fn pairwise_sampling_limits_to_first_50() {
        let mut outputs = vec!["same".to_string(); 50];
        outputs.extend(vec!["diff".to_string(); 10]);
        let distance = average_pairwise_jaccard_distance(&outputs, 50);
        assert_eq!(distance, 0.0);
    }
}
