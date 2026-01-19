//! 任务历史聚合 API 路由

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
use crate::domain::models::{CheckpointListResponse, TaskHistoryResponse};
use crate::infra::db::repositories::{
    CheckpointRepo, CheckpointRepoError, IterationRepo, IterationRepoError, OptimizationTaskRepo,
};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct TaskHistoryQuery {
    /// 迭代历史最大返回条数（可选，默认 100，最大 100）
    pub iterations_limit: Option<i32>,
    /// Checkpoint 最大返回条数（可选，默认 20，最大 100）
    pub checkpoints_limit: Option<u32>,
    /// Checkpoint 偏移量（可选，默认 0）
    pub checkpoints_offset: Option<u32>,
    /// 是否包含已归档的 Checkpoint（默认 false）
    pub include_archived: Option<bool>,
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

/// 获取任务历史聚合（迭代历史 + 回滚候选）
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/history",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        TaskHistoryQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<TaskHistoryResponse>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "history"
)]
pub(crate) async fn get_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<TaskHistoryQuery>,
    current_user: CurrentUser,
) -> ApiResponse<TaskHistoryResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "history:aggregate",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询任务历史聚合"
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

    let iterations_limit = query
        .iterations_limit
        .filter(|value| *value > 0)
        .map(|value| value.min(100));
    let checkpoints_limit = query.checkpoints_limit.unwrap_or(20).min(100) as usize;
    let checkpoints_offset = query.checkpoints_offset.unwrap_or(0) as usize;
    let include_archived = query.include_archived.unwrap_or(false);

    let iterations = match IterationRepo::list_by_task_id(
        &state.db,
        user_id,
        &task_id,
        iterations_limit,
    )
    .await
    {
        Ok(list) => list,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            return ApiResponse::err(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "任务不存在或无权访问",
            );
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询迭代历史失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询迭代历史失败",
            );
        }
    };

    let total = match CheckpointRepo::count_checkpoints_by_task_with_archived(
        &state.db,
        &task_id,
        include_archived,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 总数失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            );
        }
    };

    let current_branch_id =
        match CheckpointRepo::get_latest_active_branch_id(&state.db, &task_id).await {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询 checkpoint 分支失败"
                );
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询 checkpoint 失败",
                );
            }
        };

    let checkpoints = match CheckpointRepo::list_checkpoint_summaries(
        &state.db,
        &task_id,
        include_archived,
        checkpoints_limit,
        checkpoints_offset,
    )
    .await
    {
        Ok(list) => list,
        Err(CheckpointRepoError::Database(err)) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 列表失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            );
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 列表失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            );
        }
    };

    ApiResponse::ok(TaskHistoryResponse {
        iterations,
        checkpoints: CheckpointListResponse {
            checkpoints,
            total,
            current_branch_id,
        },
    })
}

/// 创建任务历史聚合路由
pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_history))
}
