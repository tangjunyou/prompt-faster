//! 历史事件仓库

use chrono::DateTime;
use serde_json::json;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::str::FromStr;
use thiserror::Error;
use tracing::info;

use crate::domain::models::{
    Actor, EventType, HistoryEvent, HistoryEventFilter, TimelineEntry, TimelineEntryType,
};
use crate::domain::types::unix_ms_to_iso8601;
use crate::shared::time::now_millis;

#[derive(Debug, Error)]
pub enum HistoryEventRepoError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("JSON 解析错误: {0}")]
    JsonParse(String),
    #[error("数据解析错误: {0}")]
    InvalidData(String),
}

#[derive(Debug, sqlx::FromRow)]
struct HistoryEventRow {
    id: String,
    task_id: String,
    event_type: String,
    actor: String,
    details: Option<String>,
    iteration: Option<i64>,
    correlation_id: Option<String>,
    created_at: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct TimelineRow {
    id: String,
    entry_type: String,
    timestamp: i64,
    iteration: Option<i64>,
    title: String,
    description: Option<String>,
    actor: Option<String>,
    details: Option<String>,
    pass_rate: Option<f64>,
    passed_cases: Option<i64>,
    total_cases: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct HistoryEventRepo;

impl HistoryEventRepo {
    pub async fn create_event(
        pool: &SqlitePool,
        event: &HistoryEvent,
    ) -> Result<(), HistoryEventRepoError> {
        let created_at = parse_iso_to_unix_ms(&event.created_at).unwrap_or_else(now_millis);
        let details = serialize_optional_json(&event.details)?;
        let iteration_state = event
            .iteration
            .map(|value| value.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        sqlx::query(
            r#"
            INSERT INTO history_events (
                id, task_id, event_type, actor, details, iteration, correlation_id, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.id)
        .bind(&event.task_id)
        .bind(event.event_type.as_str())
        .bind(event.actor.as_str())
        .bind(details)
        .bind(event.iteration.map(|value| value as i64))
        .bind(&event.correlation_id)
        .bind(created_at)
        .execute(pool)
        .await?;

        info!(
            correlation_id = %event.correlation_id.as_deref().unwrap_or("unknown"),
            user_id = %event.actor.as_str(),
            task_id = %event.task_id,
            event_id = %event.id,
            event_type = %event.event_type.as_str(),
            actor = %event.actor.as_str(),
            action = "history_event_created",
            prev_state = "N/A",
            new_state = "N/A",
            iteration_state = %iteration_state,
            timestamp = created_at,
            "保存历史事件"
        );

        Ok(())
    }

    pub async fn list_events(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryEvent>, HistoryEventRepoError> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, task_id, event_type, actor, details, iteration, correlation_id, created_at \
             FROM history_events WHERE task_id = ",
        );
        qb.push_bind(task_id);
        apply_event_filters(&mut qb, filter);
        qb.push(" ORDER BY created_at DESC LIMIT ");
        qb.push_bind(limit as i64);
        qb.push(" OFFSET ");
        qb.push_bind(offset as i64);

        let rows: Vec<HistoryEventRow> = qb.build_query_as().fetch_all(pool).await?;
        Ok(rows
            .into_iter()
            .map(row_to_event)
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub async fn count_events(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
    ) -> Result<u32, HistoryEventRepoError> {
        let mut qb: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) as count FROM history_events WHERE task_id = ");
        qb.push_bind(task_id);
        apply_event_filters(&mut qb, filter);

        let row: (i64,) = qb.build_query_as().fetch_one(pool).await?;
        Ok(row.0.max(0) as u32)
    }

    pub async fn get_event_by_id(
        pool: &SqlitePool,
        event_id: &str,
    ) -> Result<Option<HistoryEvent>, HistoryEventRepoError> {
        let row: Option<HistoryEventRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, event_type, actor, details, iteration, correlation_id, created_at
            FROM history_events
            WHERE id = ?
            "#,
        )
        .bind(event_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_event(row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_timeline_entries(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TimelineEntry>, HistoryEventRepoError> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, entry_type, timestamp, iteration, title, description, actor, details, pass_rate, passed_cases, total_cases FROM (",
        );

        qb.push(
            "SELECT id, 'iteration' AS entry_type, started_at AS timestamp, round AS iteration, ",
        );
        qb.push("'iteration' AS title, status AS description, NULL AS actor, NULL AS details, ");
        qb.push(
            "pass_rate AS pass_rate, passed_cases AS passed_cases, total_cases AS total_cases ",
        );
        qb.push("FROM iterations WHERE task_id = ");
        qb.push_bind(task_id);
        apply_timeline_filters(&mut qb, filter, TimelineFilterScope::Iteration);

        qb.push(" UNION ALL ");
        qb.push("SELECT id, 'checkpoint' AS entry_type, created_at AS timestamp, iteration AS iteration, ");
        qb.push("'checkpoint' AS title, state AS description, NULL AS actor, NULL AS details, ");
        qb.push("NULL AS pass_rate, NULL AS passed_cases, NULL AS total_cases ");
        qb.push("FROM checkpoints WHERE task_id = ");
        qb.push_bind(task_id);
        apply_timeline_filters(&mut qb, filter, TimelineFilterScope::Checkpoint);

        qb.push(" UNION ALL ");
        qb.push(
            "SELECT id, 'event' AS entry_type, created_at AS timestamp, iteration AS iteration, ",
        );
        qb.push("event_type AS title, NULL AS description, actor AS actor, details AS details, ");
        qb.push("NULL AS pass_rate, NULL AS passed_cases, NULL AS total_cases ");
        qb.push("FROM history_events WHERE task_id = ");
        qb.push_bind(task_id);
        apply_timeline_filters(&mut qb, filter, TimelineFilterScope::Event);

        qb.push(") ORDER BY timestamp DESC LIMIT ");
        qb.push_bind(limit as i64);
        qb.push(" OFFSET ");
        qb.push_bind(offset as i64);

        let rows: Vec<TimelineRow> = qb.build_query_as().fetch_all(pool).await?;
        rows.into_iter().map(row_to_timeline_entry).collect()
    }

    pub async fn count_timeline_entries(
        pool: &SqlitePool,
        task_id: &str,
        filter: &HistoryEventFilter,
    ) -> Result<u32, HistoryEventRepoError> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT COUNT(*) as count FROM (");

        qb.push("SELECT id FROM iterations WHERE task_id = ");
        qb.push_bind(task_id);
        apply_timeline_filters(&mut qb, filter, TimelineFilterScope::Iteration);

        qb.push(" UNION ALL ");
        qb.push("SELECT id FROM checkpoints WHERE task_id = ");
        qb.push_bind(task_id);
        apply_timeline_filters(&mut qb, filter, TimelineFilterScope::Checkpoint);

        qb.push(" UNION ALL ");
        qb.push("SELECT id FROM history_events WHERE task_id = ");
        qb.push_bind(task_id);
        apply_timeline_filters(&mut qb, filter, TimelineFilterScope::Event);

        qb.push(")");

        let row: (i64,) = qb.build_query_as().fetch_one(pool).await?;
        Ok(row.0.max(0) as u32)
    }
}

#[derive(Debug, Clone, Copy)]
enum TimelineFilterScope {
    Iteration,
    Checkpoint,
    Event,
}

fn apply_event_filters(qb: &mut QueryBuilder<Sqlite>, filter: &HistoryEventFilter) {
    if let Some(event_types) = filter
        .event_types
        .as_ref()
        .filter(|items| !items.is_empty())
    {
        qb.push(" AND event_type IN (");
        let mut separated = qb.separated(", ");
        for event_type in event_types {
            separated.push_bind(event_type.as_str());
        }
        qb.push(")");
    }
    if let Some(actor) = filter.actor.as_ref() {
        qb.push(" AND actor = ");
        qb.push_bind(actor.as_str());
    }
    if let Some(min) = filter.iteration_min {
        qb.push(" AND iteration IS NOT NULL AND iteration >= ");
        qb.push_bind(min as i64);
    }
    if let Some(max) = filter.iteration_max {
        qb.push(" AND iteration IS NOT NULL AND iteration <= ");
        qb.push_bind(max as i64);
    }
    if let Some(start) = filter.time_start {
        qb.push(" AND created_at >= ");
        qb.push_bind(start);
    }
    if let Some(end) = filter.time_end {
        qb.push(" AND created_at <= ");
        qb.push_bind(end);
    }
}

fn apply_timeline_filters(
    qb: &mut QueryBuilder<Sqlite>,
    filter: &HistoryEventFilter,
    scope: TimelineFilterScope,
) {
    let iteration_col = match scope {
        TimelineFilterScope::Iteration => "round",
        TimelineFilterScope::Checkpoint | TimelineFilterScope::Event => "iteration",
    };
    let time_col = match scope {
        TimelineFilterScope::Iteration => "started_at",
        TimelineFilterScope::Checkpoint | TimelineFilterScope::Event => "created_at",
    };

    if let Some(min) = filter.iteration_min {
        qb.push(" AND ");
        qb.push(iteration_col);
        qb.push(" IS NOT NULL AND ");
        qb.push(iteration_col);
        qb.push(" >= ");
        qb.push_bind(min as i64);
    }
    if let Some(max) = filter.iteration_max {
        qb.push(" AND ");
        qb.push(iteration_col);
        qb.push(" IS NOT NULL AND ");
        qb.push(iteration_col);
        qb.push(" <= ");
        qb.push_bind(max as i64);
    }
    if let Some(start) = filter.time_start {
        qb.push(" AND ");
        qb.push(time_col);
        qb.push(" >= ");
        qb.push_bind(start);
    }
    if let Some(end) = filter.time_end {
        qb.push(" AND ");
        qb.push(time_col);
        qb.push(" <= ");
        qb.push_bind(end);
    }

    if matches!(scope, TimelineFilterScope::Event) {
        if let Some(event_types) = filter
            .event_types
            .as_ref()
            .filter(|items| !items.is_empty())
        {
            qb.push(" AND event_type IN (");
            let mut separated = qb.separated(", ");
            for event_type in event_types {
                separated.push_bind(event_type.as_str());
            }
            qb.push(")");
        }
        if let Some(actor) = filter.actor.as_ref() {
            qb.push(" AND actor = ");
            qb.push_bind(actor.as_str());
        }
    } else if filter.event_types.is_some() || filter.actor.is_some() {
        // 如果筛选条件只对事件有效，则非事件条目不参与
        qb.push(" AND 1 = 0");
    }
}

fn row_to_event(row: HistoryEventRow) -> Result<HistoryEvent, HistoryEventRepoError> {
    let event_type = EventType::from_str(&row.event_type)
        .map_err(|err| HistoryEventRepoError::InvalidData(err))?;
    let actor =
        Actor::from_str(&row.actor).map_err(|err| HistoryEventRepoError::InvalidData(err))?;
    let details = parse_optional_json(row.details.as_deref())?;

    Ok(HistoryEvent {
        id: row.id,
        task_id: row.task_id,
        event_type,
        actor,
        details,
        iteration: row.iteration.map(|value| value as u32),
        correlation_id: row.correlation_id,
        created_at: unix_ms_to_iso8601(row.created_at),
    })
}

fn row_to_timeline_entry(row: TimelineRow) -> Result<TimelineEntry, HistoryEventRepoError> {
    let entry_type = match row.entry_type.as_str() {
        "iteration" => TimelineEntryType::Iteration,
        "checkpoint" => TimelineEntryType::Checkpoint,
        "event" => TimelineEntryType::Event,
        other => {
            return Err(HistoryEventRepoError::InvalidData(format!(
                "unknown timeline entry_type: {other}"
            )));
        }
    };

    let mut details = parse_optional_json(row.details.as_deref())?;
    let mut description = row.description.clone();

    if matches!(entry_type, TimelineEntryType::Iteration) {
        if let (Some(pass_rate), Some(passed_cases), Some(total_cases)) =
            (row.pass_rate, row.passed_cases, row.total_cases)
        {
            let percent = (pass_rate * 100.0).max(0.0);
            let status = row
                .description
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            description = Some(format!(
                "{} · {:.1}% ({}/{})",
                status, percent, passed_cases, total_cases
            ));
            details = Some(json!({
                "status": status,
                "passRate": pass_rate,
                "passedCases": passed_cases,
                "totalCases": total_cases,
            }));
        }
    } else if matches!(entry_type, TimelineEntryType::Checkpoint) && details.is_none() {
        if let Some(state) = row.description.clone() {
            details = Some(json!({ "state": state }));
        }
    }

    Ok(TimelineEntry {
        id: row.id,
        entry_type,
        timestamp: unix_ms_to_iso8601(row.timestamp),
        iteration: row.iteration.map(|value| value as u32),
        title: row.title,
        description,
        actor: row.actor,
        details,
    })
}

fn serialize_optional_json(
    value: &Option<serde_json::Value>,
) -> Result<Option<String>, HistoryEventRepoError> {
    match value {
        Some(v) => serde_json::to_string(v)
            .map(Some)
            .map_err(|err| HistoryEventRepoError::JsonParse(err.to_string())),
        None => Ok(None),
    }
}

fn parse_optional_json(
    value: Option<&str>,
) -> Result<Option<serde_json::Value>, HistoryEventRepoError> {
    match value {
        Some(raw) => serde_json::from_str(raw)
            .map(Some)
            .map_err(|err| HistoryEventRepoError::JsonParse(err.to_string())),
        None => Ok(None),
    }
}

fn parse_iso_to_unix_ms(value: &str) -> Option<i64> {
    if let Ok(ms) = value.parse::<i64>() {
        return Some(ms);
    }
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.timestamp_millis())
        .ok()
}
