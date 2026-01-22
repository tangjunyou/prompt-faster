//! 迭代历史数据仓库
//!
//! 提供迭代历史数据的查询功能。

use sqlx::SqlitePool;
use thiserror::Error;
use tracing::{info, warn};

use crate::domain::models::DiversityAnalysisResult;
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

/// 迭代摘要 + 产物
#[derive(Debug)]
pub struct IterationSummaryWithArtifacts {
    pub summary: IterationHistorySummary,
    pub artifacts: IterationArtifacts,
}

/// 迭代摘要 + 产物 + 评估结果（诊断报告使用）
#[derive(Debug)]
pub struct IterationSummaryWithArtifactsAndEvaluations {
    pub summary: IterationHistorySummary,
    pub artifacts: IterationArtifacts,
    pub evaluation_results: Vec<EvaluationResultSummary>,
    pub completed_at: Option<i64>,
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

    /// 按任务 ID 查询历史迭代列表（包含产物，按轮次倒序）
    ///
    /// 可选传入状态过滤（例如仅 completed）
    pub async fn list_with_artifacts_by_task_id(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
        status_filter: Option<&str>,
    ) -> Result<Vec<IterationSummaryWithArtifacts>, IterationRepoError> {
        let limit = limit
            .filter(|value| *value > 0)
            .unwrap_or(Self::DEFAULT_LIMIT)
            .min(Self::DEFAULT_LIMIT);
        let offset = offset.filter(|value| *value >= 0).unwrap_or(0);

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

        let mut sql = String::from(
            r#"
            SELECT id, task_id, round, started_at, completed_at, status,
                   artifacts, evaluation_results, reflection_summary,
                   pass_rate, total_cases, passed_cases, created_at
            FROM iterations
            WHERE task_id = ?
            "#,
        );
        if status_filter.is_some() {
            sql.push_str(" AND status = ?");
        }
        sql.push_str(" ORDER BY round DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, IterationRow>(&sql).bind(task_id);
        if let Some(status) = status_filter {
            query = query.bind(status);
        }
        let rows: Vec<IterationRow> = query.bind(limit).bind(offset).fetch_all(pool).await?;

        info!(
            task_id = %task_id,
            user_id = %user_id,
            count = rows.len(),
            "查询历史迭代列表（含产物）"
        );

        Ok(rows
            .into_iter()
            .map(|row| IterationSummaryWithArtifacts {
                summary: Self::row_to_summary_ref(&row),
                artifacts: Self::parse_artifacts(&row.artifacts),
            })
            .collect())
    }

    /// 按任务 ID 查询迭代列表（包含产物 + 评估结果，按轮次升序）
    pub async fn list_with_artifacts_and_results_by_task_id(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
        status_filter: Option<&str>,
    ) -> Result<Vec<IterationSummaryWithArtifactsAndEvaluations>, IterationRepoError> {
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

        let mut sql = String::from(
            r#"
            SELECT id, task_id, round, started_at, completed_at, status,
                   artifacts, evaluation_results, reflection_summary,
                   pass_rate, total_cases, passed_cases, created_at
            FROM iterations
            WHERE task_id = ?
            "#,
        );
        if status_filter.is_some() {
            sql.push_str(" AND status = ?");
        }
        sql.push_str(" ORDER BY round ASC");

        let mut query = sqlx::query_as::<_, IterationRow>(&sql).bind(task_id);
        if let Some(status) = status_filter {
            query = query.bind(status);
        }
        let rows: Vec<IterationRow> = query.fetch_all(pool).await?;

        info!(
            task_id = %task_id,
            user_id = %user_id,
            count = rows.len(),
            "查询迭代列表（含产物 + 评估结果）"
        );

        Ok(rows
            .into_iter()
            .map(|row| IterationSummaryWithArtifactsAndEvaluations {
                summary: Self::row_to_summary_ref(&row),
                artifacts: Self::parse_artifacts(&row.artifacts),
                evaluation_results: Self::parse_evaluation_results(&row.evaluation_results),
                completed_at: row.completed_at,
                created_at: row.created_at,
            })
            .collect())
    }

    /// 统计任务迭代总数
    pub async fn count_by_task_id(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
    ) -> Result<i32, IterationRepoError> {
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

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM iterations WHERE task_id = ?")
            .bind(task_id)
            .fetch_one(pool)
            .await?;

        Ok(count as i32)
    }

    /// 获取通过率最高的已完成迭代（含产物）
    pub async fn find_best_completed_with_artifacts_by_task_id(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
    ) -> Result<Option<IterationSummaryWithArtifacts>, IterationRepoError> {
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
            WHERE task_id = ? AND status = 'completed'
            ORDER BY pass_rate DESC, round DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|row| IterationSummaryWithArtifacts {
            summary: Self::row_to_summary_ref(&row),
            artifacts: Self::parse_artifacts(&row.artifacts),
        }))
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

    /// 更新指定轮次的多样性分析结果（内部使用，无权限校验）
    pub async fn update_diversity_analysis_for_round(
        pool: &SqlitePool,
        task_id: &str,
        round: u32,
        analysis: &DiversityAnalysisResult,
    ) -> Result<(), IterationRepoError> {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, artifacts
            FROM iterations
            WHERE task_id = ?1 AND round = ?2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .bind(round as i32)
        .fetch_optional(pool)
        .await?;

        let Some((id, artifacts_raw)) = row else {
            return Err(IterationRepoError::NotFound);
        };

        let mut artifacts = Self::parse_artifacts(&artifacts_raw);
        artifacts.diversity_analysis = Some(analysis.clone());
        artifacts.updated_at = crate::shared::ws::chrono_timestamp();

        let artifacts_json = serde_json::to_string(&artifacts)
            .map_err(|err| IterationRepoError::JsonParse(err.to_string()))?;

        sqlx::query("UPDATE iterations SET artifacts = ?1 WHERE id = ?2")
            .bind(artifacts_json)
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
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

    fn row_to_summary_ref(row: &IterationRow) -> IterationHistorySummary {
        IterationHistorySummary {
            id: row.id.clone(),
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
        let artifacts = Self::parse_artifacts(&row.artifacts);

        // 解析评估结果 JSON
        let evaluation_results = Self::parse_evaluation_results(&row.evaluation_results);

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

    fn parse_artifacts(raw: &Option<String>) -> IterationArtifacts {
        match raw {
            Some(json) if !json.trim().is_empty() => {
                serde_json::from_str::<IterationArtifacts>(json).unwrap_or_else(|e| {
                    warn!(error = %e, "解析 artifacts JSON 失败，使用空产物");
                    IterationArtifacts::empty()
                })
            }
            _ => IterationArtifacts::empty(),
        }
    }

    fn parse_evaluation_results(raw: &Option<String>) -> Vec<EvaluationResultSummary> {
        match raw {
            Some(json) if !json.trim().is_empty() => {
                serde_json::from_str::<Vec<EvaluationResultSummary>>(json).unwrap_or_else(|e| {
                    warn!(error = %e, "解析 evaluation_results JSON 失败，使用空数组");
                    Vec::new()
                })
            }
            _ => Vec::new(),
        }
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
