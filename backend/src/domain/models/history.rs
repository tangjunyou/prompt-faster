use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::models::CheckpointListResponse;
use crate::domain::models::history_event::HistoryEvent;
use crate::domain::types::IterationHistorySummary;

/// 任务历史聚合响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TaskHistoryResponse {
    pub iterations: Vec<IterationHistorySummary>,
    pub checkpoints: CheckpointListResponse,
}

/// 历史事件列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct HistoryEventResponse {
    pub events: Vec<HistoryEvent>,
    pub total: u32,
    pub has_more: bool,
}

/// 时间线条目类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum TimelineEntryType {
    Iteration,
    Checkpoint,
    Event,
}

/// 时间线条目
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TimelineEntry {
    pub id: String,
    pub entry_type: TimelineEntryType,
    pub timestamp: String,
    pub iteration: Option<u32>,
    pub title: String,
    pub description: Option<String>,
    pub actor: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// 时间线响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TimelineResponse {
    pub entries: Vec<TimelineEntry>,
    pub total: u32,
    pub has_more: bool,
}

/// 历史导出数据
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct HistoryExportData {
    pub task: TaskExportMeta,
    pub iterations: Vec<IterationExportEntry>,
    pub checkpoints: Vec<crate::domain::models::CheckpointSummary>,
    pub events: Vec<HistoryEvent>,
    pub branches: Vec<BranchInfo>,
    pub truncated: bool,
    pub event_total: u32,
    pub checkpoint_total: u32,
    pub export_limit: u32,
    pub exported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TaskExportMeta {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationExportEntry {
    pub iteration: u32,
    pub prompt: Option<String>,
    pub rule_system: Option<String>,
    pub pass_rate: Option<f64>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct BranchInfo {
    pub branch_id: String,
    pub parent_branch_id: Option<String>,
    pub created_at: String,
    pub checkpoint_count: u32,
}
