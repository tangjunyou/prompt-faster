use std::collections::HashMap;

use prompt_faster::domain::models::{
    CheckpointEntity, IterationState, LineageType, PassRateSummary, RuleSystem,
};
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
        archived_at: None,
        archive_reason: None,
        pass_rate_summary: None,
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

#[tokio::test]
async fn checkpoint_repo_lists_summaries_with_archived_filter() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let mut checkpoint = build_checkpoint_entity();
    checkpoint.id = "checkpoint-1".to_string();
    checkpoint.created_at = 10;
    checkpoint.pass_rate_summary = Some(PassRateSummary {
        total_cases: 10,
        passed_cases: 7,
        pass_rate: 0.7,
    });
    seed_user_workspace_task(&pool, "user-1", "workspace-1", &checkpoint.task_id).await;
    CheckpointRepo::create_checkpoint(&pool, checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let mut archived = build_checkpoint_entity();
    archived.id = "checkpoint-2".to_string();
    archived.created_at = 20;
    archived.archived_at = Some(30);
    archived.archive_reason = Some("rollback".to_string());
    archived.pass_rate_summary = Some(PassRateSummary {
        total_cases: 10,
        passed_cases: 5,
        pass_rate: 0.5,
    });
    CheckpointRepo::create_checkpoint(&pool, archived)
        .await
        .expect("创建 checkpoint 失败");

    let active = CheckpointRepo::list_checkpoint_summaries(&pool, "task-1", false, 100, 0)
        .await
        .expect("查询 summaries 失败");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "checkpoint-1");

    let all = CheckpointRepo::list_checkpoint_summaries(&pool, "task-1", true, 100, 0)
        .await
        .expect("查询 summaries 失败");
    assert_eq!(all.len(), 2);
    assert!(all.iter().any(|item| item.id == "checkpoint-2"));
}

#[tokio::test]
async fn checkpoint_repo_archives_only_after_target_on_same_branch() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let mut target = build_checkpoint_entity();
    target.id = "checkpoint-target".to_string();
    target.created_at = 100;
    target.branch_id = "branch-a".to_string();
    seed_user_workspace_task(&pool, "user-1", "workspace-1", &target.task_id).await;
    CheckpointRepo::create_checkpoint(&pool, target)
        .await
        .expect("创建 checkpoint 失败");

    let mut after = build_checkpoint_entity();
    after.id = "checkpoint-after".to_string();
    after.created_at = 200;
    after.branch_id = "branch-a".to_string();
    CheckpointRepo::create_checkpoint(&pool, after)
        .await
        .expect("创建 checkpoint 失败");

    let mut other_branch = build_checkpoint_entity();
    other_branch.id = "checkpoint-other-branch".to_string();
    other_branch.created_at = 300;
    other_branch.branch_id = "branch-b".to_string();
    CheckpointRepo::create_checkpoint(&pool, other_branch)
        .await
        .expect("创建 checkpoint 失败");

    let archived =
        CheckpointRepo::archive_checkpoints_after(&pool, "task-1", "checkpoint-target", "rollback")
            .await
            .expect("归档失败");
    assert_eq!(archived, 1);

    let summaries = CheckpointRepo::list_checkpoint_summaries(&pool, "task-1", true, 100, 0)
        .await
        .expect("查询 summaries 失败");
    let archived_item = summaries
        .iter()
        .find(|item| item.id == "checkpoint-after")
        .expect("应包含归档项");
    assert!(archived_item.archived_at.is_some());
    assert_eq!(archived_item.archive_reason.as_deref(), Some("rollback"));

    let other_item = summaries
        .iter()
        .find(|item| item.id == "checkpoint-other-branch")
        .expect("应包含其他分支项");
    assert!(other_item.archived_at.is_none());
}

#[tokio::test]
async fn checkpoint_repo_get_checkpoint_with_summary() {
    let pool = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("运行 migrations 失败");

    let mut checkpoint = build_checkpoint_entity();
    checkpoint.id = "checkpoint-with-summary".to_string();
    checkpoint.pass_rate_summary = Some(PassRateSummary {
        total_cases: 8,
        passed_cases: 6,
        pass_rate: 0.75,
    });
    seed_user_workspace_task(&pool, "user-1", "workspace-1", &checkpoint.task_id).await;
    CheckpointRepo::create_checkpoint(&pool, checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let item = CheckpointRepo::get_checkpoint_with_summary(&pool, "checkpoint-with-summary")
        .await
        .expect("查询 checkpoint 失败")
        .expect("checkpoint 应存在");
    assert_eq!(item.checkpoint.id, "checkpoint-with-summary");
    assert_eq!(
        item.pass_rate_summary.as_ref().map(|s| s.pass_rate),
        Some(0.75)
    );
}
