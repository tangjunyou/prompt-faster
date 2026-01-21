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
use prompt_faster::domain::models::{
    EvaluatorType, ExecutionTargetType, OptimizationTaskConfig, OptimizationTaskMode,
    TaskReference, TestCase,
};
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::db::repositories::{
    CreateOptimizationTaskInput, OptimizationTaskRepo, TestSetRepo, WorkspaceRepo,
};
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;
use std::sync::{Mutex, OnceLock};

const TEST_MASTER_PASSWORD: &str = "test_master_password_for_integration";

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn set_test_env_var(key: &str, value: &str) {
    // Safety: env vars are process-global; we serialize access via env_lock in tests.
    unsafe {
        std::env::set_var(key, value);
    }
}

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

async fn find_user_id(db: &sqlx::SqlitePool, username: &str) -> String {
    sqlx::query_scalar("SELECT id FROM users WHERE username = ?1")
        .bind(username)
        .fetch_one(db)
        .await
        .expect("查询用户 ID 失败")
}

async fn create_task_with_test_set(
    db: &sqlx::SqlitePool,
    user_id: &str,
    cases: Vec<TestCase>,
) -> (String, String) {
    let workspace = WorkspaceRepo::create(db, user_id, "ws", None)
        .await
        .expect("创建工作区失败");
    let test_set = TestSetRepo::create(db, &workspace.id, "ts", None, &cases, None, None)
        .await
        .expect("创建测试集失败");
    let created = OptimizationTaskRepo::create_scoped(
        db,
        CreateOptimizationTaskInput {
            user_id,
            workspace_id: &workspace.id,
            name: "task-1",
            description: None,
            goal: "g",
            execution_target_type: ExecutionTargetType::Example,
            task_mode: OptimizationTaskMode::Fixed,
            test_set_ids: std::slice::from_ref(&test_set.id),
            teacher_prompt_version_id: None,
        },
    )
    .await
    .expect("创建任务失败");
    (workspace.id, created.task.id)
}

async fn create_prompt_version(app: &Router, token: &str, content: &str) -> String {
    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts",
            json!({"content": content}),
        ),
        token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    body["data"]["id"].as_str().unwrap().to_string()
}

fn sample_exact_cases(prompt_len: usize) -> Vec<TestCase> {
    let mut input = std::collections::HashMap::new();
    input.insert("x".to_string(), serde_json::Value::String("1".to_string()));
    vec![
        TestCase {
            id: "tc-1".to_string(),
            input: input.clone(),
            reference: TaskReference::Exact {
                expected: format!(
                    "example_execution_target: test_case_id=tc-1 prompt_len={prompt_len} input_keys_count=1"
                ),
            },
            split: None,
            metadata: None,
        },
        TestCase {
            id: "tc-2".to_string(),
            input,
            reference: TaskReference::Exact {
                expected: format!(
                    "example_execution_target: test_case_id=tc-2 prompt_len={prompt_len} input_keys_count=1"
                ),
            },
            split: None,
            metadata: None,
        },
    ]
}

fn sample_exact_cases_with_count(prompt_len: usize, count: usize) -> Vec<TestCase> {
    let mut cases = Vec::with_capacity(count);
    for idx in 0..count {
        let mut input = std::collections::HashMap::new();
        input.insert("x".to_string(), serde_json::Value::String("1".to_string()));
        let id = format!("tc-{}", idx + 1);
        cases.push(TestCase {
            id: id.clone(),
            input,
            reference: TaskReference::Exact {
                expected: format!(
                    "example_execution_target: test_case_id={} prompt_len={} input_keys_count=1",
                    id, prompt_len
                ),
            },
            split: None,
            metadata: None,
        });
    }
    cases
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

#[tokio::test]
async fn test_validate_prompt_returns_errors() {
    let app = setup_test_app().await;
    let token = register_user(&app, "user5", "password").await;

    let empty_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/validate",
            json!({"content": "  "}),
        ),
        &token,
    );
    let empty_resp = app.clone().oneshot(empty_req).await.unwrap();
    assert_eq!(empty_resp.status(), StatusCode::OK);
    let empty_json = read_json_body(empty_resp).await;
    assert_eq!(empty_json["data"]["isValid"].as_bool().unwrap(), false);
    assert!(empty_json["data"]["errors"].as_array().unwrap().len() >= 1);

    let long_content = "a".repeat(100 * 1024 + 1);
    let long_req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/validate",
            json!({"content": long_content}),
        ),
        &token,
    );
    let long_resp = app.clone().oneshot(long_req).await.unwrap();
    assert_eq!(long_resp.status(), StatusCode::OK);
    let long_json = read_json_body(long_resp).await;
    assert_eq!(long_json["data"]["isValid"].as_bool().unwrap(), false);
}

