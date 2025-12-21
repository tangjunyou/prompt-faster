//! WebSocket 事件定义
//! 格式：{domain}:{action}

use serde::{Deserialize, Serialize};

/// WebSocket 消息结构
#[derive(Serialize, Deserialize, Debug)]
pub struct WsMessage<T> {
    /// 事件类型：{domain}:{action}
    #[serde(rename = "type")]
    pub event_type: String,
    /// 消息负载
    pub payload: T,
    /// ISO 8601 时间戳
    pub timestamp: String,
    /// 追踪 ID (AR2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl<T> WsMessage<T> {
    /// 创建新的 WebSocket 消息
    pub fn new(event_type: impl Into<String>, payload: T) -> Self {
        Self {
            event_type: event_type.into(),
            payload,
            timestamp: chrono_timestamp(),
            correlation_id: None,
        }
    }

    /// 设置 correlationId
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// 获取 ISO 8601 时间戳
fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        1970 + secs / 31536000,
        1,
        1,
        (secs % 86400) / 3600,
        (secs % 3600) / 60,
        secs % 60,
        millis
    )
}
