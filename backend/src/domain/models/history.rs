use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::models::CheckpointListResponse;
use crate::domain::types::IterationHistorySummary;

/// 任务历史聚合响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TaskHistoryResponse {
    pub iterations: Vec<IterationHistorySummary>,
    pub checkpoints: CheckpointListResponse,
}
