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

fn sample_constrained_cases_json() -> Value {
    json!([
      {
        "id": "case-1",
        "input": { "prompt": "写一段欢迎文案" },
        "reference": {
          "Constrained": {
            "core_request": "友好、简洁、鼓励探索",
            "constraints": [
              { "name": "length", "description": "长度限制", "params": { "minChars": 30, "maxChars": 120 }, "weight": null }
            ],
            "quality_dimensions": []
          }
        },
        "split": null,
        "metadata": null
      }
    ])
}

fn sample_hybrid_cases_json() -> Value {
    json!([
      {
        "id": "case-1",
        "input": { "text": "hi" },
        "reference": {
          "Hybrid": {
            "exact_parts": { "expected": "ok" },
            "constraints": []
          }
        },
        "split": null,
        "metadata": null
      }
    ])
}

fn default_output_config_json() -> Value {
    json!({
        "strategy": "single",
        "conflict_alert_threshold": 3,
        "auto_recommend": true
    })
}

fn default_evaluator_config_json() -> Value {
    json!({
        "evaluator_type": "auto",
        "exact_match": { "case_sensitive": false },
        "semantic_similarity": { "threshold_percent": 85 },
        "constraint_check": { "strict": true },
        "teacher_model": { "llm_judge_samples": 1 }
    })
}

fn default_advanced_data_split_json() -> Value {
    json!({
        "strategy": "percent",
        "k_fold_folds": 5,
        "sampling_strategy": "random"
    })
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
            json!({"name": name, "description": null, "cases": cases}),
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
    task_mode: &str,
    test_set_ids: Vec<String>,
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
                "task_mode": task_mode,
                "test_set_ids": test_set_ids
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

#[tokio::test]
async fn test_unauthorized_access_returns_401() {
    let app = setup_test_app().await;

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/workspaces/w1/optimization-tasks")
        .body(Body::empty())
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_and_list_happy_path() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_user_a", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;

    // Create
    let create_req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
            json!({
                "name": "task-1",
                "description": "d",
                "goal": "g",
                "execution_target_type": "dify",
                "task_mode": "fixed",
                "test_set_ids": [test_set_id]
            }),
        ),
        &token,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);

    // List
    let list_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
        ),
        &token,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_trims_test_set_ids() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_trim_ts_ids", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let test_set_id =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;

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
                "test_set_ids": [format!("  {}  ", test_set_id)]
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_workspace_other_user_returns_workspace_not_found() {
    let app = setup_test_app().await;
    let token_a = register_user(&app, "opt_task_user_ws_a", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;

    let token_b = register_user(&app, "opt_task_user_ws_b", "TestPass123!").await;

    let list_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
        ),
        &token_b,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(list_resp).await;
    assert_eq!(body["error"]["code"], "WORKSPACE_NOT_FOUND");
}

#[tokio::test]
async fn test_create_with_missing_test_set_returns_test_set_not_found() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_missing_ts", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
            json!({
                "name": "task-1",
                "description": null,
                "goal": "g",
                "execution_target_type": "dify",
                "task_mode": "fixed",
                "test_set_ids": ["missing"]
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "TEST_SET_NOT_FOUND");
}

