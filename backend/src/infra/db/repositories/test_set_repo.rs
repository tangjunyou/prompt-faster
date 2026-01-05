use sqlx::SqlitePool;
use thiserror::Error;

use crate::domain::models::{TestCase, TestSet};
use crate::shared::time::now_millis;

#[derive(Error, Debug)]
pub enum TestSetRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("JSON 序列化/反序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("测试集未找到")]
    NotFound,
}

pub struct TestSetRepo;

#[derive(Debug, Clone)]
pub struct TestSetSummary {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub cases_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TestSetRepo {
    pub async fn create(
        pool: &SqlitePool,
        workspace_id: &str,
        name: &str,
        description: Option<&str>,
        cases: &[TestCase],
        dify_config_json: Option<&str>,
        generic_config_json: Option<&str>,
    ) -> Result<TestSet, TestSetRepoError> {
        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();
        let cases_json = serde_json::to_string(cases)?;

        sqlx::query(
            r#"
            INSERT INTO test_sets (id, workspace_id, name, description, cases_json, dify_config_json, generic_config_json, is_template, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)
            "#,
        )
        .bind(&id)
        .bind(workspace_id)
        .bind(name)
        .bind(description)
        .bind(&cases_json)
        .bind(dify_config_json)
        .bind(generic_config_json)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(TestSet {
            id,
            workspace_id: workspace_id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            cases: cases.to_vec(),
            dify_config_json: dify_config_json.map(|s| s.to_string()),
            generic_config_json: generic_config_json.map(|s| s.to_string()),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list_by_workspace(
        pool: &SqlitePool,
        workspace_id: &str,
    ) -> Result<Vec<TestSet>, TestSetRepoError> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, i64, i64)>(
            r#"
            SELECT id, workspace_id, name, description, cases_json, created_at, updated_at
            FROM test_sets
            WHERE workspace_id = ?1
              AND is_template = 0
            ORDER BY created_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for (id, workspace_id, name, description, cases_json, created_at, updated_at) in rows {
            let cases: Vec<TestCase> = serde_json::from_str(&cases_json)?;
            out.push(TestSet {
                id,
                workspace_id,
                name,
                description,
                cases,
                dify_config_json: None,
                generic_config_json: None,
                created_at,
                updated_at,
            });
        }
        Ok(out)
    }

    pub async fn list_summaries_by_workspace(
        pool: &SqlitePool,
        workspace_id: &str,
    ) -> Result<Vec<TestSetSummary>, TestSetRepoError> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, i64, i64)>(
            r#"
            SELECT id, workspace_id, name, description, cases_json, created_at, updated_at
            FROM test_sets
            WHERE workspace_id = ?1
              AND is_template = 0
            ORDER BY created_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for (id, workspace_id, name, description, cases_json, created_at, updated_at) in rows {
            let value: serde_json::Value = serde_json::from_str(&cases_json)?;
            let cases_count = value.as_array().map(|a| a.len()).unwrap_or(0) as u32;
            out.push(TestSetSummary {
                id,
                workspace_id,
                name,
                description,
                cases_count,
                created_at,
                updated_at,
            });
        }
        Ok(out)
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        workspace_id: &str,
        test_set_id: &str,
    ) -> Result<TestSet, TestSetRepoError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
            ),
        >(
            r#"
            SELECT id, workspace_id, name, description, cases_json, dify_config_json, generic_config_json, created_at, updated_at
            FROM test_sets
            WHERE workspace_id = ?1 AND id = ?2 AND is_template = 0
            "#,
        )
        .bind(workspace_id)
        .bind(test_set_id)
        .fetch_optional(pool)
        .await?;

        let Some((
            id,
            workspace_id,
            name,
            description,
            cases_json,
            dify_config_json,
            generic_config_json,
            created_at,
            updated_at,
        )) = row
        else {
            return Err(TestSetRepoError::NotFound);
        };

        let cases: Vec<TestCase> = serde_json::from_str(&cases_json)?;

        Ok(TestSet {
            id,
            workspace_id,
            name,
            description,
            cases,
            dify_config_json,
            generic_config_json,
            created_at,
            updated_at,
        })
    }

