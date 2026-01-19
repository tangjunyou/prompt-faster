//! WebSocket 连接管理

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tracing::{info, warn};

use crate::api::state::AppState;
use crate::core::iteration_engine::events::record_event_async;
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::domain::models::{Actor, EventType};
use crate::domain::types::RunControlState;
use crate::infra::db::repositories::{OptimizationTaskRepo, OptimizationTaskRepoError};
use crate::shared::ws::{
    ArtifactGetAckPayload, ArtifactGetPayload, ArtifactUpdateAckPayload, ArtifactUpdatePayload,
    ArtifactUpdatedPayload, CMD_ARTIFACT_GET, CMD_ARTIFACT_UPDATE, CMD_GUIDANCE_SEND,
    CMD_TASK_PAUSE, CMD_TASK_RESUME, EVT_ARTIFACT_GET_ACK, EVT_ARTIFACT_UPDATE_ACK,
    EVT_ARTIFACT_UPDATED, EVT_GUIDANCE_SEND_ACK, EVT_GUIDANCE_SENT, EVT_ITERATION_PAUSED,
    EVT_TASK_PAUSE_ACK, EVT_TASK_RESUME_ACK, GuidanceSendAckPayload, GuidanceSendPayload,
    GuidanceSentPayload, IterationPausedPayload, TaskControlAckPayload, TaskControlPayload,
    WsMessage,
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
    if let Some(token) = query
        .token
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
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
        CMD_ARTIFACT_GET => Some(EVT_ARTIFACT_GET_ACK),
        CMD_ARTIFACT_UPDATE => Some(EVT_ARTIFACT_UPDATE_ACK),
        CMD_GUIDANCE_SEND => Some(EVT_GUIDANCE_SEND_ACK),
        _ => None,
    }
}

