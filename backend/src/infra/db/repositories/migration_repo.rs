use sqlx::SqlitePool;
use thiserror::Error;

use crate::shared::time::now_millis;

#[derive(Error, Debug)]
pub enum MigrationRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationResult {
    pub migrated_api_credentials: u64,
    pub migrated_teacher_settings: u64,
}

pub struct MigrationRepo;

impl MigrationRepo {
    pub async fn migrate_legacy_default_user_data(
        pool: &SqlitePool,
        target_user_id: &str,
    ) -> Result<MigrationResult, MigrationRepoError> {
        let now = now_millis();
        let mut tx = pool.begin().await?;

        let migrated_api_credentials = sqlx::query(
            r#"
            UPDATE api_credentials
            SET user_id = ?1,
                updated_at = ?2
            WHERE user_id = 'default_user'
              AND NOT EXISTS (
                SELECT 1
                FROM api_credentials ac2
                WHERE ac2.user_id = ?1
                  AND ac2.credential_type = api_credentials.credential_type
              )
            "#,
        )
        .bind(target_user_id)
        .bind(now)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        let migrated_teacher_settings = sqlx::query(
            r#"
            UPDATE teacher_model_settings
            SET user_id = ?1,
                updated_at = ?2
            WHERE user_id = 'default_user'
              AND NOT EXISTS (
                SELECT 1
                FROM teacher_model_settings t2
                WHERE t2.user_id = ?1
              )
            "#,
        )
        .bind(target_user_id)
        .bind(now)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        tx.commit().await?;

        Ok(MigrationResult {
            migrated_api_credentials,
            migrated_teacher_settings,
        })
    }
}
