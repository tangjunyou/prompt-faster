//! 优化任务（OptimizationTask）领域模型（以 workspace 为第一隔离边界）

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ExecutionTargetType {
    Dify,
    Generic,
    Example,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum OptimizationTaskMode {
    Fixed,
    Creative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum OptimizationTaskStatus {
    /// 草稿（初始状态）
    Draft,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成（自动达到目标）
    Completed,
    /// 已终止（用户手动终止）
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export_to = "models/")]
pub struct OptimizationTaskEntity {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub goal: String,
    pub execution_target_type: ExecutionTargetType,
    pub task_mode: OptimizationTaskMode,
    pub status: OptimizationTaskStatus,
    pub config_json: Option<String>,
    pub final_prompt: Option<String>,
    #[ts(type = "number | null")]
    pub terminated_at: Option<i64>,
    pub selected_iteration_id: Option<String>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}
