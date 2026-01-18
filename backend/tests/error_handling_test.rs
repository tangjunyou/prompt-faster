//! 错误处理集成测试 (Story 1.8 Task 7)
//!
//! 测试目标:
//! 1. 验证所有错误响应符合统一结构 { error: { code, message, details? } }
//! 2. 验证错误码符合 error_codes 规范
//! 3. 验证 message 字段可读
//! 4. 验证 details 仅在开发环境出现
//! 5. 验证 HTTP 状态码映射正确

use axum::Router;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use axum::middleware;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::net::SocketAddr;
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
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

        checkpoint_cache_limit: 10,

        checkpoint_memory_alert_threshold: 10,
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
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&body).expect("序列化 JSON 失败"),
        ))
        .expect("构建请求失败");

    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 3001))));

    request
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

#[tokio::test]
async fn test_generic_llm_connection_error_does_not_leak_upstream_body() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(500).set_body_string("TOPSECRET_DO_NOT_ECHO"))
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/test-connection/generic-llm",
            json!({
              "base_url": mock_server.uri(),
              "api_key": "sk-test-123",
              "provider": "siliconflow"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let body = read_json_body(response).await;
    let raw = body.to_string();
    assert!(
        !raw.contains("TOPSECRET_DO_NOT_ECHO"),
        "response leaked upstream body"
    );
}

#[tokio::test]
async fn test_validation_error_empty_username() {
    let app = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/register",
            json!({"username": "", "password": "password123"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = read_json_body(response).await;
    assert_error_response_structure(&body, "VALIDATION_ERROR");

    let message = body["error"]["message"].as_str().unwrap_or("");
    assert!(message.contains("用户名"));
}

#[tokio::test]
async fn test_auth_invalid_credentials() {
    let app = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/login",
            json!({"username": "nonexistent_user", "password": "wrong_password"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = read_json_body(response).await;
    assert_error_response_structure(&body, "AUTH_FAILED");
    assert_eq!(body["error"]["message"].as_str(), Some("用户名或密码错误"));
}

#[tokio::test]
async fn test_resource_not_found_workspace() {
    let app = setup_test_app().await;
    let token = register_user(&app, "test_error_user", "password123").await;

    let create_response = app
        .clone()
        .oneshot(with_bearer(
            build_json_request(
                "POST",
                "/api/v1/workspaces",
                json!({"name": "Test Workspace"}),
            ),
            &token,
        ))
        .await
        .unwrap();

    let workspace_id = read_json_body(create_response).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let _delete1 = app
        .clone()
        .oneshot(with_bearer(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/workspaces/{}", workspace_id))
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(with_bearer(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/workspaces/{}", workspace_id))
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = read_json_body(response).await;
    assert_error_response_structure(&body, "WORKSPACE_NOT_FOUND");
}

#[tokio::test]
async fn test_unauthorized_access() {
    let app = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = read_json_body(response).await;
    assert_error_response_structure(&body, "UNAUTHORIZED");
}

#[tokio::test]
async fn test_error_response_structure() {
    let app = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/register",
            json!({"username": "", "password": "123"}),
        ))
        .await
        .unwrap();

    let body = read_json_body(response).await;
    assert!(body.is_object());
    assert!(body.get("error").is_some());

    let error = body["error"].as_object().expect("Error must be an object");
    assert!(error.contains_key("code"));
    assert!(error.contains_key("message"));

    assert!(error.get("code").unwrap().is_string());
    assert!(error.get("message").unwrap().is_string());
}

#[tokio::test]
async fn test_username_conflict() {
    let app = setup_test_app().await;

    let _ = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/register",
            json!({"username": "conflict_test_user", "password": "password123"}),
        ))
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/register",
            json!({"username": "conflict_test_user", "password": "password456"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = read_json_body(response).await;
    assert_error_response_structure(&body, "USERNAME_CONFLICT");
}

#[tokio::test]
async fn test_http_status_code_mapping() {
    let app = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/register",
            json!({"username": "", "password": "123"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(build_json_request(
            "POST",
            "/api/v1/auth/login",
            json!({"username": "wrong", "password": "wrong"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let token = register_user(&app, "test_user_status_map", "password123").await;
    let response = app
        .clone()
        .oneshot(with_bearer(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/workspaces/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

fn assert_error_response_structure(body: &Value, expected_code: &str) {
    assert!(body.is_object(), "Response must be an object");
    assert!(
        body.get("error").is_some(),
        "Response must contain 'error' field"
    );

    let error_map = body["error"].as_object().expect("Error must be an object");

    assert!(
        error_map.contains_key("code"),
        "Error must contain 'code' field"
    );
    let code = error_map.get("code").expect("Code field must exist");
    assert!(code.is_string(), "Code must be a string");
    assert_eq!(
        code.as_str().expect("Code must be a string"),
        expected_code,
        "Error code must match expected value"
    );

    assert!(
        error_map.contains_key("message"),
        "Error must contain 'message' field"
    );
    let message = error_map.get("message").expect("Message field must exist");
    assert!(message.is_string(), "Message must be a string");
    assert!(
        !message
            .as_str()
            .expect("Message must be a string")
            .is_empty(),
        "Message should not be empty"
    );
}
