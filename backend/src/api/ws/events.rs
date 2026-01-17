//! WebSocket 事件定义（从 shared 复用）
//! 格式：{domain}:{action}

pub use crate::shared::ws::{
    ArtifactGetAckPayload, ArtifactGetPayload, ArtifactUpdateAckPayload, ArtifactUpdatePayload,
    ArtifactUpdatedPayload, CMD_ARTIFACT_GET, CMD_ARTIFACT_UPDATE, CMD_TASK_PAUSE, CMD_TASK_RESUME,
    EVT_ARTIFACT_GET_ACK, EVT_ARTIFACT_UPDATE_ACK, EVT_ARTIFACT_UPDATED, EVT_ITERATION_PAUSED,
    EVT_ITERATION_RESUMED, EVT_TASK_PAUSE_ACK, EVT_TASK_RESUME_ACK, IterationPausedPayload,
    IterationResumedPayload, TaskControlAckPayload, TaskControlPayload, WsMessage,
};
