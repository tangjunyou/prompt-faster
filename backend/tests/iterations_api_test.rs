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
use prompt_faster::api::routes::{auth, health, iterations, user_auth, workspaces};
use prompt_faster::api::state::AppState;
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

    let protected_iterations_routes = iterations::router().layer(middleware::from_fn_with_state(
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
        .nest(
            "/api/v1/tasks/{task_id}/iterations",
            protected_iterations_routes,
        )
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
            json!({ "name": name, "description": null, "cases": cases }),
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

async fn insert_iteration(db: &sqlx::SqlitePool, task_id: &str, iteration_id: &str, round: i32) {
    let artifacts = json!({
        "patterns": [
            {"id": "p1", "pattern": "pattern", "source": "system", "confidence": 0.8}
        ],
        "candidatePrompts": [
            {"id": "c1", "content": "prompt", "source": "system", "isBest": true, "score": 0.9}
        ],
        "userGuidance": null,
        "updatedAt": "2025-01-01T12:00:00Z"
    })
    .to_string();

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
    .bind(iteration_id)
    .bind(task_id)
    .bind(round)
    .bind(1700000000000_i64)
    .bind(1700000005000_i64)
    .bind("completed")
    .bind(artifacts)
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
async fn test_list_empty_returns_empty_array() {
    let (app, _db) = setup_test_app_with_db().await;
    let token = register_user(&app, "iter_list_empty", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, test_set_id).await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{}/iterations", task_id)),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert!(body["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_forbidden_for_other_user() {
    let (app, _db) = setup_test_app_with_db().await;
    let token_a = register_user(&app, "iter_list_a", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;
    let test_set_id = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token_a,
        "ts",
        sample_exact_cases_json(),
    )
    .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token_a, test_set_id).await;

    let token_b = register_user(&app, "iter_list_b", "TestPass123!").await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{}/iterations", task_id)),
        &token_b,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_detail_not_found_returns_404() {
    let (app, _db) = setup_test_app_with_db().await;
    let token = register_user(&app, "iter_detail_missing", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, test_set_id).await;

    let req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/tasks/{}/iterations/iter-missing", task_id),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_returns_round_desc() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "iter_list_desc", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, test_set_id).await;

    insert_iteration(&db, &task_id, "iter-1", 1).await;
    insert_iteration(&db, &task_id, "iter-2", 2).await;

    let req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/tasks/{}/iterations", task_id)),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    let data = body["data"].as_array().expect("data 不是数组");
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["round"], 2);
    assert_eq!(data[1]["round"], 1);
}
