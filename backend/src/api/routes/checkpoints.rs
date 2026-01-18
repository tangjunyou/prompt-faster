//! Checkpoint API 路由

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Router, routing::get};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::core::iteration_engine::checkpoint::verify_checksum;
use crate::domain::models::{CheckpointEntity, CheckpointListResponse, CheckpointResponse};
use crate::domain::types::unix_ms_to_iso8601;
use crate::infra::db::repositories::{CheckpointRepo, CheckpointRepoError, OptimizationTaskRepo};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListCheckpointsQuery {
    /// 最大返回条数（默认 20，最大 100）
    pub limit: Option<u32>,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn current_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn to_checkpoint_response(entity: &CheckpointEntity) -> CheckpointResponse {
    let prompt_preview = if entity.prompt.chars().count() > 200 {
        entity.prompt.chars().take(200).collect::<String>()
    } else {
        entity.prompt.clone()
    };
    let state = serde_json::to_value(entity.state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let run_control_state = serde_json::to_value(entity.run_control_state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    CheckpointResponse {
        id: entity.id.clone(),
        task_id: entity.task_id.clone(),
        iteration: entity.iteration,
        state,
        run_control_state,
        prompt_preview,
        has_artifacts: entity
            .artifacts
            .as_ref()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        has_user_guidance: entity.user_guidance.is_some(),
        checksum: entity.checksum.clone(),
        integrity_ok: verify_checksum(entity),
        created_at: unix_ms_to_iso8601(entity.created_at),
    }
}

/// 获取任务的 Checkpoint 列表
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/checkpoints",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        ListCheckpointsQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<CheckpointListResponse>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "checkpoints"
)]
pub(crate) async fn list_checkpoints(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<ListCheckpointsQuery>,
    current_user: CurrentUser,
) -> ApiResponse<CheckpointListResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "checkpoint:list",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询 checkpoint 列表"
    );

    if OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id)
        .await
        .is_err()
    {
        return ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        );
    }

    let limit = query.limit.unwrap_or(20).min(100);
    match CheckpointRepo::list_checkpoints_by_task(&state.db, &task_id, limit).await {
        Ok(checkpoints) => {
            let total = CheckpointRepo::count_checkpoints_by_task(&state.db, &task_id)
                .await
                .unwrap_or(checkpoints.len() as u32);
            let items: Vec<CheckpointResponse> =
                checkpoints.iter().map(to_checkpoint_response).collect();
            ApiResponse::ok(CheckpointListResponse {
                total,
                checkpoints: items,
            })
        }
        Err(CheckpointRepoError::Database(e)) => {
            warn!(correlation_id = %correlation_id, error = %e, "查询 checkpoint 列表失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            )
        }
        Err(e) => {
            warn!(correlation_id = %correlation_id, error = %e, "查询 checkpoint 列表失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            )
        }
    }
}

/// 获取单个 Checkpoint 详情
#[utoipa::path(
    get,
    path = "/api/v1/checkpoints/{checkpoint_id}",
    params(
        ("checkpoint_id" = String, Path, description = "Checkpoint ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<CheckpointResponse>),
        (status = 401, description = "未授权"),
        (status = 404, description = "Checkpoint 不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "checkpoints"
)]
pub(crate) async fn get_checkpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(checkpoint_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<CheckpointResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        checkpoint_id = %checkpoint_id,
        action = "checkpoint:detail",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询 checkpoint 详情"
    );

    let checkpoint =
        match CheckpointRepo::get_checkpoint_for_user(&state.db, user_id, &checkpoint_id).await {
            Ok(Some(checkpoint)) => checkpoint,
            Ok(None) => {
                return ApiResponse::err(
                    StatusCode::NOT_FOUND,
                    error_codes::RESOURCE_NOT_FOUND,
                    "Checkpoint 不存在",
                );
            }
            Err(e) => {
                warn!(correlation_id = %correlation_id, error = %e, "查询 checkpoint 失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询 checkpoint 失败",
                );
            }
        };

    ApiResponse::ok(to_checkpoint_response(&checkpoint))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/{checkpoint_id}", get(get_checkpoint))
}

pub fn task_router() -> Router<AppState> {
    Router::new().route("/", get(list_checkpoints))
}
