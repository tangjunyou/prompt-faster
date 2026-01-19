//! Checkpoint Repository

use serde::{Serialize, de::DeserializeOwned};
use sqlx::SqlitePool;
use thiserror::Error;
use tracing::info;

use crate::domain::models::{
    CheckpointEntity, CheckpointSummary, CheckpointWithSummary, IterationState, LineageType,
    PassRateSummary, RuleSystem,
};
use crate::domain::types::{IterationArtifacts, RunControlState, UserGuidance, unix_ms_to_iso8601};
use crate::shared::time::now_millis;

#[derive(Debug, Error)]
pub enum CheckpointRepoError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("JSON 解析错误: {0}")]
    JsonParse(String),
}

#[derive(Debug, sqlx::FromRow)]
struct CheckpointRow {
    id: String,
    task_id: String,
    iteration: i64,
    state: String,
    run_control_state: String,
    prompt: String,
    rule_system: String,
    artifacts: Option<String>,
    user_guidance: Option<String>,
    branch_id: String,
    parent_id: Option<String>,
    lineage_type: String,
    branch_description: Option<String>,
    checksum: String,
    created_at: i64,
    archived_at: Option<i64>,
    archive_reason: Option<String>,
    pass_rate_summary: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct CheckpointSummaryRow {
    id: String,
    task_id: String,
    iteration: i64,
    state: String,
    created_at: i64,
    archived_at: Option<i64>,
    archive_reason: Option<String>,
    pass_rate_summary: Option<String>,
    branch_id: String,
    parent_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CheckpointRepo;

impl CheckpointRepo {
    pub async fn create_checkpoint(
        pool: &SqlitePool,
        checkpoint: CheckpointEntity,
    ) -> Result<CheckpointEntity, CheckpointRepoError> {
        let state = serialize_json(&checkpoint.state)?;
        let run_control_state = serialize_json(&checkpoint.run_control_state)?;
        let rule_system = serialize_json(&checkpoint.rule_system)?;
        let artifacts = serialize_optional_json(&checkpoint.artifacts)?;
        let user_guidance = serialize_optional_json(&checkpoint.user_guidance)?;
        let lineage_type = serialize_json(&checkpoint.lineage_type)?;
        let pass_rate_summary = serialize_optional_json(&checkpoint.pass_rate_summary)?;

        sqlx::query(
            r#"
            INSERT INTO checkpoints (
                id, task_id, iteration, state, run_control_state, prompt,
                rule_system, artifacts, user_guidance, branch_id, parent_id,
                lineage_type, branch_description, checksum, created_at,
                archived_at, archive_reason, pass_rate_summary
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&checkpoint.id)
        .bind(&checkpoint.task_id)
        .bind(checkpoint.iteration as i64)
        .bind(state)
        .bind(run_control_state)
        .bind(&checkpoint.prompt)
        .bind(rule_system)
        .bind(artifacts)
        .bind(user_guidance)
        .bind(&checkpoint.branch_id)
        .bind(&checkpoint.parent_id)
        .bind(lineage_type)
        .bind(&checkpoint.branch_description)
        .bind(&checkpoint.checksum)
        .bind(checkpoint.created_at)
        .bind(checkpoint.archived_at)
        .bind(&checkpoint.archive_reason)
        .bind(pass_rate_summary)
        .execute(pool)
        .await?;

        info!(
            task_id = %checkpoint.task_id,
            checkpoint_id = %checkpoint.id,
            "保存 checkpoint"
        );

        Ok(checkpoint)
    }

    pub async fn get_checkpoint_by_id(
        pool: &SqlitePool,
        id: &str,
    ) -> Result<Option<CheckpointEntity>, CheckpointRepoError> {
        let row: Option<CheckpointRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, iteration, state, run_control_state, prompt,
                   rule_system, artifacts, user_guidance, branch_id, parent_id,
                   lineage_type, branch_description, checksum, created_at,
                   archived_at, archive_reason, pass_rate_summary
            FROM checkpoints
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        row.map(row_to_entity).transpose()
    }

    pub async fn get_checkpoint_for_user(
        pool: &SqlitePool,
        user_id: &str,
        checkpoint_id: &str,
    ) -> Result<Option<CheckpointEntity>, CheckpointRepoError> {
        let row: Option<CheckpointRow> = sqlx::query_as(
            r#"
            SELECT c.id, c.task_id, c.iteration, c.state, c.run_control_state, c.prompt,
                   c.rule_system, c.artifacts, c.user_guidance, c.branch_id, c.parent_id,
                   c.lineage_type, c.branch_description, c.checksum, c.created_at,
                   c.archived_at, c.archive_reason, c.pass_rate_summary
            FROM checkpoints c
            JOIN optimization_tasks t ON t.id = c.task_id
            JOIN workspaces w ON w.id = t.workspace_id
            WHERE c.id = ?1 AND w.user_id = ?2
            "#,
        )
        .bind(checkpoint_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        row.map(row_to_entity).transpose()
    }

    pub async fn list_checkpoints_by_task(
        pool: &SqlitePool,
        task_id: &str,
        limit: u32,
    ) -> Result<Vec<CheckpointEntity>, CheckpointRepoError> {
        let rows: Vec<CheckpointRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, iteration, state, run_control_state, prompt,
                   rule_system, artifacts, user_guidance, branch_id, parent_id,
                   lineage_type, branch_description, checksum, created_at,
                   archived_at, archive_reason, pass_rate_summary
            FROM checkpoints
            WHERE task_id = ?
              AND archived_at IS NULL
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(task_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(row_to_entity).collect()
    }

    pub async fn list_checkpoint_summaries(
        pool: &SqlitePool,
        task_id: &str,
        include_archived: bool,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<CheckpointSummary>, CheckpointRepoError> {
        let rows: Vec<CheckpointSummaryRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, iteration, state, created_at,
                   archived_at, archive_reason, pass_rate_summary, branch_id, parent_id
            FROM checkpoints
            WHERE task_id = ?
              AND (? OR archived_at IS NULL)
            ORDER BY created_at DESC, id DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(task_id)
        .bind(include_archived)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let pass_rate_summary = parse_optional_json::<PassRateSummary>(row.pass_rate_summary)?;
            let state = parse_json::<IterationState>(&row.state)?;
            items.push(CheckpointSummary {
                id: row.id,
                task_id: row.task_id,
                iteration: row.iteration as u32,
                state: iteration_state_label(&state),
                pass_rate_summary,
                created_at: unix_ms_to_iso8601(row.created_at),
                archived_at: row.archived_at.map(unix_ms_to_iso8601),
                archive_reason: row.archive_reason,
                branch_id: row.branch_id,
                parent_id: row.parent_id,
            });
        }

        Ok(items)
    }

    pub async fn count_checkpoints_by_task_with_archived(
        pool: &SqlitePool,
        task_id: &str,
        include_archived: bool,
    ) -> Result<u32, CheckpointRepoError> {
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM checkpoints
            WHERE task_id = ?
              AND (? OR archived_at IS NULL)
            "#,
        )
        .bind(task_id)
        .bind(include_archived)
        .fetch_one(pool)
        .await?;

        Ok(total as u32)
    }

    pub async fn count_archived_by_reason(
        pool: &SqlitePool,
        task_id: &str,
        reason: &str,
    ) -> Result<u32, CheckpointRepoError> {
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM checkpoints
            WHERE task_id = ?
              AND archived_at IS NOT NULL
              AND archive_reason = ?
            "#,
        )
        .bind(task_id)
        .bind(reason)
        .fetch_one(pool)
        .await?;

        Ok(total as u32)
    }

    pub async fn get_latest_active_branch_id(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<Option<String>, CheckpointRepoError> {
        let branch_id: Option<String> = sqlx::query_scalar(
            r#"
            SELECT branch_id
            FROM checkpoints
            WHERE task_id = ?
              AND archived_at IS NULL
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        Ok(branch_id)
    }

    pub async fn archive_checkpoints_after(
        pool: &SqlitePool,
        task_id: &str,
        checkpoint_id: &str,
        reason: &str,
    ) -> Result<u32, CheckpointRepoError> {
        let target: Option<(i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT created_at, branch_id, iteration
            FROM checkpoints
            WHERE id = ? AND task_id = ?
            "#,
        )
        .bind(checkpoint_id)
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        let Some((created_at, branch_id, iteration)) = target else {
            return Ok(0);
        };

        let candidates: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM checkpoints
            WHERE task_id = ?
              AND branch_id = ?
              AND (created_at > ? OR (created_at = ? AND iteration > ?))
              AND archived_at IS NULL
            "#,
        )
        .bind(task_id)
        .bind(&branch_id)
        .bind(created_at)
        .bind(created_at)
        .bind(iteration)
        .fetch_one(pool)
        .await?;

        if candidates == 0 {
            return Ok(0);
        }

        let now = now_millis();
        sqlx::query(
            r#"
            UPDATE checkpoints
            SET archived_at = ?, archive_reason = ?
            WHERE task_id = ?
              AND branch_id = ?
              AND (created_at > ? OR (created_at = ? AND iteration > ?))
              AND archived_at IS NULL
            "#,
        )
        .bind(now)
        .bind(reason)
        .bind(task_id)
        .bind(branch_id)
        .bind(created_at)
        .bind(created_at)
        .bind(iteration)
        .execute(pool)
        .await?;

        Ok(candidates as u32)
    }

    pub(crate) async fn archive_checkpoints_after_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_id: &str,
        checkpoint_id: &str,
        reason: &str,
    ) -> Result<u32, CheckpointRepoError> {
        let target: Option<(i64, String, i64)> = sqlx::query_as(
            r#"
            SELECT created_at, branch_id, iteration
            FROM checkpoints
            WHERE id = ? AND task_id = ?
            "#,
        )
        .bind(checkpoint_id)
        .bind(task_id)
        .fetch_optional(tx.as_mut())
        .await?;

        let Some((created_at, branch_id, iteration)) = target else {
            return Ok(0);
        };

        let candidates: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM checkpoints
            WHERE task_id = ?
              AND branch_id = ?
              AND (created_at > ? OR (created_at = ? AND iteration > ?))
              AND archived_at IS NULL
            "#,
        )
        .bind(task_id)
        .bind(&branch_id)
        .bind(created_at)
        .bind(created_at)
        .bind(iteration)
        .fetch_one(tx.as_mut())
        .await?;

        if candidates == 0 {
            return Ok(0);
        }

        let now = now_millis();
        sqlx::query(
            r#"
            UPDATE checkpoints
            SET archived_at = ?, archive_reason = ?
            WHERE task_id = ?
              AND branch_id = ?
              AND (created_at > ? OR (created_at = ? AND iteration > ?))
              AND archived_at IS NULL
            "#,
        )
        .bind(now)
        .bind(reason)
        .bind(task_id)
        .bind(branch_id)
        .bind(created_at)
        .bind(created_at)
        .bind(iteration)
        .execute(tx.as_mut())
        .await?;

        Ok(candidates as u32)
    }

    pub async fn update_branch_info(
        pool: &SqlitePool,
        checkpoint_id: &str,
        branch_id: &str,
        parent_id: Option<&str>,
    ) -> Result<(), CheckpointRepoError> {
        sqlx::query(
            r#"
            UPDATE checkpoints
            SET branch_id = ?, parent_id = ?
            WHERE id = ?
            "#,
        )
        .bind(branch_id)
        .bind(parent_id)
        .bind(checkpoint_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub(crate) async fn update_branch_info_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        checkpoint_id: &str,
        branch_id: &str,
        parent_id: Option<&str>,
    ) -> Result<(), CheckpointRepoError> {
        sqlx::query(
            r#"
            UPDATE checkpoints
            SET branch_id = ?, parent_id = ?
            WHERE id = ?
            "#,
        )
        .bind(branch_id)
        .bind(parent_id)
        .bind(checkpoint_id)
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    pub async fn update_checksum(
        pool: &SqlitePool,
        checkpoint_id: &str,
        checksum: &str,
    ) -> Result<(), CheckpointRepoError> {
        sqlx::query(
            r#"
            UPDATE checkpoints
            SET checksum = ?
            WHERE id = ?
            "#,
        )
        .bind(checksum)
        .bind(checkpoint_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub(crate) async fn update_checksum_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        checkpoint_id: &str,
        checksum: &str,
    ) -> Result<(), CheckpointRepoError> {
        sqlx::query(
            r#"
            UPDATE checkpoints
            SET checksum = ?
            WHERE id = ?
            "#,
        )
        .bind(checksum)
        .bind(checkpoint_id)
        .execute(tx.as_mut())
        .await?;

        Ok(())
    }

    pub async fn get_checkpoint_with_summary(
        pool: &SqlitePool,
        checkpoint_id: &str,
    ) -> Result<Option<CheckpointWithSummary>, CheckpointRepoError> {
        let row: Option<CheckpointRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, iteration, state, run_control_state, prompt,
                   rule_system, artifacts, user_guidance, branch_id, parent_id,
                   lineage_type, branch_description, checksum, created_at,
                   archived_at, archive_reason, pass_rate_summary
            FROM checkpoints
            WHERE id = ?
            "#,
        )
        .bind(checkpoint_id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let pass_rate_summary =
            parse_optional_json::<PassRateSummary>(row.pass_rate_summary.clone())?;
        let archived_at = row.archived_at.map(unix_ms_to_iso8601);
        let archive_reason = row.archive_reason.clone();
        let checkpoint = row_to_entity(row)?;

        Ok(Some(CheckpointWithSummary {
            checkpoint,
            pass_rate_summary,
            archived_at,
            archive_reason,
        }))
    }

    pub async fn count_checkpoints_by_task(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<u32, CheckpointRepoError> {
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM checkpoints
            WHERE task_id = ?
              AND archived_at IS NULL
            "#,
        )
        .bind(task_id)
        .fetch_one(pool)
        .await?;

        Ok(total as u32)
    }

    pub async fn delete_old_checkpoints(
        pool: &SqlitePool,
        task_id: &str,
        keep_count: u32,
    ) -> Result<u32, CheckpointRepoError> {
        let result = sqlx::query(
            r#"
            WITH to_delete AS (
                SELECT id
                FROM checkpoints
                WHERE task_id = ?
                ORDER BY created_at DESC
                LIMIT -1 OFFSET ?
            )
            DELETE FROM checkpoints
            WHERE id IN (SELECT id FROM to_delete)
            "#,
        )
        .bind(task_id)
        .bind(keep_count as i64)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as u32)
    }
}

fn serialize_json<T: Serialize>(value: &T) -> Result<String, CheckpointRepoError> {
    serde_json::to_string(value).map_err(|err| CheckpointRepoError::JsonParse(err.to_string()))
}

fn serialize_optional_json<T: Serialize>(
    value: &Option<T>,
) -> Result<Option<String>, CheckpointRepoError> {
    value.as_ref().map(serialize_json).transpose()
}

fn parse_json<T: DeserializeOwned>(value: &str) -> Result<T, CheckpointRepoError> {
    serde_json::from_str(value).map_err(|err| CheckpointRepoError::JsonParse(err.to_string()))
}

fn parse_optional_json<T: DeserializeOwned>(
    value: Option<String>,
) -> Result<Option<T>, CheckpointRepoError> {
    value.map(|raw| parse_json(&raw)).transpose()
}

fn row_to_entity(row: CheckpointRow) -> Result<CheckpointEntity, CheckpointRepoError> {
    Ok(CheckpointEntity {
        id: row.id,
        task_id: row.task_id,
        iteration: row.iteration as u32,
        state: parse_json::<IterationState>(&row.state)?,
        run_control_state: parse_json::<RunControlState>(&row.run_control_state)?,
        prompt: row.prompt,
        rule_system: parse_json::<RuleSystem>(&row.rule_system)?,
        artifacts: parse_optional_json::<IterationArtifacts>(row.artifacts)?,
        user_guidance: parse_optional_json::<UserGuidance>(row.user_guidance)?,
        branch_id: row.branch_id,
        parent_id: row.parent_id,
        lineage_type: parse_json::<LineageType>(&row.lineage_type)?,
        branch_description: row.branch_description,
        checksum: row.checksum,
        created_at: row.created_at,
        archived_at: row.archived_at,
        archive_reason: row.archive_reason,
        pass_rate_summary: parse_optional_json::<PassRateSummary>(row.pass_rate_summary)?,
    })
}

fn iteration_state_label(state: &IterationState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}
