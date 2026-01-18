use sqlx::Row;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteRow;
use thiserror::Error;

use crate::domain::models::OptimizationTaskConfig;
use crate::domain::models::optimization_task_config::{
    OPTIMIZATION_TASK_CONFIG_MAX_JSON_BYTES, serialize_config_with_existing_extra,
};
use crate::domain::models::{
    ExecutionTargetType, OptimizationTaskEntity, OptimizationTaskMode, OptimizationTaskStatus,
};
use crate::shared::time::now_millis;

#[derive(Error, Debug)]
pub enum OptimizationTaskRepoError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("工作区未找到")]
    WorkspaceNotFound,

    #[error("测试集未找到")]
    TestSetNotFound,

    #[error("优化任务未找到")]
    NotFound,

    #[error("任务配置无效: {0}")]
    InvalidConfig(String),
}

pub struct OptimizationTaskRepo;

#[derive(Debug, Clone)]
pub struct OptimizationTaskWithTestSets {
    pub task: OptimizationTaskEntity,
    pub test_set_ids: Vec<String>,
}

pub struct CreateOptimizationTaskInput<'a> {
    pub user_id: &'a str,
    pub workspace_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub goal: &'a str,
    pub execution_target_type: ExecutionTargetType,
    pub task_mode: OptimizationTaskMode,
    pub test_set_ids: &'a [String],
}

fn parse_execution_target_type(raw: &str) -> Option<ExecutionTargetType> {
    match raw {
        "dify" => Some(ExecutionTargetType::Dify),
        "generic" => Some(ExecutionTargetType::Generic),
        "example" => Some(ExecutionTargetType::Example),
        _ => None,
    }
}

fn parse_task_mode(raw: &str) -> Option<OptimizationTaskMode> {
    match raw {
        "fixed" => Some(OptimizationTaskMode::Fixed),
        "creative" => Some(OptimizationTaskMode::Creative),
        _ => None,
    }
}

fn parse_task_status(raw: &str) -> Option<OptimizationTaskStatus> {
    match raw {
        "draft" => Some(OptimizationTaskStatus::Draft),
        "running" => Some(OptimizationTaskStatus::Running),
        "paused" => Some(OptimizationTaskStatus::Paused),
        "completed" => Some(OptimizationTaskStatus::Completed),
        "terminated" => Some(OptimizationTaskStatus::Terminated),
        _ => None,
    }
}

fn serialize_execution_target_type(value: ExecutionTargetType) -> &'static str {
    match value {
        ExecutionTargetType::Dify => "dify",
        ExecutionTargetType::Generic => "generic",
        ExecutionTargetType::Example => "example",
    }
}

fn serialize_task_mode(value: OptimizationTaskMode) -> &'static str {
    match value {
        OptimizationTaskMode::Fixed => "fixed",
        OptimizationTaskMode::Creative => "creative",
    }
}

fn serialize_task_status(value: OptimizationTaskStatus) -> &'static str {
    match value {
        OptimizationTaskStatus::Draft => "draft",
        OptimizationTaskStatus::Running => "running",
        OptimizationTaskStatus::Paused => "paused",
        OptimizationTaskStatus::Completed => "completed",
        OptimizationTaskStatus::Terminated => "terminated",
    }
}