#[tokio::test]
async fn test_mode_validation_fixed_rejects_constrained() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_fixed_constrained", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let constrained_ts = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token,
        "ts-constrained",
        sample_constrained_cases_json(),
    )
    .await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
            json!({
                "name": "task-1",
                "description": null,
                "goal": "g",
                "execution_target_type": "dify",
                "task_mode": "fixed",
                "test_set_ids": [constrained_ts]
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_mode_validation_creative_rejects_exact() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_creative_exact", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let exact_ts = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token,
        "ts-exact",
        sample_exact_cases_json(),
    )
    .await;

    let req = with_bearer(
        build_json_request(
            "POST",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
            json!({
                "name": "task-1",
                "description": null,
                "goal": "g",
                "execution_target_type": "generic",
                "task_mode": "creative",
                "test_set_ids": [exact_ts]
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_mode_validation_hybrid_allowed_for_fixed_and_creative() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_hybrid_allowed", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let hybrid_ts = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token,
        "ts-hybrid",
        sample_hybrid_cases_json(),
    )
    .await;

    for (idx, task_mode) in ["fixed", "creative"].into_iter().enumerate() {
        let req = with_bearer(
            build_json_request(
                "POST",
                &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
                json!({
                    "name": format!("task-{}", idx + 1),
                    "description": null,
                    "goal": "g",
                    "execution_target_type": "dify",
                    "task_mode": task_mode,
                    "test_set_ids": [hybrid_ts]
                }),
            ),
            &token,
        );
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_create_with_multiple_test_sets_then_get_reflects_all() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_multi_ts", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token,
        "ts-1",
        sample_exact_cases_json(),
    )
    .await;
    let ts2 = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token,
        "ts-2",
        sample_exact_cases_json(),
    )
    .await;

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
                "test_set_ids": [ts1, ts2]
            }),
        ),
        &token,
    );
    let create_resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::OK);
    let create_body = read_json_body(create_resp).await;
    let task_id = create_body["data"]["id"]
        .as_str()
        .expect("缺少 id")
        .to_string();

    let get_req = with_bearer(
        build_empty_request(
            "GET",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}",
                workspace_id, task_id
            ),
        ),
        &token,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = read_json_body(get_resp).await;
    assert_eq!(
        get_body["data"]["test_set_ids"].as_array().unwrap().len(),
        2
    );
}

#[tokio::test]
async fn test_get_task_returns_normalized_default_config_includes_new_fields_when_config_json_null()
{
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_get_default_cfg", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let get_req = with_bearer(
        build_empty_request(
            "GET",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}",
                workspace_id, task_id
            ),
        ),
        &token,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = read_json_body(get_resp).await;

    assert_eq!(get_body["data"]["config"]["candidate_prompt_count"], 5);
    assert_eq!(
        get_body["data"]["config"]["diversity_injection_threshold"],
        3
    );
    assert_eq!(
        get_body["data"]["config"]["output_config"],
        default_output_config_json()
    );
    assert_eq!(
        get_body["data"]["config"]["evaluator_config"],
        default_evaluator_config_json()
    );
    assert_eq!(
        get_body["data"]["config"]["advanced_data_split"],
        default_advanced_data_split_json()
    );
}

#[tokio::test]
async fn test_get_task_fills_default_for_new_fields_when_old_config_missing_them() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "opt_task_get_legacy_cfg", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let legacy_config = json!({
        "schema_version": 1,
        "initial_prompt": null,
        "max_iterations": 10,
        "pass_threshold_percent": 95,
        "data_split": {
            "train_percent": 80,
            "validation_percent": 20,
            "holdout_percent": 0
        }
    })
    .to_string();

    sqlx::query("UPDATE optimization_tasks SET config_json = ?1 WHERE id = ?2")
        .bind(legacy_config)
        .bind(&task_id)
        .execute(&db)
        .await
        .expect("写入 legacy config_json 失败");

    let get_req = with_bearer(
        build_empty_request(
            "GET",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}",
                workspace_id, task_id
            ),
        ),
        &token,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = read_json_body(get_resp).await;

    assert_eq!(get_body["data"]["config"]["candidate_prompt_count"], 5);
    assert_eq!(
        get_body["data"]["config"]["diversity_injection_threshold"],
        3
    );
    assert_eq!(
        get_body["data"]["config"]["output_config"],
        default_output_config_json()
    );
    assert_eq!(
        get_body["data"]["config"]["evaluator_config"],
        default_evaluator_config_json()
    );
    assert_eq!(
        get_body["data"]["config"]["advanced_data_split"],
        default_advanced_data_split_json()
    );
}

