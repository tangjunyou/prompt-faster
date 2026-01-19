use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

use prompt_faster::api::middleware::correlation_id::correlation_id_middleware;
use prompt_faster::api::middleware::{LoginAttemptStore, SessionStore, auth_middleware};
use prompt_faster::api::routes::{auth, recovery, user_auth};
use prompt_faster::api::state::AppState;
use prompt_faster::core::iteration_engine::checkpoint::compute_checksum;
use prompt_faster::domain::models::{
    CheckpointCreateRequest, CheckpointEntity, ConnectivityStatus, IterationState, LineageType,
    RuleSystem, TaskReference, TestCase,
};
use prompt_faster::domain::types::{IterationArtifacts, RunControlState};
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::db::repositories::{
    CheckpointRepo, OptimizationTaskRepo, TestSetRepo, WorkspaceRepo,
};
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::connectivity::{
    check_connectivity_status, record_connectivity_failure, record_connectivity_success,
};
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;
use prompt_faster::shared::time::now_millis;

async fn setup_test_app_with_db() -> (Router, sqlx::SqlitePool) {
    let db = create_pool("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("运行 migrations 失败");

    let http_client = create_http_client().expect("创建 HTTP 客户端失败");
    let config = Arc::new(AppConfig {
        database_url: "sqlite::memory:".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 0,
        log_level: "info".to_string(),
        is_dev: true,
        cors_origins: vec![],
        is_docker: false,
        allow_http_base_url: true,
        allow_localhost_base_url: true,
        allow_private_network_base_url: true,
        checkpoint_cache_limit: 10,
        checkpoint_memory_alert_threshold: 10,
    });
    let api_key_manager = Arc::new(ApiKeyManager::new(None));
    let session_store = SessionStore::new(24);
    let login_attempt_store = LoginAttemptStore::default();

    let state = AppState {
        db: db.clone(),
        http_client,
        config,
        api_key_manager,
        session_store,
        login_attempt_store,
    };

    let session_store_for_middleware = state.session_store.clone();

    let protected_routes = auth::protected_router().layer(middleware::from_fn_with_state(
        session_store_for_middleware.clone(),
        auth_middleware,
    ));
    let protected_user_auth_routes = user_auth::protected_router().layer(
        middleware::from_fn_with_state(session_store_for_middleware.clone(), auth_middleware),
    );
    let protected_recovery_routes = recovery::router().layer(middleware::from_fn_with_state(
        session_store_for_middleware,
        auth_middleware,
    ));
    let protected_task_recovery_routes = recovery::task_router().layer(
        middleware::from_fn_with_state(state.session_store.clone(), auth_middleware),
    );

    let router = Router::<AppState>::new()
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .nest("/api/v1/recovery", protected_recovery_routes)
        .nest("/api/v1/tasks/{task_id}", protected_task_recovery_routes)
        .nest("/api/v1", recovery::connectivity_router())
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware));

    (router, db)
}

async fn read_json_body(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("读取响应 body 失败")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("解析 JSON 失败")
}

fn build_json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&body).expect("序列化 JSON 失败"),
        ))
        .expect("构建请求失败")
}

fn build_empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn with_bearer(mut req: Request<Body>, token: &str) -> Request<Body> {
    req.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    req
}

async fn register_user(app: &Router, username: &str, password: &str) -> (String, String) {
    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );
    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    let register_json = read_json_body(register_resp).await;
    let token = register_json["data"]["session_token"]
        .as_str()
        .unwrap()
        .to_string();
    let user_id = register_json["data"]["user"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    (token, user_id)
}

fn build_checkpoint(task_id: &str, iteration: u32) -> CheckpointEntity {
    build_checkpoint_with(task_id, iteration, None, now_millis())
}

fn build_checkpoint_with(
    task_id: &str,
    iteration: u32,
    checksum_override: Option<String>,
    created_at: i64,
) -> CheckpointEntity {
    let req = CheckpointCreateRequest {
        task_id: task_id.to_string(),
        iteration,
        state: IterationState::RunningTests,
        run_control_state: RunControlState::Running,
        prompt: "p".to_string(),
        rule_system: RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: std::collections::HashMap::new(),
            version: 1,
        },
        artifacts: Some(IterationArtifacts::default()),
        user_guidance: None,
        branch_id: task_id.to_string(),
        parent_id: None,
        lineage_type: LineageType::Automatic,
        branch_description: None,
    };
    let checksum = checksum_override.unwrap_or_else(|| compute_checksum(&req));

    CheckpointEntity {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: req.task_id,
        iteration: req.iteration,
        state: req.state,
        run_control_state: req.run_control_state,
        prompt: req.prompt,
        rule_system: req.rule_system,
        artifacts: req.artifacts,
        user_guidance: req.user_guidance,
        branch_id: req.branch_id,
        parent_id: req.parent_id,
        lineage_type: req.lineage_type,
        branch_description: req.branch_description,
        checksum,
        created_at,
        archived_at: None,
        archive_reason: None,
        pass_rate_summary: None,
    }
}

