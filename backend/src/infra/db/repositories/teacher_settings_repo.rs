//! 老师模型参数仓储
//! 负责 teacher_model_settings 表的数据访问

use sqlx::SqlitePool;
use thiserror::Error;

use crate::shared::time::now_millis;

/// 老师模型参数记录（数据库行）
#[derive(Debug, Clone)]
pub struct TeacherSettingsRecord {
    pub id: String,
    pub user_id: String,
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 新建或更新老师模型参数的输入
#[derive(Debug, Clone)]
pub struct UpsertTeacherSettingsInput {
    pub user_id: String,
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i32,
}

/// 老师模型参数仓储错误
#[derive(Error, Debug)]
pub enum TeacherSettingsRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("老师模型参数未找到: user_id={0}")]
    NotFound(String),
}

/// 老师模型参数仓储
pub struct TeacherSettingsRepo;

impl TeacherSettingsRepo {
    /// Upsert 老师模型参数（插入或更新）
    ///
    /// 使用 `ON CONFLICT DO UPDATE` 实现 upsert，确保每个用户只有一条设置记录。
    pub async fn upsert(
        pool: &SqlitePool,
        input: UpsertTeacherSettingsInput,
    ) -> Result<TeacherSettingsRecord, TeacherSettingsRepoError> {
        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO teacher_model_settings (
                id, user_id, temperature, top_p, max_tokens, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(user_id) DO UPDATE SET
                temperature = excluded.temperature,
                top_p = excluded.top_p,
                max_tokens = excluded.max_tokens,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&id)
        .bind(&input.user_id)
        .bind(input.temperature)
        .bind(input.top_p)
        .bind(input.max_tokens)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // 查询刚刚插入/更新的记录
        Self::find_by_user(pool, &input.user_id).await
    }

    /// 根据用户 ID 查找老师模型参数
    pub async fn find_by_user(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<TeacherSettingsRecord, TeacherSettingsRepoError> {
        let row = sqlx::query_as::<_, (String, String, f64, f64, i32, i64, i64)>(
            r#"
            SELECT id, user_id, temperature, top_p, max_tokens, created_at, updated_at
            FROM teacher_model_settings
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((id, user_id, temperature, top_p, max_tokens, created_at, updated_at)) => {
                Ok(TeacherSettingsRecord {
                    id,
                    user_id,
                    temperature,
                    top_p,
                    max_tokens,
                    created_at,
                    updated_at,
                })
            }
            None => Err(TeacherSettingsRepoError::NotFound(user_id.to_string())),
        }
    }

    /// 获取老师模型参数（不存在时返回默认值）
    pub async fn get_or_default(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<TeacherSettingsRecord, TeacherSettingsRepoError> {
        match Self::find_by_user(pool, user_id).await {
            Ok(record) => Ok(record),
            Err(TeacherSettingsRepoError::NotFound(_)) => {
                // 返回默认值
                Ok(TeacherSettingsRecord {
                    id: String::new(),
                    user_id: user_id.to_string(),
                    temperature: 0.7,
                    top_p: 0.9,
                    max_tokens: 2048,
                    created_at: 0,
                    updated_at: 0,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// 删除老师模型参数
    pub async fn delete(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<bool, TeacherSettingsRepoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM teacher_model_settings
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
