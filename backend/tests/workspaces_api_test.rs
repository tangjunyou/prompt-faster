use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

use prompt_faster::api::middleware::correlation_id::correlation_id_middleware;
use prompt_faster::api::middleware::{LoginAttemptStore, SessionStore, auth_middleware};
use prompt_faster::api::routes::{auth, health, user_auth, workspaces};
use prompt_faster::api::state::AppState;
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;

const TEST_MASTER_PASSWORD: &str = "test_master_password_for_integration";

async fn setup_test_app() -> Router {
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
        db,
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
        session_store_for_middleware,
        auth_middleware,
    ));

    Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .nest("/api/v1/workspaces", protected_workspaces_routes)
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware))
}

async fn setup_test_app_with_db() -> (Router, SqlitePool) {
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
        session_store_for_middleware,
        auth_middleware,
    ));

    let app = Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .nest("/api/v1/workspaces", protected_workspaces_routes)
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware));

    (app, db)
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
        .expect("构建请求失败")
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

async fn create_workspace(app: &Router, token: &str, name: &str) -> String {
    let create_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces",
            json!({"name": name, "description": null}),
        ),
        token,
    );

    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);

    let create_json = read_json_body(create_resp).await;
    create_json["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string()
}

async fn create_test_set(app: &Router, token: &str, workspace_id: &str, name: &str) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": name, "description": null, "cases": []}),
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
    token: &str,
    workspace_id: &str,
    name: &str,
    test_set_id: &str,
) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
            json!({
              "name": name,
              "description": null,
              "goal": "g",
              "execution_target_type": "dify",
              "task_mode": "fixed",
              "test_set_ids": [test_set_id]
            }),
        ),
        token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

#[tokio::test]
async fn test_user_b_cannot_access_user_a_workspace_by_id() {
    let app = setup_test_app().await;

    let token_a = register_user(&app, "test_workspace_user_a", "TestPass123!").await;
    let token_b = register_user(&app, "test_workspace_user_b", "TestPass123!").await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces",
            json!({"name": "A Workspace", "description": "desc"}),
        ),
        &token_a,
    );

    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);

    let create_json = read_json_body(create_resp).await;
    let workspace_id = create_json["data"]["id"].as_str().expect("缺少 id");

    let get_req_b = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/workspaces/{}", workspace_id))
            .body(Body::empty())
            .unwrap(),
        &token_b,
    );

    let get_resp_b = app.clone().oneshot(get_req_b).await.unwrap();
    assert_eq!(get_resp_b.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_user_a_workspace_list_does_not_include_user_b_data() {
    let app = setup_test_app().await;

    let token_a = register_user(&app, "test_workspace_list_user_a", "TestPass123!").await;
    let token_b = register_user(&app, "test_workspace_list_user_b", "TestPass123!").await;

    let create_req_a = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces",
            json!({"name": "A Workspace", "description": "desc"}),
        ),
        &token_a,
    );
    let create_resp_a = app.clone().oneshot(create_req_a).await.unwrap();
    assert_eq!(create_resp_a.status(), StatusCode::OK);
    let create_json_a = read_json_body(create_resp_a).await;
    let workspace_id_a = create_json_a["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    let create_req_b = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces",
            json!({"name": "B Workspace", "description": "desc"}),
        ),
        &token_b,
    );
    let create_resp_b = app.clone().oneshot(create_req_b).await.unwrap();
    assert_eq!(create_resp_b.status(), StatusCode::OK);
    let create_json_b = read_json_body(create_resp_b).await;
    let workspace_id_b = create_json_b["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    let list_req_a = with_bearer(
        Request::builder()
            .method("GET")
            .uri("/api/v1/workspaces")
            .body(Body::empty())
            .unwrap(),
        &token_a,
    );

    let list_resp_a = app.clone().oneshot(list_req_a).await.unwrap();
    assert_eq!(list_resp_a.status(), StatusCode::OK);

    let list_json_a = read_json_body(list_resp_a).await;
    let list = list_json_a["data"].as_array().expect("data 应为数组");

    assert!(
        list.iter()
            .any(|w| w["id"].as_str() == Some(workspace_id_a.as_str()))
    );
    assert!(
        !list
            .iter()
            .any(|w| w["id"].as_str() == Some(workspace_id_b.as_str()))
    );
}

