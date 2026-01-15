//! WebSocket 事件定义
//! 格式：{domain}:{action}

use serde::{Deserialize, Serialize};

/// WebSocket 消息结构
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| now.unix_timestamp().to_string())
}

#[cfg(test)]
mod tests {
    use super::WsMessage;

    #[test]
    fn ws_message_serializes_with_camel_case_and_correlation_id() {
        let msg = WsMessage::new("iteration:started", serde_json::json!({ "ok": true }))
            .with_correlation_id("cid-123");

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
