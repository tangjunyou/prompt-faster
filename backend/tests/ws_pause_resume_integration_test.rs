use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;

use prompt_faster::api::state::AppState;
use prompt_faster::api::ws;
use prompt_faster::core::iteration_engine::pause_state::global_pause_registry;
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
    .bind("ws_test_user")
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

    let paused_evt = read_message_of_type(&mut socket, "iteration:paused").await;
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
