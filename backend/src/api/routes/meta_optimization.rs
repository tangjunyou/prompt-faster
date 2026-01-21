//! 元优化 API

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post, put},
};
use serde::Deserialize;
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::core::meta_optimization_service::{
    MetaOptimizationServiceError, create_prompt_version, get_historical_tasks_for_meta_optimization,
    get_overview, get_prompt_by_id, list_prompt_versions, preview_prompt, set_active_prompt,
    validate_prompt,
};
use crate::domain::models::{
    CreateTeacherPromptInput, MetaOptimizationOverview, MetaOptimizationTaskSummary, TeacherPrompt,
    TeacherPromptVersion, PromptPreviewRequest, PromptPreviewResponse, PromptValidationRequest,
    PromptValidationResult,
};
use crate::infra::db::repositories::TeacherPromptRepo;
use crate::shared::error_codes;
use crate::shared::time::now_millis;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;

#[derive(Debug, Deserialize, IntoParams)]
pub struct PromptListQuery {
    /// 返回条数（默认 50，最大 100）
    pub limit: Option<i64>,
    /// 偏移量（默认 0）
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MetaOptimizationTaskQuery {
    /// 返回条数（默认 50，最大 100）
    pub limit: Option<i64>,
    /// 偏移量（默认 0）
    pub offset: Option<i64>,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn normalize_limit_offset<T: serde::Serialize>(
    query: &PromptListQuery,
) -> Result<(i64, i64), ApiResponse<T>> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    if limit <= 0 || limit > MAX_LIMIT {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            format!("limit 必须在 1-{} 范围内", MAX_LIMIT),
        ));
    }
    let offset = query.offset.unwrap_or(0);
    if offset < 0 {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "offset 不能小于 0",
        ));
    }
    Ok((limit, offset))
}

fn normalize_task_limit_offset<T: serde::Serialize>(
    query: &MetaOptimizationTaskQuery,
) -> Result<(i64, i64), ApiResponse<T>> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    if limit <= 0 || limit > MAX_LIMIT {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            format!("limit 必须在 1-{} 范围内", MAX_LIMIT),
        ));
    }
    let offset = query.offset.unwrap_or(0);
    if offset < 0 {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "offset 不能小于 0",
        ));
    }
    Ok((limit, offset))
}

async fn map_prompt_not_found<T: serde::Serialize>(
    pool: &sqlx::SqlitePool,
    version_id: &str,
) -> ApiResponse<T> {
    match TeacherPromptRepo::exists_by_id(pool, version_id).await {
        Ok(true) => ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "无权访问该版本",
        ),
        Ok(false) => ApiResponse::err(
            StatusCode::NOT_FOUND,
            error_codes::RESOURCE_NOT_FOUND,
            "版本不存在",
        ),
        Err(err) => {
            warn!(error = %err, "检查 Prompt 版本存在性失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 Prompt 版本失败",
            )
        }
    }
}

fn log_action(
    correlation_id: &str,
    user_id: &str,
    version_id: &str,
    action: &str,
    prev_state: &str,
    new_state: &str,
) {
    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        version_id = %version_id,
        action = action,
        prev_state = prev_state,
        new_state = new_state,
        task_id = "N/A",
        iteration_state = "N/A",
        timestamp = now_millis(),
        "元优化操作"
    );
}

/// 创建老师模型 Prompt 版本
#[utoipa::path(
    post,
    path = "/api/v1/meta-optimization/prompts",
    request_body = CreateTeacherPromptInput,
    responses(
        (status = 200, description = "创建成功", body = ApiSuccess<TeacherPromptVersion>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn create_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Json(input): Json<CreateTeacherPromptInput>,
) -> ApiResponse<TeacherPromptVersion> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    match create_prompt_version(&state.db, user_id, input).await {
        Ok(version) => {
            log_action(
                &correlation_id,
                user_id,
                &version.id,
                "meta_optimization:create_prompt",
                "N/A",
                "created",
            );
            ApiResponse::ok(version)
        }
        Err(err) => map_service_error(&correlation_id, err, None, &state.db).await,
    }
}

