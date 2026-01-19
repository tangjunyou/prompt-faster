use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ts_rs::TS;
use utoipa::ToSchema;

/// 历史事件类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum EventType {
    IterationStarted,
    IterationCompleted,
    EvaluationCompleted,
    UserPause,
    UserResume,
    UserEdit,
    UserGuidance,
    Rollback,
    CheckpointSaved,
    ErrorOccurred,
    ConfigChanged,
    TaskTerminated,
    CheckpointRecovered,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IterationStarted => "iteration_started",
            Self::IterationCompleted => "iteration_completed",
            Self::EvaluationCompleted => "evaluation_completed",
            Self::UserPause => "user_pause",
            Self::UserResume => "user_resume",
            Self::UserEdit => "user_edit",
            Self::UserGuidance => "user_guidance",
            Self::Rollback => "rollback",
            Self::CheckpointSaved => "checkpoint_saved",
            Self::ErrorOccurred => "error_occurred",
            Self::ConfigChanged => "config_changed",
            Self::TaskTerminated => "task_terminated",
            Self::CheckpointRecovered => "checkpoint_recovered",
        }
    }
}

impl FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iteration_started" => Ok(Self::IterationStarted),
            "iteration_completed" => Ok(Self::IterationCompleted),
            "evaluation_completed" => Ok(Self::EvaluationCompleted),
            "user_pause" => Ok(Self::UserPause),
            "user_resume" => Ok(Self::UserResume),
            "user_edit" => Ok(Self::UserEdit),
            "user_guidance" => Ok(Self::UserGuidance),
            "rollback" => Ok(Self::Rollback),
            "checkpoint_saved" => Ok(Self::CheckpointSaved),
            "error_occurred" => Ok(Self::ErrorOccurred),
            "config_changed" => Ok(Self::ConfigChanged),
            "task_terminated" => Ok(Self::TaskTerminated),
            "checkpoint_recovered" => Ok(Self::CheckpointRecovered),
            other => Err(format!("unknown event_type: {other}")),
        }
    }
}

/// 操作者
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum Actor {
    System,
    User,
}

impl Actor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
        }
    }
}

impl FromStr for Actor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "system" => Ok(Self::System),
            "user" => Ok(Self::User),
            other => Err(format!("unknown actor: {other}")),
        }
    }
}

/// 历史事件（API 表达）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct HistoryEvent {
    pub id: String,
    pub task_id: String,
    pub event_type: EventType,
    pub actor: Actor,
    pub details: Option<serde_json::Value>,
    pub iteration: Option<u32>,
    pub correlation_id: Option<String>,
    pub created_at: String,
}

/// 历史事件筛选条件
#[derive(Debug, Clone, Default)]
pub struct HistoryEventFilter {
    pub event_types: Option<Vec<EventType>>,
    pub actor: Option<Actor>,
    pub iteration_min: Option<u32>,
    pub iteration_max: Option<u32>,
    pub time_start: Option<i64>,
    pub time_end: Option<i64>,
}
