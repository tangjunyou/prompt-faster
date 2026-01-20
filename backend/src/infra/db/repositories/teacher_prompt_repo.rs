//! 老师模型 Prompt 版本仓储
//! 负责 teacher_prompts 表的数据访问

use sqlx::SqlitePool;
use thiserror::Error;

use crate::domain::models::TeacherPromptStats;
use crate::shared::time::now_millis;

#[derive(Debug, Clone)]
pub struct TeacherPromptRecord {
    pub id: String,
    pub user_id: String,
    pub version: i32,
    pub content: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct TeacherPromptVersionRecord {
    pub id: String,
    pub version: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct TeacherPromptVersionWithStatsRecord {
    pub id: String,
    pub version: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
    pub total_tasks: i32,
    pub successful_tasks: i32,
    pub average_pass_rate: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CreateTeacherPromptRecordInput {
    pub content: String,
    pub description: Option<String>,
    pub activate: bool,
}

#[derive(Error, Debug)]
pub enum TeacherPromptRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Prompt 版本不存在")]
    NotFound,
}

pub struct TeacherPromptRepo;

impl TeacherPromptRepo {
    pub async fn create(
        pool: &SqlitePool,
        user_id: &str,
        input: CreateTeacherPromptRecordInput,
    ) -> Result<TeacherPromptRecord, TeacherPromptRepoError> {
        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();
        let is_active = if input.activate { 1 } else { 0 };

        let mut tx = pool.begin().await?;

        if input.activate {
            sqlx::query(
                r#"
                UPDATE teacher_prompts
                SET is_active = 0
                WHERE user_id = ?1
                "#,
            )
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO teacher_prompts (id, user_id, version, content, description, is_active, created_at, updated_at)
            SELECT ?1, ?2, COALESCE(MAX(version), 0) + 1, ?3, ?4, ?5, ?6, ?6
            FROM teacher_prompts
            WHERE user_id = ?2
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(&input.content)
        .bind(input.description.as_deref())
        .bind(is_active)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Self::find_by_id(pool, &id, user_id).await
    }

    pub async fn list_by_user(
        pool: &SqlitePool,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TeacherPromptVersionRecord>, TeacherPromptRepoError> {
        let rows = sqlx::query_as::<_, (String, i32, Option<String>, i64, i64)>(
            r#"
            SELECT id, version, description, is_active, created_at
            FROM teacher_prompts
            WHERE user_id = ?1
            ORDER BY version DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, version, description, is_active, created_at)| TeacherPromptVersionRecord {
                    id,
                    version,
                    description,
                    is_active: is_active != 0,
                    created_at,
                },
            )
            .collect())
    }

    pub async fn list_with_stats_by_user(
        pool: &SqlitePool,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TeacherPromptVersionWithStatsRecord>, TeacherPromptRepoError> {
        let rows = sqlx::query_as::<_, (String, i32, Option<String>, i64, i64, i64, i64, Option<f64>)>(
            r#"
            WITH task_pass_rates AS (
                SELECT
                    ot.id AS task_id,
                    ot.teacher_prompt_version_id AS version_id,
                    COALESCE(selected.pass_rate, latest.pass_rate) AS pass_rate
                FROM optimization_tasks ot
                JOIN workspaces w ON w.id = ot.workspace_id
                LEFT JOIN iterations selected ON selected.id = ot.selected_iteration_id
                LEFT JOIN (
                    SELECT i1.task_id, i1.pass_rate
                    FROM iterations i1
                    JOIN (
                        SELECT task_id, MAX(round) AS max_round
                        FROM iterations
                        WHERE status = 'completed'
                        GROUP BY task_id
                    ) m ON m.task_id = i1.task_id AND i1.round = m.max_round
                    WHERE i1.status = 'completed'
                ) latest ON latest.task_id = ot.id
                WHERE w.user_id = ?1 AND ot.teacher_prompt_version_id IS NOT NULL
            )
            SELECT
                tp.id,
                tp.version,
                tp.description,
                tp.is_active,
                tp.created_at,
                COUNT(task_pass_rates.task_id) AS total_tasks,
                COALESCE(SUM(CASE WHEN task_pass_rates.pass_rate >= 1.0 THEN 1 ELSE 0 END), 0) AS successful_tasks,
                AVG(task_pass_rates.pass_rate) AS average_pass_rate
            FROM teacher_prompts tp
            LEFT JOIN task_pass_rates ON task_pass_rates.version_id = tp.id
            WHERE tp.user_id = ?1
            GROUP BY tp.id
            ORDER BY tp.version DESC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    version,
                    description,
                    is_active,
                    created_at,
                    total_tasks,
                    successful_tasks,
                    average_pass_rate,
                )| TeacherPromptVersionWithStatsRecord {
                    id,
                    version,
                    description,
                    is_active: is_active != 0,
                    created_at,
                    total_tasks: total_tasks as i32,
                    successful_tasks: successful_tasks as i32,
                    average_pass_rate,
                },
            )
            .collect())
    }

    pub async fn count_by_user(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<i32, TeacherPromptRepoError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM teacher_prompts WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count as i32)
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: &str,
        user_id: &str,
    ) -> Result<TeacherPromptRecord, TeacherPromptRepoError> {
        let row =
            sqlx::query_as::<_, (String, String, i32, String, Option<String>, i64, i64, i64)>(
                r#"
            SELECT id, user_id, version, content, description, is_active, created_at, updated_at
            FROM teacher_prompts
            WHERE id = ?1 AND user_id = ?2
            "#,
            )
            .bind(id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        match row {
            Some((
                id,
                user_id,
                version,
                content,
                description,
                is_active,
                created_at,
                updated_at,
            )) => Ok(TeacherPromptRecord {
                id,
                user_id,
                version,
                content,
                description,
                is_active: is_active != 0,
                created_at,
                updated_at,
            }),
            None => Err(TeacherPromptRepoError::NotFound),
        }
    }

    pub async fn exists_by_id(pool: &SqlitePool, id: &str) -> Result<bool, TeacherPromptRepoError> {
        let exists: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM teacher_prompts WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(exists.is_some())
    }

    pub async fn find_active(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<Option<TeacherPromptRecord>, TeacherPromptRepoError> {
        let row =
            sqlx::query_as::<_, (String, String, i32, String, Option<String>, i64, i64, i64)>(
                r#"
            SELECT id, user_id, version, content, description, is_active, created_at, updated_at
            FROM teacher_prompts
            WHERE user_id = ?1 AND is_active = 1
            LIMIT 1
            "#,
            )
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        Ok(row.map(
            |(id, user_id, version, content, description, is_active, created_at, updated_at)| {
                TeacherPromptRecord {
                    id,
                    user_id,
                    version,
                    content,
                    description,
                    is_active: is_active != 0,
                    created_at,
                    updated_at,
                }
            },
        ))
    }

    pub async fn set_active(
        pool: &SqlitePool,
        id: &str,
        user_id: &str,
    ) -> Result<TeacherPromptRecord, TeacherPromptRepoError> {
        let now = now_millis();
        let mut tx = pool.begin().await?;

        let exists: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM teacher_prompts WHERE id = ?1 AND user_id = ?2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        if exists.is_none() {
            return Err(TeacherPromptRepoError::NotFound);
        }

        sqlx::query(
            r#"
            UPDATE teacher_prompts
            SET is_active = 0
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE teacher_prompts
            SET is_active = 1, updated_at = ?3
            WHERE id = ?2 AND user_id = ?1
            "#,
        )
        .bind(user_id)
        .bind(id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Self::find_by_id(pool, id, user_id).await
    }

    pub async fn calculate_stats(
        pool: &SqlitePool,
        version_id: &str,
        user_id: &str,
    ) -> Result<TeacherPromptStats, TeacherPromptRepoError> {
        let row: Option<(i32, i64, i64, Option<f64>)> = sqlx::query_as(
            r#"
            WITH task_pass_rates AS (
                SELECT
                    ot.id AS task_id,
                    ot.teacher_prompt_version_id AS version_id,
                    COALESCE(selected.pass_rate, latest.pass_rate) AS pass_rate
                FROM optimization_tasks ot
                JOIN workspaces w ON w.id = ot.workspace_id
                LEFT JOIN iterations selected ON selected.id = ot.selected_iteration_id
                LEFT JOIN (
                    SELECT i1.task_id, i1.pass_rate
                    FROM iterations i1
                    JOIN (
                        SELECT task_id, MAX(round) AS max_round
                        FROM iterations
                        WHERE status = 'completed'
                        GROUP BY task_id
                    ) m ON m.task_id = i1.task_id AND i1.round = m.max_round
                    WHERE i1.status = 'completed'
                ) latest ON latest.task_id = ot.id
                WHERE w.user_id = ?1 AND ot.teacher_prompt_version_id = ?2
            )
            SELECT
                tp.version,
                COUNT(task_pass_rates.task_id) AS total_tasks,
                COALESCE(SUM(CASE WHEN task_pass_rates.pass_rate >= 1.0 THEN 1 ELSE 0 END), 0) AS successful_tasks,
                AVG(task_pass_rates.pass_rate) AS average_pass_rate
            FROM teacher_prompts tp
            LEFT JOIN task_pass_rates ON task_pass_rates.version_id = tp.id
            WHERE tp.id = ?2 AND tp.user_id = ?1
            GROUP BY tp.id
            "#,
        )
        .bind(user_id)
        .bind(version_id)
        .fetch_optional(pool)
        .await?;

        let Some((version, total_tasks, successful_tasks, average_pass_rate)) = row else {
            return Err(TeacherPromptRepoError::NotFound);
        };

        let total_tasks = total_tasks as i32;
        let successful_tasks = successful_tasks as i32;
        let success_rate = if total_tasks == 0 {
            None
        } else {
            Some(successful_tasks as f64 / total_tasks as f64)
        };

        Ok(TeacherPromptStats {
            version_id: version_id.to_string(),
            version,
            total_tasks,
            successful_tasks,
            success_rate,
            average_pass_rate,
        })
    }
}