fn sample_case() -> TestCase {
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

async fn create_task_with_test_set(db: &sqlx::SqlitePool, user_id: &str) -> String {
    let workspace = WorkspaceRepo::create(db, user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set = TestSetRepo::create(db, &workspace.id, "ts", None, &[sample_case()], None, None)
        .await
        .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    OptimizationTaskRepo::update_status(
        db,
        &created.task.id,
        prompt_faster::domain::models::OptimizationTaskStatus::Running,
    )
    .await
    .expect("更新状态失败");

    created.task.id
}

#[tokio::test]
async fn recovery_unfinished_tasks_and_recover_flow() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "user1", "pass123").await;

    let workspace = WorkspaceRepo::create(&db, &user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    OptimizationTaskRepo::update_status(
        &db,
        &created.task.id,
        prompt_faster::domain::models::OptimizationTaskStatus::Running,
    )
    .await
    .expect("更新状态失败");

    let checkpoint = build_checkpoint(&created.task.id, 1);
    CheckpointRepo::create_checkpoint(&db, checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let req = with_bearer(
        build_empty_request("GET", "/api/v1/recovery/unfinished-tasks"),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["total"].as_u64().unwrap(), 1);
    assert!(body["data"]["tasks"][0]["checkpointId"].is_string());

    let recover_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/recovery/tasks/{}/recover", created.task.id),
            json!({}),
        ),
        &token,
    );
    let recover_resp = app.clone().oneshot(recover_req).await.unwrap();
    assert_eq!(recover_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn recovery_permission_denied_for_non_owner() {
    let (app, db) = setup_test_app_with_db().await;
    let (token1, user_id1) = register_user(&app, "user1", "pass123").await;
    let (token2, _user_id2) = register_user(&app, "user2", "pass123").await;

    let workspace = WorkspaceRepo::create(&db, &user_id1, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id1,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    let recover_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/recovery/tasks/{}/recover", created.task.id),
            json!({}),
        ),
        &token2,
    );
    let recover_resp = app.clone().oneshot(recover_req).await.unwrap();
    assert_eq!(recover_resp.status(), StatusCode::FORBIDDEN);

    let list_req = with_bearer(
        build_empty_request("GET", "/api/v1/recovery/unfinished-tasks"),
        &token1,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn recovery_falls_back_to_previous_checkpoint() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "user1", "pass123").await;

    let workspace = WorkspaceRepo::create(&db, &user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    let valid = build_checkpoint_with(&created.task.id, 1, None, 1000);
    CheckpointRepo::create_checkpoint(&db, valid.clone())
        .await
        .expect("创建 checkpoint 失败");

    let invalid = build_checkpoint_with(&created.task.id, 2, Some("bad".to_string()), 2000);
    CheckpointRepo::create_checkpoint(&db, invalid)
        .await
        .expect("创建 checkpoint 失败");

    let recover_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/recovery/tasks/{}/recover", created.task.id),
            json!({}),
        ),
        &token,
    );
    let recover_resp = app.clone().oneshot(recover_req).await.unwrap();
    assert_eq!(recover_resp.status(), StatusCode::OK);
    let body = read_json_body(recover_resp).await;
    assert_eq!(body["data"]["iteration"].as_u64().unwrap(), 1);
    assert_eq!(body["data"]["checkpointId"].as_str().unwrap(), valid.id);
}

#[tokio::test]
async fn unfinished_tasks_use_first_valid_checkpoint() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "user1", "pass123").await;

    let workspace = WorkspaceRepo::create(&db, &user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    OptimizationTaskRepo::update_status(
        &db,
        &created.task.id,
        prompt_faster::domain::models::OptimizationTaskStatus::Running,
    )
    .await
    .expect("更新状态失败");

    let valid = build_checkpoint_with(&created.task.id, 1, None, 1000);
    CheckpointRepo::create_checkpoint(&db, valid.clone())
        .await
        .expect("创建 checkpoint 失败");

    let invalid = build_checkpoint_with(&created.task.id, 2, Some("bad".to_string()), 2000);
    CheckpointRepo::create_checkpoint(&db, invalid)
        .await
        .expect("创建 checkpoint 失败");

    let req = with_bearer(
        build_empty_request("GET", "/api/v1/recovery/unfinished-tasks"),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["total"].as_u64().unwrap(), 1);
    assert_eq!(body["data"]["tasks"][0]["checkpointId"], valid.id);
}