#[tokio::test]
async fn test_preview_prompt_requires_auth() {
    let app = setup_test_app().await;
    let req = build_json_request(
        "POST",
        "/api/v1/meta-optimization/prompts/preview",
        json!({"content": "hi", "taskIds": ["task-1"], "testCaseIds": []}),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_preview_prompt_success() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user6", "password").await;
    let user_id = find_user_id(&db, "user6").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::Example;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/preview",
            json!({"content": prompt, "taskIds": [task_id], "testCaseIds": []}),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["totalPassed"].as_i64().unwrap(), 2);
    assert_eq!(body["data"]["totalFailed"].as_i64().unwrap(), 0);
    assert_eq!(body["data"]["results"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["results"][0]["testCaseId"], "tc-1");
}

#[tokio::test]
async fn test_preview_prompt_rejects_over_limit() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user7", "password").await;
    let user_id = find_user_id(&db, "user7").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (_workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/preview",
            json!({
                "content": prompt,
                "taskIds": [task_id],
                "testCaseIds": ["tc-1", "tc-2", "tc-3", "tc-4"]
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_preview_prompt_timeout() {
    struct EnvGuard;
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var("PROMPT_FASTER_TEACHER_MODEL_DELAY_MS");
                std::env::remove_var("PROMPT_FASTER_PREVIEW_TIMEOUT_SECS");
            }
        }
    }

    let _guard = EnvGuard;
    let _lock = env_lock();
    set_test_env_var("PROMPT_FASTER_TEACHER_MODEL_DELAY_MS", "2000");
    set_test_env_var("PROMPT_FASTER_PREVIEW_TIMEOUT_SECS", "1");

    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user8", "password").await;
    let user_id = find_user_id(&db, "user8").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::TeacherModel;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/preview",
            json!({"content": prompt, "taskIds": [task_id], "testCaseIds": []}),
        ),
        &token,
    );

    let app_clone = app.clone();
    let handle = tokio::spawn(async move { app_clone.oneshot(req).await.unwrap() });

    let resp = handle.await.unwrap();
    assert_eq!(resp.status(), StatusCode::GATEWAY_TIMEOUT);
}

#[tokio::test]
async fn test_compare_prompt_requires_auth() {
    let app = setup_test_app().await;
    let req = build_json_request(
        "POST",
        "/api/v1/meta-optimization/prompts/compare",
        json!({
            "versionIdA": "va",
            "versionIdB": "vb",
            "taskIds": ["task-1"],
            "testCaseIds": []
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_compare_prompt_success() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user9", "password").await;
    let user_id = find_user_id(&db, "user9").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::Example;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let version_a = create_prompt_version(&app, &token, prompt).await;
    let version_b = create_prompt_version(&app, &token, prompt).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["versionAContent"], prompt);
    assert_eq!(body["data"]["versionBContent"], prompt);
    assert_eq!(body["data"]["caseComparisons"].as_array().unwrap().len(), 2);
    assert_eq!(body["data"]["summary"]["passRateDiff"], 0.0);
    assert_eq!(body["data"]["summary"]["outputDiffCases"], 0);
}

#[tokio::test]
async fn test_compare_prompt_output_diff_cases() {
    let (app, db) = setup_test_app_with_db().await;
    let _lock = env_lock();
    let token = register_user(&app, "user17", "password").await;
    let user_id = find_user_id(&db, "user17").await;

    let prompt_a = "hi";
    let prompt_b = "hello world";
    let cases = sample_exact_cases(prompt_a.chars().count());
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::TeacherModel;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let version_a = create_prompt_version(&app, &token, prompt_a).await;
    let version_b = create_prompt_version(&app, &token, prompt_b).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["summary"]["outputDiffCases"], 2);
    assert_eq!(body["data"]["summary"]["improvedCases"], 0);
    assert_eq!(body["data"]["summary"]["regressedCases"], 0);
    assert_eq!(body["data"]["summary"]["unchangedCases"], 0);
    assert_eq!(body["data"]["caseComparisons"][0]["isDifferent"], true);
    assert_eq!(
        body["data"]["caseComparisons"][0]["differenceNote"],
        "两版本均通过，但输出内容存在差异"
    );
}

#[tokio::test]
async fn test_compare_prompt_both_failed() {
    let (app, db) = setup_test_app_with_db().await;
    let _lock = env_lock();
    let token = register_user(&app, "user18", "password").await;
    let user_id = find_user_id(&db, "user18").await;

    let prompt_a = "hello";
    let prompt_b = "world";
    let cases = sample_exact_cases(1);
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::Example;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let version_a = create_prompt_version(&app, &token, prompt_a).await;
    let version_b = create_prompt_version(&app, &token, prompt_b).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["summary"]["improvedCases"], 0);
    assert_eq!(body["data"]["summary"]["regressedCases"], 0);
    assert_eq!(body["data"]["summary"]["outputDiffCases"], 0);
    assert_eq!(body["data"]["summary"]["unchangedCases"], 2);
    assert!(
        body["data"]["caseComparisons"][0]["differenceNote"]
            .as_str()
            .unwrap_or("")
            .contains("两版本均失败")
    );
}

