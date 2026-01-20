use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// 诊断报告
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiagnosticReport {
    pub task_id: String,
    pub task_name: String,
    pub status: String,
    pub summary: DiagnosticSummary,
    pub turning_points: Vec<TurningPoint>,
    pub improvement_suggestions: Vec<String>,
    pub failed_cases: Vec<FailedCaseSummary>,
}

/// 诊断摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiagnosticSummary {
    pub total_iterations: u32,
    pub failed_iterations: u32,
    pub success_iterations: u32,
    pub common_failure_reasons: Vec<FailureReasonEntry>,
    pub natural_language_explanation: String,
}

/// 失败原因条目
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct FailureReasonEntry {
    pub reason: String,
    pub count: u32,
    pub percentage: f64,
}

/// 关键转折点
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TurningPoint {
    pub round: u32,
    pub event_type: TurningPointType,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_rate_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_rate_after: Option<f64>,
    pub timestamp: String,
}

/// 转折点类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum TurningPointType {
    Improvement,
    Regression,
    Breakthrough,
}

/// 失败用例摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct FailedCaseSummary {
    pub case_id: String,
    pub input_preview: String,
    pub failure_reason: String,
    pub iteration_round: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_case_id: Option<String>,
}

/// 失败用例详情
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct FailedCaseDetail {
    pub case_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_case_id: Option<String>,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_output: Option<String>,
    pub failure_reason: String,
    pub iteration_round: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_used: Option<String>,
    pub diff_segments: Vec<DiffSegment>,
}

/// 差异片段
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiffSegment {
    pub segment_type: DiffSegmentType,
    pub content: String,
    pub start_index: u32,
    pub end_index: u32,
}

/// 差异类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum DiffSegmentType {
    Added,
    Removed,
    Unchanged,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_report_serialization() {
        let report = DiagnosticReport {
            task_id: "task-1".to_string(),
            task_name: "Demo".to_string(),
            status: "completed".to_string(),
            summary: DiagnosticSummary {
                total_iterations: 3,
                failed_iterations: 1,
                success_iterations: 2,
                common_failure_reasons: vec![FailureReasonEntry {
                    reason: "format".to_string(),
                    count: 2,
                    percentage: 66.7,
                }],
                natural_language_explanation: "原因示例".to_string(),
            },
            turning_points: vec![TurningPoint {
                round: 2,
                event_type: TurningPointType::Improvement,
                description: "提升".to_string(),
                pass_rate_before: Some(0.3),
                pass_rate_after: Some(0.5),
                timestamp: "2025-01-01T00:00:00Z".to_string(),
            }],
            improvement_suggestions: vec!["补充格式示例".to_string()],
            failed_cases: vec![FailedCaseSummary {
                case_id: "iter-1:case-1".to_string(),
                input_preview: "input".to_string(),
                failure_reason: "format".to_string(),
                iteration_round: 1,
                test_case_id: Some("case-1".to_string()),
            }],
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"taskId\":\"task-1\""));
        assert!(json.contains("\"taskName\":\"Demo\""));
        assert!(json.contains("\"turningPoints\""));
        assert!(json.contains("\"improvementSuggestions\""));
        assert!(json.contains("\"failedCases\""));
        assert!(json.contains("\"commonFailureReasons\""));
        assert!(json.contains("\"naturalLanguageExplanation\""));
    }

    #[test]
    fn test_turning_point_type_serialization() {
        let json = serde_json::to_string(&TurningPointType::Breakthrough).unwrap();
        assert_eq!(json, "\"breakthrough\"");
    }

    #[test]
    fn test_diff_segment_type_serialization() {
        let json = serde_json::to_string(&DiffSegmentType::Removed).unwrap();
        assert_eq!(json, "\"removed\"");
    }
}