#[tokio::test]
async fn recovery_metrics_endpoint_returns_counts() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "user1", "pass123").await;

    let workspace = WorkspaceRepo::create(&db, &user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    OptimizationTaskRepo::update_status(
        &db,
        &created.task.id,
        prompt_faster::domain::models::OptimizationTaskStatus::Running,
    )
    .await
    .expect("更新状态失败");

    let checkpoint = build_checkpoint(&created.task.id, 1);
    CheckpointRepo::create_checkpoint(&db, checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let recover_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/recovery/tasks/{}/recover", created.task.id),
            json!({}),
        ),
        &token,
    );
    let recover_resp = app.clone().oneshot(recover_req).await.unwrap();
    assert_eq!(recover_resp.status(), StatusCode::OK);

    let metrics_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/recovery/tasks/{}/metrics", created.task.id),
        ),
        &token,
    );
    let metrics_resp = app.clone().oneshot(metrics_req).await.unwrap();
    assert_eq!(metrics_resp.status(), StatusCode::OK);
    let metrics_body = read_json_body(metrics_resp).await;
    assert_eq!(metrics_body["data"]["attemptCount"].as_u64().unwrap(), 1);
    assert_eq!(metrics_body["data"]["successCount"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn recovery_supports_legacy_checkpoint_payload() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "user1", "pass123").await;

    let workspace = WorkspaceRepo::create(&db, &user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    let legacy_req = CheckpointCreateRequest {
        task_id: created.task.id.clone(),
        iteration: 1,
        state: IterationState::RunningTests,
        run_control_state: RunControlState::Running,
        prompt: "legacy".to_string(),
        rule_system: RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: std::collections::HashMap::new(),
            version: 1,
        },
        artifacts: None,
        user_guidance: None,
        branch_id: created.task.id.clone(),
        parent_id: None,
        lineage_type: LineageType::Automatic,
        branch_description: None,
    };
    let checksum = compute_checksum(&legacy_req);
    let legacy_checkpoint = CheckpointEntity {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: legacy_req.task_id.clone(),
        iteration: legacy_req.iteration,
        state: legacy_req.state,
        run_control_state: legacy_req.run_control_state,
        prompt: legacy_req.prompt,
        rule_system: legacy_req.rule_system,
        artifacts: legacy_req.artifacts,
        user_guidance: legacy_req.user_guidance,
        branch_id: legacy_req.branch_id,
        parent_id: legacy_req.parent_id,
        lineage_type: legacy_req.lineage_type,
        branch_description: legacy_req.branch_description,
        checksum,
        created_at: now_millis(),
        archived_at: None,
        archive_reason: None,
        pass_rate_summary: None,
    };
    CheckpointRepo::create_checkpoint(&db, legacy_checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let recover_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/recovery/tasks/{}/recover", created.task.id),
            json!({}),
        ),
        &token,
    );
    let recover_resp = app.clone().oneshot(recover_req).await.unwrap();
    assert_eq!(recover_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn recovery_works_after_offline_checkpoint_save() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "user1", "pass123").await;

    record_connectivity_failure(ConnectivityStatus::Offline, "offline".to_string()).await;
    assert!(matches!(
        check_connectivity_status().await,
        ConnectivityStatus::Offline
    ));

    let workspace = WorkspaceRepo::create(&db, &user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set =
        TestSetRepo::create(&db, &workspace.id, "ts", None, &[sample_case()], None, None)
            .await
            .expect("创建测试集失败");

    let created = OptimizationTaskRepo::create_scoped(
        &db,
        prompt_faster::infra::db::repositories::CreateOptimizationTaskInput {
            user_id: &user_id,
            workspace_id: &workspace.id,
            name: "task",
            description: None,
            goal: "goal",
            execution_target_type: prompt_faster::domain::models::ExecutionTargetType::Example,
            task_mode: prompt_faster::domain::models::OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
        },
    )
    .await
    .expect("创建任务失败");

    let checkpoint = build_checkpoint(&created.task.id, 1);
    CheckpointRepo::create_checkpoint(&db, checkpoint)
        .await
        .expect("离线保存 checkpoint 失败");

    record_connectivity_success().await;
    assert!(matches!(
        check_connectivity_status().await,
        ConnectivityStatus::Online
    ));

    let recover_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/recovery/tasks/{}/recover", created.task.id),
            json!({}),
        ),
        &token,
    );
    let recover_resp = app.clone().oneshot(recover_req).await.unwrap();
    assert_eq!(recover_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn rollback_forbidden_for_non_owner() {
    let (app, db) = setup_test_app_with_db().await;
    let (token_owner, owner_id) = register_user(&app, "rollback_owner", "pass123").await;
    let (token_other, _other_id) = register_user(&app, "rollback_other", "pass123").await;

    let task_id = create_task_with_test_set(&db, &owner_id).await;
    let checkpoint = build_checkpoint(&task_id, 1);
    let checkpoint_id = checkpoint.id.clone();
    CheckpointRepo::create_checkpoint(&db, checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/tasks/{}/rollback", task_id),
            json!({
                "checkpointId": checkpoint_id,
                "confirm": true
            }),
        ),
        &token_other,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let confirm_owner_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/tasks/{}/rollback", task_id),
            json!({
                "checkpointId": checkpoint_id,
                "confirm": false
            }),
        ),
        &token_owner,
    );
    let confirm_owner_resp = app.clone().oneshot(confirm_owner_req).await.unwrap();
    assert_eq!(confirm_owner_resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rollback_rejects_checksum_mismatch() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "rollback_bad_checksum", "pass123").await;
    let task_id = create_task_with_test_set(&db, &user_id).await;

    let bad_checkpoint =
        build_checkpoint_with(&task_id, 1, Some("bad_checksum".to_string()), now_millis());
    let checkpoint_id = bad_checkpoint.id.clone();
    CheckpointRepo::create_checkpoint(&db, bad_checkpoint)
        .await
        .expect("创建 checkpoint 失败");

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/tasks/{}/rollback", task_id),
            json!({
                "checkpointId": checkpoint_id,
                "confirm": true
            }),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body = read_json_body(resp).await;
    assert_eq!(
        body["error"]["message"].as_str(),
        Some("该 Checkpoint 数据已损坏，无法回滚")
    );
}

