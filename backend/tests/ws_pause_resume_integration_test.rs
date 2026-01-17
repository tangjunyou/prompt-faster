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
use prompt_faster::core::optimization_engine::checkpoint_pause_if_requested;
use prompt_faster::core::iteration_engine::pause_state::global_pause_registry;
use prompt_faster::domain::models::{IterationState, OutputLength, Rule, RuleSystem, RuleTags};
use prompt_faster::domain::types::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, RunControlState,
    EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT,
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

async fn read_message_of_type<S>(socket: &mut WebSocketStream<S>, target_type: &str) -> Value
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let msg = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(Ok(frame)) = socket.next().await {
            if let tokio_tungstenite::tungstenite::Message::Text(text) = frame {
                let value: Value = serde_json::from_str(&text).expect("parse message");
                if value.get("type").and_then(|v| v.as_str()) == Some(target_type) {
                    return value;
                }
            }
        }
        panic!("socket closed before receiving {target_type}");
    })
    .await
    .expect("timeout waiting for message");
    msg
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

fn sample_artifacts_payload() -> Value {
    serde_json::json!({
        "patterns": [
            {
                "id": "p1",
                "pattern": "rule-1",
                "source": "system",
                "confidence": null
            }
        ],
        "candidatePrompts": [
            {
                "id": "c1",
                "content": "prompt-1",
                "source": "system",
                "score": null,
                "isBest": true
            }
        ],
        "updatedAt": "2026-01-17T00:00:00Z"
    })
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
        iteration: 1,
        state: IterationState::RunningTests,
        run_control_state: RunControlState::Running,
        test_cases: vec![],
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions,
    }
}

#[tokio::test]
async fn ws_pause_resume_ack_and_events() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-ws-1";
    let workspace_id = "ws-1";
    let task_id = "task-1";
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

    // 发送 pause 命令
    let pause_cmd = serde_json::json!({
        "type": "task:pause",
        "payload": { "taskId": task_id },
        "correlationId": "cid-1"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            pause_cmd.to_string(),
        ))
        .await
        .expect("send pause");

    let pause_ack = read_message_of_type(&mut socket, "task:pause:ack").await;
    assert_eq!(pause_ack["correlationId"], "cid-1");
    assert_eq!(pause_ack["payload"]["ok"], true);
    assert_eq!(pause_ack["payload"]["applied"], true);
    assert_eq!(pause_ack["payload"]["targetState"], "paused");

    // 模拟安全点暂停
    let controller = global_pause_registry().get_or_create(task_id).await;
    controller
        .checkpoint_pause(
            1,
            "running_tests",
            Some("cid-1"),
            serde_json::json!({
                "taskId": task_id,
                "iteration": 1,
                "iterationState": "running_tests",
                "runControlState": "paused"
            }),
        )
        .await
        .expect("checkpoint pause");

    let paused_evt =
        read_message_of_type_for_task(&mut socket, "iteration:paused", task_id).await;
    assert_eq!(paused_evt["payload"]["taskId"], task_id);
    assert_eq!(paused_evt["correlationId"], "cid-1");

    // 发送 resume 命令
    let resume_cmd = serde_json::json!({
        "type": "task:resume",
        "payload": { "taskId": task_id },
        "correlationId": "cid-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            resume_cmd.to_string(),
        ))
        .await
        .expect("send resume");

    let resume_ack = read_message_of_type(&mut socket, "task:resume:ack").await;
    assert_eq!(resume_ack["correlationId"], "cid-2");
    assert_eq!(resume_ack["payload"]["ok"], true);
    assert_eq!(resume_ack["payload"]["applied"], true);
    assert_eq!(resume_ack["payload"]["targetState"], "running");

    let resumed_evt = read_message_of_type(&mut socket, "iteration:resumed").await;
    assert_eq!(resumed_evt["payload"]["taskId"], task_id);
    assert_eq!(resumed_evt["correlationId"], "cid-2");
}

#[tokio::test]
async fn ws_artifact_get_and_update_ack() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-ws-artifact";
    let workspace_id = "ws-artifact";
    let task_id = "task-artifact";
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
    controller.request_pause("cid-1", user_id).await;
    controller
        .checkpoint_pause(
            1,
            "running_tests",
            Some("cid-1"),
            serde_json::json!({
                "taskId": task_id,
                "iteration": 1,
                "iterationState": "running_tests",
                "runControlState": "paused",
                "artifacts": sample_artifacts_payload(),
            }),
        )
        .await
        .expect("checkpoint pause");

    let get_cmd = serde_json::json!({
        "type": "artifact:get",
        "payload": { "taskId": task_id },
        "correlationId": "cid-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            get_cmd.to_string(),
        ))
        .await
        .expect("send artifact:get");

    let get_ack = read_message_of_type(&mut socket, "artifact:get:ack").await;
    assert_eq!(get_ack["correlationId"], "cid-2");
    assert_eq!(get_ack["payload"]["ok"], true);
    assert_eq!(get_ack["payload"]["artifacts"]["patterns"][0]["id"], "p1");

    let update_cmd = serde_json::json!({
        "type": "artifact:update",
        "payload": { "taskId": task_id, "artifacts": sample_artifacts_payload() },
        "correlationId": "cid-3"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            update_cmd.to_string(),
        ))
        .await
        .expect("send artifact:update");

    let update_ack = read_message_of_type(&mut socket, "artifact:update:ack").await;
    assert_eq!(update_ack["correlationId"], "cid-3");
    assert_eq!(update_ack["payload"]["ok"], true);
    assert_eq!(update_ack["payload"]["applied"], true);
}

