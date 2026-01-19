//! 历史事件记录辅助

use serde_json::Value;
use thiserror::Error;
use tracing::warn;
use uuid::Uuid;

use crate::domain::models::{Actor, EventType, HistoryEvent};
use crate::domain::types::unix_ms_to_iso8601;
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::infra::db::pool::global_db_pool;
use crate::infra::db::repositories::{HistoryEventRepo, HistoryEventRepoError};
use crate::shared::time::now_millis;

#[derive(Debug, Error)]
pub enum RecordEventError {
    #[error("数据库未初始化")]
    DatabaseNotInitialized,
    #[error("仓库错误: {0}")]
    Repo(#[from] HistoryEventRepoError),
}

pub async fn record_event(
    task_id: &str,
    event_type: EventType,
    actor: Actor,
    details: Option<Value>,
    iteration: Option<u32>,
    correlation_id: Option<String>,
) -> Result<(), RecordEventError> {
    let pool = global_db_pool().ok_or(RecordEventError::DatabaseNotInitialized)?;
    let now = now_millis();
    let correlation_id = correlation_id.filter(|value| !value.trim().is_empty());
    let correlation_id = match correlation_id {
        Some(value) => Some(value),
        None => {
            let registry = global_pause_registry();
            let controller = registry.get_or_create(task_id).await;
            controller.get_last_correlation_id().await
        }
    };
    let correlation_id = correlation_id.filter(|value| !value.trim().is_empty());
    let correlation_id = match correlation_id {
        Some(value) => Some(value),
        None => {
            warn!(
                task_id = %task_id,
                event_type = %event_type.as_str(),
                "历史事件缺少 correlation_id，已降级为 unknown"
            );
            Some("unknown".to_string())
        }
    };
    let event = HistoryEvent {
        id: Uuid::new_v4().to_string(),
        task_id: task_id.to_string(),
        event_type,
        actor,
        details,
        iteration,
        correlation_id,
        created_at: unix_ms_to_iso8601(now),
    };

    HistoryEventRepo::create_event(&pool, &event).await?;
    Ok(())
}

pub fn record_event_async(
    task_id: String,
    event_type: EventType,
    actor: Actor,
    details: Option<Value>,
    iteration: Option<u32>,
    correlation_id: Option<String>,
) {
    tokio::spawn(async move {
        if let Err(err) = record_event(
            &task_id,
            event_type,
            actor,
            details,
            iteration,
            correlation_id,
        )
        .await
        {
            warn!(
                task_id = %task_id,
                error = %err,
                "记录历史事件失败"
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn record_event_async_does_not_block_on_missing_db() {
        record_event_async(
            "task-1".to_string(),
            EventType::ErrorOccurred,
            Actor::System,
            None,
            None,
            Some("cid-test".to_string()),
        );
        // Give spawned task a chance to run; test passes if no panic.
        sleep(Duration::from_millis(10)).await;
    }
}
