use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{info, warn};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::domain::models::{
    AdvancedDataSplitConfig, DataSplitPercentConfig, EvaluatorConfig, ExecutionMode,
    ExecutionTargetType, OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION, OptimizationTaskConfig,
    OptimizationTaskMode, OptimizationTaskStatus, OutputConfig, TaskReference, TeacherLlmConfig,
};
use crate::infra::db::repositories::{
    CreateOptimizationTaskInput, OptimizationTaskRepo, OptimizationTaskRepoError,
    TeacherPromptRepo, TeacherPromptRepoError, TestSetRepo, TestSetRepoError, WorkspaceRepo,
    WorkspaceRepoError,
};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct MetaOptimizationTaskHint {
    /// 是否使用当前活跃的老师模型 Prompt 版本
    #[serde(default)]
    pub use_active_teacher_prompt: bool,
}

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct CreateOptimizationTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub goal: String,
    pub execution_target_type: String,
    pub task_mode: String,
    pub test_set_ids: Vec<String>,
    #[serde(default)]
    pub meta_optimization: Option<MetaOptimizationTaskHint>,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct OptimizationTaskResponse {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub goal: String,
    pub execution_target_type: ExecutionTargetType,
    pub task_mode: OptimizationTaskMode,
    pub status: OptimizationTaskStatus,
    pub test_set_ids: Vec<String>,
    pub config: OptimizationTaskConfig,
    pub final_prompt: Option<String>,
    #[ts(type = "number | null")]
    pub terminated_at: Option<i64>,
    pub selected_iteration_id: Option<String>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct OptimizationTaskListItemResponse {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub goal: String,
    pub execution_target_type: ExecutionTargetType,
    pub task_mode: OptimizationTaskMode,
    pub status: OptimizationTaskStatus,
    pub teacher_model_display_name: String,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct UpdateOptimizationTaskConfigRequest {
    pub initial_prompt: Option<String>,
    pub max_iterations: u32,
    pub pass_threshold_percent: u8,
    pub candidate_prompt_count: u32,
    pub diversity_injection_threshold: u32,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,
    pub train_percent: u8,
    pub validation_percent: u8,
    pub output_config: OutputConfig,
    pub evaluator_config: EvaluatorConfig,
    #[serde(default)]
    pub teacher_llm: TeacherLlmConfig,
    pub advanced_data_split: AdvancedDataSplitConfig,
}

fn default_max_concurrency() -> u32 {
    OptimizationTaskConfig::default().max_concurrency
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn warn_if_invalid_config_json(
    correlation_id: &str,
    user_id: &str,
    workspace_id: &str,
    task_id: &str,
    raw: Option<&str>,
) {
    let Some(raw) = raw else {
        return;
    };
    if raw.trim().is_empty() {
        return;
    }
    if let Err(e) = serde_json::from_str::<JsonValue>(raw) {
        warn!(
            correlation_id = %correlation_id,
            user_id = %user_id,
            workspace_id = %workspace_id,
            task_id = %task_id,
            config_json_len = raw.len(),
            error = %e,
            "检测到 optimization_tasks.config_json 非法 JSON（回显将回退为默认配置）"
        );
    }
}

fn workspace_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::WORKSPACE_NOT_FOUND,
        "工作区不存在",
    )
}

fn test_set_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::TEST_SET_NOT_FOUND,
        "测试集不存在",
    )
}

fn optimization_task_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::OPTIMIZATION_TASK_NOT_FOUND,
        "优化任务不存在",
    )
}

async fn ensure_workspace_exists<T: Serialize>(
    state: &AppState,
    workspace_id: &str,
    user_id: &str,
) -> Result<(), ApiResponse<T>> {
    match WorkspaceRepo::find_by_id(&state.db, workspace_id, user_id).await {
        Ok(_) => Ok(()),
        Err(WorkspaceRepoError::NotFound) => Err(workspace_not_found()),
        Err(e) => {
            warn!(error = %e, "查询工作区失败");
            Err(ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询工作区失败",
            ))
        }
    }
}

fn validate_name<T: Serialize>(name: &str) -> Result<(), ApiResponse<T>> {
    let name = name.trim();

    if name.is_empty() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "任务名称不能为空",
        ));
    }

    if name.chars().count() > 128 {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "任务名称不能超过 128 个字符",
        ));
    }

    Ok(())
}

fn validate_goal<T: Serialize>(goal: &str) -> Result<(), ApiResponse<T>> {
    if goal.trim().is_empty() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "优化目标不能为空",
        ));
    }
    Ok(())
}

