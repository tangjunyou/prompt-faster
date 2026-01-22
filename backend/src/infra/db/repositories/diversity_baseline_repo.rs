use sqlx::{Row, SqlitePool};
use thiserror::Error;

use crate::domain::models::{DiversityBaseline, DiversityMetrics};
use crate::domain::types::unix_ms_to_iso8601;
use crate::shared::time::now_millis;

#[derive(Error, Debug)]
pub enum DiversityBaselineRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("多样性基准线解析失败: {0}")]
    ParseError(#[from] serde_json::Error),
}

pub struct DiversityBaselineRepo;

impl DiversityBaselineRepo {
    pub async fn get_by_task_id(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<Option<DiversityBaseline>, DiversityBaselineRepoError> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, metrics_json, recorded_at, iteration
            FROM diversity_baselines
            WHERE task_id = ?1
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(parse_baseline_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn insert_if_absent(
        pool: &SqlitePool,
        task_id: &str,
        metrics: &DiversityMetrics,
        iteration: u32,
    ) -> Result<(), DiversityBaselineRepoError> {
        let now = now_millis();
        let metrics_json = serde_json::to_string(metrics)?;
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO diversity_baselines (id, task_id, metrics_json, recorded_at, iteration)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(task_id) DO NOTHING
            "#,
        )
        .bind(&id)
        .bind(task_id)
        .bind(metrics_json)
        .bind(now)
        .bind(iteration as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(
        pool: &SqlitePool,
        task_id: &str,
        metrics: &DiversityMetrics,
        iteration: u32,
    ) -> Result<DiversityBaseline, DiversityBaselineRepoError> {
        let now = now_millis();
        let metrics_json = serde_json::to_string(metrics)?;
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO diversity_baselines (id, task_id, metrics_json, recorded_at, iteration)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(task_id) DO UPDATE SET
                metrics_json = excluded.metrics_json,
                recorded_at = excluded.recorded_at,
                iteration = excluded.iteration
            "#,
        )
        .bind(&id)
        .bind(task_id)
        .bind(metrics_json)
        .bind(now)
        .bind(iteration as i64)
        .execute(pool)
        .await?;

        Self::get_by_task_id(pool, task_id)
            .await?
            .ok_or_else(|| DiversityBaselineRepoError::DatabaseError(sqlx::Error::RowNotFound))
    }
}

fn parse_baseline_row(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<DiversityBaseline, DiversityBaselineRepoError> {
    let id: String = row.try_get("id")?;
    let task_id: String = row.try_get("task_id")?;
    let metrics_json: String = row.try_get("metrics_json")?;
    let metrics: DiversityMetrics = serde_json::from_str(&metrics_json)?;
    let recorded_at: i64 = row.try_get("recorded_at")?;
    let iteration: i64 = row.try_get("iteration")?;
    Ok(DiversityBaseline {
        id,
        task_id,
        metrics,
        recorded_at: unix_ms_to_iso8601(recorded_at),
        iteration: iteration.max(0) as u32,
    })
}
