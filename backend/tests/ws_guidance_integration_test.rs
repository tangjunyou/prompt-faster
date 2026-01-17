use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;

use prompt_faster::api::state::AppState;
use prompt_faster::api::ws;
use prompt_faster::core::iteration_engine::pause_state::{PauseController, global_pause_registry};
use prompt_faster::core::optimization_engine::checkpoint_pause_if_requested;
use prompt_faster::core::optimization_engine::clear_user_guidance_from_context;
use prompt_faster::domain::models::{IterationState, OutputLength, Rule, RuleSystem, RuleTags};
use prompt_faster::domain::types::{
    EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_USER_GUIDANCE, ExecutionTargetConfig,
    OptimizationConfig, OptimizationContext, RunControlState, UserGuidance,
};
use prompt_faster::infra::db::pool::create_pool;
use prompt_faster::infra::external::api_key_manager::ApiKeyManager;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::shared::config::AppConfig;
use prompt_faster::shared::time::now_millis;

async fn setup_test_app_with_db() -> (Router, AppState) {
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
    let api_key_manager = Arc::new(ApiKeyManager::new(None));

    let session_store = prompt_faster::api::middleware::SessionStore::new(24);
    let login_attempt_store = prompt_faster::api::middleware::LoginAttemptStore::default();

    let state = AppState {
        db: db.clone(),
        http_client,
        config,
        api_key_manager,
        session_store,
        login_attempt_store,
    };

    let router = Router::<AppState>::new()
        .nest("/api/v1", ws::router())
        .with_state(state.clone());

    (router, state)
}

async fn seed_user_workspace_task(
    db: &SqlitePool,
    user_id: &str,
    workspace_id: &str,
    task_id: &str,
) {
    let now = now_millis();
    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(user_id)
    .bind(user_id)
    .bind("hashed")
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("insert user");

    sqlx::query(
        r#"
        INSERT INTO workspaces (id, user_id, name, description, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(workspace_id)
    .bind(user_id)
    .bind("ws-test")
    .bind(None::<String>)
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("insert workspace");

    sqlx::query(
        r#"
        INSERT INTO optimization_tasks
          (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, created_at, updated_at)
        VALUES
          (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)
        "#,
    )
    .bind(task_id)
    .bind(workspace_id)
    .bind("ws-task")
    .bind(None::<String>)
    .bind("goal")
    .bind("generic")
    .bind("fixed")
    .bind("draft")
    .bind(now)
    .bind(now)
    .execute(db)
    .await
    .expect("insert task");
}

async fn read_message_of_type_for_task<S>(
    socket: &mut WebSocketStream<S>,
    target_type: &str,
    task_id: &str,
) -> Value
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let msg = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(Ok(frame)) = socket.next().await {
            if let tokio_tungstenite::tungstenite::Message::Text(text) = frame {
                let value: Value = serde_json::from_str(&text).expect("parse message");
                if value.get("type").and_then(|v| v.as_str()) == Some(target_type)
                    && value
                        .get("payload")
                        .and_then(|p| p.get("taskId"))
                        .and_then(|v| v.as_str())
                        == Some(task_id)
                {
                    return value;
                }
            }
        }
        panic!("socket closed before receiving {target_type} for {task_id}");
    })
    .await
    .expect("timeout waiting for message");
    msg
}

async fn wait_for_pause(controller: &Arc<PauseController>) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if controller.is_paused() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("pause timeout");
}

fn build_test_context(task_id: &str) -> OptimizationContext {
    let rule = Rule {
        id: "rule-1".to_string(),
        description: "old-rule".to_string(),
        tags: RuleTags {
            output_format: vec![],
            output_structure: vec![],
            output_length: OutputLength::Flexible,
            semantic_focus: vec![],
            key_concepts: vec![],
            must_include: vec![],
            must_exclude: vec![],
            tone: None,
            extra: HashMap::new(),
        },
        source_test_cases: vec![],
        abstraction_level: 0,
        parent_rules: vec![],
        verified: false,
        verification_score: 0.4,
        ir: None,
    };
    let rule_system = RuleSystem {
        rules: vec![rule],
        conflict_resolution_log: vec![],
        merge_log: vec![],
        coverage_map: HashMap::new(),
        version: 1,
    };

    let mut extensions = HashMap::new();
    extensions.insert(
        EXT_BEST_CANDIDATE_PROMPT.to_string(),
        serde_json::Value::String("best-prompt".to_string()),
    );
    extensions.insert(
        EXT_BEST_CANDIDATE_INDEX.to_string(),
        serde_json::Value::Number(2.into()),
    );

    OptimizationContext {
        task_id: task_id.to_string(),
        execution_target_config: ExecutionTargetConfig::default(),
        current_prompt: "current-prompt".to_string(),
        rule_system,
        iteration: 2,
        state: IterationState::RunningTests,
        run_control_state: RunControlState::Running,
        test_cases: vec![],
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions,
    }
}