fn parse_execution_target_type<T: Serialize>(
    raw: &str,
) -> Result<ExecutionTargetType, ApiResponse<T>> {
    match raw.trim() {
        "dify" => Ok(ExecutionTargetType::Dify),
        "generic" => Ok(ExecutionTargetType::Generic),
        "example" => Ok(ExecutionTargetType::Example),
        _ => Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "execution_target_type 仅允许 dify | generic | example",
        )),
    }
}

fn parse_task_mode<T: Serialize>(raw: &str) -> Result<OptimizationTaskMode, ApiResponse<T>> {
    match raw.trim() {
        "fixed" => Ok(OptimizationTaskMode::Fixed),
        "creative" => Ok(OptimizationTaskMode::Creative),
        _ => Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "task_mode 仅允许 fixed | creative",
        )),
    }
}

fn normalize_test_set_ids<T: Serialize>(
    test_set_ids: &[String],
) -> Result<Vec<String>, ApiResponse<T>> {
    if test_set_ids.is_empty() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "test_set_ids 至少需要 1 个测试集",
        ));
    }

    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::with_capacity(test_set_ids.len());
    for id in test_set_ids {
        let normalized_id = id.trim();
        if normalized_id.is_empty() {
            return Err(ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "test_set_ids 中存在空 id",
            ));
        }
        if !seen.insert(normalized_id.to_string()) {
            return Err(ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "test_set_ids 中存在重复 id",
            ));
        }
        normalized.push(normalized_id.to_string());
    }

    Ok(normalized)
}

fn validate_mode_against_reference<T: Serialize>(
    task_mode: OptimizationTaskMode,
    reference: &TaskReference,
) -> Result<(), ApiResponse<T>> {
    match task_mode {
        OptimizationTaskMode::Fixed => {
            if matches!(reference, TaskReference::Constrained { .. }) {
                return Err(ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "Fixed 模式不允许关联包含 Constrained reference 的测试集",
                ));
            }
        }
        OptimizationTaskMode::Creative => {
            if matches!(reference, TaskReference::Exact { .. }) {
                return Err(ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "Creative 模式不允许关联包含 Exact reference 的测试集",
                ));
            }
        }
    }
    Ok(())
}

async fn load_and_validate_test_sets<T: Serialize>(
    state: &AppState,
    user_id: &str,
    workspace_id: &str,
    test_set_ids: &[String],
    task_mode: OptimizationTaskMode,
) -> Result<(), ApiResponse<T>> {
    for test_set_id in test_set_ids {
        let test_set =
            match TestSetRepo::find_by_id_scoped(&state.db, user_id, workspace_id, test_set_id)
                .await
            {
                Ok(ts) => ts,
                Err(TestSetRepoError::NotFound) => return Err(test_set_not_found()),
                Err(e) => {
                    warn!(error = %e, "查询测试集失败");
                    return Err(ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询测试集失败",
                    ));
                }
            };

        for case in &test_set.cases {
            validate_mode_against_reference::<T>(task_mode, &case.reference)?;
        }
    }

    Ok(())
}

