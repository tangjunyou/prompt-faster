//! WebSocket 连接管理

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tracing::{info, warn};

use crate::api::state::AppState;
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::domain::types::RunControlState;
use crate::infra::db::repositories::{OptimizationTaskRepo, OptimizationTaskRepoError};
use crate::shared::ws::{
    IterationPausedPayload, TaskControlAckPayload, TaskControlPayload, WsMessage, CMD_TASK_PAUSE,
    CMD_TASK_RESUME, EVT_ITERATION_PAUSED, EVT_TASK_PAUSE_ACK, EVT_TASK_RESUME_ACK,
};
use crate::shared::ws_bus::global_ws_bus;

#[derive(Debug, Deserialize)]
struct WsQuery {
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientCommand<T> {
    #[serde(rename = "type")]
    event_type: String,
    payload: T,
    correlation_id: String,
}

fn extract_token(headers: &HeaderMap, query: &WsQuery) -> Option<String> {
    if let Some(token) = query.token.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        return Some(token.to_string());
    }

    let auth_header = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    auth_header
        .strip_prefix("Bearer ")
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

async fn validate_task_ownership(
    state: &AppState,
    user_id: &str,
    task_id: &str,
) -> Result<(), OptimizationTaskRepoError> {
    OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, task_id).await?;
    Ok(())
}

fn ack_event_type(event_type: &str) -> Option<&'static str> {
    match event_type {
        CMD_TASK_PAUSE => Some(EVT_TASK_PAUSE_ACK),
        CMD_TASK_RESUME => Some(EVT_TASK_RESUME_ACK),
        _ => None,
    }
}

