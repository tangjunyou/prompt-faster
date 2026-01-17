//! 迭代历史数据仓库
//!
//! 提供迭代历史数据的查询功能。

use sqlx::SqlitePool;
use thiserror::Error;
use tracing::{info, warn};

use crate::domain::types::{
    EvaluationResultSummary, IterationArtifacts, IterationHistoryDetail, IterationHistorySummary,
    unix_ms_to_iso8601,
};

/// 迭代历史数据库行
#[derive(Debug, sqlx::FromRow)]
pub struct IterationRow {
    pub id: String,
    pub task_id: String,
    pub round: i32,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub status: String,
    pub artifacts: Option<String>,
    pub evaluation_results: Option<String>,
    pub reflection_summary: Option<String>,
    pub pass_rate: f64,
    pub total_cases: i32,
    pub passed_cases: i32,
    pub created_at: i64,
}

/// 迭代仓库错误
#[derive(Debug, Error)]
pub enum IterationRepoError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("迭代记录不存在")]
    NotFound,
    #[error("任务不存在或无权访问")]
    TaskNotFoundOrForbidden,
    #[error("JSON 解析错误: {0}")]
    JsonParse(String),
}

/// 迭代历史仓库
pub struct IterationRepo;

impl IterationRepo {
    /// 默认最大返回条数（防御性限制）
    const DEFAULT_LIMIT: i32 = 100;

    /// 按任务 ID 查询历史迭代列表（按轮次倒序）
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    /// - `user_id`: 当前用户 ID（用于权限校验）
    /// - `task_id`: 优化任务 ID
    /// - `limit`: 最大返回条数（可选，默认 100）
    pub async fn list_by_task_id(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<IterationHistorySummary>, IterationRepoError> {
        let limit = limit
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_LIMIT)
            .min(Self::DEFAULT_LIMIT);

        // 首先验证任务归属权
        let task_exists: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT ot.id
            FROM optimization_tasks ot
            JOIN workspaces w ON ot.workspace_id = w.id
            WHERE ot.id = ? AND w.user_id = ?
            "#,
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if task_exists.is_none() {
            return Err(IterationRepoError::TaskNotFoundOrForbidden);
        }

        let rows: Vec<IterationRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, round, started_at, completed_at, status,
                   artifacts, evaluation_results, reflection_summary,
                   pass_rate, total_cases, passed_cases, created_at
            FROM iterations
            WHERE task_id = ?
            ORDER BY round DESC
            LIMIT ?
            "#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        info!(
            task_id = %task_id,
            user_id = %user_id,
            count = rows.len(),
            "查询历史迭代列表"
        );

        Ok(rows.into_iter().map(Self::row_to_summary).collect())
    }

    /// 按 ID 查询单个迭代详情
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    /// - `user_id`: 当前用户 ID（用于权限校验）
    /// - `task_id`: 优化任务 ID
    /// - `iteration_id`: 迭代 ID
    pub async fn get_by_id(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
        iteration_id: &str,
    ) -> Result<IterationHistoryDetail, IterationRepoError> {
        // 首先验证任务归属权
        let task_exists: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT ot.id
            FROM optimization_tasks ot
            JOIN workspaces w ON ot.workspace_id = w.id
            WHERE ot.id = ? AND w.user_id = ?
            "#,
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if task_exists.is_none() {
            return Err(IterationRepoError::TaskNotFoundOrForbidden);
        }

        let row: Option<IterationRow> = sqlx::query_as(
            r#"
            SELECT id, task_id, round, started_at, completed_at, status,
                   artifacts, evaluation_results, reflection_summary,
                   pass_rate, total_cases, passed_cases, created_at
            FROM iterations
            WHERE id = ? AND task_id = ?
            "#,
        )
        .bind(iteration_id)
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                info!(
                    task_id = %task_id,
                    iteration_id = %iteration_id,
                    user_id = %user_id,
                    "查询迭代详情"
                );
                Self::row_to_detail(row)
            }
            None => Err(IterationRepoError::NotFound),
        }
    }

    /// 将数据库行转换为摘要
    fn row_to_summary(row: IterationRow) -> IterationHistorySummary {
        IterationHistorySummary {
            id: row.id,
            round: row.round,
            started_at: unix_ms_to_iso8601(row.started_at),
            completed_at: row.completed_at.map(unix_ms_to_iso8601),
            pass_rate: row.pass_rate,
            total_cases: row.total_cases,
            passed_cases: row.passed_cases,
            status: row.status.parse().unwrap_or_default(),
        }
    }

    /// 将数据库行转换为详情
    fn row_to_detail(row: IterationRow) -> Result<IterationHistoryDetail, IterationRepoError> {
        // 解析产物 JSON
        let artifacts = match &row.artifacts {
            Some(json) if !json.trim().is_empty() => {
                serde_json::from_str::<IterationArtifacts>(json).unwrap_or_else(|e| {
                    warn!(error = %e, "解析 artifacts JSON 失败，使用空产物");
                    IterationArtifacts::empty()
                })
            }
            _ => IterationArtifacts::empty(),
        };

        // 解析评估结果 JSON
        let evaluation_results = match &row.evaluation_results {
            Some(json) if !json.trim().is_empty() => {
                serde_json::from_str::<Vec<EvaluationResultSummary>>(json).unwrap_or_else(|e| {
                    warn!(error = %e, "解析 evaluation_results JSON 失败，使用空数组");
                    Vec::new()
                })
            }
            _ => Vec::new(),
        };

        Ok(IterationHistoryDetail {
            id: row.id,
            round: row.round,
            started_at: unix_ms_to_iso8601(row.started_at),
            completed_at: row.completed_at.map(unix_ms_to_iso8601),
            pass_rate: row.pass_rate,
            total_cases: row.total_cases,
            passed_cases: row.passed_cases,
            status: row.status.parse().unwrap_or_default(),
            artifacts,
            evaluation_results,
            reflection_summary: row.reflection_summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::IterationStatus;

    #[test]
    fn test_row_to_summary() {
        let row = IterationRow {
            id: "iter-1".to_string(),
            task_id: "task-1".to_string(),
            round: 1,
            started_at: 1705507200000,
            completed_at: Some(1705507500000),
            status: "completed".to_string(),
            artifacts: None,
            evaluation_results: None,
            reflection_summary: None,
            pass_rate: 0.85,
            total_cases: 10,
            passed_cases: 8,
            created_at: 1705507200000,
        };

        let summary = IterationRepo::row_to_summary(row);
        assert_eq!(summary.id, "iter-1");
        assert_eq!(summary.round, 1);
        assert_eq!(summary.pass_rate, 0.85);
        assert_eq!(summary.status, IterationStatus::Completed);
    }

    #[test]
    fn test_row_to_detail_with_empty_json() {
        let row = IterationRow {
            id: "iter-1".to_string(),
            task_id: "task-1".to_string(),
            round: 1,
            started_at: 1705507200000,
            completed_at: None,
            status: "running".to_string(),
            artifacts: None,
            evaluation_results: None,
            reflection_summary: Some("测试反思".to_string()),
            pass_rate: 0.0,
            total_cases: 0,
            passed_cases: 0,
            created_at: 1705507200000,
        };

        let detail = IterationRepo::row_to_detail(row).unwrap();
        assert_eq!(detail.id, "iter-1");
        assert!(detail.artifacts.is_empty());
        assert!(detail.evaluation_results.is_empty());
        assert_eq!(detail.reflection_summary, Some("测试反思".to_string()));
    }
}