/// 获取老师模型 Prompt 版本列表
#[utoipa::path(
    get,
    path = "/api/v1/meta-optimization/prompts",
    params(PromptListQuery),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<Vec<TeacherPromptVersion>>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn list_prompts(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Query(query): Query<PromptListQuery>,
) -> ApiResponse<Vec<TeacherPromptVersion>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    let (limit, offset) = match normalize_limit_offset::<Vec<TeacherPromptVersion>>(&query) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    log_action(
        &correlation_id,
        user_id,
        "N/A",
        "meta_optimization:list_prompts",
        "N/A",
        "N/A",
    );

    match list_prompt_versions(&state.db, user_id, limit, offset).await {
        Ok(list) => ApiResponse::ok(list),
        Err(err) => map_service_error(&correlation_id, err, None, &state.db).await,
    }
}

/// 获取老师模型 Prompt 版本详情
#[utoipa::path(
    get,
    path = "/api/v1/meta-optimization/prompts/{id}",
    params(
        ("id" = String, Path, description = "Prompt 版本 ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<TeacherPrompt>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问"),
        (status = 404, description = "版本不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn get_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<TeacherPrompt> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    log_action(
        &correlation_id,
        user_id,
        &id,
        "meta_optimization:get_prompt",
        "N/A",
        "N/A",
    );

    match get_prompt_by_id(&state.db, user_id, &id).await {
        Ok(prompt) => ApiResponse::ok(prompt),
        Err(err) => map_service_error(&correlation_id, err, Some(&id), &state.db).await,
    }
}

/// 设为活跃版本
#[utoipa::path(
    put,
    path = "/api/v1/meta-optimization/prompts/{id}/activate",
    params(
        ("id" = String, Path, description = "Prompt 版本 ID")
    ),
    responses(
        (status = 200, description = "更新成功", body = ApiSuccess<TeacherPrompt>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问"),
        (status = 404, description = "版本不存在"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn activate_prompt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<TeacherPrompt> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    let prev_active = TeacherPromptRepo::find_active(&state.db, user_id)
        .await
        .ok()
        .and_then(|v| v.map(|p| p.id));

    match set_active_prompt(&state.db, user_id, &id).await {
        Ok(prompt) => {
            log_action(
                &correlation_id,
                user_id,
                &id,
                "meta_optimization:activate_prompt",
                prev_active.as_deref().unwrap_or("N/A"),
                &id,
            );
            ApiResponse::ok(prompt)
        }
        Err(err) => map_service_error(&correlation_id, err, Some(&id), &state.db).await,
    }
}

/// 获取统计概览
#[utoipa::path(
    get,
    path = "/api/v1/meta-optimization/stats",
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<MetaOptimizationOverview>),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn get_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
) -> ApiResponse<MetaOptimizationOverview> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    log_action(
        &correlation_id,
        user_id,
        "N/A",
        "meta_optimization:get_stats",
        "N/A",
        "N/A",
    );

    match get_overview(&state.db, user_id).await {
        Ok(overview) => ApiResponse::ok(overview),
        Err(err) => map_service_error(&correlation_id, err, None, &state.db).await,
    }
}

/// 获取历史任务列表（元优化选择入口）
#[utoipa::path(
    get,
    path = "/api/v1/meta-optimization/tasks",
    params(MetaOptimizationTaskQuery),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<Vec<MetaOptimizationTaskSummary>>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn list_historical_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Query(query): Query<MetaOptimizationTaskQuery>,
) -> ApiResponse<Vec<MetaOptimizationTaskSummary>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    let (limit, offset) =
        match normalize_task_limit_offset::<Vec<MetaOptimizationTaskSummary>>(&query) {
            Ok(v) => v,
            Err(resp) => return resp,
        };

    log_action(
        &correlation_id,
        user_id,
        "N/A",
        "meta_optimization:list_tasks",
        "N/A",
        "N/A",
    );

    match get_historical_tasks_for_meta_optimization(&state.db, user_id, limit, offset).await {
        Ok(list) => ApiResponse::ok(list),
        Err(err) => map_service_error(&correlation_id, err, None, &state.db).await,
    }
}