#[tokio::test]
async fn rollback_archives_following_checkpoints_and_updates_branch() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "rollback_success", "pass123").await;
    let task_id = create_task_with_test_set(&db, &user_id).await;

    let base_time = now_millis();
    let target = build_checkpoint_with(&task_id, 1, None, base_time - 1000);
    let target_id = target.id.clone();
    let later = build_checkpoint_with(&task_id, 2, None, base_time + 1000);
    let later_id = later.id.clone();

    CheckpointRepo::create_checkpoint(&db, target)
        .await
        .expect("创建 checkpoint 失败");
    CheckpointRepo::create_checkpoint(&db, later)
        .await
        .expect("创建 checkpoint 失败");

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/tasks/{}/rollback", task_id),
            json!({
                "checkpointId": target_id,
                "confirm": true
            }),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(
        body["data"]["checkpointId"].as_str(),
        Some(target_id.as_str())
    );
    assert_eq!(body["data"]["archivedCount"].as_u64(), Some(1));

    let new_branch_id = body["data"]["newBranchId"]
        .as_str()
        .expect("缺少 newBranchId")
        .to_string();
    assert!(!new_branch_id.is_empty());

    let updated_target = CheckpointRepo::get_checkpoint_by_id(&db, &target_id)
        .await
        .expect("查询 checkpoint 失败")
        .expect("checkpoint 不存在");
    assert_eq!(updated_target.branch_id, new_branch_id);

    let archived_checkpoint = CheckpointRepo::get_checkpoint_by_id(&db, &later_id)
        .await
        .expect("查询 checkpoint 失败")
        .expect("checkpoint 不存在");
    assert!(archived_checkpoint.archived_at.is_some());
    let expected_reason = format!("rollback_to_checkpoint_{}", target_id);
    assert_eq!(
        archived_checkpoint.archive_reason.as_deref(),
        Some(expected_reason.as_str())
    );
}

#[tokio::test]
async fn rollback_rejects_archived_checkpoint() {
    let (app, db) = setup_test_app_with_db().await;
    let (token, user_id) = register_user(&app, "rollback_archived", "pass123").await;
    let task_id = create_task_with_test_set(&db, &user_id).await;

    let base_time = now_millis();
    let target = build_checkpoint_with(&task_id, 1, None, base_time - 2000);
    let target_id = target.id.clone();
    let archived_target = build_checkpoint_with(&task_id, 2, None, base_time - 1000);
    let archived_id = archived_target.id.clone();
    let later = build_checkpoint_with(&task_id, 3, None, base_time + 1000);

    CheckpointRepo::create_checkpoint(&db, target)
        .await
        .expect("创建 checkpoint 失败");
    CheckpointRepo::create_checkpoint(&db, archived_target)
        .await
        .expect("创建 checkpoint 失败");
    CheckpointRepo::create_checkpoint(&db, later)
        .await
        .expect("创建 checkpoint 失败");

    CheckpointRepo::archive_checkpoints_after(&db, &task_id, &target_id, "rolled_back")
        .await
        .expect("归档 checkpoint 失败");

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/tasks/{}/rollback", task_id),
            json!({
                "checkpointId": archived_id,
                "confirm": true
            }),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body = read_json_body(resp).await;
    assert_eq!(
        body["error"]["message"].as_str(),
        Some("该 Checkpoint 已归档，无法回滚")
    );
}
