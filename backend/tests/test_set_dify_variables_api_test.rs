use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&body).expect("序列化 JSON 失败"),
        ))
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

async fn create_test_set(app: &Router, token: &str, workspace_id: &str) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({
                "name": "ts",
                "description": null,
                "cases": [
                    {
                        "id": "case-1",
                        "input": { "foo": "bar" },
                        "reference": { "Exact": { "expected": "ok" } },
                        "split": "train",
                        "metadata": null
                    }
                ]
            }),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

async fn save_auth_config(app: &Router, token: &str, dify_base_url: &str, dify_key: &str) {
    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/auth/config",
            json!({
                "dify": { "base_url": dify_base_url, "api_key": dify_key },
                "generic_llm": { "provider": "siliconflow", "base_url": "https://api.example.com", "api_key": "sk-llm-test" },
                "teacher_settings": { "temperature": 0.7, "top_p": 0.9, "max_tokens": 256 }
            }),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_refresh_dify_variables_happy_path() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/parameters"))
        .and(header("Authorization", "Bearer sk-dify-test"))
        .and(header("x-correlation-id", "test-correlation-id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "user_input_form": [
                { "text-input": { "label": "System", "variable": "system_prompt", "required": true, "default": "hi" } },
                { "number": { "label": "K", "variable": "k", "required": false, "default": 3 } },
                { "checkbox": { "label": "Flag", "variable": "flag", "required": true } }
            ]
        })))
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;
    let token = register_user(&app, "dify_vars_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id = create_test_set(&app, &token, &workspace_id).await;
    save_auth_config(&app, &token, &mock_server.uri(), "sk-dify-test").await;

    let mut req = with_bearer(
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/variables/refresh",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    req.headers_mut()
        .insert("x-correlation-id", "test-correlation-id".parse().unwrap());

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["variables"].as_array().unwrap().len(), 3);
    assert_eq!(body["data"]["variables"][0]["name"], "system_prompt");
    assert_eq!(body["data"]["variables"][0]["component"], "text-input");
}

#[tokio::test]
async fn test_refresh_dify_variables_upstream_401_is_mapped_to_502_and_sanitized() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/parameters"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code": "unauthorized",
            "message": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;
    let token = register_user(&app, "dify_vars_user2", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id = create_test_set(&app, &token, &workspace_id).await;
    save_auth_config(&app, &token, &mock_server.uri(), "sk-dify-test").await;

    let req = with_bearer(
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/variables/refresh",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "UPSTREAM_ERROR");
    assert!(body["error"]["message"].as_str().unwrap().contains("Dify"));
}

#[tokio::test]
async fn test_refresh_dify_variables_upstream_403_is_mapped_to_502_and_friendy_message() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/parameters"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "code": "forbidden",
            "message": "Forbidden"
        })))
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;
    let token = register_user(&app, "dify_vars_user_403", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id = create_test_set(&app, &token, &workspace_id).await;
    save_auth_config(&app, &token, &mock_server.uri(), "sk-dify-test").await;

    let req = with_bearer(
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/variables/refresh",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "UPSTREAM_ERROR");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("拒绝访问")
    );
}

#[tokio::test]
async fn test_refresh_dify_variables_parse_error_is_mapped_to_502_and_friendy_message() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/parameters"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{not-json"))
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;
    let token = register_user(&app, "dify_vars_user_parse", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id = create_test_set(&app, &token, &workspace_id).await;
    save_auth_config(&app, &token, &mock_server.uri(), "sk-dify-test").await;

    let req = with_bearer(
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/variables/refresh",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "UPSTREAM_ERROR");
    assert!(body["error"]["message"].as_str().unwrap().contains("解析"));
}

#[tokio::test]
async fn test_save_dify_config_persists_and_can_be_read_back() {
    let app = setup_test_app().await;
    let token = register_user(&app, "dify_cfg_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id = create_test_set(&app, &token, &workspace_id).await;

    let save_req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/config",
                workspace_id, test_set_id
            ),
            json!({
                "targetPromptVariable": "system_prompt",
                "bindings": {
                    "k": { "source": "fixed", "value": 3 },
                    "flag": { "source": "testCaseInput", "inputKey": "flag" }
                }
            }),
        ),
        &token,
    );

    let save_resp = app.clone().oneshot(save_req).await.unwrap();
    assert_eq!(save_resp.status(), StatusCode::OK);
    let save_body = read_json_body(save_resp).await;
    assert_eq!(
        save_body["data"]["difyConfig"]["targetPromptVariable"],
        "system_prompt"
    );

    let get_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = read_json_body(get_resp).await;
    assert_eq!(
        get_body["data"]["dify_config"]["targetPromptVariable"],
        "system_prompt"
    );
}

#[tokio::test]
async fn test_dify_config_is_copied_to_template() {
    let app = setup_test_app().await;
    let token = register_user(&app, "dify_tpl_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id = create_test_set(&app, &token, &workspace_id).await;

    let save_cfg_req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/config",
                workspace_id, test_set_id
            ),
            json!({
                "targetPromptVariable": "system_prompt",
                "bindings": {
                    "k": { "source": "fixed", "value": 3 }
                }
            }),
        ),
        &token,
    );
    let save_cfg_resp = app.clone().oneshot(save_cfg_req).await.unwrap();
    assert_eq!(save_cfg_resp.status(), StatusCode::OK);

    let save_tpl_req = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_id, test_set_id
            ),
            json!({ "name": "tpl", "description": null }),
        ),
        &token,
    );
    let save_tpl_resp = app.clone().oneshot(save_tpl_req).await.unwrap();
    assert_eq!(save_tpl_resp.status(), StatusCode::OK);
    let save_tpl_body = read_json_body(save_tpl_resp).await;
    let template_id = save_tpl_body["data"]["id"].as_str().unwrap().to_string();

    let get_tpl_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/workspaces/{}/test-set-templates/{}",
                workspace_id, template_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let get_tpl_resp = app.clone().oneshot(get_tpl_req).await.unwrap();
    assert_eq!(get_tpl_resp.status(), StatusCode::OK);
    let get_tpl_body = read_json_body(get_tpl_resp).await;
    assert_eq!(
        get_tpl_body["data"]["dify_config"]["targetPromptVariable"],
        "system_prompt"
    );
}

#[tokio::test]
async fn test_cross_user_refresh_returns_404() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/parameters"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "user_input_form": [] })),
        )
        .mount(&mock_server)
        .await;

    let app = setup_test_app().await;
    let token_a = register_user(&app, "dify_cross_a", "TestPass123!").await;
    let token_b = register_user(&app, "dify_cross_b", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;
    let test_set_id = create_test_set(&app, &token_a, &workspace_id).await;
    save_auth_config(&app, &token_a, &mock_server.uri(), "sk-dify-test").await;

    let req = with_bearer(
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/variables/refresh",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token_b,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "WORKSPACE_NOT_FOUND");
}
