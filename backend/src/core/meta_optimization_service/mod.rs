//! 元优化服务

use sqlx::SqlitePool;
use thiserror::Error;

use crate::domain::models::{
    CreateTeacherPromptInput, MetaOptimizationOverview, MetaOptimizationTaskSummary, TeacherPrompt,
    TeacherPromptStats, TeacherPromptVersion,
};
use crate::domain::types::unix_ms_to_iso8601;
use crate::infra::db::repositories::{
    CreateTeacherPromptRecordInput, TeacherPromptRecord, TeacherPromptRepo, TeacherPromptRepoError,
    TeacherPromptVersionWithStatsRecord,
};

#[derive(Debug, Error)]
pub enum MetaOptimizationServiceError {
    #[error("Prompt 版本不存在或无权访问")]
    NotFoundOrForbidden,
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("仓库错误: {0}")]
    Repo(String),
}

fn record_to_prompt(record: TeacherPromptRecord) -> TeacherPrompt {
    TeacherPrompt {
        id: record.id,
        user_id: record.user_id,
        version: record.version,
        content: record.content,
        description: record.description,
        is_active: record.is_active,
        created_at: unix_ms_to_iso8601(record.created_at),
        updated_at: unix_ms_to_iso8601(record.updated_at),
    }
}

fn version_with_stats_record_to_version(
    record: TeacherPromptVersionWithStatsRecord,
) -> (TeacherPromptVersion, TeacherPromptStats) {
    let success_rate = if record.total_tasks == 0 {
        None
    } else {
        Some(record.successful_tasks as f64 / record.total_tasks as f64)
    };

    let version = TeacherPromptVersion {
        id: record.id.clone(),
        version: record.version,
        description: record.description.clone(),
        is_active: record.is_active,
        success_rate,
        task_count: record.total_tasks,
        created_at: unix_ms_to_iso8601(record.created_at),
    };

    let stats = TeacherPromptStats {
        version_id: record.id,
        version: record.version,
        total_tasks: record.total_tasks,
        successful_tasks: record.successful_tasks,
        success_rate,
        average_pass_rate: record.average_pass_rate,
    };

    (version, stats)
}

fn map_repo_error(err: TeacherPromptRepoError) -> MetaOptimizationServiceError {
    match err {
        TeacherPromptRepoError::NotFound => MetaOptimizationServiceError::NotFoundOrForbidden,
        TeacherPromptRepoError::DatabaseError(e) => MetaOptimizationServiceError::Database(e),
    }
}

pub async fn create_prompt_version(
    pool: &SqlitePool,
    user_id: &str,
    input: CreateTeacherPromptInput,
) -> Result<TeacherPromptVersion, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::create(
        pool,
        user_id,
        CreateTeacherPromptRecordInput {
            content: input.content,
            description: input.description,
            activate: input.activate,
        },
    )
    .await
    .map_err(map_repo_error)?;

    let stats = TeacherPromptRepo::calculate_stats(pool, &record.id, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(TeacherPromptVersion {
        id: record.id,
        version: record.version,
        description: record.description,
        is_active: record.is_active,
        success_rate: stats.success_rate,
        task_count: stats.total_tasks,
        created_at: unix_ms_to_iso8601(record.created_at),
    })
}

