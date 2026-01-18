//! Checkpoint Repository

use serde::{Serialize, de::DeserializeOwned};
use sqlx::SqlitePool;
use thiserror::Error;
use tracing::info;

use crate::domain::models::CheckpointEntity;
use crate::domain::models::{IterationState, LineageType, RuleSystem};
use crate::domain::types::{IterationArtifacts, RunControlState, UserGuidance};

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

        sqlx::query(
            r#"
            INSERT INTO checkpoints (
                id, task_id, iteration, state, run_control_state, prompt,
                rule_system, artifacts, user_guidance, branch_id, parent_id,
                lineage_type, branch_description, checksum, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                   lineage_type, branch_description, checksum, created_at
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
                   c.lineage_type, c.branch_description, c.checksum, c.created_at
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
                   lineage_type, branch_description, checksum, created_at
            FROM checkpoints
            WHERE task_id = ?
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

    pub async fn count_checkpoints_by_task(
        pool: &SqlitePool,
        task_id: &str,
    ) -> Result<u32, CheckpointRepoError> {
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(1)
            FROM checkpoints
            WHERE task_id = ?
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
    })
}
