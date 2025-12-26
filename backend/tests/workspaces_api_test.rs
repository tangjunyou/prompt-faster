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
    let api_key_manager = Arc::new(ApiKeyManager::new(TEST_MASTER_PASSWORD.to_string()));

    let session_store = SessionStore::new(24);
    let login_attempt_store = LoginAttemptStore::default();

    let state = AppState {
        db,
        http_client,
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
