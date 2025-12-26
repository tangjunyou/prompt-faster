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
use sqlx::SqlitePool;

use prompt_faster::api::middleware::correlation_id::correlation_id_middleware;
use prompt_faster::api::middleware::{LoginAttemptStore, SessionStore, auth_middleware};
use prompt_faster::api::routes::{auth, health, user_auth};
use prompt_faster::api::state::AppState;
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;

const TEST_MASTER_PASSWORD: &str = "test_master_password_for_integration";

async fn setup_test_app_with_db() -> (Router, SqlitePool) {
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
        db: db.clone(),
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
        middleware::from_fn_with_state(session_store_for_middleware, auth_middleware),
    );

    let router = Router::<AppState>::new()
        .nest("/api/v1", health::router::<AppState>())
        .nest("/api/v1/auth", auth::public_router())
        .nest("/api/v1/auth", protected_routes)
        .nest("/api/v1/auth", user_auth::public_router())
        .nest("/api/v1/auth", protected_user_auth_routes)
        .with_state(state)
        .layer(middleware::from_fn(correlation_id_middleware));

    (router, db)
}

async fn setup_test_app() -> Router {
    setup_test_app_with_db().await.0
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

fn with_connect_info(mut req: Request<Body>, addr: SocketAddr) -> Request<Body> {
    req.extensions_mut().insert(ConnectInfo(addr));
    req
}

fn with_bearer(mut req: Request<Body>, token: &str) -> Request<Body> {
    req.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );
    req
}

#[tokio::test]
async fn test_register_login_me_success() {
    let app = setup_test_app().await;

    let username = "test_user_register_login_me";
    let password = "TestPass123!";

    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );

    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    let register_json = read_json_body(register_resp).await;
    let register_token = register_json["data"]["session_token"]
        .as_str()
        .expect("缺少 session_token")
        .to_string();

    let logout_req = with_bearer(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/logout")
            .body(Body::empty())
            .unwrap(),
        &register_token,
    );

    let logout_resp = app.clone().oneshot(logout_req).await.unwrap();
    assert_eq!(logout_resp.status(), StatusCode::OK);

    let login_addr: SocketAddr = ([127, 0, 0, 1], 12345).into();
    let login_req = with_connect_info(
        build_json_request(
            "POST",
            "/api/v1/auth/login",
            json!({"username": username, "password": password}),
        ),
        login_addr,
    );

    let login_resp = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);

    let login_json = read_json_body(login_resp).await;
    let token = login_json["data"]["session_token"]
        .as_str()
        .expect("缺少 session_token")
        .to_string();

    let me_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .body(Body::empty())
            .unwrap(),
        &token,
    );

    let me_resp = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(me_resp.status(), StatusCode::OK);

    let me_json = read_json_body(me_resp).await;
    assert_eq!(me_json["data"]["username"].as_str(), Some(username));
}

#[tokio::test]
async fn test_register_migrates_legacy_default_user_data_for_first_user() {
    let (app, db) = setup_test_app_with_db().await;

    let now = 1_i64;

    let legacy_credential_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO api_credentials (
            id, user_id, credential_type, provider, base_url,
            encrypted_api_key, nonce, salt, created_at, updated_at
        )
        VALUES (?1, 'default_user', 'dify', NULL, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(legacy_credential_id)
    .bind("https://legacy.example")
    .bind(vec![1_u8, 2, 3])
    .bind(vec![0_u8; 12])
    .bind(vec![0_u8; 16])
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .expect("插入 legacy api_credentials 失败");

    let legacy_teacher_settings_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO teacher_model_settings (
            id, user_id, temperature, top_p, max_tokens, created_at, updated_at
        )
        VALUES (?1, 'default_user', 0.7, 0.9, 2048, ?2, ?3)
        "#,
    )
    .bind(legacy_teacher_settings_id)
    .bind(now)
    .bind(now)
    .execute(&db)
    .await
    .expect("插入 legacy teacher_model_settings 失败");

    let username = "test_user_migration_first_user";
    let password = "TestPass123!";

    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );

    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);
    let register_json = read_json_body(register_resp).await;
    let user_id = register_json["data"]["user"]["id"]
        .as_str()
        .expect("缺少 user.id");

    let legacy_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM api_credentials WHERE user_id = 'default_user'
        "#,
    )
    .fetch_one(&db)
    .await
    .expect("查询 legacy api_credentials 失败");
    assert_eq!(legacy_count.0, 0);

    let migrated_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM api_credentials WHERE user_id = ?1 AND credential_type = 'dify'
        "#,
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .expect("查询迁移后的 api_credentials 失败");
    assert_eq!(migrated_count.0, 1);

    let legacy_settings_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM teacher_model_settings WHERE user_id = 'default_user'
        "#,
    )
    .fetch_one(&db)
    .await
    .expect("查询 legacy teacher_model_settings 失败");
    assert_eq!(legacy_settings_count.0, 0);

    let migrated_settings_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM teacher_model_settings WHERE user_id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_one(&db)
    .await
    .expect("查询迁移后的 teacher_model_settings 失败");
    assert_eq!(migrated_settings_count.0, 1);
}