async fn handle_command(
    state: &AppState,
    user_id: &str,
    text: &str,
    ack_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
) {
    // 先解析基础结构获取命令类型
    let base: ClientCommand<serde_json::Value> = match serde_json::from_str(text) {
        Ok(cmd) => cmd,
        Err(err) => {
            warn!(error = %err, "WS command parse failed");
            return;
        }
    };

    // 根据命令类型分发处理
    match base.event_type.as_str() {
        CMD_ARTIFACT_GET => {
            handle_artifact_get(state, user_id, text, ack_sender).await;
            return;
        }
        CMD_ARTIFACT_UPDATE => {
            handle_artifact_update(state, user_id, text, ack_sender).await;
            return;
        }
        CMD_GUIDANCE_SEND => {
            handle_guidance_send(state, user_id, text, ack_sender).await;
            return;
        }
        _ => {}
    }

    // 处理任务控制命令（pause/resume）
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
    let snapshot = controller.get_snapshot().await;
    let iteration_state = snapshot
        .as_ref()
        .map(|s| s.stage.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let context_snapshot = snapshot.as_ref().map(|s| s.context_snapshot.clone());
    let snapshot_iteration = snapshot.as_ref().map(|s| s.iteration);

    match cmd.event_type.as_str() {
        CMD_TASK_PAUSE => {
            let accepted = controller.request_pause(&cmd.correlation_id, user_id).await;
            let current_state = if controller.is_paused() {
                RunControlState::Paused
            } else {
                RunControlState::Running
            };
            let new_state = if accepted {
                RunControlState::Paused
            } else {
                prev_state
            };
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
            if accepted {
                record_event_async(
                    task_id.clone(),
                    EventType::UserPause,
                    Actor::User,
                    Some(serde_json::json!({ "source": "ws" })),
                    snapshot_iteration,
                    Some(cmd.correlation_id.clone()),
                );
            }
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
            let new_state = if accepted {
                RunControlState::Running
            } else {
                prev_state
            };
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
            if accepted {
                record_event_async(
                    task_id.clone(),
                    EventType::UserResume,
                    Actor::User,
                    Some(serde_json::json!({ "source": "ws" })),
                    snapshot_iteration,
                    Some(cmd.correlation_id.clone()),
                );
            }
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

/// 处理 artifact:get 命令
async fn handle_artifact_get(
    state: &AppState,
    user_id: &str,
    text: &str,
    ack_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
) {
    let cmd: ClientCommand<ArtifactGetPayload> = match serde_json::from_str(text) {
        Ok(cmd) => cmd,
        Err(err) => {
            warn!(error = %err, "artifact:get command parse failed");
            return;
        }
    };

    if cmd.correlation_id.trim().is_empty() {
        warn!("artifact:get command missing correlationId");
        return;
    }

    let task_id = cmd.payload.task_id.trim().to_string();
    if task_id.is_empty() {
        let ack = ArtifactGetAckPayload {
            task_id: "".to_string(),
            ok: false,
            artifacts: None,
            reason: Some("missing_task_id".to_string()),
        };
        let msg = WsMessage::new(EVT_ARTIFACT_GET_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    // 权限校验
    if let Err(err) = validate_task_ownership(state, user_id, &task_id).await {
        warn!(
            correlation_id = %cmd.correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            error = %err,
            "artifact:get rejected: task ownership check failed"
        );
        let ack = ArtifactGetAckPayload {
            task_id: task_id.clone(),
            ok: false,
            artifacts: None,
            reason: Some("task_not_found_or_forbidden".to_string()),
        };
        let msg = WsMessage::new(EVT_ARTIFACT_GET_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    let registry = global_pause_registry();
    let controller = registry.get_or_create(&task_id).await;

    // 获取产物
    let artifacts = controller.get_artifacts().await;

    info!(
        correlation_id = %cmd.correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        has_artifacts = artifacts.is_some(),
        "artifact:get request"
    );

    let ack = ArtifactGetAckPayload {
        task_id: task_id.clone(),
        ok: true,
        artifacts,
        reason: None,
    };
    let msg = WsMessage::new(EVT_ARTIFACT_GET_ACK, ack, cmd.correlation_id.clone());
    if let Ok(text) = serde_json::to_string(&msg) {
        let _ = ack_sender.send(Message::Text(text.into()));
    }
}

/// 处理 artifact:update 命令
async fn handle_artifact_update(
    state: &AppState,
    user_id: &str,
    text: &str,
    ack_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
) {
    let cmd: ClientCommand<ArtifactUpdatePayload> = match serde_json::from_str(text) {
        Ok(cmd) => cmd,
        Err(err) => {
            warn!(error = %err, "artifact:update command parse failed");
            return;
        }
    };

    if cmd.correlation_id.trim().is_empty() {
        warn!("artifact:update command missing correlationId");
        return;
    }

    let task_id = cmd.payload.task_id.trim().to_string();
    if task_id.is_empty() {
        let ack = ArtifactUpdateAckPayload {
            task_id: "".to_string(),
            ok: false,
            applied: false,
            artifacts: None,
            reason: Some("missing_task_id".to_string()),
        };
        let msg = WsMessage::new(EVT_ARTIFACT_UPDATE_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    // 权限校验
    if let Err(err) = validate_task_ownership(state, user_id, &task_id).await {
        warn!(
            correlation_id = %cmd.correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            error = %err,
            "artifact:update rejected: task ownership check failed"
        );
        let ack = ArtifactUpdateAckPayload {
            task_id: task_id.clone(),
            ok: false,
            applied: false,
            artifacts: None,
            reason: Some("task_not_found_or_forbidden".to_string()),
        };
        let msg = WsMessage::new(EVT_ARTIFACT_UPDATE_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    let registry = global_pause_registry();
    let controller = registry.get_or_create(&task_id).await;
    let snapshot_iteration = controller.get_snapshot().await.map(|s| s.iteration);

    // 状态二次校验：必须处于 Paused 状态
    if !controller.is_paused() {
        warn!(
            correlation_id = %cmd.correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            "artifact:update rejected: task not paused"
        );
        let ack = ArtifactUpdateAckPayload {
            task_id: task_id.clone(),
            ok: false,
            applied: false,
            artifacts: None,
            reason: Some("task_not_paused".to_string()),
        };
        let msg = WsMessage::new(EVT_ARTIFACT_UPDATE_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    // 执行更新
    match controller
        .update_artifacts(&cmd.payload.artifacts, &cmd.correlation_id, user_id)
        .await
    {
        Ok(updated_artifacts) => {
            info!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                "artifact:update success"
            );
            record_event_async(
                task_id.clone(),
                EventType::UserEdit,
                Actor::User,
                Some(serde_json::json!({
                    "field": "artifacts",
                    "edit_type": "manual",
                })),
                snapshot_iteration,
                Some(cmd.correlation_id.clone()),
            );

            // 发送 ACK
            let ack = ArtifactUpdateAckPayload {
                task_id: task_id.clone(),
                ok: true,
                applied: true,
                artifacts: Some(updated_artifacts.clone()),
                reason: None,
            };
            let msg = WsMessage::new(EVT_ARTIFACT_UPDATE_ACK, ack, cmd.correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ack_sender.send(Message::Text(text.into()));
            }

            // 广播 artifact:updated 事件
            let broadcast_payload = ArtifactUpdatedPayload {
                task_id: task_id.clone(),
                artifacts: updated_artifacts,
                edited_by: user_id.to_string(),
            };
            let broadcast_msg = WsMessage::new(
                EVT_ARTIFACT_UPDATED,
                broadcast_payload,
                cmd.correlation_id.clone(),
            );
            if let Ok(text) = serde_json::to_string(&broadcast_msg) {
                global_ws_bus().publish(text);
            }
        }
        Err(err) => {
            warn!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                error = %err,
                "artifact:update failed"
            );
            let ack = ArtifactUpdateAckPayload {
                task_id: task_id.clone(),
                ok: false,
                applied: false,
                artifacts: None,
                reason: Some(err.to_string()),
            };
            let msg = WsMessage::new(EVT_ARTIFACT_UPDATE_ACK, ack, cmd.correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ack_sender.send(Message::Text(text.into()));
            }
        }
    }
}

/// 处理 guidance:send 命令
async fn handle_guidance_send(
    state: &AppState,
    user_id: &str,
    text: &str,
    ack_sender: &tokio::sync::mpsc::UnboundedSender<Message>,
) {
    let cmd: ClientCommand<GuidanceSendPayload> = match serde_json::from_str(text) {
        Ok(cmd) => cmd,
        Err(err) => {
            warn!(error = %err, "guidance:send command parse failed");
            return;
        }
    };

    if cmd.correlation_id.trim().is_empty() {
        warn!("guidance:send command missing correlationId");
        return;
    }

    let task_id = cmd.payload.task_id.trim().to_string();
    if task_id.is_empty() {
        let ack = GuidanceSendAckPayload {
            task_id: "".to_string(),
            ok: false,
            guidance_id: None,
            status: None,
            reason: Some("missing_task_id".to_string()),
        };
        let msg = WsMessage::new(EVT_GUIDANCE_SEND_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    // 权限校验
    if let Err(err) = validate_task_ownership(state, user_id, &task_id).await {
        warn!(
            correlation_id = %cmd.correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            error = %err,
            "guidance:send rejected: task ownership check failed"
        );
        let ack = GuidanceSendAckPayload {
            task_id: task_id.clone(),
            ok: false,
            guidance_id: None,
            status: None,
            reason: Some("task_not_found_or_forbidden".to_string()),
        };
        let msg = WsMessage::new(EVT_GUIDANCE_SEND_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    let registry = global_pause_registry();
    let controller = registry.get_or_create(&task_id).await;

    // 状态校验：必须处于 Paused 状态
    if !controller.is_paused() {
        warn!(
            correlation_id = %cmd.correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            "guidance:send rejected: task not paused"
        );
        let ack = GuidanceSendAckPayload {
            task_id: task_id.clone(),
            ok: false,
            guidance_id: None,
            status: None,
            reason: Some("task_not_paused".to_string()),
        };
        let msg = WsMessage::new(EVT_GUIDANCE_SEND_ACK, ack, cmd.correlation_id.clone());
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = ack_sender.send(Message::Text(text.into()));
        }
        return;
    }

    let snapshot = controller.get_snapshot().await;
    let iteration_state = snapshot
        .as_ref()
        .map(|s| s.stage.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let snapshot_iteration = snapshot.as_ref().map(|s| s.iteration);
    let timestamp = crate::shared::ws::chrono_timestamp();

    // 执行引导更新
    match controller
        .update_guidance(&cmd.payload.content, &cmd.correlation_id, user_id)
        .await
    {
        Ok(guidance) => {
            info!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                action = "guidance_send",
                prev_state = ?RunControlState::Paused,
                new_state = ?RunControlState::Paused,
                iteration_state = %iteration_state,
                timestamp = %timestamp,
                guidance_id = %guidance.id,
                guidance_preview = %guidance.content_preview(),
                "guidance:send success"
            );
            record_event_async(
                task_id.clone(),
                EventType::UserGuidance,
                Actor::User,
                Some(serde_json::json!({
                    "field": "user_guidance",
                    "edit_type": "manual",
                    "guidance_id": guidance.id,
                })),
                snapshot_iteration,
                Some(cmd.correlation_id.clone()),
            );

            // 发送 ACK
            let ack = GuidanceSendAckPayload {
                task_id: task_id.clone(),
                ok: true,
                guidance_id: Some(guidance.id.clone()),
                status: Some(format!("{:?}", guidance.status).to_lowercase()),
                reason: None,
            };
            let msg = WsMessage::new(EVT_GUIDANCE_SEND_ACK, ack, cmd.correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ack_sender.send(Message::Text(text.into()));
            }

            // 广播 guidance:sent 事件
            let broadcast_payload = GuidanceSentPayload {
                task_id: task_id.clone(),
                guidance_id: guidance.id.clone(),
                content_preview: guidance.content_preview(),
                status: format!("{:?}", guidance.status).to_lowercase(),
                created_at: guidance.created_at.clone(),
                sent_by: user_id.to_string(),
            };
            let broadcast_msg = WsMessage::new(
                EVT_GUIDANCE_SENT,
                broadcast_payload,
                cmd.correlation_id.clone(),
            );
            if let Ok(text) = serde_json::to_string(&broadcast_msg) {
                global_ws_bus().publish(text);
            }
        }
        Err(err) => {
            warn!(
                correlation_id = %cmd.correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                error = %err,
                "guidance:send failed"
            );
            let ack = GuidanceSendAckPayload {
                task_id: task_id.clone(),
                ok: false,
                guidance_id: None,
                status: None,
                reason: Some(err.to_string()),
            };
            let msg = WsMessage::new(EVT_GUIDANCE_SEND_ACK, ack, cmd.correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ack_sender.send(Message::Text(text.into()));
            }
        }
    }
}

async fn send_paused_snapshots(sender: &tokio::sync::mpsc::UnboundedSender<Message>) -> bool {
    let registry = global_pause_registry();
    let snapshots = registry.get_all_paused_snapshots().await;

    for snapshot in snapshots {
        let payload = IterationPausedPayload {
            task_id: snapshot.task_id.clone(),
            paused_at: snapshot.paused_at.clone(),
            stage: snapshot.stage.clone(),
            iteration: snapshot.iteration,
        };
        let msg = WsMessage::new(
            EVT_ITERATION_PAUSED,
            payload,
            snapshot.correlation_id.clone(),
        );
        let Ok(text) = serde_json::to_string(&msg) else {
            continue;
        };
        if sender.send(Message::Text(text.into())).is_err() {
            return false;
        }
    }

    true
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: String) {
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
