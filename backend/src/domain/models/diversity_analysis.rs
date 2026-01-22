//! 多样性分析相关 DTO

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum DiversityTrend {
    Improved,
    Declined,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiversityMetrics {
    pub lexical_diversity: f64,
    pub structural_diversity: f64,
    pub semantic_diversity: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct BaselineComparison {
    pub overall_diff: f64,
    pub lexical_diff: f64,
    pub structural_diff: f64,
    pub semantic_diff: f64,
    pub trend: DiversityTrend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum DiversityWarningLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiversityWarning {
    pub level: DiversityWarningLevel,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiversitySuggestion {
    pub suggestion_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiversityAnalysisResult {
    pub metrics: DiversityMetrics,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_comparison: Option<BaselineComparison>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<DiversityWarning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<DiversitySuggestion>,
    pub analyzed_at: String,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiversityBaseline {
    pub id: String,
    pub task_id: String,
    pub metrics: DiversityMetrics,
    pub recorded_at: String,
    pub iteration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiversityConfig {
    pub enabled: bool,
    pub warning_threshold: f64,
    pub compute_lexical: bool,
    pub compute_structural: bool,
    pub compute_semantic: bool,
}

impl Default for DiversityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            warning_threshold: 0.3,
            compute_lexical: true,
            compute_structural: true,
            compute_semantic: false,
        }
    }
}
