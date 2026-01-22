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
use prompt_faster::api::routes::{auth, diversity, health, user_auth, workspaces};
use prompt_faster::api::state::AppState;
use prompt_faster::domain::models::{DiversityAnalysisResult, DiversityMetrics};
use prompt_faster::domain::types::IterationArtifacts;
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;

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

    let protected_diversity_routes = diversity::router().layer(middleware::from_fn_with_state(
        session_store_for_middleware,
        auth_middleware,
    ));

    let router = Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .nest("/api/v1/workspaces", protected_workspaces_routes)
        .nest("/api/v1/tasks/{task_id}/diversity", protected_diversity_routes)
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

async fn fetch_user_id(db: &sqlx::SqlitePool, username: &str) -> String {
    let row: (String,) = sqlx::query_as("SELECT id FROM users WHERE username = ?1")
        .bind(username)
        .fetch_one(db)
        .await
        .expect("查询 user_id 失败");
    row.0
}

async fn seed_task(
    db: &sqlx::SqlitePool,
    workspace_id: &str,
    user_id: &str,
    task_id: &str,
) {
    let now = prompt_faster::shared::time::now_millis();
    sqlx::query(
        r#"
        INSERT INTO optimization_tasks
            (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, created_at, updated_at)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
    )
    .bind(task_id)
    .bind(workspace_id)
    .bind("diversity-task")
    .bind(None::<String>)
    .bind("goal")
    .bind("example")
    .bind("creative")
    .bind("running")
    .bind(None::<String>)
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("插入优化任务失败");

    sqlx::query("UPDATE workspaces SET user_id = ?1 WHERE id = ?2")
        .bind(user_id)
        .bind(workspace_id)
        .execute(db)
        .await
        .expect("更新 workspace user_id 失败");
}

async fn seed_iteration_with_diversity(
    db: &sqlx::SqlitePool,
    task_id: &str,
    round: i32,
    metrics: DiversityMetrics,
) {
    let analysis = DiversityAnalysisResult {
        metrics,
        baseline_comparison: None,
        warnings: Vec::new(),
        suggestions: Vec::new(),
        analyzed_at: "2025-01-01T00:00:00Z".to_string(),
        sample_count: 2,
    };
    let mut artifacts = IterationArtifacts::default();
    artifacts.diversity_analysis = Some(analysis);
    artifacts.updated_at = "2025-01-01T00:00:00Z".to_string();
    let artifacts_json = serde_json::to_string(&artifacts).expect("serialize artifacts");

    let evaluation_results = json!([
        {"testCaseId": "case-1", "passed": true, "score": 0.95, "failureReason": null}
    ])
    .to_string();

    sqlx::query(
        r#"
        INSERT INTO iterations (
          id, task_id, round, started_at, completed_at, status,
          artifacts, evaluation_results, reflection_summary,
          pass_rate, total_cases, passed_cases, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
    )
    .bind(format!("iter-{task_id}-{round}"))
    .bind(task_id)
    .bind(round)
    .bind(1700000000000_i64)
    .bind(1700000005000_i64)
    .bind("completed")
    .bind(artifacts_json)
    .bind(evaluation_results)
    .bind("reflection")
    .bind(0.9_f64)
    .bind(10_i32)
    .bind(9_i32)
    .bind(1700000000000_i64)
    .execute(db)
    .await
    .expect("插入 iteration 失败");
}

#[tokio::test]
async fn diversity_returns_forbidden_for_non_owner() {
    let (app, db) = setup_test_app_with_db().await;
    let token_owner = register_user(&app, "div_owner", "TestPass123!").await;
    let token_other = register_user(&app, "div_other", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token_owner).await;
    let owner_id = fetch_user_id(&db, "div_owner").await;

    let task_id = "task-diversity-403";
    seed_task(&db, &workspace_id, &owner_id, task_id).await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{task_id}/diversity")),
        &token_other,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn diversity_returns_not_found_when_analysis_missing() {
    let (app, db) = setup_test_app_with_db().await;
    let token_owner = register_user(&app, "div_owner_404", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token_owner).await;
    let owner_id = fetch_user_id(&db, "div_owner_404").await;

    let task_id = "task-diversity-404";
    seed_task(&db, &workspace_id, &owner_id, task_id).await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{task_id}/diversity")),
        &token_owner,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn diversity_returns_analysis_for_owner() {
    let (app, db) = setup_test_app_with_db().await;
    let token_owner = register_user(&app, "div_owner_ok", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token_owner).await;
    let owner_id = fetch_user_id(&db, "div_owner_ok").await;

    let task_id = "task-diversity-ok";
    seed_task(&db, &workspace_id, &owner_id, task_id).await;
    seed_iteration_with_diversity(
        &db,
        task_id,
        1,
        DiversityMetrics {
            lexical_diversity: 0.4,
            structural_diversity: 0.5,
            semantic_diversity: 0.0,
            overall_score: 0.45,
        },
    )
    .await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{task_id}/diversity")),
        &token_owner,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["metrics"]["overallScore"].as_f64(), Some(0.45));
}

#[tokio::test]
async fn diversity_baseline_upsert_updates_latest_round() {
    let (app, db) = setup_test_app_with_db().await;
    let token_owner = register_user(&app, "div_owner_baseline", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token_owner).await;
    let owner_id = fetch_user_id(&db, "div_owner_baseline").await;

    let task_id = "task-diversity-baseline";
    seed_task(&db, &workspace_id, &owner_id, task_id).await;
    seed_iteration_with_diversity(
        &db,
        task_id,
        1,
        DiversityMetrics {
            lexical_diversity: 0.2,
            structural_diversity: 0.3,
            semantic_diversity: 0.0,
            overall_score: 0.25,
        },
    )
    .await;

    let req = with_bearer(
        build_empty_request("POST", &format!("/api/v1/tasks/{task_id}/diversity/baseline")),
        &token_owner,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["iteration"].as_i64(), Some(1));

    seed_iteration_with_diversity(
        &db,
        task_id,
        2,
        DiversityMetrics {
            lexical_diversity: 0.8,
            structural_diversity: 0.7,
            semantic_diversity: 0.0,
            overall_score: 0.75,
        },
    )
    .await;

    let req = with_bearer(
        build_empty_request("POST", &format!("/api/v1/tasks/{task_id}/diversity/baseline")),
        &token_owner,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["iteration"].as_i64(), Some(2));
    assert_eq!(body["data"]["metrics"]["overallScore"].as_f64(), Some(0.75));
}
