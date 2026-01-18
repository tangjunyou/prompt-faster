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

async fn create_workspace(app: &Router, token: &str, name: &str) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces",
            json!({"name": name, "description": "desc"}),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

fn sample_cases_json() -> Value {
    json!([
      {
        "id": "case-1",
        "input": { "text": "hi" },
        "reference": { "Exact": { "expected": "ok" } },
        "split": "train",
        "metadata": null
      }
    ])
}

async fn create_test_set(app: &Router, token: &str, workspace_id: &str, name: &str) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": name, "description": "d", "cases": sample_cases_json()}),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

async fn create_test_set_with_description(
    app: &Router,
    token: &str,
    workspace_id: &str,
    name: &str,
    description: &str,
) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": name, "description": description, "cases": sample_cases_json()}),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().expect("缺少 id").to_string()
}

#[tokio::test]
async fn test_save_as_template_list_get_happy_path() {
    let app = setup_test_app().await;
    let token = register_user(&app, "tpl_user_a", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token, "ws-a").await;
    let test_set_id = create_test_set(&app, &token, &workspace_id, "ts1").await;

    // Save as template
    let save_req = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_id, test_set_id
            ),
            json!({"name": "tpl1", "description": null}),
        ),
        &token,
    );
    let save_resp = app.clone().oneshot(save_req).await.unwrap();
    assert_eq!(save_resp.status(), StatusCode::OK);
    let save_body = read_json_body(save_resp).await;
    let template_id = save_body["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    // List templates
    let list_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/workspaces/{}/test-set-templates",
                workspace_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = read_json_body(list_resp).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 1);
    assert_eq!(list_body["data"][0]["cases_count"], 1);

    // Get template detail
    let get_req = with_bearer(
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
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = read_json_body(get_resp).await;
    assert_eq!(get_body["data"]["id"], template_id);
    assert_eq!(get_body["data"]["cases"].as_array().unwrap().len(), 1);

    // Ensure templates do not appear in test-sets list
    let list_test_sets_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/workspaces/{}/test-sets", workspace_id))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let list_test_sets_resp = app.clone().oneshot(list_test_sets_req).await.unwrap();
    assert_eq!(list_test_sets_resp.status(), StatusCode::OK);
    let list_test_sets_body = read_json_body(list_test_sets_resp).await;
    assert_eq!(list_test_sets_body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_save_as_template_name_validation_returns_400() {
    let app = setup_test_app().await;
    let token = register_user(&app, "tpl_validate_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token, "ws-validate").await;
    let test_set_id = create_test_set(&app, &token, &workspace_id, "ts1").await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_id, test_set_id
            ),
            json!({"name": "   ", "description": null}),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_cross_user_workspace_returns_workspace_not_found() {
    let app = setup_test_app().await;
    let token_a = register_user(&app, "tpl_user_a2", "TestPass123!").await;
    let token_b = register_user(&app, "tpl_user_b2", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a, "ws-cross").await;

    // B tries to list templates in A's workspace
    let list_req_b = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/workspaces/{}/test-set-templates",
                workspace_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token_b,
    );
    let list_resp_b = app.clone().oneshot(list_req_b).await.unwrap();
    assert_eq!(list_resp_b.status(), StatusCode::NOT_FOUND);
    let list_body_b = read_json_body(list_resp_b).await;
    assert_eq!(list_body_b["error"]["code"], "WORKSPACE_NOT_FOUND");

    // B tries to save-as-template in A's workspace
    let test_set_id = create_test_set(&app, &token_a, &workspace_id, "ts1").await;
    let save_req_b = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_id, test_set_id
            ),
            json!({"name": "tpl1", "description": null}),
        ),
        &token_b,
    );
    let save_resp_b = app.clone().oneshot(save_req_b).await.unwrap();
    assert_eq!(save_resp_b.status(), StatusCode::NOT_FOUND);
    let save_body_b = read_json_body(save_resp_b).await;
    assert_eq!(save_body_b["error"]["code"], "WORKSPACE_NOT_FOUND");
}

#[tokio::test]
async fn test_cross_workspace_template_get_returns_template_not_found() {
    let app = setup_test_app().await;
    let token = register_user(&app, "tpl_ws_user", "TestPass123!").await;
    let workspace_a = create_workspace(&app, &token, "ws-a").await;
    let workspace_b = create_workspace(&app, &token, "ws-b").await;
    let test_set_id = create_test_set(&app, &token, &workspace_a, "ts1").await;

    let save_req = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_a, test_set_id
            ),
            json!({"name": "tpl1", "description": null}),
        ),
        &token,
    );
    let save_resp = app.clone().oneshot(save_req).await.unwrap();
    assert_eq!(save_resp.status(), StatusCode::OK);
    let save_body = read_json_body(save_resp).await;
    let template_id = save_body["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    let get_req_wrong_ws = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/workspaces/{}/test-set-templates/{}",
                workspace_b, template_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let resp = app.clone().oneshot(get_req_wrong_ws).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "TEST_SET_NOT_FOUND");
}

#[tokio::test]
async fn test_save_as_template_cross_workspace_returns_test_set_not_found() {
    let app = setup_test_app().await;
    let token = register_user(&app, "tpl_cross_ws_save_user", "TestPass123!").await;
    let workspace_a = create_workspace(&app, &token, "ws-a").await;
    let workspace_b = create_workspace(&app, &token, "ws-b").await;
    let test_set_id = create_test_set(&app, &token, &workspace_a, "ts1").await;

    let save_req_wrong_ws = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_b, test_set_id
            ),
            json!({"name": "tpl1", "description": null}),
        ),
        &token,
    );
    let resp = app.clone().oneshot(save_req_wrong_ws).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "TEST_SET_NOT_FOUND");
}

#[tokio::test]
async fn test_save_as_template_null_description_does_not_fallback_to_source_description() {
    let app = setup_test_app().await;
    let token = register_user(&app, "tpl_desc_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token, "ws-desc").await;
    let test_set_id =
        create_test_set_with_description(&app, &token, &workspace_id, "ts1", "source description")
            .await;

    let save_req = with_bearer(
        build_json_request(
            "POST",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/save-as-template",
                workspace_id, test_set_id
            ),
            json!({"name": "tpl1", "description": null}),
        ),
        &token,
    );
    let resp = app.clone().oneshot(save_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["description"], Value::Null);
}