    pub async fn find_by_id_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        test_set_id: &str,
    ) -> Result<TestSet, TestSetRepoError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
            ),
        >(
            r#"
            SELECT ts.id, ts.workspace_id, ts.name, ts.description, ts.cases_json, ts.dify_config_json, ts.generic_config_json, ts.created_at, ts.updated_at
            FROM test_sets ts
            JOIN workspaces w ON w.id = ts.workspace_id
            WHERE ts.workspace_id = ?1 AND ts.id = ?2 AND ts.is_template = 0 AND w.user_id = ?3
            "#,
        )
        .bind(workspace_id)
        .bind(test_set_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some((
            id,
            workspace_id,
            name,
            description,
            cases_json,
            dify_config_json,
            generic_config_json,
            created_at,
            updated_at,
        )) = row
        else {
            return Err(TestSetRepoError::NotFound);
        };

        let cases: Vec<TestCase> = serde_json::from_str(&cases_json)?;

        Ok(TestSet {
            id,
            workspace_id,
            name,
            description,
            cases,
            dify_config_json,
            generic_config_json,
            created_at,
            updated_at,
        })
    }

    pub async fn update(
        pool: &SqlitePool,
        workspace_id: &str,
        test_set_id: &str,
        name: &str,
        description: Option<&str>,
        cases: &[TestCase],
    ) -> Result<TestSet, TestSetRepoError> {
        let now = now_millis();
        let cases_json = serde_json::to_string(cases)?;

        let result = sqlx::query(
            r#"
            UPDATE test_sets
            SET name = ?1, description = ?2, cases_json = ?3, updated_at = ?4
            WHERE workspace_id = ?5 AND id = ?6 AND is_template = 0
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(&cases_json)
        .bind(now)
        .bind(workspace_id)
        .bind(test_set_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TestSetRepoError::NotFound);
        }

        // Return canonical row data (including created_at) after update.
        Self::find_by_id(pool, workspace_id, test_set_id).await
    }

    pub async fn update_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        test_set_id: &str,
        name: &str,
        description: Option<&str>,
        cases: &[TestCase],
    ) -> Result<TestSet, TestSetRepoError> {
        let now = now_millis();
        let cases_json = serde_json::to_string(cases)?;

        let result = sqlx::query(
            r#"
            UPDATE test_sets
            SET name = ?1, description = ?2, cases_json = ?3, updated_at = ?4
            WHERE workspace_id = ?5 AND id = ?6
              AND is_template = 0
              AND EXISTS (SELECT 1 FROM workspaces w WHERE w.id = ?5 AND w.user_id = ?7)
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(&cases_json)
        .bind(now)
        .bind(workspace_id)
        .bind(test_set_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TestSetRepoError::NotFound);
        }

        Self::find_by_id_scoped(pool, user_id, workspace_id, test_set_id).await
    }

    pub async fn delete(
        pool: &SqlitePool,
        workspace_id: &str,
        test_set_id: &str,
    ) -> Result<bool, TestSetRepoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM test_sets
            WHERE workspace_id = ?1 AND id = ?2 AND is_template = 0
            "#,
        )
        .bind(workspace_id)
        .bind(test_set_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        test_set_id: &str,
    ) -> Result<bool, TestSetRepoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM test_sets
            WHERE workspace_id = ?1 AND id = ?2
              AND is_template = 0
              AND EXISTS (SELECT 1 FROM workspaces w WHERE w.id = ?1 AND w.user_id = ?3)
            "#,
        )
        .bind(workspace_id)
        .bind(test_set_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_template_summaries_by_workspace_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
    ) -> Result<Vec<TestSetSummary>, TestSetRepoError> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, i64, i64)>(
            r#"
            SELECT ts.id, ts.workspace_id, ts.name, ts.description, ts.cases_json, ts.created_at, ts.updated_at
            FROM test_sets ts
            JOIN workspaces w ON w.id = ts.workspace_id
            WHERE ts.workspace_id = ?1
              AND ts.is_template = 1
              AND w.user_id = ?2
            ORDER BY ts.created_at DESC
            "#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for (id, workspace_id, name, description, cases_json, created_at, updated_at) in rows {
            let value: serde_json::Value = serde_json::from_str(&cases_json)?;
            let cases_count = value.as_array().map(|a| a.len()).unwrap_or(0) as u32;
            out.push(TestSetSummary {
                id,
                workspace_id,
                name,
                description,
                cases_count,
                created_at,
                updated_at,
            });
        }
        Ok(out)
    }

    pub async fn find_template_by_id_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        template_id: &str,
    ) -> Result<TestSet, TestSetRepoError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
            ),
        >(
            r#"
            SELECT ts.id, ts.workspace_id, ts.name, ts.description, ts.cases_json, ts.dify_config_json, ts.generic_config_json, ts.created_at, ts.updated_at
            FROM test_sets ts
            JOIN workspaces w ON w.id = ts.workspace_id
            WHERE ts.workspace_id = ?1
              AND ts.id = ?2
              AND ts.is_template = 1
              AND w.user_id = ?3
            "#,
        )
        .bind(workspace_id)
        .bind(template_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some((
            id,
            workspace_id,
            name,
            description,
            cases_json,
            dify_config_json,
            generic_config_json,
            created_at,
            updated_at,
        )) = row
        else {
            return Err(TestSetRepoError::NotFound);
        };

        let cases: Vec<TestCase> = serde_json::from_str(&cases_json)?;

        Ok(TestSet {
            id,
            workspace_id,
            name,
            description,
            cases,
            dify_config_json,
            generic_config_json,
            created_at,
            updated_at,
        })
    }

    pub async fn create_template_from_test_set_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        source_test_set_id: &str,
        template_name: &str,
        template_description: Option<&str>,
    ) -> Result<TestSet, TestSetRepoError> {
        let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            r#"
            SELECT ts.cases_json, ts.dify_config_json, ts.generic_config_json
            FROM test_sets ts
            JOIN workspaces w ON w.id = ts.workspace_id
            WHERE ts.workspace_id = ?1
              AND ts.id = ?2
              AND ts.is_template = 0
              AND w.user_id = ?3
            "#,
        )
        .bind(workspace_id)
        .bind(source_test_set_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some((cases_json, dify_config_json, generic_config_json)) = row else {
            return Err(TestSetRepoError::NotFound);
        };

        let now = now_millis();
        let id = uuid::Uuid::new_v4().to_string();
        let description = template_description.map(|s| s.to_string());

        sqlx::query(
            r#"
            INSERT INTO test_sets (id, workspace_id, name, description, cases_json, dify_config_json, generic_config_json, is_template, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?9)
            "#,
        )
        .bind(&id)
        .bind(workspace_id)
        .bind(template_name)
        .bind(description.as_deref())
        .bind(&cases_json)
        .bind(dify_config_json.as_deref())
        .bind(generic_config_json.as_deref())
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        let cases: Vec<TestCase> = serde_json::from_str(&cases_json)?;

        Ok(TestSet {
            id,
            workspace_id: workspace_id.to_string(),
            name: template_name.to_string(),
            description,
            cases,
            dify_config_json,
            generic_config_json,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn update_dify_config_json_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        test_set_id: &str,
        dify_config_json: Option<&str>,
    ) -> Result<(), TestSetRepoError> {
        let now = now_millis();

        let result = sqlx::query(
            r#"
            UPDATE test_sets
            SET dify_config_json = ?1, updated_at = ?2
            WHERE workspace_id = ?3 AND id = ?4
              AND is_template = 0
              AND EXISTS (SELECT 1 FROM workspaces w WHERE w.id = ?3 AND w.user_id = ?5)
            "#,
        )
        .bind(dify_config_json)
        .bind(now)
        .bind(workspace_id)
        .bind(test_set_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TestSetRepoError::NotFound);
        }

        Ok(())
    }

    pub async fn update_generic_config_json_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        test_set_id: &str,
        generic_config_json: Option<&str>,
    ) -> Result<(), TestSetRepoError> {
        let now = now_millis();

        let result = sqlx::query(
            r#"
            UPDATE test_sets
            SET generic_config_json = ?1, updated_at = ?2
            WHERE workspace_id = ?3 AND id = ?4
              AND is_template = 0
              AND EXISTS (SELECT 1 FROM workspaces w WHERE w.id = ?3 AND w.user_id = ?5)
            "#,
        )
        .bind(generic_config_json)
        .bind(now)
        .bind(workspace_id)
        .bind(test_set_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(TestSetRepoError::NotFound);
        }

        Ok(())
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

    async fn insert_workspace(pool: &SqlitePool, id: &str, user_id: &str) {
        sqlx::query(
            r#"
            INSERT INTO workspaces (id, user_id, name, description, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind("ws")
        .bind("desc")
        .bind(0_i64)
        .bind(0_i64)
        .execute(pool)
        .await
        .expect("插入测试 workspace 失败");
    }

    fn sample_cases() -> Vec<TestCase> {
        use crate::domain::models::{DataSplit, TaskReference};
        use std::collections::HashMap;

        vec![TestCase {
            id: "case-1".to_string(),
            input: HashMap::from([("text".to_string(), serde_json::Value::String("hi".into()))]),
            reference: TaskReference::Exact {
                expected: "ok".to_string(),
            },
            split: Some(DataSplit::Train),
            metadata: None,
        }]
    }

    #[tokio::test]
    async fn test_create_find_list_delete() {
        let pool = setup_test_db().await;
        insert_user(&pool, "u1", "user1").await;
        insert_workspace(&pool, "w1", "u1").await;

        let created =
            TestSetRepo::create(&pool, "w1", "ts1", Some("d"), &sample_cases(), None, None)
                .await
                .expect("创建失败");

        let loaded = TestSetRepo::find_by_id(&pool, "w1", &created.id)
            .await
            .expect("查询失败");
        assert_eq!(loaded.name, "ts1");
        assert_eq!(loaded.cases.len(), 1);

        let list = TestSetRepo::list_by_workspace(&pool, "w1")
            .await
            .expect("列表失败");
        assert_eq!(list.len(), 1);

        let summaries = TestSetRepo::list_summaries_by_workspace(&pool, "w1")
            .await
            .expect("列表 summary 失败");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].cases_count, 1);

        let deleted = TestSetRepo::delete(&pool, "w1", &created.id)
            .await
            .expect("删除失败");
        assert!(deleted);
    }
}