#[tokio::test]
async fn test_wrong_username_and_wrong_password_return_same_generic_error() {
    let app = setup_test_app().await;

    let username = "test_user_generic_error";
    let password = "TestPass123!";

    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );
    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    let addr: SocketAddr = ([127, 0, 0, 1], 22222).into();

    let wrong_username_req = with_connect_info(
        build_json_request(
            "POST",
            "/api/v1/auth/login",
            json!({"username": "non_existent_user", "password": password}),
        ),
        addr,
    );
    let wrong_username_resp = app.clone().oneshot(wrong_username_req).await.unwrap();
    assert_eq!(wrong_username_resp.status(), StatusCode::UNAUTHORIZED);
    let wrong_username_json = read_json_body(wrong_username_resp).await;

    let wrong_password_req = with_connect_info(
        build_json_request(
            "POST",
            "/api/v1/auth/login",
            json!({"username": username, "password": "wrong_password"}),
        ),
        addr,
    );
    let wrong_password_resp = app.clone().oneshot(wrong_password_req).await.unwrap();
    assert_eq!(wrong_password_resp.status(), StatusCode::UNAUTHORIZED);
    let wrong_password_json = read_json_body(wrong_password_resp).await;

    assert_eq!(
        wrong_username_json["error"]["code"].as_str(),
        Some("AUTH_FAILED")
    );
    assert_eq!(
        wrong_username_json["error"]["message"].as_str(),
        Some("用户名或密码错误")
    );

    assert_eq!(
        wrong_password_json["error"]["code"].as_str(),
        Some("AUTH_FAILED")
    );
    assert_eq!(
        wrong_password_json["error"]["message"].as_str(),
        Some("用户名或密码错误")
    );
}

#[tokio::test]
async fn test_logout_invalidates_token() {
    let app = setup_test_app().await;

    let username = "test_user_logout";
    let password = "TestPass123!";

    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );

    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    let register_json = read_json_body(register_resp).await;
    let token = register_json["data"]["session_token"]
        .as_str()
        .expect("缺少 session_token")
        .to_string();

    let logout_req = with_bearer(
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/logout")
            .body(Body::empty())
            .unwrap(),
        &token,
    );

    let logout_resp = app.clone().oneshot(logout_req).await.unwrap();
    assert_eq!(logout_resp.status(), StatusCode::OK);

    let me_req = with_bearer(
        Request::builder()
            .method("GET")
            .uri("/api/v1/auth/me")
            .body(Body::empty())
            .unwrap(),
        &token,
    );

    let me_resp = app.clone().oneshot(me_req).await.unwrap();
    assert_eq!(me_resp.status(), StatusCode::UNAUTHORIZED);

    let me_json = read_json_body(me_resp).await;
    assert_eq!(me_json["error"]["code"].as_str(), Some("UNAUTHORIZED"));
}

#[tokio::test]
async fn test_login_attempt_protection_returns_generic_error() {
    let app = setup_test_app().await;

    let username = "test_user_rate_limit";
    let password = "TestPass123!";

    let register_req = build_json_request(
        "POST",
        "/api/v1/auth/register",
        json!({"username": username, "password": password}),
    );

    let register_resp = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_resp.status(), StatusCode::OK);

    let addr: SocketAddr = ([127, 0, 0, 1], 33333).into();

    for _ in 0..5 {
        let req = with_connect_info(
            build_json_request(
                "POST",
                "/api/v1/auth/login",
                json!({"username": username, "password": "wrong_password"}),
            ),
            addr,
        );
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let json = read_json_body(resp).await;
        assert_eq!(json["error"]["code"].as_str(), Some("AUTH_FAILED"));
        assert_eq!(json["error"]["message"].as_str(), Some("用户名或密码错误"));
    }

    let blocked_req = with_connect_info(
        build_json_request(
            "POST",
            "/api/v1/auth/login",
            json!({"username": username, "password": "wrong_password"}),
        ),
        addr,
    );

    let blocked_resp = app.clone().oneshot(blocked_req).await.unwrap();
    assert_eq!(blocked_resp.status(), StatusCode::UNAUTHORIZED);

    let blocked_json = read_json_body(blocked_resp).await;
    assert_eq!(blocked_json["error"]["code"].as_str(), Some("AUTH_FAILED"));
    assert_eq!(
        blocked_json["error"]["message"].as_str(),
        Some("用户名或密码错误")
    );
}