#[tokio::test]
async fn test_update_task_config_preserves_unknown_extra_fields_in_config_json() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "opt_task_cfg_preserve_extra", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let config_with_extra = json!({
        "schema_version": 1,
        "initial_prompt": null,
        "max_iterations": 10,
        "pass_threshold_percent": 95,
        "candidate_prompt_count": 5,
        "diversity_injection_threshold": 3,
        "data_split": {
            "train_percent": 80,
            "validation_percent": 20,
            "holdout_percent": 0
        },
        "future_field": 123
    })
    .to_string();

    sqlx::query("UPDATE optimization_tasks SET config_json = ?1 WHERE id = ?2")
        .bind(config_with_extra)
        .bind(&task_id)
        .execute(&db)
        .await
        .expect("写入 config_json 失败");

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 4,
                "diversity_injection_threshold": 2,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["config"]["candidate_prompt_count"], 4);
    assert_eq!(body["data"]["config"]["diversity_injection_threshold"], 2);

    let updated_config_json: Option<String> =
        sqlx::query_scalar("SELECT config_json FROM optimization_tasks WHERE id = ?1")
            .bind(&task_id)
            .fetch_optional(&db)
            .await
            .expect("查询 config_json 失败");
    let updated_config_json = updated_config_json.expect("缺少 config_json");
    let updated_value: Value =
        serde_json::from_str(&updated_config_json).expect("解析更新后的 config_json 失败");
    assert_eq!(updated_value["future_field"], 123);
}

#[tokio::test]
async fn test_update_task_config_normalizes_empty_initial_prompt() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_cfg_empty_prompt", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": "",
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert!(body["data"]["config"]["initial_prompt"].is_null());
    assert_eq!(body["data"]["config"]["max_iterations"], 10);
    assert_eq!(body["data"]["config"]["pass_threshold_percent"], 95);
    assert_eq!(body["data"]["config"]["candidate_prompt_count"], 5);
    assert_eq!(body["data"]["config"]["diversity_injection_threshold"], 3);
}

#[tokio::test]
async fn test_update_task_config_rejects_when_existing_config_json_invalid_to_avoid_data_loss() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "opt_task_cfg_bad_json", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    sqlx::query("UPDATE optimization_tasks SET config_json = ?1 WHERE id = ?2")
        .bind("{not-json")
        .bind(&task_id)
        .execute(&db)
        .await
        .expect("写入 invalid config_json 失败");

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("任务配置解析失败"),
        "期望错误消息包含“任务配置解析失败”，实际: {msg}"
    );

    let current_config_json: Option<String> =
        sqlx::query_scalar("SELECT config_json FROM optimization_tasks WHERE id = ?1")
            .bind(&task_id)
            .fetch_optional(&db)
            .await
            .expect("查询 config_json 失败");
    assert_eq!(
        current_config_json.as_deref(),
        Some("{not-json"),
        "更新失败时不应覆盖既有 config_json"
    );
}

#[tokio::test]
async fn test_update_task_config_accepts_new_fields_boundary_values() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_cfg_new_fields_boundary", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 1,
                "diversity_injection_threshold": 10,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": {
                  "strategy": "multi",
                  "conflict_alert_threshold": 10,
                  "auto_recommend": false
                },
                "evaluator_config": {
                  "evaluator_type": "teacher_model",
                  "exact_match": { "case_sensitive": false },
                  "semantic_similarity": { "threshold_percent": 85 },
                  "constraint_check": { "strict": true },
                  "teacher_model": { "llm_judge_samples": 5 }
                },
                "advanced_data_split": {
                  "strategy": "k_fold",
                  "k_fold_folds": 10,
                  "sampling_strategy": "stratified"
                }
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["config"]["candidate_prompt_count"], 1);
    assert_eq!(body["data"]["config"]["diversity_injection_threshold"], 10);
    assert_eq!(body["data"]["config"]["output_config"]["strategy"], "multi");
    assert_eq!(
        body["data"]["config"]["output_config"]["conflict_alert_threshold"],
        10
    );
    assert_eq!(
        body["data"]["config"]["output_config"]["auto_recommend"],
        false
    );
    assert_eq!(
        body["data"]["config"]["evaluator_config"]["evaluator_type"],
        "teacher_model"
    );
    assert_eq!(
        body["data"]["config"]["evaluator_config"]["teacher_model"]["llm_judge_samples"],
        5
    );
    assert_eq!(
        body["data"]["config"]["advanced_data_split"]["strategy"],
        "k_fold"
    );
    assert_eq!(
        body["data"]["config"]["advanced_data_split"]["k_fold_folds"],
        10
    );
}

