use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use serde::Serialize;
use tracing::{info, warn};

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::domain::models::{DiversityAnalysisResult, DiversityBaseline, OptimizationTaskMode};
use crate::infra::db::repositories::{
    DiversityBaselineRepo, DiversityBaselineRepoError, IterationRepo, IterationRepoError,
    OptimizationTaskRepo, OptimizationTaskRepoError,
};
use crate::shared::error_codes;
use crate::shared::time::now_millis;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_diversity_analysis))
        .route("/baseline", post(record_diversity_baseline))
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

async fn task_exists(pool: &sqlx::SqlitePool, task_id: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT id FROM optimization_tasks WHERE id = ?")
        .bind(task_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

fn log_action(
    correlation_id: &str,
    user_id: &str,
    task_id: &str,
    action: &str,
    prev_state: &str,
    new_state: &str,
    iteration_state: &str,
) {
    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = action,
        prev_state = prev_state,
        new_state = new_state,
        iteration_state = iteration_state,
        timestamp = now_millis(),
        "diversity action"
    );
}

async fn ensure_creative_task<T>(
    state: &AppState,
    user_id: &str,
    task_id: &str,
    correlation_id: &str,
) -> Result<(), ApiResponse<T>>
where
    T: Serialize,
{
    match OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, task_id).await {
        Ok(task) => {
            if task.task_mode != OptimizationTaskMode::Creative {
                return Err(ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "多样性分析仅支持创意任务",
                ));
            }
            Ok(())
        }
        Err(OptimizationTaskRepoError::NotFound) => match task_exists(&state.db, task_id).await {
            Ok(true) => Err(ApiResponse::err(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "无权访问该任务",
            )),
            Ok(false) => Err(ApiResponse::err(
                StatusCode::NOT_FOUND,
                error_codes::OPTIMIZATION_TASK_NOT_FOUND,
                "优化任务不存在",
            )),
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "检查任务存在性失败"
                );
                Err(ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询任务失败",
                ))
            }
        },
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询任务失败"
            );
            Err(ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询任务失败",
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/diversity",
    params(("task_id" = String, Path, description = "优化任务 ID")),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<DiversityAnalysisResult>),
        (status = 400, description = "非创意任务", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 403, description = "无权访问", body = ApiError),
        (status = 404, description = "分析结果不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "diversity"
)]
pub async fn get_diversity_analysis(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<DiversityAnalysisResult> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    log_action(
        &correlation_id,
        user_id,
        &task_id,
        "get_diversity_analysis",
        "N/A",
        "N/A",
        "N/A",
    );

    if let Err(resp) = ensure_creative_task(&state, user_id, &task_id, &correlation_id).await {
        return resp;
    }

    let iterations = match IterationRepo::list_with_artifacts_by_task_id(
        &state.db,
        user_id,
        &task_id,
        Some(1),
        Some(0),
        Some("completed"),
    )
    .await
    {
        Ok(iterations) => iterations,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            match task_exists(&state.db, &task_id).await {
                Ok(true) => {
                    return ApiResponse::err(
                        StatusCode::FORBIDDEN,
                        error_codes::FORBIDDEN,
                        "无权访问该任务",
                    );
                }
                Ok(false) => {
                    return ApiResponse::err(
                        StatusCode::NOT_FOUND,
                        error_codes::OPTIMIZATION_TASK_NOT_FOUND,
                        "优化任务不存在",
                    );
                }
                Err(err) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %err,
                        "检查任务存在性失败"
                    );
                    return ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询任务失败",
                    );
                }
            }
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询迭代记录失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询迭代记录失败",
            );
        }
    };

    let Some(iteration) = iterations.first() else {
        return ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "多样性分析暂不可用",
        );
    };
    let Some(analysis) = iteration.artifacts.diversity_analysis.clone() else {
        return ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "多样性分析暂不可用",
        );
    };

    ApiResponse::ok(analysis)
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{task_id}/diversity/baseline",
    params(("task_id" = String, Path, description = "优化任务 ID")),
    responses(
        (status = 200, description = "记录成功", body = ApiSuccess<DiversityBaseline>),
        (status = 400, description = "非创意任务或缺少分析", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 403, description = "无权访问", body = ApiError),
        (status = 404, description = "分析结果不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "diversity"
)]
pub async fn record_diversity_baseline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<DiversityBaseline> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    log_action(
        &correlation_id,
        user_id,
        &task_id,
        "record_diversity_baseline",
        "N/A",
        "N/A",
        "N/A",
    );

    if let Err(resp) = ensure_creative_task(&state, user_id, &task_id, &correlation_id).await {
        return resp;
    }

    let iterations = match IterationRepo::list_with_artifacts_by_task_id(
        &state.db,
        user_id,
        &task_id,
        Some(1),
        Some(0),
        Some("completed"),
    )
    .await
    {
        Ok(iterations) => iterations,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            match task_exists(&state.db, &task_id).await {
                Ok(true) => {
                    return ApiResponse::err(
                        StatusCode::FORBIDDEN,
                        error_codes::FORBIDDEN,
                        "无权访问该任务",
                    );
                }
                Ok(false) => {
                    return ApiResponse::err(
                        StatusCode::NOT_FOUND,
                        error_codes::OPTIMIZATION_TASK_NOT_FOUND,
                        "优化任务不存在",
                    );
                }
                Err(err) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %err,
                        "检查任务存在性失败"
                    );
                    return ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询任务失败",
                    );
                }
            }
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询迭代记录失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询迭代记录失败",
            );
        }
    };

    let Some(iteration) = iterations.first() else {
        return ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "暂无多样性分析可记录为基准线",
        );
    };
    let Some(analysis) = iteration.artifacts.diversity_analysis.clone() else {
        return ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "暂无多样性分析可记录为基准线",
        );
    };

    let existing = DiversityBaselineRepo::get_by_task_id(&state.db, &task_id).await;
    let prev_state = match existing {
        Ok(Some(_)) => "baseline_exists",
        Ok(None) => "baseline_missing",
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询基准线失败"
            );
            "baseline_unknown"
        }
    };

    let baseline = match DiversityBaselineRepo::upsert(
        &state.db,
        &task_id,
        &analysis.metrics,
        iteration.summary.round.max(0) as u32,
    )
    .await
    {
        Ok(baseline) => baseline,
        Err(DiversityBaselineRepoError::DatabaseError(err)) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "记录多样性基准线失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "记录多样性基准线失败",
            );
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "记录多样性基准线失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "记录多样性基准线失败",
            );
        }
    };

    log_action(
        &correlation_id,
        user_id,
        &task_id,
        "record_diversity_baseline",
        prev_state,
        "baseline_upserted",
        "completed",
    );

    ApiResponse::ok(baseline)
}
