//! 历史迭代 API 路由
//!
//! 提供历史迭代查询功能的 RESTful API 端点。

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Router, routing::get};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;
use utoipa::IntoParams;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::domain::types::{IterationHistoryDetail, IterationHistorySummary};
use crate::infra::db::repositories::{IterationRepo, IterationRepoError};
use crate::shared::error_codes;

/// 历史迭代列表查询参数
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListIterationsQuery {
    /// 最大返回条数（可选，默认 100，最大 100）
    pub limit: Option<i32>,
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

/// 获取历史迭代列表
///
/// 按轮次倒序返回指定任务的历史迭代摘要列表。
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/iterations",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        ListIterationsQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<Vec<IterationHistorySummary>>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "iterations"
)]
pub(crate) async fn list_iterations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<ListIterationsQuery>,
    current_user: CurrentUser,
) -> ApiResponse<Vec<IterationHistorySummary>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "history:list",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询历史迭代列表"
    );

    match IterationRepo::list_by_task_id(&state.db, user_id, &task_id, query.limit).await {
        Ok(iterations) => {
            info!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                count = iterations.len(),
                iteration_state = "read_only",
                prev_state = "N/A",
                new_state = "N/A",
                timestamp = current_unix_ms(),
                "历史迭代列表查询成功"
            );
            ApiResponse::ok(iterations)
        }
        Err(IterationRepoError::TaskNotFoundOrForbidden) => ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        ),
        Err(e) => {
            tracing::warn!(
                correlation_id = %correlation_id,
                error = %e,
                "查询历史迭代列表失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询历史迭代失败",
            )
        }
    }
}

/// 获取单个迭代详情
///
/// 返回指定迭代的完整详情，包含产物、评估结果和反思总结。
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/iterations/{iteration_id}",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        ("iteration_id" = String, Path, description = "迭代 ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<IterationHistoryDetail>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 404, description = "迭代不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "iterations"
)]
pub(crate) async fn get_iteration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((task_id, iteration_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<IterationHistoryDetail> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        iteration_id = %iteration_id,
        action = "history:detail",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询迭代详情"
    );

    match IterationRepo::get_by_id(&state.db, user_id, &task_id, &iteration_id).await {
        Ok(detail) => {
            info!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                iteration_id = %iteration_id,
                round = detail.round,
                iteration_state = detail.status.as_str(),
                prev_state = "N/A",
                new_state = "N/A",
                timestamp = current_unix_ms(),
                "迭代详情查询成功"
            );
            ApiResponse::ok(detail)
        }
        Err(IterationRepoError::TaskNotFoundOrForbidden) => ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        ),
        Err(IterationRepoError::NotFound) => ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "迭代记录不存在",
        ),
        Err(e) => {
            tracing::warn!(
                correlation_id = %correlation_id,
                error = %e,
                "查询迭代详情失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询迭代详情失败",
            )
        }
    }
}

/// 创建历史迭代路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_iterations))
        .route("/{iteration_id}", get(get_iteration))
}