async fn handle_command(
    state: &AppState,
    user_id: &str,
    text: &str,
    ack_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
) {
    let cmd: ClientCommand<TaskControlPayload> = match serde_json::from_str(text) {
        Ok(cmd) => cmd,
        Err(err) => {
            warn!(error = %err, "WS command parse failed");
            return;
        }
    };

    if cmd.correlation_id.trim().is_empty() {
        warn!("WS command missing correlationId");
        return;
    }

    let task_id = cmd.payload.task_id.trim().to_string();
    if task_id.is_empty() {
        warn!(correlation_id = %cmd.correlation_id, user_id = %user_id, "WS command missing taskId");
        if let Some(event_type) = ack_event_type(cmd.event_type.as_str()) {
            let ack = TaskControlAckPayload {
                task_id: "".to_string(),
                ok: false,
                applied: false,
                current_state: RunControlState::Idle,
                target_state: RunControlState::Idle,
                reason: Some("missing_task_id".to_string()),
                context_snapshot: None,
            };
            let msg = WsMessage::new(event_type, ack, cmd.correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ack_sender.send(Message::Text(text.into()));
            }
        }
        return;
    }

    if let Err(err) = validate_task_ownership(state, user_id, &task_id).await {
        warn!(
            correlation_id = %cmd.correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            error = %err,
            "WS command rejected: task ownership check failed"
        );
        if let Some(event_type) = ack_event_type(cmd.event_type.as_str()) {
            let ack = TaskControlAckPayload {
                task_id: task_id.clone(),
                ok: false,
                applied: false,
                current_state: RunControlState::Idle,
                target_state: RunControlState::Idle,
                reason: Some("task_not_found_or_forbidden".to_string()),
                context_snapshot: None,
            };
            let msg = WsMessage::new(event_type, ack, cmd.correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ack_sender.send(Message::Text(text.into()));
            }
        }
        return;
    }

    let registry = global_pause_registry();
    let controller = registry.get_or_create(&task_id).await;
    let prev_state = if controller.is_paused() {
        RunControlState::Paused
    } else {
        RunControlState::Running
    };
    let iteration_state = controller
        .get_snapshot()
        .await
        .map(|s| s.stage)
        .unwrap_or_else(|| "unknown".to_string());
    let context_snapshot = controller
        .get_snapshot()
        .await
        .map(|s| s.context_snapshot);

    match cmd.event_type.as_str() {
        CMD_TASK_PAUSE => {
            let accepted = controller
                .request_pause(&cmd.correlation_id, user_id)
                .await;
            let current_state = if controller.is_paused() {
                RunControlState::Paused
            } else {
                RunControlState::Running
            };
            let new_state = if accepted { RunControlState::Paused } else { prev_state };
            info!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                action = "pause",
                prev_state = ?prev_state,
                new_state = ?new_state,
                iteration_state = %iteration_state,
                accepted = accepted,
                "WS pause request"
            );
            if let Some(event_type) = ack_event_type(cmd.event_type.as_str()) {
                let reason = if accepted {
                    None
                } else if controller.is_paused() {
                    Some("already_paused".to_string())
                } else {
                    Some("already_requested".to_string())
                };
                let ack = TaskControlAckPayload {
                    task_id: task_id.clone(),
                    ok: true,
                    applied: accepted,
                    current_state,
                    target_state: RunControlState::Paused,
                    reason,
                    context_snapshot,
                };
                let msg = WsMessage::new(event_type, ack, cmd.correlation_id.clone());
                if let Ok(text) = serde_json::to_string(&msg) {
                    let _ = ack_sender.send(Message::Text(text.into()));
                }
            }
        }
        CMD_TASK_RESUME => {
            let accepted = controller
                .request_resume(&cmd.correlation_id, user_id)
                .await;
            let current_state = if controller.is_paused() {
                RunControlState::Paused
            } else {
                RunControlState::Running
            };
            let new_state = if accepted { RunControlState::Running } else { prev_state };
            info!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                action = "resume",
                prev_state = ?prev_state,
                new_state = ?new_state,
                iteration_state = %iteration_state,
                accepted = accepted,
                "WS resume request"
            );
            if let Some(event_type) = ack_event_type(cmd.event_type.as_str()) {
                let reason = if accepted {
                    None
                } else {
                    Some("not_paused".to_string())
                };
                let ack = TaskControlAckPayload {
                    task_id: task_id.clone(),
                    ok: true,
                    applied: accepted,
                    current_state,
                    target_state: RunControlState::Running,
                    reason,
                    context_snapshot,
                };
                let msg = WsMessage::new(event_type, ack, cmd.correlation_id.clone());
                if let Ok(text) = serde_json::to_string(&msg) {
                    let _ = ack_sender.send(Message::Text(text.into()));
                }
            }
        }
        other => {
            warn!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                event_type = %other,
                "WS command ignored: unsupported type"
            );
            if let Some(event_type) = ack_event_type(other) {
                let ack = TaskControlAckPayload {
                    task_id: task_id.clone(),
                    ok: false,
                    applied: false,
                    current_state: RunControlState::Idle,
                    target_state: RunControlState::Idle,
                    reason: Some("unsupported_type".to_string()),
                    context_snapshot: None,
                };
                let msg = WsMessage::new(event_type, ack, cmd.correlation_id.clone());
                if let Ok(text) = serde_json::to_string(&msg) {
                    let _ = ack_sender.send(Message::Text(text.into()));
                }
            }
        }
    }
}

async fn send_paused_snapshots(
    sender: &tokio::sync::mpsc::UnboundedSender<Message>,
) -> bool {
    let registry = global_pause_registry();
    let snapshots = registry.get_all_paused_snapshots().await;

    for snapshot in snapshots {
        let payload = IterationPausedPayload {
            task_id: snapshot.task_id.clone(),
            paused_at: snapshot.paused_at.clone(),
            stage: snapshot.stage.clone(),
            iteration: snapshot.iteration,
        };
        let msg = WsMessage::new(EVT_ITERATION_PAUSED, payload, snapshot.correlation_id.clone());
        let Ok(text) = serde_json::to_string(&msg) else {
            continue;
        };
        if sender.send(Message::Text(text.into())).is_err() {
            return false;
        }
    }

    true
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: String,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    if !send_paused_snapshots(&tx).await {
        return;
    }

    let mut bus_rx = global_ws_bus().subscribe();
    let tx_for_bus = tx.clone();
    let send_task = tokio::spawn(async move {
        loop {
            match bus_rx.recv().await {
                Ok(msg) => {
                    if tx_for_bus.send(Message::Text(msg.into())).is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });

    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => handle_command(&state, &user_id, &text, &tx).await,
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    write_task.abort();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
) -> Response {
    let Some(token) = extract_token(&headers, &query) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let session = match state.session_store.validate_session(&token).await {
        Some(session) => session,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let user_id = session.user_id;
    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
        .into_response()
}

pub fn router() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}
