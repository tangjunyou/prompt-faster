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
use prompt_faster::api::routes::{auth, health, meta_optimization, user_auth};
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

    let protected_meta_routes = meta_optimization::router().layer(middleware::from_fn_with_state(
        session_store_for_middleware,
        auth_middleware,
    ));

    let router = Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .nest("/api/v1/meta-optimization", protected_meta_routes)
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware));

    (router, db)
}

async fn setup_test_app() -> Router {
    let (app, _db) = setup_test_app_with_db().await;
    app
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

#[tokio::test]
async fn test_create_list_and_get_prompt() {
    let app = setup_test_app().await;
    let token = register_user(&app, "user1", "password").await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts",
            json!({"content": "prompt-v1", "description": "first"}),
        ),
        &token,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let create_json = read_json_body(create_resp).await;
    let prompt_id = create_json["data"]["id"].as_str().unwrap().to_string();

    let list_req = with_bearer(
        build_empty_request("GET", "/api/v1/meta-optimization/prompts"),
        &token,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_json = read_json_body(list_resp).await;
    assert_eq!(list_json["data"].as_array().unwrap().len(), 1);

    let get_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/meta-optimization/prompts/{}", prompt_id),
        ),
        &token,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activate_prompt_and_stats() {
    let app = setup_test_app().await;
    let token = register_user(&app, "user2", "password").await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts",
            json!({"content": "prompt-v1"}),
        ),
        &token,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    let create_json = read_json_body(create_resp).await;
    let prompt_id = create_json["data"]["id"].as_str().unwrap().to_string();

    let create_req2 = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts",
            json!({"content": "prompt-v2", "activate": false}),
        ),
        &token,
    );
    let create_resp2 = app.clone().oneshot(create_req2).await.unwrap();
    let create_json2 = read_json_body(create_resp2).await;
    let prompt_id2 = create_json2["data"]["id"].as_str().unwrap().to_string();

    let activate_req = with_bearer(
        build_json_request(
            "PUT",
            &format!("/api/v1/meta-optimization/prompts/{}/activate", prompt_id2),
            json!({}),
        ),
        &token,
    );
    let activate_resp = app.clone().oneshot(activate_req).await.unwrap();
    assert_eq!(activate_resp.status(), StatusCode::OK);

    let stats_req = with_bearer(
        build_empty_request("GET", "/api/v1/meta-optimization/stats"),
        &token,
    );
    let stats_resp = app.clone().oneshot(stats_req).await.unwrap();
    assert_eq!(stats_resp.status(), StatusCode::OK);
    let stats_json = read_json_body(stats_resp).await;
    assert_eq!(stats_json["data"]["totalVersions"].as_i64().unwrap(), 2);

    // ensure activate didn't delete first version
    assert_ne!(prompt_id, prompt_id2);
}

#[tokio::test]
async fn test_prompt_forbidden_for_other_user() {
    let app = setup_test_app().await;
    let token1 = register_user(&app, "user3", "password").await;
    let token2 = register_user(&app, "user4", "password").await;

    let create_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts",
            json!({"content": "prompt-v1"}),
        ),
        &token1,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    let create_json = read_json_body(create_resp).await;
    let prompt_id = create_json["data"]["id"].as_str().unwrap().to_string();

    let get_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/meta-optimization/prompts/{}", prompt_id),
        ),
        &token2,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::FORBIDDEN);
}