#[tokio::test]
async fn ws_guidance_send_rejected_when_not_paused() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-guidance-not-paused";
    let workspace_id = "ws-guidance-not-paused";
    let task_id = "task-guidance-not-paused";
    seed_user_workspace_task(&state.db, user_id, workspace_id, task_id).await;

    let token = state
        .session_store
        .create_session(user_id.to_string(), None)
        .await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("serve");
    });

    let ws_url = format!("ws://{}/api/v1/ws?token={}", addr, token);
    let (mut socket, _) = connect_async(ws_url).await.expect("connect ws");

    let cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": "请更正式" },
        "correlationId": "cid-guidance-1"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            cmd.to_string(),
        ))
        .await
        .expect("send guidance");

    let ack = read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    assert_eq!(ack["payload"]["ok"], false);
    assert_eq!(ack["payload"]["reason"].as_str(), Some("task_not_paused"));
}

#[tokio::test]
async fn ws_guidance_send_rejected_when_not_owner() {
    let (app, state) = setup_test_app_with_db().await;

    let owner_id = "user-guidance-owner";
    let workspace_id = "ws-guidance-owner";
    let task_id = "task-guidance-owner";
    seed_user_workspace_task(&state.db, owner_id, workspace_id, task_id).await;

    let other_id = "user-guidance-other";
    seed_user_workspace_task(
        &state.db,
        other_id,
        "ws-guidance-other",
        "task-guidance-other",
    )
    .await;

    let token = state
        .session_store
        .create_session(other_id.to_string(), None)
        .await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("serve");
    });

    let ws_url = format!("ws://{}/api/v1/ws?token={}", addr, token);
    let (mut socket, _) = connect_async(ws_url).await.expect("connect ws");

    let cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": "请更正式" },
        "correlationId": "cid-guidance-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            cmd.to_string(),
        ))
        .await
        .expect("send guidance");

    let ack = read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    assert_eq!(ack["payload"]["ok"], false);
    assert_eq!(
        ack["payload"]["reason"].as_str(),
        Some("task_not_found_or_forbidden")
    );
}

#[tokio::test]
async fn ws_guidance_send_validates_input() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-guidance-validate";
    let workspace_id = "ws-guidance-validate";
    let task_id = "task-guidance-validate";
    seed_user_workspace_task(&state.db, user_id, workspace_id, task_id).await;

    let token = state
        .session_store
        .create_session(user_id.to_string(), None)
        .await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("serve");
    });

    let ws_url = format!("ws://{}/api/v1/ws?token={}", addr, token);
    let (mut socket, _) = connect_async(ws_url).await.expect("connect ws");

    let controller = global_pause_registry().get_or_create(task_id).await;
    controller
        .request_pause("cid-guidance-validate", user_id)
        .await;
    controller
        .checkpoint_pause(
            1,
            "running_tests",
            Some("cid-guidance-validate"),
            serde_json::json!({ "artifacts": { "patterns": [], "candidatePrompts": [], "updatedAt": "2026-01-17T00:00:00Z" } }),
        )
        .await
        .expect("checkpoint pause");

    let empty_cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": "   " },
        "correlationId": "cid-guidance-validate-1"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            empty_cmd.to_string(),
        ))
        .await
        .expect("send guidance empty");

    let empty_ack = read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    assert_eq!(empty_ack["payload"]["ok"], false);
    assert!(
        empty_ack["payload"]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("不能为空")
    );

    let too_long = "a".repeat(UserGuidance::MAX_CONTENT_LENGTH + 1);
    let long_cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": too_long },
        "correlationId": "cid-guidance-validate-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            long_cmd.to_string(),
        ))
        .await
        .expect("send guidance long");

    let long_ack = read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    assert_eq!(long_ack["payload"]["ok"], false);
    assert!(
        long_ack["payload"]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("最大长度")
    );
}

