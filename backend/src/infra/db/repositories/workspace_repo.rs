use sqlx::SqlitePool;
use thiserror::Error;

use crate::domain::models::Workspace;
use crate::shared::time::now_millis;

#[derive(Error, Debug)]
pub enum WorkspaceRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("工作区未找到")]
    NotFound,
}

pub struct WorkspaceRepo;

impl WorkspaceRepo {
    pub async fn create(
        pool: &SqlitePool,
        user_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<Workspace, WorkspaceRepoError> {
        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO workspaces (id, user_id, name, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(Workspace {
            id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<Workspace, WorkspaceRepoError> {
        let row = sqlx::query_as::<_, (String, String, String, Option<String>, i64, i64)>(
            r#"
            SELECT id, user_id, name, description, created_at, updated_at
            FROM workspaces
            WHERE id = ?1 AND user_id = ?2
            "#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some((id, user_id, name, description, created_at, updated_at)) => Ok(Workspace {
                id,
                user_id,
                name,
                description,
                created_at,
                updated_at,
            }),
            None => Err(WorkspaceRepoError::NotFound),
        }
    }

    pub async fn find_all_by_user(
        pool: &SqlitePool,
        user_id: &str,
    ) -> Result<Vec<Workspace>, WorkspaceRepoError> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, i64, i64)>(
            r#"
            SELECT id, user_id, name, description, created_at, updated_at
            FROM workspaces
            WHERE user_id = ?1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, user_id, name, description, created_at, updated_at)| Workspace {
                id,
                user_id,
                name,
                description,
                created_at,
                updated_at,
            })
            .collect())
    }

    pub async fn delete(
        pool: &SqlitePool,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<bool, WorkspaceRepoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM workspaces
            WHERE id = ?1 AND user_id = ?2
            "#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::db::pool::create_pool;

    async fn setup_test_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("创建测试数据库失败");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("运行 migrations 失败");
        pool
    }

    async fn insert_user(pool: &SqlitePool, id: &str, username: &str) {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind("test_hash")
        .bind(0_i64)
        .bind(0_i64)
        .execute(pool)
        .await
        .expect("插入测试用户失败");
    }

    #[tokio::test]
    async fn test_create_and_find_by_id_success() {
        let pool = setup_test_db().await;

        let user_id = "u1";
        insert_user(&pool, user_id, "user1").await;
        let created = WorkspaceRepo::create(&pool, user_id, "ws", Some("desc"))
            .await
            .expect("创建工作区失败");

        let loaded = WorkspaceRepo::find_by_id(&pool, &created.id, user_id)
            .await
            .expect("查询工作区失败");

        assert_eq!(loaded.id, created.id);
        assert_eq!(loaded.user_id, user_id);
        assert_eq!(loaded.name, "ws");
        assert_eq!(loaded.description.as_deref(), Some("desc"));
    }

    #[tokio::test]
    async fn test_find_all_by_user_filters() {
        let pool = setup_test_db().await;

        let u1 = "u1";
        let u2 = "u2";

        insert_user(&pool, u1, "user1").await;
        insert_user(&pool, u2, "user2").await;

        WorkspaceRepo::create(&pool, u1, "ws1", None)
            .await
            .expect("创建工作区失败");
        WorkspaceRepo::create(&pool, u2, "ws2", None)
            .await
            .expect("创建工作区失败");

        let list_u1 = WorkspaceRepo::find_all_by_user(&pool, u1)
            .await
            .expect("查询列表失败");

        assert_eq!(list_u1.len(), 1);
        assert_eq!(list_u1[0].user_id, u1);
        assert_eq!(list_u1[0].name, "ws1");
    }

    #[tokio::test]
    async fn test_find_by_id_other_user_returns_not_found() {
        let pool = setup_test_db().await;

        insert_user(&pool, "u1", "user1").await;
        insert_user(&pool, "u2", "user2").await;

        let created = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");

        let err = WorkspaceRepo::find_by_id(&pool, &created.id, "u2")
            .await
            .expect_err("应返回 NotFound");

        assert!(matches!(err, WorkspaceRepoError::NotFound));
    }

    #[tokio::test]
    async fn test_delete_requires_user_id() {
        let pool = setup_test_db().await;

        insert_user(&pool, "u1", "user1").await;
        insert_user(&pool, "u2", "user2").await;

        let created = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");

        let deleted_wrong = WorkspaceRepo::delete(&pool, &created.id, "u2")
            .await
            .expect("删除失败");
        assert!(!deleted_wrong);

        let deleted_ok = WorkspaceRepo::delete(&pool, &created.id, "u1")
            .await
            .expect("删除失败");
        assert!(deleted_ok);
    }
}