#[tokio::test]
async fn test_update_task_config_rejects_out_of_range_values() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_cfg_invalid", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let invalid_payloads = vec![
        json!({
            "initial_prompt": null,
            "max_iterations": 0,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 101,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 70,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 0,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 0,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 11,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 11,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": {
                "strategy": "single",
                "conflict_alert_threshold": 0,
                "auto_recommend": true
            },
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": {
                "evaluator_type": "semantic_similarity",
                "exact_match": { "case_sensitive": false },
                "semantic_similarity": { "threshold_percent": 101 },
                "constraint_check": { "strict": true },
                "teacher_model": { "llm_judge_samples": 1 }
            },
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": {
                "evaluator_type": "teacher_model",
                "exact_match": { "case_sensitive": false },
                "semantic_similarity": { "threshold_percent": 85 },
                "constraint_check": { "strict": true },
                "teacher_model": { "llm_judge_samples": 0 }
            },
            "advanced_data_split": default_advanced_data_split_json()
        }),
        json!({
            "initial_prompt": null,
            "max_iterations": 10,
            "pass_threshold_percent": 95,
            "candidate_prompt_count": 5,
            "diversity_injection_threshold": 3,
            "train_percent": 80,
            "validation_percent": 20,
            "output_config": default_output_config_json(),
            "evaluator_config": default_evaluator_config_json(),
            "advanced_data_split": {
                "strategy": "k_fold",
                "k_fold_folds": 1,
                "sampling_strategy": "random"
            }
        }),
    ];

    for payload in invalid_payloads {
        let req = with_bearer(
            build_json_request(
                "PUT",
                &format!(
                    "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                    workspace_id, task_id
                ),
                payload,
            ),
            &token,
        );
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = read_json_body(resp).await;
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn test_teacher_llm_model_is_task_scoped_and_list_includes_display_name() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_teacher_llm_scoped", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;

    let task_a = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1.clone()]).await;
    let task_b = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    // Update task A teacher_llm
    let update_req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_a
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "teacher_llm": { "model_id": "  gpt-4  " },
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let update_resp = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_resp.status(), StatusCode::OK);
    let update_body = read_json_body(update_resp).await;
    assert_eq!(
        update_body["data"]["config"]["teacher_llm"]["model_id"].as_str(),
        Some("gpt-4")
    );

    // GET task should also reflect normalized value
    let get_req = with_bearer(
        build_empty_request(
            "GET",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}",
                workspace_id, task_a
            ),
        ),
        &token,
    );
    let get_resp = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::OK);
    let get_body = read_json_body(get_resp).await;
    assert_eq!(
        get_body["data"]["config"]["teacher_llm"]["model_id"].as_str(),
        Some("gpt-4")
    );

    // List should include display name without extra calls (computed server-side)
    let list_req = with_bearer(
        build_empty_request(
            "GET",
            &format!("/api/v1/workspaces/{}/optimization-tasks", workspace_id),
        ),
        &token,
    );
    let list_resp = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let body = read_json_body(list_resp).await;
    let items = body["data"].as_array().expect("data 不是数组");

    let mut by_id = std::collections::HashMap::new();
    for item in items {
        let id = item["id"].as_str().unwrap_or_default().to_string();
        by_id.insert(id, item.clone());
    }

    assert_eq!(
        by_id[&task_a]["teacher_model_display_name"].as_str(),
        Some("gpt-4")
    );
    assert_eq!(
        by_id[&task_b]["teacher_model_display_name"].as_str(),
        Some("系统默认")
    );
}

