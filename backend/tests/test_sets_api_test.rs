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

#[tokio::test]
async fn test_unauthorized_access_returns_401() {
    let app = setup_test_app().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/w1/test-sets")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_in_missing_workspace_returns_workspace_not_found() {
    let app = setup_test_app().await;
    let token = register_user(&app, "ts_user_a", "TestPass123!").await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/workspaces/nonexistent/test-sets",
            json!({"name": "ts", "description": null, "cases": sample_cases_json()}),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "WORKSPACE_NOT_FOUND");
}

#[tokio::test]
async fn test_crud_happy_path() {
    let app = setup_test_app().await;
    let token = register_user(&app, "ts_crud_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    // Create
    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "ts1", "description": "d", "cases": sample_cases_json()}),
        ),
        &token,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let create_body = read_json_body(create_resp).await;
    let test_set_id = create_body["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    // List
    let list_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/workspaces/{}/test-sets", workspace_id))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = read_json_body(list_resp).await;
    assert!(list_body["data"].as_array().unwrap().len() >= 1);
    assert_eq!(list_body["data"][0]["cases_count"], 1);

    // Get
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

    // Update
    let update_req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ),
            json!({"name": "ts1-updated", "description": null, "cases": sample_cases_json()}),
        ),
        &token,
    );
    let update_resp = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);
    let update_body = read_json_body(update_resp).await;
    assert_eq!(update_body["data"]["name"], "ts1-updated");

    // Delete
    let delete_req = with_bearer(
        Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token,
    );
    let delete_resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(delete_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cross_user_get_returns_test_set_not_found() {
    let app = setup_test_app().await;
    let token_a = register_user(&app, "ts_user_a2", "TestPass123!").await;
    let token_b = register_user(&app, "ts_user_b2", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "ts1", "description": null, "cases": sample_cases_json()}),
        ),
        &token_a,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let body = read_json_body(create_resp).await;
    let test_set_id = body["data"]["id"].as_str().unwrap().to_string();

    let get_req_b = with_bearer(
        Request::builder()
            .method("GET")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token_b,
    );
    let resp_b = app.clone().oneshot(get_req_b).await.unwrap();
    assert_eq!(resp_b.status(), StatusCode::NOT_FOUND);
    let resp_body = read_json_body(resp_b).await;
    assert_eq!(resp_body["error"]["code"], "TEST_SET_NOT_FOUND");
}

#[tokio::test]
async fn test_cross_user_update_returns_test_set_not_found() {
    let app = setup_test_app().await;
    let token_a = register_user(&app, "ts_user_a_update", "TestPass123!").await;
    let token_b = register_user(&app, "ts_user_b_update", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "ts1", "description": null, "cases": sample_cases_json()}),
        ),
        &token_a,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let body = read_json_body(create_resp).await;
    let test_set_id = body["data"]["id"].as_str().unwrap().to_string();

    let update_req_b = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ),
            json!({"name": "ts1-updated", "description": null, "cases": sample_cases_json()}),
        ),
        &token_b,
    );
    let resp_b = app.clone().oneshot(update_req_b).await.unwrap();
    assert_eq!(resp_b.status(), StatusCode::NOT_FOUND);
    let resp_body = read_json_body(resp_b).await;
    assert_eq!(resp_body["error"]["code"], "TEST_SET_NOT_FOUND");
}

#[tokio::test]
async fn test_update_test_set_preserves_dify_config() {
    let app = setup_test_app().await;
    let token = register_user(&app, "ts_update_keep_cfg", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    // Create test set
    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "ts1", "description": null, "cases": sample_cases_json()}),
        ),
        &token,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let create_body = read_json_body(create_resp).await;
    let test_set_id = create_body["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    // Save dify config
    let save_cfg_req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}/dify/config",
                workspace_id, test_set_id
            ),
            json!({
                "targetPromptVariable": "system_prompt",
                "bindings": { "k": { "source": "fixed", "value": 3 } }
            }),
        ),
        &token,
    );
    let save_cfg_resp = app.clone().oneshot(save_cfg_req).await.unwrap();
    assert_eq!(save_cfg_resp.status(), StatusCode::OK);

    // Update test set
    let update_req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ),
            json!({"name": "ts1-updated", "description": null, "cases": sample_cases_json()}),
        ),
        &token,
    );
    let update_resp = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);
    let update_body = read_json_body(update_resp).await;
    assert_eq!(update_body["data"]["name"], "ts1-updated");
    assert_eq!(
        update_body["data"]["dify_config"]["targetPromptVariable"],
        "system_prompt"
    );

    // Get test set again
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
async fn test_cross_user_delete_returns_test_set_not_found() {
    let app = setup_test_app().await;
    let token_a = register_user(&app, "ts_user_a_delete", "TestPass123!").await;
    let token_b = register_user(&app, "ts_user_b_delete", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "ts1", "description": null, "cases": sample_cases_json()}),
        ),
        &token_a,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let body = read_json_body(create_resp).await;
    let test_set_id = body["data"]["id"].as_str().unwrap().to_string();

    let delete_req_b = with_bearer(
        Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/v1/workspaces/{}/test-sets/{}",
                workspace_id, test_set_id
            ))
            .body(Body::empty())
            .unwrap(),
        &token_b,
    );
    let resp_b = app.clone().oneshot(delete_req_b).await.unwrap();
    assert_eq!(resp_b.status(), StatusCode::NOT_FOUND);
    let resp_body = read_json_body(resp_b).await;
    assert_eq!(resp_body["error"]["code"], "TEST_SET_NOT_FOUND");
}

#[tokio::test]
async fn test_name_validation_returns_400() {
    let app = setup_test_app().await;
    let token = register_user(&app, "ts_validate_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "   ", "description": null, "cases": sample_cases_json()}),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_cases_validation_returns_400_for_non_array() {
    let app = setup_test_app().await;
    let token = register_user(&app, "ts_cases_validate_user", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({"name": "ts", "description": null, "cases": {"not": "array"}}),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_cases_validation_returns_400_for_missing_reference() {
    let app = setup_test_app().await;
    let token = register_user(&app, "ts_cases_validate_user2", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/test-sets", workspace_id),
            json!({
                "name": "ts",
                "description": null,
                "cases": [{ "id": "case-1", "input": { "text": "hi" } }]
            }),
        ),
        &token,
    );

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}
