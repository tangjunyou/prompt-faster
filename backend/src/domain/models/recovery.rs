//! 断点恢复相关模型

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// 未完成任务信息
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct UnfinishedTask {
    pub task_id: String,
    pub task_name: String,
    /// 最近一次 Checkpoint ID
    pub checkpoint_id: String,
    /// 最近一次 Checkpoint 时间（ISO 8601）
    pub last_checkpoint_at: String,
    pub iteration: u32,
    /// 迭代状态（string）
    pub state: String,
    /// 运行控制状态（string）
    pub run_control_state: String,
}

/// 未完成任务列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct UnfinishedTasksResponse {
    pub tasks: Vec<UnfinishedTask>,
    pub total: u32,
}

/// 恢复请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RecoveryRequest {
    /// 可选指定 Checkpoint，默认使用最近的
    pub checkpoint_id: Option<String>,
}

/// 恢复响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RecoveryResponse {
    pub success: bool,
    pub task_id: String,
    pub checkpoint_id: String,
    pub iteration: u32,
    pub state: String,
    pub run_control_state: String,
    pub message: String,
}

/// 通过率摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PassRateSummary {
    pub total_cases: u32,
    pub passed_cases: u32,
    pub pass_rate: f64, // 0.0 - 1.0
}

/// Checkpoint 摘要（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointSummary {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: String,
    pub pass_rate_summary: Option<PassRateSummary>,
    pub created_at: String,
    pub archived_at: Option<String>,
    pub archive_reason: Option<String>,
    pub branch_id: String,
    pub parent_id: Option<String>,
}

/// Checkpoint 详情（含摘要字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointWithSummary {
    pub checkpoint: crate::domain::models::CheckpointEntity,
    pub pass_rate_summary: Option<PassRateSummary>,
    pub archived_at: Option<String>,
    pub archive_reason: Option<String>,
}

/// 回滚请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RollbackRequest {
    pub checkpoint_id: String,
    pub confirm: bool,
}

/// 回滚响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RollbackResponse {
    pub success: bool,
    pub task_id: String,
    pub checkpoint_id: String,
    pub new_branch_id: String,
    pub archived_count: u32,
    pub iteration: u32,
    pub state: String,
    pub message: String,
}

/// 恢复率统计
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct RecoveryMetrics {
    pub task_id: String,
    pub success_count: u64,
    pub attempt_count: u64,
    /// 成功率（0.0 ~ 1.0）
    pub recovery_rate: f64,
    /// 最近一次更新（ISO 8601）
    pub updated_at: String,
}

/// 网络连接状态
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ConnectivityStatus {
    Online,
    Offline,
    Limited,
}

/// 网络状态响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct ConnectivityResponse {
    pub status: ConnectivityStatus,
    pub last_checked_at: String,
    pub message: Option<String>,
    pub available_features: Vec<String>,
    pub restricted_features: Vec<String>,
}
