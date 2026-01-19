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
use prompt_faster::api::routes::{auth, history, user_auth, workspaces};
use prompt_faster::api::state::AppState;
use prompt_faster::core::iteration_engine::checkpoint::compute_checksum;
use prompt_faster::domain::models::{
    CheckpointCreateRequest, CheckpointEntity, IterationState, LineageType, RuleSystem,
};
use prompt_faster::domain::types::{IterationArtifacts, RunControlState, UserGuidance};
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::db::repositories::CheckpointRepo;
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;
use prompt_faster::shared::time::now_millis;

const TEST_MASTER_PASSWORD: &str = "test_master_password_for_integration";

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
    let api_key_manager = Arc::new(ApiKeyManager::new(Some(TEST_MASTER_PASSWORD.to_string())));

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
    let protected_workspaces_routes = workspaces::router().layer(middleware::from_fn_with_state(
        session_store_for_middleware.clone(),
        auth_middleware,
    ));
    let protected_history_routes = history::router().layer(middleware::from_fn_with_state(
        session_store_for_middleware,
        auth_middleware,
    ));

    let router = Router::<AppState>::new()
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .nest("/api/v1/workspaces", protected_workspaces_routes)
        .nest("/api/v1/tasks/{task_id}/history", protected_history_routes)
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

async fn register_user(app: &Router, username: &str, password: &str) -> String {
    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );

    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    let register_json = read_json_body(register_resp).await;
    register_json["data"]["session_token"]
        .as_str()
        .expect("缺少 session_token")
        .to_string()
}

async fn create_workspace(app: &Router, token: &str) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces",
            json!({"name": "ws", "description": "desc"}),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

fn sample_exact_cases_json() -> Value {
    json!([
      {
        "id": "case-1",
        "input": { "text": "hi" },
        "reference": { "Exact": { "expected": "ok" } },
        "split": null,
        "metadata": null
      }
    ])
}

async fn create_test_set_with_cases(
    app: &Router,
    workspace_id: &str,
    token: &str,
    name: &str,
    cases: Value,
) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": name, "description": null, "cases": cases}),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

async fn create_optimization_task(
    app: &Router,
    workspace_id: &str,
    token: &str,
    test_set_id: String,
) -> String {
    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
            json!({
                "name": "task-1",
                "description": null,
                "goal": "g",
                "execution_target_type": "dify",
                "task_mode": "fixed",
                "test_set_ids": [test_set_id]
            }),
        ),
        token,
    );

    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let create_body = read_json_body(create_resp).await;
    create_body["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string()
}

fn build_rule_system() -> RuleSystem {
    RuleSystem {
        rules: Vec::new(),
        conflict_resolution_log: Vec::new(),
        merge_log: Vec::new(),
        coverage_map: std::collections::HashMap::new(),
        version: 1,
    }
}

async fn insert_checkpoint(
    db: &sqlx::SqlitePool,
    task_id: &str,
    checkpoint_id: &str,
    iteration: u32,
) -> String {
    let checkpoint = CheckpointEntity {
        id: checkpoint_id.to_string(),
        task_id: task_id.to_string(),
        iteration,
        state: IterationState::Completed,
        run_control_state: RunControlState::Running,
        prompt: "prompt".to_string(),
        rule_system: build_rule_system(),
        artifacts: Some(IterationArtifacts::empty()),
        user_guidance: Some(UserGuidance::new("guidance")),
        branch_id: "branch".to_string(),
        parent_id: None,
        lineage_type: LineageType::Automatic,
        branch_description: None,
        checksum: String::new(),
        created_at: now_millis() + iteration as i64,
        archived_at: None,
        archive_reason: None,
        pass_rate_summary: None,
    };

    let checksum = compute_checksum(&CheckpointCreateRequest {
        task_id: checkpoint.task_id.clone(),
        iteration: checkpoint.iteration,
        state: checkpoint.state,
        run_control_state: checkpoint.run_control_state,
        prompt: checkpoint.prompt.clone(),
        rule_system: checkpoint.rule_system.clone(),
        artifacts: checkpoint.artifacts.clone(),
        user_guidance: checkpoint.user_guidance.clone(),
        branch_id: checkpoint.branch_id.clone(),
        parent_id: checkpoint.parent_id.clone(),
        lineage_type: checkpoint.lineage_type.clone(),
        branch_description: checkpoint.branch_description.clone(),
    });
    let saved = CheckpointRepo::create_checkpoint(
        db,
        CheckpointEntity {
            checksum,
            ..checkpoint
        },
    )
    .await
    .expect("插入 checkpoint 失败");
    saved.id
}

async fn insert_iteration(db: &sqlx::SqlitePool, task_id: &str, round: i32) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_millis();
    sqlx::query(
        r#"
        INSERT INTO iterations (
            id, task_id, round, started_at, completed_at, status,
            artifacts, evaluation_results, reflection_summary,
            pass_rate, total_cases, passed_cases, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(task_id)
    .bind(round)
    .bind(now - 1000)
    .bind(Some(now))
    .bind("completed")
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(0.8_f64)
    .bind(10_i32)
    .bind(8_i32)
    .bind(now)
    .execute(db)
    .await
    .expect("插入 iteration 失败");
    id
}

#[tokio::test]
async fn test_history_combines_iterations_and_checkpoints() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "history_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, test_set_id).await;

    let _iteration_id = insert_iteration(&db, &task_id, 1).await;
    let checkpoint_id = insert_checkpoint(&db, &task_id, "checkpoint-1", 1).await;
    let _ = insert_checkpoint(&db, &task_id, "checkpoint-2", 2).await;
    CheckpointRepo::archive_checkpoints_after(&db, &task_id, &checkpoint_id, "rolled_back")
        .await
        .expect("归档 checkpoint 失败");

    let req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/tasks/{}/history?include_archived=true", task_id),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    let iterations = body["data"]["iterations"].as_array().unwrap();
    assert_eq!(iterations.len(), 1);
    assert_eq!(iterations[0]["round"].as_i64(), Some(1));

    let checkpoints = &body["data"]["checkpoints"];
    assert_eq!(checkpoints["total"].as_u64(), Some(2));
    let items = checkpoints["checkpoints"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(checkpoints["currentBranchId"].as_str(), Some("branch"));
    let archived = items
        .iter()
        .find(|item| item["archivedAt"].is_string())
        .expect("缺少归档 checkpoint");
    assert_eq!(archived["archiveReason"].as_str(), Some("rolled_back"));
}

#[tokio::test]
async fn test_history_forbidden_for_other_user() {
    let (app, db) = setup_test_app_with_db().await;
    let token_owner = register_user(&app, "history_owner", "TestPass123!").await;
    let token_other = register_user(&app, "history_other", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token_owner).await;
    let test_set_id = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token_owner,
        "ts",
        sample_exact_cases_json(),
    )
    .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token_owner, test_set_id).await;
    let _ = insert_iteration(&db, &task_id, 1).await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{}/history", task_id)),
        &token_other,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