#[tokio::test]
async fn test_compare_prompt_rejects_same_version() {
    let (app, _db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user10", "password").await;

    let version_id = create_prompt_version(&app, &token, "hi").await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_id,
                "versionIdB": version_id,
                "taskIds": ["task-1"],
                "testCaseIds": []
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_compare_prompt_version_not_found() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user11", "password").await;
    let user_id = find_user_id(&db, "user11").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (_workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let version_a = create_prompt_version(&app, &token, prompt).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": "missing",
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_compare_prompt_forbidden() {
    let (app, db) = setup_test_app_with_db().await;
    let token1 = register_user(&app, "user12", "password").await;
    let token2 = register_user(&app, "user13", "password").await;
    let user_id2 = find_user_id(&db, "user13").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (_workspace_id, task_id) = create_task_with_test_set(&db, &user_id2, cases).await;

    let version_a = create_prompt_version(&app, &token1, prompt).await;
    let version_b = create_prompt_version(&app, &token2, prompt).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token2,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_compare_prompt_rejects_over_limit() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user14", "password").await;
    let user_id = find_user_id(&db, "user14").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (_workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let version_a = create_prompt_version(&app, &token, prompt).await;
    let version_b = create_prompt_version(&app, &token, prompt).await;

    let test_case_ids: Vec<String> = (1..=11).map(|i| format!("tc-{}", i)).collect();

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": test_case_ids
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_compare_prompt_sampling_order() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user15", "password").await;
    let user_id = find_user_id(&db, "user15").await;

    let prompt = "hi";
    let cases = sample_exact_cases_with_count(prompt.chars().count(), 12);
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::Example;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let version_a = create_prompt_version(&app, &token, prompt).await;
    let version_b = create_prompt_version(&app, &token, prompt).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    let ids: Vec<String> = body["data"]["caseComparisons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["testCaseId"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids.len(), 10);
    assert_eq!(ids[0], "tc-1");
    assert_eq!(ids[9], "tc-10");
}

#[tokio::test]
async fn test_compare_prompt_timeout() {
    struct EnvGuard;
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var("PROMPT_FASTER_TEACHER_MODEL_DELAY_MS");
                std::env::remove_var("PROMPT_FASTER_COMPARE_TIMEOUT_SECS");
                std::env::remove_var("PROMPT_FASTER_PREVIEW_TIMEOUT_SECS");
            }
        }
    }

    let _guard = EnvGuard;
    let _lock = env_lock();
    set_test_env_var("PROMPT_FASTER_TEACHER_MODEL_DELAY_MS", "2000");
    set_test_env_var("PROMPT_FASTER_COMPARE_TIMEOUT_SECS", "1");
    set_test_env_var("PROMPT_FASTER_PREVIEW_TIMEOUT_SECS", "5");

    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "user16", "password").await;
    let user_id = find_user_id(&db, "user16").await;

    let prompt = "hi";
    let cases = sample_exact_cases(prompt.chars().count());
    let (workspace_id, task_id) = create_task_with_test_set(&db, &user_id, cases).await;

    let mut config = OptimizationTaskConfig::default();
    config.evaluator_config.evaluator_type = EvaluatorType::TeacherModel;
    let _ =
        OptimizationTaskRepo::update_config_scoped(&db, &user_id, &workspace_id, &task_id, config)
            .await
            .expect("更新任务配置失败");

    let version_a = create_prompt_version(&app, &token, prompt).await;
    let version_b = create_prompt_version(&app, &token, prompt).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            "/api/v1/meta-optimization/prompts/compare",
            json!({
                "versionIdA": version_a,
                "versionIdB": version_b,
                "taskIds": [task_id],
                "testCaseIds": []
            }),
        ),
        &token,
    );

    let app_clone = app.clone();
    let handle = tokio::spawn(async move { app_clone.oneshot(req).await.unwrap() });

    let resp = handle.await.unwrap();
    assert_eq!(resp.status(), StatusCode::GATEWAY_TIMEOUT);
}