#[tokio::test]
async fn test_teacher_llm_model_id_validation_and_normalization() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_teacher_llm_validation", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;

    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    // model_id with control character should be rejected (trim does not remove \u0007)
    let invalid_control = "gpt\u{0007}-4";
    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "teacher_llm": { "model_id": invalid_control },
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");

    // model_id too long should be rejected
    let long_id = "a".repeat(129);
    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "teacher_llm": { "model_id": long_id },
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");

    // model_id whitespace should normalize to null (system default)
    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "teacher_llm": { "model_id": "   " },
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_json_body(resp).await;
    assert_eq!(body["data"]["config"]["teacher_llm"]["model_id"], serde_json::Value::Null);
}

#[tokio::test]
async fn test_update_task_config_not_found() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_cfg_not_found", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, "missing-task"
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "OPTIMIZATION_TASK_NOT_FOUND");
}

#[tokio::test]
async fn test_update_task_config_other_user_returns_workspace_not_found() {
    let app = setup_test_app().await;

    let token_a = register_user(&app, "opt_task_cfg_cross_user_a", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token_a).await;
    let ts1 = create_test_set_with_cases(
        &app,
        &workspace_id,
        &token_a,
        "ts",
        sample_exact_cases_json(),
    )
    .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token_a, "fixed", vec![ts1]).await;

    let token_b = register_user(&app, "opt_task_cfg_cross_user_b", "TestPass123!").await;

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token_b,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "WORKSPACE_NOT_FOUND");
}

#[tokio::test]
async fn test_update_task_config_rejects_too_large_initial_prompt_bytes() {
    let app = setup_test_app().await;
    let token = register_user(&app, "opt_task_cfg_prompt_too_large", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let prompt = "a".repeat(20_001);

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": prompt,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_update_task_config_rejects_oversize_config_json() {
    let (app, db) = setup_test_app_with_db().await;
    let token = register_user(&app, "opt_task_cfg_oversize_json", "TestPass123!").await;
    let workspace_id = create_workspace(&app, &token).await;
    let ts1 =
        create_test_set_with_cases(&app, &workspace_id, &token, "ts", sample_exact_cases_json())
            .await;
    let task_id = create_optimization_task(&app, &workspace_id, &token, "fixed", vec![ts1]).await;

    let extra_blob = "x".repeat(40_000);
    let big_config = json!({
        "schema_version": 1,
        "initial_prompt": null,
        "max_iterations": 10,
        "pass_threshold_percent": 95,
        "data_split": {
            "train_percent": 80,
            "validation_percent": 20,
            "holdout_percent": 0
        },
        "extra_blob": extra_blob
    })
    .to_string();

    sqlx::query("UPDATE optimization_tasks SET config_json = ?1 WHERE id = ?2")
        .bind(big_config)
        .bind(&task_id)
        .execute(&db)
        .await
        .expect("写入 oversized config_json 失败");

    let req = with_bearer(
        build_json_request(
            "PUT",
            &format!(
                "/api/v1/workspaces/{}/optimization-tasks/{}/config",
                workspace_id, task_id
            ),
            json!({
                "initial_prompt": null,
                "max_iterations": 10,
                "pass_threshold_percent": 95,
                "candidate_prompt_count": 5,
                "diversity_injection_threshold": 3,
                "train_percent": 80,
                "validation_percent": 20,
                "output_config": default_output_config_json(),
                "evaluator_config": default_evaluator_config_json(),
                "advanced_data_split": default_advanced_data_split_json()
            }),
        ),
        &token,
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = read_json_body(resp).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains("任务配置过大"),
        "期望错误消息包含“任务配置过大”，实际: {msg}"
    );
}
