//! Checkpoint 相关模型

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ts_rs::TS;

use crate::domain::models::{IterationState, LineageType, RuleSystem};
use crate::domain::types::{IterationArtifacts, RunControlState, UserGuidance};

pub use crate::domain::models::algorithm::Checkpoint;

/// Checkpoint 数据库实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointEntity {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: IterationState,
    pub run_control_state: RunControlState,
    pub prompt: String,
    pub rule_system: RuleSystem,
    pub artifacts: Option<IterationArtifacts>,
    pub user_guidance: Option<UserGuidance>,
    pub branch_id: String,
    pub parent_id: Option<String>,
    pub lineage_type: LineageType,
    pub branch_description: Option<String>,
    pub checksum: String,
    pub created_at: i64,
}

/// Checkpoint 全量结构（与数据库实体一致）
pub type CheckpointFull = CheckpointEntity;

/// Checkpoint 创建请求（内部使用）
#[derive(Debug, Clone)]
pub struct CheckpointCreateRequest {
    pub task_id: String,
    pub iteration: u32,
    pub state: IterationState,
    pub run_control_state: RunControlState,
    pub prompt: String,
    pub rule_system: RuleSystem,
    pub artifacts: Option<IterationArtifacts>,
    pub user_guidance: Option<UserGuidance>,
    pub branch_id: String,
    pub parent_id: Option<String>,
    pub lineage_type: LineageType,
    pub branch_description: Option<String>,
}

/// Checkpoint API 响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointResponse {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: String,
    pub run_control_state: String,
    pub prompt_preview: String,
    pub has_artifacts: bool,
    pub has_user_guidance: bool,
    pub checksum: String,
    pub integrity_ok: bool,
    pub created_at: String,
}

/// Checkpoint 列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CheckpointListResponse {
    pub checkpoints: Vec<CheckpointResponse>,
    pub total: u32,
}
