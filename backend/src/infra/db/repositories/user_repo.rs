//! 用户仓储
//! 负责 users 表的数据访问（Repository 是唯一数据库访问点）

use sqlx::SqlitePool;
use thiserror::Error;

use crate::domain::models::User;
use crate::shared::time::now_millis;

/// 用户仓储错误
#[derive(Error, Debug)]
pub enum UserRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("用户未找到")]
    NotFound,

    #[error("用户名已存在")]
    UsernameConflict,
}

/// 用户仓储
pub struct UserRepo;

impl UserRepo {
    /// 创建用户
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    /// - `username`: 用户名
    /// - `password_hash`: Argon2 PHC 格式的密码哈希
    ///
    /// # 返回
    /// - 成功时返回创建的用户
    /// - 用户名冲突时返回 `UsernameConflict` 错误
    ///
    /// # 注意
    /// 登录接口不得区分"用户不存在 vs 密码错误"（AC #3）
    pub async fn create_user(
        pool: &SqlitePool,
        username: &str,
        password_hash: &str,
    ) -> Result<User, UserRepoError> {
        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();

        let result = sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&id)
        .bind(username)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(User::new(
                id,
                username.to_string(),
                password_hash.to_string(),
                now,
            )),
            Err(sqlx::Error::Database(db_err)) => {
                // SQLite 唯一性约束违反错误码: SQLITE_CONSTRAINT_UNIQUE (2067)
                if db_err.message().contains("UNIQUE constraint failed") {
                    Err(UserRepoError::UsernameConflict)
                } else {
                    Err(UserRepoError::DatabaseError(sqlx::Error::Database(db_err)))
                }
            }
            Err(e) => Err(UserRepoError::DatabaseError(e)),
        }
    }

    /// 根据用户名查找用户
    ///
    /// # 注意
    /// 此方法仅供内部认证使用，调用者不得向客户端暴露"用户是否存在"的信息（AC #3）
    pub async fn find_by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<User, UserRepoError> {
        let row = sqlx::query_as::<_, (String, String, String, i64, i64)>(
            r#"
            SELECT id, username, password_hash, created_at, updated_at
            FROM users
            WHERE username = ?1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((id, username, password_hash, created_at, updated_at)) => Ok(User {
                id,
                username,
                password_hash,
                created_at,
                updated_at,
            }),
            None => Err(UserRepoError::NotFound),
        }
    }

    /// 根据用户 ID 查找用户
    pub async fn find_by_id(pool: &SqlitePool, user_id: &str) -> Result<User, UserRepoError> {
        let row = sqlx::query_as::<_, (String, String, String, i64, i64)>(
            r#"
            SELECT id, username, password_hash, created_at, updated_at
            FROM users
            WHERE id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((id, username, password_hash, created_at, updated_at)) => Ok(User {
                id,
                username,
                password_hash,
                created_at,
                updated_at,
            }),
            None => Err(UserRepoError::NotFound),
        }
    }

    /// 检查是否存在任何用户
    ///
    /// 用于判断系统是否首次启动（需要注册流程）
    pub async fn has_any_user(pool: &SqlitePool) -> Result<bool, UserRepoError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM users
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(count.0 > 0)
    }

    /// 获取第一个注册的用户
    ///
    /// 用于历史数据迁移（将 default_user 的记录归属到首个注册用户）
    pub async fn get_first_user(pool: &SqlitePool) -> Result<Option<User>, UserRepoError> {
        let row = sqlx::query_as::<_, (String, String, String, i64, i64)>(
            r#"
            SELECT id, username, password_hash, created_at, updated_at
            FROM users
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|(id, username, password_hash, created_at, updated_at)| User {
            id,
            username,
            password_hash,
            created_at,
            updated_at,
        }))
    }
}
