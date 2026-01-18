use sqlx::{Row, SqlitePool};
use thiserror::Error;

use crate::domain::models::RecoveryMetrics;
use crate::domain::types::unix_ms_to_iso8601;
use crate::shared::time::now_millis;

#[derive(Error, Debug)]
pub enum RecoveryMetricsRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub struct RecoveryMetricsRepo;

impl RecoveryMetricsRepo {
    pub async fn record_attempt(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<RecoveryMetrics, RecoveryMetricsRepoError> {
        let now = now_millis();
        sqlx::query(
            r#"
            INSERT INTO recovery_metrics (task_id, success_count, attempt_count, created_at, updated_at)
            VALUES (?1, 0, 1, ?2, ?2)
            ON CONFLICT(task_id) DO UPDATE SET
                attempt_count = attempt_count + 1,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(task_id)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_metrics(pool, task_id).await
    }

    pub async fn record_success(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<RecoveryMetrics, RecoveryMetricsRepoError> {
        let now = now_millis();
        sqlx::query(
            r#"
            INSERT INTO recovery_metrics (task_id, success_count, attempt_count, created_at, updated_at)
            VALUES (?1, 1, 0, ?2, ?2)
            ON CONFLICT(task_id) DO UPDATE SET
                success_count = success_count + 1,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(task_id)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_metrics(pool, task_id).await
    }

    pub async fn get_metrics(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<RecoveryMetrics, RecoveryMetricsRepoError> {
        let row = sqlx::query(
            r#"
            SELECT task_id, success_count, attempt_count, updated_at
            FROM recovery_metrics
            WHERE task_id = ?1
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let success_count: i64 = row.try_get("success_count")?;
            let attempt_count: i64 = row.try_get("attempt_count")?;
            let updated_at: i64 = row.try_get("updated_at")?;
            return Ok(RecoveryMetrics {
                task_id: task_id.to_string(),
                success_count: success_count.max(0) as u64,
                attempt_count: attempt_count.max(0) as u64,
                recovery_rate: compute_recovery_rate(success_count, attempt_count),
                updated_at: unix_ms_to_iso8601(updated_at),
            });
        }

        Ok(RecoveryMetrics {
            task_id: task_id.to_string(),
            success_count: 0,
            attempt_count: 0,
            recovery_rate: 0.0,
            updated_at: unix_ms_to_iso8601(now_millis()),
        })
    }
}

fn compute_recovery_rate(success: i64, attempt: i64) -> f64 {
    if attempt <= 0 {
        0.0
    } else {
        (success.max(0) as f64) / (attempt as f64)
    }
}