#[tokio::test]
async fn ws_artifact_update_rejected_when_not_paused_or_owner() {
    let (app, state) = setup_test_app_with_db().await;

    let owner_id = "user-ws-owner";
    let workspace_id = "ws-owner";
    let task_id = "task-owner";
    seed_user_workspace_task(&state.db, owner_id, workspace_id, task_id).await;

    let other_id = "user-ws-other";
    seed_user_workspace_task(&state.db, other_id, "ws-other", "task-other").await;

    let other_token = state
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

    let ws_url = format!("ws://{}/api/v1/ws?token={}", addr, other_token);
    let (mut socket, _) = connect_async(ws_url).await.expect("connect ws");

    let update_cmd = serde_json::json!({
        "type": "artifact:update",
        "payload": { "taskId": task_id, "artifacts": sample_artifacts_payload() },
        "correlationId": "cid-4"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            update_cmd.to_string(),
        ))
        .await
        .expect("send artifact:update");

    let update_ack = read_message_of_type(&mut socket, "artifact:update:ack").await;
    assert_eq!(update_ack["payload"]["ok"], false);
    assert_eq!(
        update_ack["payload"]["reason"].as_str(),
        Some("task_not_found_or_forbidden")
    );
}

#[tokio::test]
async fn ws_artifact_update_rejected_when_not_paused() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-ws-not-paused";
    let workspace_id = "ws-not-paused";
    let task_id = "task-not-paused";
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

    let update_cmd = serde_json::json!({
        "type": "artifact:update",
        "payload": { "taskId": task_id, "artifacts": sample_artifacts_payload() },
        "correlationId": "cid-5"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            update_cmd.to_string(),
        ))
        .await
        .expect("send artifact:update");

    let update_ack = read_message_of_type(&mut socket, "artifact:update:ack").await;
    assert_eq!(update_ack["payload"]["ok"], false);
    assert_eq!(update_ack["payload"]["reason"].as_str(), Some("task_not_paused"));
}

#[tokio::test]
async fn ws_pause_edit_resume_flow_applies_artifacts() {
    let (app, state) = setup_test_app_with_db().await;

    let user_id = "user-ws-flow";
    let workspace_id = "ws-flow";
    let task_id = "task-flow";
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
    controller.reset().await;

    let pause_cmd = serde_json::json!({
        "type": "task:pause",
        "payload": { "taskId": task_id },
        "correlationId": "cid-flow-1"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            pause_cmd.to_string(),
        ))
        .await
        .expect("send pause");

    let pause_ack = read_message_of_type(&mut socket, "task:pause:ack").await;
    assert_eq!(pause_ack["payload"]["ok"], true);

    let ctx = build_test_context(task_id);
    let engine_task = tokio::spawn(async move {
        let mut ctx = ctx;
        let paused = checkpoint_pause_if_requested(&mut ctx)
            .await
            .expect("checkpoint pause");
        (paused, ctx)
    });

    let paused_evt =
        read_message_of_type_for_task(&mut socket, "iteration:paused", task_id).await;
    assert_eq!(paused_evt["payload"]["taskId"], task_id);

    let update_cmd = serde_json::json!({
        "type": "artifact:update",
        "payload": {
            "taskId": task_id,
            "artifacts": {
                "patterns": [
                    {
                        "id": "rule-1",
                        "pattern": "edited-rule",
                        "source": "user_edited",
                        "confidence": 0.9
                    }
                ],
                "candidatePrompts": [
                    {
                        "id": "current",
                        "content": "current-prompt",
                        "source": "system",
                        "score": null,
                        "isBest": false
                    },
                    {
                        "id": "candidate:2",
                        "content": "edited-prompt",
                        "source": "user_edited",
                        "score": 0.7,
                        "isBest": true
                    }
                ],
                "updatedAt": "2026-01-17T00:00:00Z"
            }
        },
        "correlationId": "cid-flow-2"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            update_cmd.to_string(),
        ))
        .await
        .expect("send artifact:update");

    let update_ack = read_message_of_type(&mut socket, "artifact:update:ack").await;
    assert_eq!(update_ack["payload"]["ok"], true);

    let resume_cmd = serde_json::json!({
        "type": "task:resume",
        "payload": { "taskId": task_id },
        "correlationId": "cid-flow-3"
    });
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            resume_cmd.to_string(),
        ))
        .await
        .expect("send resume");

    let resume_ack = read_message_of_type(&mut socket, "task:resume:ack").await;
    assert_eq!(resume_ack["payload"]["ok"], true);

    let (paused, updated_ctx) = tokio::time::timeout(Duration::from_secs(5), engine_task)
        .await
        .expect("resume timeout")
        .expect("engine task join");

    assert!(paused);
    assert_eq!(updated_ctx.rule_system.rules[0].description, "edited-rule");
    assert_eq!(updated_ctx.current_prompt, "edited-prompt");
    assert_eq!(updated_ctx.run_control_state, RunControlState::Running);
    assert_eq!(
        updated_ctx
            .extensions
            .get(EXT_BEST_CANDIDATE_PROMPT)
            .and_then(|v| v.as_str()),
        Some("edited-prompt")
    );
    assert_eq!(
        updated_ctx
            .extensions
            .get(EXT_BEST_CANDIDATE_INDEX)
            .and_then(|v| v.as_u64()),
        Some(2)
    );
}
