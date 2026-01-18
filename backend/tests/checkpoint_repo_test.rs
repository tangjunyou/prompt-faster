use std::collections::HashMap;

use prompt_faster::domain::models::{CheckpointEntity, IterationState, LineageType, RuleSystem};
use prompt_faster::domain::types::{IterationArtifacts, RunControlState, UserGuidance};
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::db::repositories::CheckpointRepo;
use prompt_faster::shared::time::now_millis;
use sqlx::SqlitePool;

fn build_rule_system() -> RuleSystem {
    RuleSystem {
        rules: Vec::new(),
        conflict_resolution_log: Vec::new(),
        merge_log: Vec::new(),
        coverage_map: HashMap::new(),
        version: 1,
    }
}

fn build_checkpoint_entity() -> CheckpointEntity {
    CheckpointEntity {
        id: "checkpoint-1".to_string(),
        task_id: "task-1".to_string(),
        iteration: 1,
        state: IterationState::Completed,
        run_control_state: RunControlState::Running,
        prompt: "prompt".to_string(),
        rule_system: build_rule_system(),
        artifacts: Some(IterationArtifacts::empty()),
        user_guidance: Some(UserGuidance::new("guidance")),
        branch_id: "branch-1".to_string(),
        parent_id: None,
        lineage_type: LineageType::Automatic,
        branch_description: None,
        checksum: "checksum".to_string(),
        created_at: 0,
    }
}

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

#[tokio::test]
async fn checkpoint_repo_create_and_list() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let checkpoint = build_checkpoint_entity();
    seed_user_workspace_task(&pool, "user-1", "workspace-1", &checkpoint.task_id).await;
    let saved = CheckpointRepo::create_checkpoint(&pool, checkpoint.clone())
        .await
        .expect("创建 checkpoint 失败");
    assert_eq!(saved.id, checkpoint.id);

    let items = CheckpointRepo::list_checkpoints_by_task(&pool, &checkpoint.task_id, 20)
        .await
        .expect("查询 checkpoints 失败");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, checkpoint.id);
}
