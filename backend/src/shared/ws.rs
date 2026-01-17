//! WebSocket 事件与消息定义（共享层）
//! 格式：{domain}:{action}

use crate::domain::types::{IterationArtifacts, RunControlState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

// ============================================================================
// WS 事件类型常量
// ============================================================================

/// 任务暂停命令
pub const CMD_TASK_PAUSE: &str = "task:pause";
/// 任务继续命令
pub const CMD_TASK_RESUME: &str = "task:resume";
/// 迭代已暂停事件
pub const EVT_ITERATION_PAUSED: &str = "iteration:paused";
/// 迭代已继续事件
pub const EVT_ITERATION_RESUMED: &str = "iteration:resumed";
/// 暂停命令 ACK
pub const EVT_TASK_PAUSE_ACK: &str = "task:pause:ack";
/// 继续命令 ACK
pub const EVT_TASK_RESUME_ACK: &str = "task:resume:ack";

/// 获取产物命令
pub const CMD_ARTIFACT_GET: &str = "artifact:get";
/// 更新产物命令
pub const CMD_ARTIFACT_UPDATE: &str = "artifact:update";
/// 获取产物 ACK
pub const EVT_ARTIFACT_GET_ACK: &str = "artifact:get:ack";
/// 更新产物 ACK
pub const EVT_ARTIFACT_UPDATE_ACK: &str = "artifact:update:ack";
/// 产物已更新事件（广播）
pub const EVT_ARTIFACT_UPDATED: &str = "artifact:updated";

// ============================================================================
// WS 命令负载
// ============================================================================

/// 暂停/继续命令负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct TaskControlPayload {
    /// 任务 ID
    pub task_id: String,
}

/// 获取产物命令负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct ArtifactGetPayload {
    /// 任务 ID
    pub task_id: String,
}

/// 更新产物命令负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct ArtifactUpdatePayload {
    /// 任务 ID
    pub task_id: String,
    /// 更新后的产物
    pub artifacts: IterationArtifacts,
}

// ============================================================================
// WS 事件负载
// ============================================================================

/// 迭代已暂停事件负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct IterationPausedPayload {
    /// 任务 ID
    pub task_id: String,
    /// 暂停时间（ISO 8601）
    pub paused_at: String,
    /// 暂停时所处的阶段
    pub stage: String,
    /// 当前迭代轮次
    pub iteration: u32,
}

/// 迭代已继续事件负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct IterationResumedPayload {
    /// 任务 ID
    pub task_id: String,
    /// 继续时间（ISO 8601）
    pub resumed_at: String,
}

/// 暂停/继续命令 ACK 负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct TaskControlAckPayload {
    /// 任务 ID
    pub task_id: String,
    /// 命令是否成功处理（包括幂等 no-op）
    pub ok: bool,
    /// 是否产生状态变更
    pub applied: bool,
    /// 当前状态
    pub current_state: RunControlState,
    /// 目标状态
    pub target_state: RunControlState,
    /// 可选的原因说明（幂等或拒绝）
    pub reason: Option<String>,
    /// 可选的上下文快照（用于诊断）
    pub context_snapshot: Option<Value>,
}

/// 获取产物 ACK 负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct ArtifactGetAckPayload {
    /// 任务 ID
    pub task_id: String,
    /// 是否成功
    pub ok: bool,
    /// 产物（成功时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<IterationArtifacts>,
    /// 失败原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// 更新产物 ACK 负载
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct ArtifactUpdateAckPayload {
    /// 任务 ID
    pub task_id: String,
    /// 是否成功
    pub ok: bool,
    /// 是否产生状态变更
    pub applied: bool,
    /// 更新后的产物（成功时返回）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<IterationArtifacts>,
    /// 失败原因
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// 产物已更新事件负载（广播给所有订阅者）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "ws/")]
pub struct ArtifactUpdatedPayload {
    /// 任务 ID
    pub task_id: String,
    /// 更新后的产物
    pub artifacts: IterationArtifacts,
    /// 编辑者 ID
    pub edited_by: String,
}

/// WebSocket 消息结构
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WsMessage<T> {
    /// 事件类型：{domain}:{action}
    #[serde(rename = "type")]
    pub event_type: String,
    /// 消息负载
    pub payload: T,
    /// ISO 8601 时间戳
    pub timestamp: String,
    /// 追踪 ID (AR2) - 必填
    pub correlation_id: String,
}

impl<T> WsMessage<T> {
    /// 创建新的 WebSocket 消息（correlationId 必填）
    pub fn new(
        event_type: impl Into<String>,
        payload: T,
        correlation_id: impl Into<String>,
    ) -> Self {
        let correlation_id = correlation_id.into();
        let correlation_id = if correlation_id.trim().is_empty() {
            format!("system-{}", chrono_timestamp())
        } else {
            correlation_id
        };
        Self {
            event_type: event_type.into(),
            payload,
            timestamp: chrono_timestamp(),
            correlation_id,
        }
    }
}

/// 获取 ISO 8601 时间戳
pub fn chrono_timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| now.unix_timestamp().to_string())
}

#[cfg(test)]
mod tests {
    use super::WsMessage;

    #[test]
    fn ws_message_serializes_with_camel_case_and_correlation_id() {
        let msg = WsMessage::new(
            "iteration:started",
            serde_json::json!({ "ok": true }),
            "cid-123",
        );

        let v = serde_json::to_value(&msg).unwrap();
        assert_eq!(v.get("type").unwrap(), "iteration:started");
        assert!(v.get("payload").is_some());
        assert!(v.get("timestamp").is_some());

        // Must be camelCase (architecture contract) – no snake_case leakage.
        assert_eq!(v.get("correlationId").unwrap(), "cid-123");
        assert!(v.get("correlation_id").is_none());

        // Basic ISO 8601 / RFC3339 shape check (avoid pulling in extra parsing features).
        let ts = v.get("timestamp").unwrap().as_str().unwrap();
        assert!(
            ts.contains('T') && (ts.ends_with('Z') || ts.contains('+') || ts.contains('-')),
            "timestamp not RFC3339-like: {ts}"
        );
    }
}
