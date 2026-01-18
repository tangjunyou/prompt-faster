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
