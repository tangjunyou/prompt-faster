//! 断点恢复 API 路由

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::{get, post}};
use serde::Serialize;
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::core::iteration_engine::recovery as recovery_core;
use crate::domain::models::{
    ConnectivityResponse, RecoveryMetrics, RecoveryRequest, RecoveryResponse,
    UnfinishedTasksResponse,
};
use crate::infra::external::connectivity::check_connectivity;
use crate::infra::db::repositories::OptimizationTaskRepo;
use crate::shared::error_codes;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AbortRecoveryResponse {
    pub success: bool,
    pub message: String,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn state_label(state: &crate::domain::models::IterationState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn run_control_state_label(state: &crate::domain::types::RunControlState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

/// 获取未完成任务列表
#[utoipa::path(
    get,
    path = "/api/v1/recovery/unfinished-tasks",
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<UnfinishedTasksResponse>),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "recovery"
)]
pub(crate) async fn list_unfinished_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
) -> ApiResponse<UnfinishedTasksResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        action = "recovery:list_unfinished",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = crate::shared::time::now_millis(),
        "查询未完成任务列表"
    );

    match recovery_core::detect_unfinished_tasks_with_pool(&state.db, &user_id).await {
        Ok(tasks) => ApiResponse::ok(UnfinishedTasksResponse {
            total: tasks.len() as u32,
            tasks,
        }),
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询未完成任务失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询未完成任务失败",
            )
        }
    }
}

/// 执行恢复操作
#[utoipa::path(
    post,
    path = "/api/v1/recovery/tasks/{task_id}/recover",
    request_body = RecoveryRequest,
    responses(
        (status = 200, description = "恢复成功", body = ApiSuccess<RecoveryResponse>),
        (status = 400, description = "请求非法"),
        (status = 403, description = "无权访问"),
        (status = 404, description = "Checkpoint 不存在"),
        (status = 409, description = "无法恢复"),
        (status = 500, description = "服务器错误")
    ),
    tag = "recovery"
)]
pub(crate) async fn recover_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
    Json(body): Json<RecoveryRequest>,
) -> ApiResponse<RecoveryResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = current_user.user_id;
    let checkpoint_id = body.checkpoint_id;

    let result = recovery_core::recover_task_with_pool(
        &state.db,
        &task_id,
        &user_id,
        &correlation_id,
        checkpoint_id.as_deref(),
    )
    .await;

    match result {
        Ok((ctx, used_checkpoint)) => {
            info!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                action = "recovery:recover",
                prev_state = ?ctx.state,
                new_state = ?ctx.state,
                iteration_state = ?ctx.state,
                timestamp = crate::shared::time::now_millis(),
                "恢复任务成功"
            );
            ApiResponse::ok(RecoveryResponse {
                success: true,
                task_id,
                checkpoint_id: used_checkpoint.id,
                iteration: ctx.iteration,
                state: state_label(&ctx.state),
                run_control_state: run_control_state_label(&ctx.run_control_state),
                message: "恢复成功".to_string(),
            })
        }
        Err(recovery_core::RecoveryError::TaskNotFound) => ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        ),
        Err(recovery_core::RecoveryError::CheckpointNotFound)
        | Err(recovery_core::RecoveryError::PauseStateNotFound) => ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::RESOURCE_NOT_FOUND,
            "未找到可用的 Checkpoint",
        ),
        Err(recovery_core::RecoveryError::NoValidCheckpoint) => ApiResponse::err(
            StatusCode::CONFLICT,
            error_codes::UPSTREAM_ERROR,
            "无法恢复，请重新开始任务",
        ),
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "恢复任务失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "恢复任务失败",
            )
        }
    }
}

/// 放弃恢复
#[utoipa::path(
    post,
    path = "/api/v1/recovery/tasks/{task_id}/abort",
    responses(
        (status = 200, description = "放弃恢复成功", body = ApiSuccess<AbortRecoveryResponse>),
        (status = 403, description = "无权访问"),
        (status = 500, description = "服务器错误")
    ),
    tag = "recovery"
)]
pub(crate) async fn abort_recovery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<AbortRecoveryResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = current_user.user_id;

    match recovery_core::abort_task_with_pool(&state.db, &task_id, &user_id, &correlation_id).await {
        Ok(_) => ApiResponse::ok(AbortRecoveryResponse {
            success: true,
            message: "已放弃恢复".to_string(),
        }),
        Err(recovery_core::RecoveryError::TaskNotFound) => ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        ),
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "放弃恢复失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "放弃恢复失败",
            )
        }
    }
}

/// 获取恢复统计信息
#[utoipa::path(
    get,
    path = "/api/v1/recovery/tasks/{task_id}/metrics",
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<RecoveryMetrics>),
        (status = 403, description = "无权访问"),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "recovery"
)]
pub(crate) async fn get_recovery_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<RecoveryMetrics> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = current_user.user_id;

    if OptimizationTaskRepo::find_by_id_for_user(&state.db, &user_id, &task_id)
        .await
        .is_err()
    {
        return ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        );
    }

    match recovery_core::get_recovery_metrics_with_pool(&state.db, &task_id).await {
        Ok(metrics) => ApiResponse::ok(metrics),
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询恢复统计失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "查询恢复统计失败",
            )
        }
    }
}

/// 获取网络连接状态
#[utoipa::path(
    get,
    path = "/api/v1/connectivity",
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<ConnectivityResponse>)
    ),
    tag = "recovery"
)]
pub(crate) async fn get_connectivity(
    State(_state): State<AppState>,
) -> ApiResponse<ConnectivityResponse> {
    let response = check_connectivity().await;
    ApiResponse::ok(response)
}

/// 创建恢复相关路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/unfinished-tasks", get(list_unfinished_tasks))
        .route("/tasks/{task_id}/recover", post(recover_task))
        .route("/tasks/{task_id}/abort", post(abort_recovery))
        .route("/tasks/{task_id}/metrics", get(get_recovery_metrics))
}

/// 创建 connectivity 路由（公开）
pub fn connectivity_router() -> Router<AppState> {
    Router::new().route("/connectivity", get(get_connectivity))
}