fn row_to_entity(row: &SqliteRow) -> Result<OptimizationTaskEntity, OptimizationTaskRepoError> {
    let execution_target_type_raw: String = row.try_get("execution_target_type")?;
    let task_mode_raw: String = row.try_get("task_mode")?;
    let status_raw: String = row.try_get("status")?;

    let execution_target_type = parse_execution_target_type(&execution_target_type_raw)
        .ok_or_else(|| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "invalid execution_target_type: {}",
                    execution_target_type_raw
                ),
            )))
        })?;
    let task_mode = parse_task_mode(&task_mode_raw).ok_or_else(|| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid task_mode: {}", task_mode_raw),
        )))
    })?;
    let status = parse_task_status(&status_raw).ok_or_else(|| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid status: {}", status_raw),
        )))
    })?;

    Ok(OptimizationTaskEntity {
        id: row.try_get("id")?,
        workspace_id: row.try_get("workspace_id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        goal: row.try_get("goal")?,
        execution_target_type,
        task_mode,
        status,
        config_json: row.try_get("config_json")?,
        final_prompt: row.try_get("final_prompt")?,
        terminated_at: row.try_get("terminated_at")?,
        selected_iteration_id: row.try_get("selected_iteration_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

impl OptimizationTaskRepo {
    pub async fn create_scoped(
        pool: &SqlitePool,
        input: CreateOptimizationTaskInput<'_>,
    ) -> Result<OptimizationTaskWithTestSets, OptimizationTaskRepoError> {
        let now = now_millis();
        let task_id = uuid::Uuid::new_v4().to_string();

        let mut tx = pool.begin().await?;

        let workspace_exists =
            sqlx::query_scalar::<_, i64>("SELECT 1 FROM workspaces WHERE id = ?1 AND user_id = ?2")
                .bind(input.workspace_id)
                .bind(input.user_id)
                .fetch_optional(&mut *tx)
                .await?
                .is_some();
        if !workspace_exists {
            return Err(OptimizationTaskRepoError::WorkspaceNotFound);
        }

        for test_set_id in input.test_set_ids {
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT 1 FROM test_sets WHERE id = ?1 AND workspace_id = ?2 AND is_template = 0",
            )
            .bind(test_set_id)
            .bind(input.workspace_id)
            .fetch_optional(&mut *tx)
            .await?
            .is_some();
            if !exists {
                return Err(OptimizationTaskRepoError::TestSetNotFound);
            }
        }

        sqlx::query(
            r#"
            INSERT INTO optimization_tasks
              (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, created_at, updated_at)
            VALUES
              (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)
            "#,
        )
        .bind(&task_id)
        .bind(input.workspace_id)
        .bind(input.name)
        .bind(input.description)
        .bind(input.goal)
        .bind(serialize_execution_target_type(input.execution_target_type))
        .bind(serialize_task_mode(input.task_mode))
        .bind(serialize_task_status(OptimizationTaskStatus::Draft))
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        for test_set_id in input.test_set_ids {
            sqlx::query(
                r#"
                INSERT INTO optimization_task_test_sets (optimization_task_id, test_set_id, created_at)
                VALUES (?1, ?2, ?3)
                "#,
            )
            .bind(&task_id)
            .bind(test_set_id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(OptimizationTaskWithTestSets {
            task: OptimizationTaskEntity {
                id: task_id,
                workspace_id: input.workspace_id.to_string(),
                name: input.name.to_string(),
                description: input.description.map(|s| s.to_string()),
                goal: input.goal.to_string(),
                execution_target_type: input.execution_target_type,
                task_mode: input.task_mode,
                status: OptimizationTaskStatus::Draft,
                config_json: None,
                final_prompt: None,
                terminated_at: None,
                selected_iteration_id: None,
                created_at: now,
                updated_at: now,
            },
            test_set_ids: input.test_set_ids.to_vec(),
        })
    }

    pub async fn list_by_workspace_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
    ) -> Result<Vec<OptimizationTaskWithTestSets>, OptimizationTaskRepoError> {
        let rows = sqlx::query(
            r#"
            SELECT t.*, otts.test_set_id AS test_set_id
            FROM optimization_tasks t
            JOIN workspaces w ON w.id = t.workspace_id
            LEFT JOIN optimization_task_test_sets otts ON otts.optimization_task_id = t.id
            WHERE t.workspace_id = ?1 AND w.user_id = ?2
            ORDER BY t.updated_at DESC, otts.created_at ASC
            "#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let mut by_id: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut tasks: Vec<OptimizationTaskWithTestSets> = Vec::new();

        for row in rows {
            let task_id: String = row.try_get("id")?;
            let idx = match by_id.get(&task_id) {
                Some(v) => *v,
                None => {
                    let task = row_to_entity(&row)?;
                    tasks.push(OptimizationTaskWithTestSets {
                        task,
                        test_set_ids: Vec::new(),
                    });
                    let idx = tasks.len() - 1;
                    by_id.insert(task_id, idx);
                    idx
                }
            };

            let test_set_id: Option<String> = row.try_get("test_set_id")?;
            if let Some(test_set_id) = test_set_id {
                tasks[idx].test_set_ids.push(test_set_id);
            }
        }

        Ok(tasks)
    }

    pub async fn find_by_id_for_user(
        pool: &SqlitePool,
        user_id: &str,
        task_id: &str,
    ) -> Result<OptimizationTaskEntity, OptimizationTaskRepoError> {
        let row = sqlx::query(
            r#"
            SELECT t.*
            FROM optimization_tasks t
            JOIN workspaces w ON w.id = t.workspace_id
            WHERE t.id = ?1 AND w.user_id = ?2
            "#,
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = row else {
            return Err(OptimizationTaskRepoError::NotFound);
        };

        row_to_entity(&row)
    }

    pub async fn find_by_id_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        task_id: &str,
    ) -> Result<OptimizationTaskWithTestSets, OptimizationTaskRepoError> {
        let row = sqlx::query(
            r#"
            SELECT t.*
            FROM optimization_tasks t
            JOIN workspaces w ON w.id = t.workspace_id
            WHERE t.id = ?1 AND t.workspace_id = ?2 AND w.user_id = ?3
            "#,
        )
        .bind(task_id)
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let Some(row) = row else {
            return Err(OptimizationTaskRepoError::NotFound);
        };

        let task = row_to_entity(&row)?;
        let rels = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT test_set_id
            FROM optimization_task_test_sets
            WHERE optimization_task_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .bind(task_id)
        .fetch_all(pool)
        .await?;

        Ok(OptimizationTaskWithTestSets {
            task,
            test_set_ids: rels.into_iter().map(|(id,)| id).collect(),
        })
    }

    pub async fn update_config_scoped(
        pool: &SqlitePool,
        user_id: &str,
        workspace_id: &str,
        task_id: &str,
        config: OptimizationTaskConfig,
    ) -> Result<OptimizationTaskWithTestSets, OptimizationTaskRepoError> {
        let now = now_millis();
        let mut tx = pool.begin().await?;

        let workspace_exists =
            sqlx::query_scalar::<_, i64>("SELECT 1 FROM workspaces WHERE id = ?1 AND user_id = ?2")
                .bind(workspace_id)
                .bind(user_id)
                .fetch_optional(&mut *tx)
                .await?
                .is_some();
        if !workspace_exists {
            return Err(OptimizationTaskRepoError::WorkspaceNotFound);
        }

        let row = sqlx::query(
            r#"
            SELECT t.*
            FROM optimization_tasks t
            WHERE t.id = ?1 AND t.workspace_id = ?2
            "#,
        )
        .bind(task_id)
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            return Err(OptimizationTaskRepoError::NotFound);
        };

        let existing_config_json: Option<String> = row.try_get("config_json")?;
        let config_json_bytes =
            serialize_config_with_existing_extra(config, existing_config_json.as_deref())
                .map_err(OptimizationTaskRepoError::InvalidConfig)?;
        if config_json_bytes.len() > OPTIMIZATION_TASK_CONFIG_MAX_JSON_BYTES {
            return Err(OptimizationTaskRepoError::InvalidConfig(format!(
                "任务配置过大（最大 {} bytes）",
                OPTIMIZATION_TASK_CONFIG_MAX_JSON_BYTES
            )));
        }
        let config_json = String::from_utf8(config_json_bytes).map_err(|_| {
            OptimizationTaskRepoError::InvalidConfig("任务配置编码失败".to_string())
        })?;

        sqlx::query(
            r#"
            UPDATE optimization_tasks
            SET config_json = ?1, updated_at = ?2
            WHERE id = ?3 AND workspace_id = ?4
            "#,
        )
        .bind(&config_json)
        .bind(now)
        .bind(task_id)
        .bind(workspace_id)
        .execute(&mut *tx)
        .await?;

        let mut task = row_to_entity(&row)?;
        task.config_json = Some(config_json);
        task.updated_at = now;

        let rels = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT test_set_id
            FROM optimization_task_test_sets
            WHERE optimization_task_id = ?1
            ORDER BY created_at ASC
            "#,
        )
        .bind(task_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(OptimizationTaskWithTestSets {
            task,
            test_set_ids: rels.into_iter().map(|(id,)| id).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{TaskReference, TestCase};
    use crate::infra::db::pool::create_pool;
    use crate::infra::db::repositories::{TestSetRepo, WorkspaceRepo};

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

    fn sample_exact_case() -> TestCase {
        TestCase {
            id: "case-1".to_string(),
            input: std::collections::HashMap::new(),
            reference: TaskReference::Exact {
                expected: "ok".to_string(),
            },
            split: None,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_create_list_get_scoped() {
        let pool = setup_test_db().await;

        insert_user(&pool, "u1", "user1").await;
        let workspace = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");

        let test_set = TestSetRepo::create(
            &pool,
            &workspace.id,
            "ts",
            None,
            &[sample_exact_case()],
            None,
            None,
        )
        .await
        .expect("创建测试集失败");

        let created = OptimizationTaskRepo::create_scoped(
            &pool,
            CreateOptimizationTaskInput {
                user_id: "u1",
                workspace_id: &workspace.id,
                name: "task",
                description: Some("desc"),
                goal: "goal",
                execution_target_type: ExecutionTargetType::Dify,
                task_mode: OptimizationTaskMode::Fixed,
                test_set_ids: std::slice::from_ref(&test_set.id),
            },
        )
        .await
        .expect("创建任务失败");

        let list = OptimizationTaskRepo::list_by_workspace_scoped(&pool, "u1", &workspace.id)
            .await
            .expect("列表失败");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].task.id, created.task.id);
        assert_eq!(list[0].test_set_ids, vec![test_set.id.clone()]);

        let loaded =
            OptimizationTaskRepo::find_by_id_scoped(&pool, "u1", &workspace.id, &created.task.id)
                .await
                .expect("获取失败");
        assert_eq!(loaded.task.name, "task");
        assert_eq!(loaded.task.description.as_deref(), Some("desc"));
        assert_eq!(loaded.task.goal, "goal");
        assert_eq!(loaded.task.execution_target_type, ExecutionTargetType::Dify);
        assert_eq!(loaded.task.task_mode, OptimizationTaskMode::Fixed);
        assert_eq!(loaded.task.status, OptimizationTaskStatus::Draft);
        assert_eq!(loaded.test_set_ids, vec![test_set.id]);
    }

    #[tokio::test]
    async fn test_create_scoped_rejects_other_workspace_test_set() {
        let pool = setup_test_db().await;

        insert_user(&pool, "u1", "user1").await;
        let ws1 = WorkspaceRepo::create(&pool, "u1", "ws1", None)
            .await
            .expect("创建工作区失败");
        let ws2 = WorkspaceRepo::create(&pool, "u1", "ws2", None)
            .await
            .expect("创建工作区失败");

        let test_set_ws2 = TestSetRepo::create(
            &pool,
            &ws2.id,
            "ts2",
            None,
            &[sample_exact_case()],
            None,
            None,
        )
        .await
        .expect("创建测试集失败");

        let err = OptimizationTaskRepo::create_scoped(
            &pool,
            CreateOptimizationTaskInput {
                user_id: "u1",
                workspace_id: &ws1.id,
                name: "task",
                description: None,
                goal: "goal",
                execution_target_type: ExecutionTargetType::Dify,
                task_mode: OptimizationTaskMode::Fixed,
                test_set_ids: &[test_set_ws2.id],
            },
        )
        .await
        .expect_err("应失败");

        assert!(matches!(err, OptimizationTaskRepoError::TestSetNotFound));
    }
}
