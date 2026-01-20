use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ResultExportFormat {
    Markdown,
    Json,
    Xml,
}

/// 迭代摘要条目
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationSummaryEntry {
    pub round: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_rate: Option<f64>,
    pub status: String,
}

/// 结果查看 DTO
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TaskResultView {
    pub task_id: String,
    pub task_name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_rate: Option<f64>,
    pub total_iterations: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    pub created_at: String,
    pub iteration_summary: Vec<IterationSummaryEntry>,
}

/// 导出结果响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct ExportResultResponse {
    pub content: String,
    pub format: ResultExportFormat,
    pub filename: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_export_format_serialization() {
        let json = serde_json::to_string(&ResultExportFormat::Markdown).unwrap();
        assert_eq!(json, "\"markdown\"");
    }

    #[test]
    fn test_task_result_view_serialization() {
        let view = TaskResultView {
            task_id: "task-1".to_string(),
            task_name: "Demo".to_string(),
            status: "completed".to_string(),
            best_prompt: Some("prompt".to_string()),
            pass_rate: Some(0.9),
            total_iterations: 3,
            completed_at: Some("2025-01-01T00:00:00Z".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            iteration_summary: vec![IterationSummaryEntry {
                round: 1,
                pass_rate: Some(0.8),
                status: "completed".to_string(),
            }],
        };

        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"taskId\":\"task-1\""));
        assert!(json.contains("\"taskName\":\"Demo\""));
        assert!(json.contains("\"bestPrompt\":\"prompt\""));
        assert!(json.contains("\"passRate\":0.9"));
        assert!(json.contains("\"totalIterations\":3"));
        assert!(json.contains("\"iterationSummary\""));
    }
}