#[tokio::test]
async fn test_delete_workspace_requires_ownership_returns_404() {
    let app = setup_test_app().await;

    let token_a = register_user(&app, "test_delete_ws_user_a", "TestPass123!").await;
    let token_b = register_user(&app, "test_delete_ws_user_b", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token_a, "A Workspace").await;

    let delete_req_b = with_bearer(
        build_empty_request("DELETE", &format!("/api/v1/workspaces/{}", workspace_id)),
        &token_b,
    );
    let delete_resp_b = app.clone().oneshot(delete_req_b).await.unwrap();
    assert_eq!(delete_resp_b.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_workspace_cascades_to_test_sets_and_optimization_tasks() {
    let (app, db) = setup_test_app_with_db().await;

    let token = register_user(&app, "test_delete_ws_cascade_user", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token, "Cascade Workspace").await;
    let test_set_id = create_test_set(&app, &token, &workspace_id, "ts").await;
    let _task_id =
        create_optimization_task(&app, &token, &workspace_id, "task", &test_set_id).await;

    let delete_req = with_bearer(
        build_empty_request("DELETE", &format!("/api/v1/workspaces/{}", workspace_id)),
        &token,
    );
    let delete_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_resp.status(), StatusCode::OK);

    let get_ws_req = with_bearer(
        build_empty_request("GET", &format!("/api/v1/workspaces/{}", workspace_id)),
        &token,
    );
    let get_ws_resp = app.clone().oneshot(get_ws_req).await.unwrap();
    assert_eq!(get_ws_resp.status(), StatusCode::NOT_FOUND);

    let test_sets_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM test_sets WHERE workspace_id = ?1")
            .bind(&workspace_id)
            .fetch_one(&db)
            .await
            .expect("查询 test_sets 失败");
    assert_eq!(test_sets_count, 0);

    let tasks_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM optimization_tasks WHERE workspace_id = ?1")
            .bind(&workspace_id)
            .fetch_one(&db)
            .await
            .expect("查询 optimization_tasks 失败");
    assert_eq!(tasks_count, 0);

    let otts_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM optimization_task_test_sets")
        .fetch_one(&db)
        .await
        .expect("查询 optimization_task_test_sets 失败");
    assert_eq!(otts_count, 0);

    let list_test_sets_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
        ),
        &token,
    );
    let list_test_sets_resp = app.clone().oneshot(list_test_sets_req).await.unwrap();
    assert_eq!(list_test_sets_resp.status(), StatusCode::NOT_FOUND);

    let list_tasks_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
        ),
        &token,
    );
    let list_tasks_resp = app.clone().oneshot(list_tasks_req).await.unwrap();
    assert_eq!(list_tasks_resp.status(), StatusCode::NOT_FOUND);

    let list_templates_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/test-set-templates", workspace_id),
        ),
        &token,
    );
    let list_templates_resp = app.clone().oneshot(list_templates_req).await.unwrap();
    assert_eq!(list_templates_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_workspace_scoped_data_isolated_between_workspaces_for_same_user() {
    let app = setup_test_app().await;

    let token = register_user(&app, "test_ws_isolation_same_user", "TestPass123!").await;

    let workspace_a = create_workspace(&app, &token, "Workspace A").await;
    let workspace_b = create_workspace(&app, &token, "Workspace B").await;

    let test_set_id_a = create_test_set(&app, &token, &workspace_a, "ts-a").await;
    let _task_id_a =
        create_optimization_task(&app, &token, &workspace_a, "task-a", &test_set_id_a).await;

    let list_test_sets_b_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_b),
        ),
        &token,
    );
    let list_test_sets_b_resp = app.clone().oneshot(list_test_sets_b_req).await.unwrap();
    assert_eq!(list_test_sets_b_resp.status(), StatusCode::OK);
    let json_b = read_json_body(list_test_sets_b_resp).await;
    assert_eq!(json_b["data"].as_array().unwrap().len(), 0);

    let list_tasks_b_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_b),
        ),
        &token,
    );
    let list_tasks_b_resp = app.clone().oneshot(list_tasks_b_req).await.unwrap();
    assert_eq!(list_tasks_b_resp.status(), StatusCode::OK);
    let json_tasks_b = read_json_body(list_tasks_b_resp).await;
    assert_eq!(json_tasks_b["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_delete_workspace_cascades_to_optimization_task_test_sets() {
    let (app, db) = setup_test_app_with_db().await;

    let token = register_user(&app, "test_delete_ws_cascade_join", "TestPass123!").await;

    let workspace_id = create_workspace(&app, &token, "Cascade Workspace").await;
    let test_set_id = create_test_set(&app, &token, &workspace_id, "ts").await;
    let _task_id =
        create_optimization_task(&app, &token, &workspace_id, "task", &test_set_id).await;

    let delete_req = with_bearer(
        build_empty_request("DELETE", &format!("/api/v1/workspaces/{}", workspace_id)),
        &token,
    );
    let delete_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_resp.status(), StatusCode::OK);

    let otts_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM optimization_task_test_sets")
        .fetch_one(&db)
        .await
        .expect("查询 optimization_task_test_sets 失败");
    assert_eq!(otts_count, 0);
}