#[tokio::test]
async fn ws_guidance_last_one_wins() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-guidance-last";
    let workspace_id = "ws-guidance-last";
    let task_id = "task-guidance-last";
    seed_user_workspace_task(&state.db, user_id, workspace_id, task_id).await;

    let token = state
        .session_store
        .create_session(user_id.to_string(), None)
        .await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("serve");
    });

    let ws_url = format!("ws://{}/api/v1/ws?token={}", addr, token);
    let (mut socket, _) = connect_async(ws_url).await.expect("connect ws");

    let controller = global_pause_registry().get_or_create(task_id).await;
    controller.request_pause("cid-guidance-last", user_id).await;
    controller
        .checkpoint_pause(
            1,
            "running_tests",
            Some("cid-guidance-last"),
            serde_json::json!({ "artifacts": { "patterns": [], "candidatePrompts": [], "updatedAt": "2026-01-17T00:00:00Z" } }),
        )
        .await
        .expect("checkpoint pause");

    let first_cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": "第一次引导" },
        "correlationId": "cid-guidance-last-1"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            first_cmd.to_string(),
        ))
        .await
        .expect("send guidance first");
    let _ = read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    let _ = read_message_of_type_for_task(&mut socket, "guidance:sent", task_id).await;

    let second_cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": "第二次引导" },
        "correlationId": "cid-guidance-last-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            second_cmd.to_string(),
        ))
        .await
        .expect("send guidance second");
    let _ = read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    let _ = read_message_of_type_for_task(&mut socket, "guidance:sent", task_id).await;

    let current = controller.get_guidance().await.expect("guidance exists");
    assert_eq!(current.content, "第二次引导");
}

#[tokio::test]
async fn ws_pause_send_guidance_resume_applies_and_clears() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-guidance-flow";
    let workspace_id = "ws-guidance-flow";
    let task_id = "task-guidance-flow";
    seed_user_workspace_task(&state.db, user_id, workspace_id, task_id).await;

    let token = state
        .session_store
        .create_session(user_id.to_string(), None)
        .await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .expect("serve");
    });

    let ws_url = format!("ws://{}/api/v1/ws?token={}", addr, token);
    let (mut socket, _) = connect_async(ws_url).await.expect("connect ws");

    let pause_cmd = serde_json::json!({
        "type": "task:pause",
        "payload": { "taskId": task_id },
        "correlationId": "cid-guidance-flow-1"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            pause_cmd.to_string(),
        ))
        .await
        .expect("send pause");

    let pause_ack = read_message_of_type_for_task(&mut socket, "task:pause:ack", task_id).await;
    assert_eq!(pause_ack["payload"]["ok"], true);

    let ctx = build_test_context(task_id);
    let engine_task = tokio::spawn(async move {
        let mut ctx = ctx;
        let paused = checkpoint_pause_if_requested(&mut ctx)
            .await
            .expect("checkpoint pause");
        (paused, ctx)
    });

    let controller = global_pause_registry().get_or_create(task_id).await;
    wait_for_pause(&controller).await;

    let guidance_cmd = serde_json::json!({
        "type": "guidance:send",
        "payload": { "taskId": task_id, "content": "请更正式" },
        "correlationId": "cid-guidance-flow-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            guidance_cmd.to_string(),
        ))
        .await
        .expect("send guidance");

    let guidance_ack =
        read_message_of_type_for_task(&mut socket, "guidance:send:ack", task_id).await;
    assert_eq!(guidance_ack["payload"]["ok"], true);
    let sent_evt = read_message_of_type_for_task(&mut socket, "guidance:sent", task_id).await;
    assert_eq!(sent_evt["payload"]["taskId"], task_id);
    assert_eq!(sent_evt["payload"]["status"], "pending");
    assert!(
        sent_evt["payload"]["contentPreview"]
            .as_str()
            .unwrap_or("")
            .contains("请更正式")
    );

    let resume_cmd = serde_json::json!({
        "type": "task:resume",
        "payload": { "taskId": task_id },
        "correlationId": "cid-guidance-flow-3"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            resume_cmd.to_string(),
        ))
        .await
        .expect("send resume");

    let resume_ack = read_message_of_type_for_task(&mut socket, "task:resume:ack", task_id).await;
    assert_eq!(resume_ack["payload"]["ok"], true);

    let applied_evt = read_message_of_type_for_task(&mut socket, "guidance:applied", task_id).await;
    assert_eq!(applied_evt["payload"]["taskId"], task_id);
    assert_eq!(applied_evt["payload"]["iteration"], 2);

    let (paused, mut updated_ctx) = tokio::time::timeout(Duration::from_secs(5), engine_task)
        .await
        .expect("resume timeout")
        .expect("engine task join");
    assert!(paused);

    let guidance = updated_ctx
        .extensions
        .get(EXT_USER_GUIDANCE)
        .cloned()
        .and_then(|v| serde_json::from_value::<UserGuidance>(v).ok())
        .expect("guidance injected");
    assert_eq!(guidance.content, "请更正式");
    assert!(matches!(
        guidance.status,
        prompt_faster::domain::types::GuidanceStatus::Applied
    ));

    let controller = global_pause_registry().get_or_create(task_id).await;
    assert!(controller.get_guidance().await.is_none());

    clear_user_guidance_from_context(&mut updated_ctx);
    assert!(updated_ctx.extensions.get(EXT_USER_GUIDANCE).is_none());
}