fn validate_task_config<T: Serialize>(
    config: &OptimizationTaskConfig,
) -> Result<(), ApiResponse<T>> {
    if let Err(msg) = config.validate() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            msg,
        ));
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/optimization-tasks",
    request_body = CreateOptimizationTaskRequest,
    params(
        ("workspace_id" = String, Path, description = "工作区 ID")
    ),
    responses(
        (status = 200, description = "创建成功", body = ApiSuccess<OptimizationTaskResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区/测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "optimization_tasks"
)]
pub(crate) async fn create_optimization_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
    Json(req): Json<CreateOptimizationTaskRequest>,
) -> ApiResponse<OptimizationTaskResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "创建优化任务");

    if let Err(resp) =
        ensure_workspace_exists::<OptimizationTaskResponse>(&state, &workspace_id, user_id).await
    {
        return resp;
    }

    if let Err(resp) = validate_name::<OptimizationTaskResponse>(&req.name) {
        return resp;
    }
    if let Err(resp) = validate_goal::<OptimizationTaskResponse>(&req.goal) {
        return resp;
    }
    let test_set_ids = match normalize_test_set_ids::<OptimizationTaskResponse>(&req.test_set_ids) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let execution_target_type =
        match parse_execution_target_type::<OptimizationTaskResponse>(&req.execution_target_type) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
    let task_mode = match parse_task_mode::<OptimizationTaskResponse>(&req.task_mode) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if let Err(resp) = load_and_validate_test_sets::<OptimizationTaskResponse>(
        &state,
        user_id,
        &workspace_id,
        &test_set_ids,
        task_mode,
    )
    .await
    {
        return resp;
    }

    let teacher_prompt_version_id = match req
        .meta_optimization
        .as_ref()
        .map(|meta| meta.use_active_teacher_prompt)
        .unwrap_or(false)
    {
        true => match TeacherPromptRepo::find_active(&state.db, user_id).await {
            Ok(Some(prompt)) => Some(prompt.id),
            Ok(None) => {
                return ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "未找到可用的老师模型 Prompt 版本",
                );
            }
            Err(TeacherPromptRepoError::DatabaseError(err)) => {
                warn!(error = %err, "查询老师模型 Prompt 版本失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询老师模型 Prompt 版本失败",
                );
            }
            Err(_) => {
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询老师模型 Prompt 版本失败",
                );
            }
        },
        false => None,
    };

    match OptimizationTaskRepo::create_scoped(
        &state.db,
        CreateOptimizationTaskInput {
            user_id,
            workspace_id: &workspace_id,
            name: req.name.trim(),
            description: req
                .description
                .as_deref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
            goal: req.goal.trim(),
            execution_target_type,
            task_mode,
            test_set_ids: &test_set_ids,
            teacher_prompt_version_id: teacher_prompt_version_id.as_deref(),
        },
    )
    .await
    {
        Ok(created) => ApiResponse::ok(OptimizationTaskResponse {
            config: OptimizationTaskConfig::default(),
            id: created.task.id,
            workspace_id: created.task.workspace_id,
            name: created.task.name,
            description: created.task.description,
            goal: created.task.goal,
            execution_target_type: created.task.execution_target_type,
            task_mode: created.task.task_mode,
            status: created.task.status,
            test_set_ids: created.test_set_ids,
            final_prompt: created.task.final_prompt,
            terminated_at: created.task.terminated_at,
            selected_iteration_id: created.task.selected_iteration_id,
            created_at: created.task.created_at,
            updated_at: created.task.updated_at,
        }),
        Err(OptimizationTaskRepoError::WorkspaceNotFound) => workspace_not_found(),
        Err(OptimizationTaskRepoError::TestSetNotFound) => test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "创建优化任务失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "创建优化任务失败",
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/optimization-tasks",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<Vec<OptimizationTaskListItemResponse>>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "optimization_tasks"
)]
pub(crate) async fn list_optimization_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<Vec<OptimizationTaskListItemResponse>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "查询优化任务列表");

    if let Err(resp) = ensure_workspace_exists::<Vec<OptimizationTaskListItemResponse>>(
        &state,
        &workspace_id,
        user_id,
    )
    .await
    {
        return resp;
    }

    match OptimizationTaskRepo::list_by_workspace_scoped(&state.db, user_id, &workspace_id).await {
        Ok(list) => ApiResponse::ok(
            list.into_iter()
                .map(|item| {
                    warn_if_invalid_config_json(
                        &correlation_id,
                        user_id,
                        &workspace_id,
                        &item.task.id,
                        item.task.config_json.as_deref(),
                    );

                    let config = OptimizationTaskConfig::normalized_from_config_json(
                        item.task.config_json.as_deref(),
                    );

                    let teacher_model_display_name = config
                        .teacher_llm
                        .model_id
                        .clone()
                        .unwrap_or_else(|| "系统默认".to_string());

                    OptimizationTaskListItemResponse {
                        id: item.task.id,
                        workspace_id: item.task.workspace_id,
                        name: item.task.name,
                        goal: item.task.goal,
                        execution_target_type: item.task.execution_target_type,
                        task_mode: item.task.task_mode,
                        status: item.task.status,
                        teacher_model_display_name,
                        created_at: item.task.created_at,
                        updated_at: item.task.updated_at,
                    }
                })
                .collect(),
        ),
        Err(OptimizationTaskRepoError::WorkspaceNotFound) => workspace_not_found(),
        Err(e) => {
            warn!(error = %e, "查询优化任务列表失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询优化任务列表失败",
            )
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("task_id" = String, Path, description = "优化任务 ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<OptimizationTaskResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "资源不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "optimization_tasks"
)]
pub(crate) async fn get_optimization_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, task_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<OptimizationTaskResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, task_id = %task_id, "获取优化任务");

    if let Err(resp) =
        ensure_workspace_exists::<OptimizationTaskResponse>(&state, &workspace_id, user_id).await
    {
        return resp;
    }

    match OptimizationTaskRepo::find_by_id_scoped(&state.db, user_id, &workspace_id, &task_id).await
    {
        Ok(task) => {
            warn_if_invalid_config_json(
                &correlation_id,
                user_id,
                &workspace_id,
                &task_id,
                task.task.config_json.as_deref(),
            );
            ApiResponse::ok(OptimizationTaskResponse {
                config: OptimizationTaskConfig::normalized_from_config_json(
                    task.task.config_json.as_deref(),
                ),
                id: task.task.id,
                workspace_id: task.task.workspace_id,
                name: task.task.name,
                description: task.task.description,
                goal: task.task.goal,
                execution_target_type: task.task.execution_target_type,
                task_mode: task.task.task_mode,
                status: task.task.status,
                test_set_ids: task.test_set_ids,
                final_prompt: task.task.final_prompt,
                terminated_at: task.task.terminated_at,
                selected_iteration_id: task.task.selected_iteration_id,
                created_at: task.task.created_at,
                updated_at: task.task.updated_at,
            })
        }
        Err(OptimizationTaskRepoError::NotFound) => optimization_task_not_found(),
        Err(OptimizationTaskRepoError::WorkspaceNotFound) => workspace_not_found(),
        Err(e) => {
            warn!(error = %e, "获取优化任务失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "获取优化任务失败",
            )
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}/config",
    request_body = UpdateOptimizationTaskConfigRequest,
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("task_id" = String, Path, description = "优化任务 ID")
    ),
    responses(
        (status = 200, description = "更新成功", body = ApiSuccess<OptimizationTaskResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "资源不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "optimization_tasks"
)]
pub(crate) async fn update_optimization_task_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, task_id)): Path<(String, String)>,
    current_user: CurrentUser,
    Json(req): Json<UpdateOptimizationTaskConfigRequest>,
) -> ApiResponse<OptimizationTaskResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    let initial_prompt_len = req.initial_prompt.as_ref().map(|s| s.len()).unwrap_or(0);
    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        workspace_id = %workspace_id,
        task_id = %task_id,
        initial_prompt_len = initial_prompt_len,
        "更新优化任务配置"
    );

    if let Err(resp) =
        ensure_workspace_exists::<OptimizationTaskResponse>(&state, &workspace_id, user_id).await
    {
        return resp;
    }

    let config = OptimizationTaskConfig {
        schema_version: OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION,
        initial_prompt: req.initial_prompt,
        max_iterations: req.max_iterations,
        pass_threshold_percent: req.pass_threshold_percent,
        candidate_prompt_count: req.candidate_prompt_count,
        diversity_injection_threshold: req.diversity_injection_threshold,
        execution_mode: req.execution_mode,
        max_concurrency: req.max_concurrency,
        data_split: DataSplitPercentConfig {
            train_percent: req.train_percent,
            validation_percent: req.validation_percent,
            holdout_percent: 0,
        },
        output_config: req.output_config,
        evaluator_config: req.evaluator_config,
        teacher_llm: req.teacher_llm,
        advanced_data_split: req.advanced_data_split,
    }
    .normalized();

    if let Err(resp) = validate_task_config::<OptimizationTaskResponse>(&config) {
        return resp;
    }

    match OptimizationTaskRepo::update_config_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &task_id,
        config,
    )
    .await
    {
        Ok(updated) => {
            warn_if_invalid_config_json(
                &correlation_id,
                user_id,
                &workspace_id,
                &task_id,
                updated.task.config_json.as_deref(),
            );
            ApiResponse::ok(OptimizationTaskResponse {
                config: OptimizationTaskConfig::normalized_from_config_json(
                    updated.task.config_json.as_deref(),
                ),
                id: updated.task.id,
                workspace_id: updated.task.workspace_id,
                name: updated.task.name,
                description: updated.task.description,
                goal: updated.task.goal,
                execution_target_type: updated.task.execution_target_type,
                task_mode: updated.task.task_mode,
                status: updated.task.status,
                test_set_ids: updated.test_set_ids,
                final_prompt: updated.task.final_prompt,
                terminated_at: updated.task.terminated_at,
                selected_iteration_id: updated.task.selected_iteration_id,
                created_at: updated.task.created_at,
                updated_at: updated.task.updated_at,
            })
        }
        Err(OptimizationTaskRepoError::NotFound) => optimization_task_not_found(),
        Err(OptimizationTaskRepoError::WorkspaceNotFound) => workspace_not_found(),
        Err(OptimizationTaskRepoError::InvalidConfig(msg)) => {
            ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg)
        }
        Err(e) => {
            warn!(error = %e, "更新优化任务配置失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "更新优化任务配置失败",
            )
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_optimization_task).get(list_optimization_tasks),
        )
        .route("/{task_id}", get(get_optimization_task))
        .route("/{task_id}/config", put(update_optimization_task_config))
}
