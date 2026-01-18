use std::collections::HashMap;

use prompt_faster::core::iteration_engine::checkpoint::save_checkpoint;
use prompt_faster::domain::models::{IterationState, RuleSystem};
use prompt_faster::domain::types::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, RunControlState,
};
use prompt_faster::infra::db::pool::{create_pool, init_global_db_pool};
use prompt_faster::infra::db::repositories::CheckpointRepo;
use prompt_faster::shared::time::now_millis;
use sqlx::SqlitePool;

async fn seed_user_workspace_task(
    db: &SqlitePool,
    user_id: &str,
    workspace_id: &str,
    task_id: &str,
) {
    let now = now_millis();
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .bind("hashed")
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("insert user");

    sqlx::query(
        r#"
        INSERT INTO workspaces (id, user_id, name, description, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(workspace_id)
    .bind(user_id)
    .bind("ws-test")
    .bind(None::<String>)
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("insert workspace");

    sqlx::query(
        r#"
        INSERT INTO optimization_tasks
          (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, created_at, updated_at)
        VALUES
          (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)
        "#,
    )
    .bind(task_id)
    .bind(workspace_id)
    .bind("ws-task")
    .bind(None::<String>)
    .bind("goal")
    .bind("generic")
    .bind("fixed")
    .bind("draft")
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("insert task");
}

fn build_context(task_id: &str) -> OptimizationContext {
    OptimizationContext {
        task_id: task_id.to_string(),
        execution_target_config: ExecutionTargetConfig::default(),
        current_prompt: "prompt".to_string(),
        rule_system: RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: HashMap::new(),
            version: 1,
        },
        iteration: 1,
        state: IterationState::Completed,
        run_control_state: RunControlState::Running,
        test_cases: vec![],
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions: HashMap::new(),
    }
}

#[tokio::test]
async fn save_checkpoint_persists_to_db() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");
    init_global_db_pool(pool.clone());

    let task_id = "task-1";
    seed_user_workspace_task(&pool, "user-1", "workspace-1", task_id).await;

    let ctx = build_context(task_id);
    let checkpoint = save_checkpoint(&ctx, "user-1", "corr-1")
        .await
        .expect("保存 checkpoint 失败");

    let list = CheckpointRepo::list_checkpoints_by_task(&pool, task_id, 10)
        .await
        .expect("查询 checkpoint 失败");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, checkpoint.id);
}