/// 预览执行 Prompt
#[utoipa::path(
    post,
    path = "/api/v1/meta-optimization/prompts/preview",
    request_body = PromptPreviewRequest,
    responses(
        (status = 200, description = "预览成功", body = ApiSuccess<PromptPreviewResponse>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn preview_prompt_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Json(request): Json<PromptPreviewRequest>,
) -> ApiResponse<PromptPreviewResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;
    let user_password = match current_user.unlock_context.as_ref() {
        Some(ctx) => ctx.password_bytes(),
        None => {
            return ApiResponse::err(
                StatusCode::UNAUTHORIZED,
                error_codes::UNAUTHORIZED,
                "会话已过期，请重新登录",
            );
        }
    };

    log_action(
        &correlation_id,
        user_id,
        "N/A",
        "meta_optimization:preview_prompt",
        "N/A",
        "preview",
    );

    match preview_prompt(
        &state.db,
        state.api_key_manager.as_ref(),
        user_id,
        user_password,
        request,
        Some(correlation_id.clone()),
    )
    .await
    {
        Ok(resp) => ApiResponse::ok(resp),
        Err(err) => map_service_error(&correlation_id, err, None, &state.db).await,
    }
}

/// 验证 Prompt 格式
#[utoipa::path(
    post,
    path = "/api/v1/meta-optimization/prompts/validate",
    request_body = PromptValidationRequest,
    responses(
        (status = 200, description = "验证完成", body = ApiSuccess<PromptValidationResult>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器错误")
    ),
    tag = "meta_optimization"
)]
pub(crate) async fn validate_prompt_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Json(request): Json<PromptValidationRequest>,
) -> ApiResponse<PromptValidationResult> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    log_action(
        &correlation_id,
        user_id,
        "N/A",
        "meta_optimization:validate_prompt",
        "N/A",
        "validated",
    );

    match validate_prompt(request) {
        Ok(result) => ApiResponse::ok(result),
        Err(err) => map_service_error(&correlation_id, err, None, &state.db).await,
    }
}

async fn map_service_error<T: serde::Serialize>(
    correlation_id: &str,
    err: MetaOptimizationServiceError,
    version_id: Option<&str>,
    pool: &sqlx::SqlitePool,
) -> ApiResponse<T> {
    match err {
        MetaOptimizationServiceError::NotFoundOrForbidden => {
            if let Some(id) = version_id {
                return map_prompt_not_found(pool, id).await;
            }
            ApiResponse::err(
                StatusCode::NOT_FOUND,
                error_codes::RESOURCE_NOT_FOUND,
                "版本不存在",
            )
        }
        MetaOptimizationServiceError::InvalidRequest(msg) => ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            msg,
        ),
        MetaOptimizationServiceError::ExecutionFailed(msg) => {
            warn!(correlation_id = %correlation_id, error = %msg, "元优化预览执行失败");
            ApiResponse::err(
                StatusCode::BAD_GATEWAY,
                error_codes::UPSTREAM_ERROR,
                "预览执行失败",
            )
        }
        MetaOptimizationServiceError::Timeout => ApiResponse::err(
            StatusCode::GATEWAY_TIMEOUT,
            error_codes::UPSTREAM_ERROR,
            "预览执行超时",
        ),
        MetaOptimizationServiceError::Encryption(msg) => {
            warn!(correlation_id = %correlation_id, error = %msg, "元优化服务解密失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::ENCRYPTION_ERROR,
                "解密 API Key 失败",
            )
        }
        MetaOptimizationServiceError::Database(err) => {
            warn!(correlation_id = %correlation_id, error = %err, "元优化服务数据库错误");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "元优化服务失败",
            )
        }
        MetaOptimizationServiceError::Repo(msg) => {
            warn!(correlation_id = %correlation_id, error = %msg, "元优化服务错误");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "元优化服务失败",
            )
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/prompts", post(create_prompt).get(list_prompts))
        .route("/prompts/preview", post(preview_prompt_handler))
        .route("/prompts/validate", post(validate_prompt_handler))
        .route("/prompts/{id}", get(get_prompt))
        .route("/prompts/{id}/activate", put(activate_prompt))
        .route("/stats", get(get_stats))
        .route("/tasks", get(list_historical_tasks))
}
