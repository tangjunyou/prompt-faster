use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// 老师模型 Prompt（数据库完整记录）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPrompt {
    pub id: String,
    pub user_id: String,
    pub version: i32,
    pub content: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 老师模型 Prompt 版本摘要（列表展示用）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPromptVersion {
    pub id: String,
    pub version: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub success_rate: Option<f64>,
    pub task_count: i32,
    pub created_at: String,
}

/// 版本统计信息
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPromptStats {
    pub version_id: String,
    pub version: i32,
    pub total_tasks: i32,
    pub successful_tasks: i32,
    pub success_rate: Option<f64>,
    pub average_pass_rate: Option<f64>,
}

/// 创建新版本的输入
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CreateTeacherPromptInput {
    pub content: String,
    pub description: Option<String>,
    #[serde(default = "default_activate")]
    pub activate: bool,
}

fn default_activate() -> bool {
    true
}

/// 元优化概览
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct MetaOptimizationOverview {
    pub total_versions: i32,
    pub active_version: Option<TeacherPromptVersion>,
    pub best_version: Option<TeacherPromptVersion>,
    pub stats: Vec<TeacherPromptStats>,
}

/// 元优化历史任务摘要（用于选择入口展示）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct MetaOptimizationTaskSummary {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub status: String,
    pub pass_rate: Option<f64>,
    pub created_at: String,
}