pub async fn list_prompt_versions(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<TeacherPromptVersion>, MetaOptimizationServiceError> {
    let records = TeacherPromptRepo::list_with_stats_by_user(pool, user_id, limit, offset)
        .await
        .map_err(map_repo_error)?;

    let mut versions = Vec::with_capacity(records.len());
    for record in records {
        let (version, _stats) = version_with_stats_record_to_version(record);
        versions.push(version);
    }

    Ok(versions)
}

pub async fn get_active_prompt(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<TeacherPrompt>, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::find_active(pool, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(record.map(record_to_prompt))
}

pub async fn set_active_prompt(
    pool: &SqlitePool,
    user_id: &str,
    version_id: &str,
) -> Result<TeacherPrompt, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::set_active(pool, version_id, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(record_to_prompt(record))
}

pub async fn get_prompt_by_id(
    pool: &SqlitePool,
    user_id: &str,
    version_id: &str,
) -> Result<TeacherPrompt, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::find_by_id(pool, version_id, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(record_to_prompt(record))
}

pub async fn get_overview(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<MetaOptimizationOverview, MetaOptimizationServiceError> {
    let total_versions = TeacherPromptRepo::count_by_user(pool, user_id)
        .await
        .map_err(map_repo_error)?;

    if total_versions == 0 {
        return Ok(MetaOptimizationOverview {
            total_versions: 0,
            active_version: None,
            best_version: None,
            stats: Vec::new(),
        });
    }

    let records =
        TeacherPromptRepo::list_with_stats_by_user(pool, user_id, total_versions as i64, 0)
        .await
        .map_err(map_repo_error)?;

    let mut stats_list = Vec::with_capacity(records.len());
    let mut versions = Vec::with_capacity(records.len());
    for record in records {
        let (version, stats) = version_with_stats_record_to_version(record);
        stats_list.push(stats);
        versions.push(version);
    }

    let active_version = versions.iter().find(|v| v.is_active).cloned();
    let best_version = versions
        .iter()
        .filter(|v| v.success_rate.is_some())
        .max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();

    Ok(MetaOptimizationOverview {
        total_versions,
        active_version,
        best_version,
        stats: stats_list,
    })
}

#[allow(dead_code)]
pub async fn get_historical_tasks_for_meta_optimization(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<MetaOptimizationTaskSummary>, MetaOptimizationServiceError> {
    let task_rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
        r#"
        SELECT ot.id, ot.workspace_id, ot.name, ot.status, ot.created_at
        FROM optimization_tasks ot
        JOIN workspaces w ON ot.workspace_id = w.id
        WHERE w.user_id = ?1
        ORDER BY ot.created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(MetaOptimizationServiceError::Database)?;

    let mut summaries = Vec::with_capacity(task_rows.len());
    for (task_id, workspace_id, name, status, created_at) in task_rows {
        let selected_iteration_id: Option<String> = sqlx::query_scalar(
            r#"
            SELECT selected_iteration_id
            FROM optimization_tasks
            WHERE id = ?1
            "#,
        )
        .bind(&task_id)
        .fetch_optional(pool)
        .await
        .map_err(MetaOptimizationServiceError::Database)?;

        let selected_pass_rate: Option<f64> = if let Some(iteration_id) = selected_iteration_id {
            sqlx::query_scalar("SELECT pass_rate FROM iterations WHERE id = ?1")
                .bind(iteration_id)
                .fetch_optional(pool)
                .await
                .map_err(MetaOptimizationServiceError::Database)?
        } else {
            None
        };

        let pass_rate = if selected_pass_rate.is_some() {
            selected_pass_rate
        } else {
            sqlx::query_scalar(
                r#"
                SELECT pass_rate
                FROM iterations
                WHERE task_id = ?1 AND status = 'completed'
                ORDER BY round DESC
                LIMIT 1
                "#,
            )
            .bind(&task_id)
            .fetch_optional(pool)
            .await
            .map_err(MetaOptimizationServiceError::Database)?
        };

        summaries.push(MetaOptimizationTaskSummary {
            id: task_id,
            workspace_id,
            name,
            status,
            pass_rate,
            created_at: unix_ms_to_iso8601(created_at),
        });
    }

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    use crate::infra::db::pool::create_pool;
    use crate::infra::db::repositories::{TeacherPromptRepo, WorkspaceRepo};
    use crate::shared::time::now_millis;

    async fn setup_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("创建测试数据库失败");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("运行 migrations 失败");
        pool
    }

    async fn insert_user(pool: &SqlitePool, user_id: &str, username: &str) {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind("hash")
        .bind(now_millis())
        .bind(now_millis())
        .execute(pool)
        .await
        .expect("插入用户失败");
    }

    async fn insert_task_with_iteration(
        pool: &SqlitePool,
        task_id: &str,
        workspace_id: &str,
        version_id: &str,
        selected_iteration_id: Option<&str>,
        pass_rate: f64,
        round: i32,
    ) {
        let now = now_millis();
        sqlx::query(
            r#"
            INSERT INTO optimization_tasks
              (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, teacher_prompt_version_id, selected_iteration_id, created_at, updated_at)
            VALUES
              (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, NULL, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(task_id)
        .bind(workspace_id)
        .bind("task")
        .bind("goal")
        .bind("example")
        .bind("fixed")
        .bind("completed")
        .bind(version_id)
        .bind(selected_iteration_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .expect("插入任务失败");

        sqlx::query(
            r#"
            INSERT INTO iterations
              (id, task_id, round, started_at, completed_at, status, artifacts, evaluation_results, reflection_summary, pass_rate, total_cases, passed_cases, created_at)
            VALUES
              (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, NULL, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(selected_iteration_id.unwrap_or(task_id))
        .bind(task_id)
        .bind(round)
        .bind(now)
        .bind(now)
        .bind("completed")
        .bind(pass_rate)
        .bind(10)
        .bind((pass_rate * 10.0).round() as i32)
        .bind(now)
        .execute(pool)
        .await
        .expect("插入迭代失败");
    }

    #[tokio::test]
    async fn test_create_and_list_versions() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let v1 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-1".to_string(),
                description: Some("first".to_string()),
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        let v2 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-2".to_string(),
                description: Some("second".to_string()),
                activate: false,
            },
        )
        .await
        .expect("创建版本失败");

        assert_eq!(v1.version, 1);
        assert_eq!(v2.version, 2);

        let list = list_prompt_versions(&pool, "u1", 50, 0)
            .await
            .expect("获取列表失败");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].version, 2);
        assert_eq!(list[1].version, 1);
    }

    #[tokio::test]
    async fn test_set_active_prompt() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let v1 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-1".to_string(),
                description: None,
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        let v2 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-2".to_string(),
                description: None,
                activate: false,
            },
        )
        .await
        .expect("创建版本失败");

        let active_before = TeacherPromptRepo::find_active(&pool, "u1")
            .await
            .expect("查询失败")
            .expect("缺少 active");
        assert_eq!(active_before.id, v1.id);

        let active = set_active_prompt(&pool, "u1", &v2.id)
            .await
            .expect("设置 active 失败");
        assert_eq!(active.id, v2.id);

        let active_after = TeacherPromptRepo::find_active(&pool, "u1")
            .await
            .expect("查询失败")
            .expect("缺少 active");
        assert_eq!(active_after.id, v2.id);
    }

    #[tokio::test]
    async fn test_success_rate_calculation() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let workspace = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");

        let version = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt".to_string(),
                description: None,
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        insert_task_with_iteration(
            &pool,
            "task-1",
            &workspace.id,
            &version.id,
            Some("iter-1"),
            1.0,
            1,
        )
        .await;

        insert_task_with_iteration(
            &pool,
            "task-2",
            &workspace.id,
            &version.id,
            None,
            0.5,
            2,
        )
        .await;

        let stats = TeacherPromptRepo::calculate_stats(&pool, &version.id, "u1")
            .await
            .expect("统计失败");

        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.successful_tasks, 1);
        assert_eq!(stats.success_rate.unwrap(), 0.5);
        assert_eq!(stats.average_pass_rate.unwrap(), 0.75);
    }

    #[tokio::test]
    async fn test_success_rate_empty_tasks_returns_null() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let version = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt".to_string(),
                description: None,
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        let stats = TeacherPromptRepo::calculate_stats(&pool, &version.id, "u1")
            .await
            .expect("统计失败");

        assert_eq!(stats.total_tasks, 0);
        assert!(stats.success_rate.is_none());
        assert!(stats.average_pass_rate.is_none());
    }

    #[tokio::test]
    async fn test_list_prompt_versions_pagination() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        for idx in 0..3 {
            let _ = create_prompt_version(
                &pool,
                "u1",
                CreateTeacherPromptInput {
                    content: format!("prompt-{idx}"),
                    description: None,
                    activate: idx == 0,
                },
            )
            .await
            .expect("创建版本失败");
        }

        let page = list_prompt_versions(&pool, "u1", 1, 1)
            .await
            .expect("获取列表失败");
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].version, 2);
    }

    #[tokio::test]
    async fn test_concurrent_version_creation_unique_versions() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let create_a = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-a".to_string(),
                description: None,
                activate: true,
            },
        );
        let create_b = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-b".to_string(),
                description: None,
                activate: false,
            },
        );

        let (a, b) = tokio::join!(create_a, create_b);
        let a = a.expect("创建版本失败");
        let b = b.expect("创建版本失败");

        let mut versions = vec![a.version, b.version];
        versions.sort_unstable();
        versions.dedup();
        assert_eq!(versions, vec![1, 2]);
    }
}
