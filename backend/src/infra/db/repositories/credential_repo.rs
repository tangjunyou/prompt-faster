//! API 凭证仓储
//! 负责 api_credentials 表的数据访问

use sqlx::SqlitePool;
use thiserror::Error;

use crate::shared::time::now_millis;

/// 凭证类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialType {
    Dify,
    GenericLlm,
}

impl CredentialType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialType::Dify => "dify",
            CredentialType::GenericLlm => "generic_llm",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "dify" => Some(CredentialType::Dify),
            "generic_llm" => Some(CredentialType::GenericLlm),
            _ => None,
        }
    }
}

/// 凭证记录（数据库行）
#[derive(Debug, Clone)]
pub struct CredentialRecord {
    pub id: String,
    pub user_id: String,
    pub credential_type: String,
    pub provider: Option<String>,
    pub base_url: String,
    pub encrypted_api_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 新建或更新凭证的输入
#[derive(Debug, Clone)]
pub struct UpsertCredentialInput {
    pub user_id: String,
    pub credential_type: CredentialType,
    pub provider: Option<String>,
    pub base_url: String,
    pub encrypted_api_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
}

/// 凭证仓储错误
#[derive(Error, Debug)]
pub enum CredentialRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("凭证未找到: user_id={user_id}, type={credential_type}")]
    NotFound {
        user_id: String,
        credential_type: String,
    },
}

/// 凭证仓储
pub struct CredentialRepo;

impl CredentialRepo {
    /// Upsert 凭证（插入或更新）
    ///
    /// 使用 `ON CONFLICT DO UPDATE` 实现 upsert，确保每个用户每种凭证类型只有一条记录。
    pub async fn upsert(
        pool: &SqlitePool,
        input: UpsertCredentialInput,
    ) -> Result<CredentialRecord, CredentialRepoError> {
        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();
        let credential_type = input.credential_type.as_str();

        // SQLite upsert: INSERT OR REPLACE
        // 注意：使用 ON CONFLICT 更新时保留原有的 created_at
        sqlx::query(
            r#"
            INSERT INTO api_credentials (
                id, user_id, credential_type, provider, base_url,
                encrypted_api_key, nonce, salt, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(user_id, credential_type) DO UPDATE SET
                provider = excluded.provider,
                base_url = excluded.base_url,
                encrypted_api_key = excluded.encrypted_api_key,
                nonce = excluded.nonce,
                salt = excluded.salt,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&id)
        .bind(&input.user_id)
        .bind(credential_type)
        .bind(&input.provider)
        .bind(&input.base_url)
        .bind(&input.encrypted_api_key)
        .bind(&input.nonce)
        .bind(&input.salt)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        // 查询刚刚插入/更新的记录
        Self::find_by_user_and_type(pool, &input.user_id, input.credential_type).await
    }

    /// 根据用户 ID 和凭证类型查找凭证
    pub async fn find_by_user_and_type(
        pool: &SqlitePool,
        user_id: &str,
        credential_type: CredentialType,
    ) -> Result<CredentialRecord, CredentialRepoError> {
        let type_str = credential_type.as_str();

        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                i64,
                i64,
            ),
        >(
            r#"
            SELECT id, user_id, credential_type, provider, base_url,
                   encrypted_api_key, nonce, salt, created_at, updated_at
            FROM api_credentials
            WHERE user_id = ?1 AND credential_type = ?2
            "#,
        )
        .bind(user_id)
        .bind(type_str)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((
                id,
                user_id,
                credential_type,
                provider,
                base_url,
                encrypted_api_key,
                nonce,
                salt,
                created_at,
                updated_at,
            )) => Ok(CredentialRecord {
                id,
                user_id,
                credential_type,
                provider,
                base_url,
                encrypted_api_key,
                nonce,
                salt,
                created_at,
                updated_at,
            }),
            None => Err(CredentialRepoError::NotFound {
                user_id: user_id.to_string(),
                credential_type: type_str.to_string(),
            }),
        }
    }

    /// 查找用户的所有凭证
    pub async fn find_all_by_user(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<Vec<CredentialRecord>, CredentialRepoError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                i64,
                i64,
            ),
        >(
            r#"
            SELECT id, user_id, credential_type, provider, base_url,
                   encrypted_api_key, nonce, salt, created_at, updated_at
            FROM api_credentials
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    user_id,
                    credential_type,
                    provider,
                    base_url,
                    encrypted_api_key,
                    nonce,
                    salt,
                    created_at,
                    updated_at,
                )| {
                    CredentialRecord {
                        id,
                        user_id,
                        credential_type,
                        provider,
                        base_url,
                        encrypted_api_key,
                        nonce,
                        salt,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    /// 删除凭证
    pub async fn delete(
        pool: &SqlitePool,
        user_id: &str,
        credential_type: CredentialType,
    ) -> Result<bool, CredentialRepoError> {
        let type_str = credential_type.as_str();

        let result = sqlx::query(
            r#"
            DELETE FROM api_credentials
            WHERE user_id = ?1 AND credential_type = ?2
            "#,
        )
        .bind(user_id)
        .bind(type_str)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
