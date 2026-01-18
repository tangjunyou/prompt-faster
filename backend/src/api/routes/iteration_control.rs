//! 迭代控制 API 路由
//!
//! 提供增加轮数、获取候选 Prompt 列表和终止任务的 RESTful API 端点。
//! 路由前缀: /api/v1/tasks/{task_id}

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, patch, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::domain::types::{
    AddRoundsRequest, AddRoundsResponse, CandidatePromptListResponse, CandidatePromptSummary,
    TerminateTaskRequest, TerminateTaskResponse, unix_ms_to_iso8601,
};
use crate::infra::db::repositories::{
    IterationRepo, IterationRepoError, IterationSummaryWithArtifacts, OptimizationTaskRepo,
};
use crate::shared::error_codes;
use crate::shared::ws::{EVT_TASK_TERMINATED, TaskTerminatedPayload, WsMessage};
use crate::shared::ws_bus::global_ws_bus;

/// 候选 Prompt 列表最大返回数量
const MAX_CANDIDATES: i32 = 100;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CandidatesQuery {
    /// 最大返回条数（默认 100）
    pub limit: Option<i32>,
    /// 偏移量（默认 0）
    pub offset: Option<i32>,
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
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

async fn task_exists(pool: &SqlitePool, task_id: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM optimization_tasks WHERE id = ?1")
        .bind(task_id)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}

/// 增加迭代轮数
///
/// PATCH /api/v1/tasks/{task_id}/config
/// 仅允许在 Running/Paused 状态下操作
#[utoipa::path(
    patch,
    path = "/api/v1/tasks/{task_id}/config",
    request_body = AddRoundsRequest,
    params(
        ("task_id" = String, Path, description = "优化任务 ID")
    ),
    responses(
        (status = 200, description = "更新成功", body = ApiSuccess<AddRoundsResponse>),
        (status = 400, description = "参数错误或状态不允许", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 403, description = "无权访问该任务", body = ApiError),
        (status = 404, description = "任务不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "iteration_control"
)]
pub(crate) async fn add_rounds(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
    Json(req): Json<AddRoundsRequest>,
) -> ApiResponse<AddRoundsResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;
    let timestamp = current_unix_ms();

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "add_rounds",
        additional_rounds = req.additional_rounds,
        timestamp = timestamp,
        "增加迭代轮数请求"
    );

    // 验证请求参数
    if let Err(msg) = req.validate() {
        warn!(
            correlation_id = %correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            error = %msg,
            "增加轮数参数校验失败"
        );
        return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg);
    }

    // 查询任务（权限校验）
    let task = match OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id).await {
        Ok(task) => task,
        Err(crate::infra::db::repositories::OptimizationTaskRepoError::NotFound) => {
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
                Err(e) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %e,
                        "检查任务存在性失败"
                    );
                    return ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询优化任务失败",
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "查询优化任务失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询优化任务失败",
            );
        }
    };

    // 状态校验：仅 Running/Paused 状态可操作
    let status = task.status;
    if !matches!(
        status,
        crate::domain::models::OptimizationTaskStatus::Running
            | crate::domain::models::OptimizationTaskStatus::Paused
    ) {
        warn!(
            correlation_id = %correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            current_status = ?status,
            "增加轮数状态校验失败：任务不在 Running/Paused 状态"
        );
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "仅在运行中或已暂停状态下可增加轮数",
        );
    }

    // 解析当前配置
    let config = crate::domain::models::OptimizationTaskConfig::normalized_from_config_json(
        task.config_json.as_deref(),
    );
    let prev_max_iterations = config.max_iterations as i32;
    let new_max_iterations = prev_max_iterations + req.additional_rounds;

    // 获取当前轮次（从最新迭代记录）
    let current_round =
        match IterationRepo::list_by_task_id(&state.db, user_id, &task_id, Some(1)).await {
            Ok(iterations) => iterations.first().map(|i| i.round).unwrap_or(0),
            Err(_) => 0,
        };

    // 更新配置
    let mut new_config = config.clone();
    new_config.max_iterations = new_max_iterations as u32;

    // 获取任务的 workspace_id
    let workspace_id = &task.workspace_id;

    match OptimizationTaskRepo::update_config_scoped(
        &state.db,
        user_id,
        workspace_id,
        &task_id,
        new_config,
    )
    .await
    {
        Ok(_) => {
            let controller = global_pause_registry().get_or_create(&task_id).await;
            controller
                .set_max_iterations_override(new_max_iterations as u32)
                .await;

            info!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                action = "add_rounds",
                prev_value = prev_max_iterations,
                new_value = new_max_iterations,
                prev_state = ?status,
                new_state = ?status,
                iteration_state = current_round,
                timestamp = current_unix_ms(),
                "增加轮数成功"
            );

            ApiResponse::ok(AddRoundsResponse {
                previous_max_iterations: prev_max_iterations,
                new_max_iterations,
                current_round,
            })
        }
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "更新任务配置失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "更新任务配置失败",
            )
        }
    }
}

/// 获取候选 Prompt 列表
///
/// GET /api/v1/tasks/{task_id}/candidates
/// 返回按通过率降序排列的候选 Prompt 列表
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/candidates",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        ("limit" = Option<i32>, Query, description = "最大返回数量（默认 100）"),
        ("offset" = Option<i32>, Query, description = "偏移量（默认 0）")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<CandidatePromptListResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "任务不存在", body = ApiError),
        (status = 403, description = "无权访问该任务", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "iteration_control"
)]
pub(crate) async fn get_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<CandidatesQuery>,
    current_user: CurrentUser,
) -> ApiResponse<CandidatePromptListResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "get_candidates",
        timestamp = current_unix_ms(),
        "获取候选 Prompt 列表"
    );

    // 查询历史迭代（仅 completed），避免 N+1
    let limit = query.limit.unwrap_or(MAX_CANDIDATES);
    let iterations: Vec<IterationSummaryWithArtifacts> =
        match IterationRepo::list_with_artifacts_by_task_id(
            &state.db,
            user_id,
            &task_id,
            Some(limit),
            query.offset,
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
                    Err(e) => {
                        warn!(
                            correlation_id = %correlation_id,
                            error = %e,
                            "检查任务存在性失败"
                        );
                        return ApiResponse::err(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            error_codes::DATABASE_ERROR,
                            "查询优化任务失败",
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %e,
                    "查询迭代历史失败"
                );
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询迭代历史失败",
                );
            }
        };

    // 提取每轮的候选 Prompt
    let mut candidates: Vec<CandidatePromptSummary> = Vec::new();

    for iteration in &iterations {
        let summary = &iteration.summary;
        let artifacts = &iteration.artifacts;
        if artifacts.candidate_prompts.is_empty() {
            continue;
        }

        // 优先选择 is_best，否则取第一条
        let best_prompt = artifacts
            .candidate_prompts
            .iter()
            .find(|p| p.is_best)
            .or_else(|| artifacts.candidate_prompts.first());

        if let Some(prompt) = best_prompt {
            let prompt_preview = CandidatePromptSummary::generate_preview(&prompt.content);
            candidates.push(CandidatePromptSummary {
                iteration_id: summary.id.clone(),
                round: summary.round,
                pass_rate: summary.pass_rate,
                passed_cases: summary.passed_cases,
                total_cases: summary.total_cases,
                prompt: prompt.content.clone(),
                prompt_preview,
                completed_at: summary
                    .completed_at
                    .clone()
                    .unwrap_or_else(|| summary.started_at.clone()),
            });
        }
    }

    // 按通过率降序排列
    candidates.sort_by(|a, b| {
        b.pass_rate
            .partial_cmp(&a.pass_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total = candidates.len() as i32;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        count = total,
        "候选 Prompt 列表查询成功"
    );

    ApiResponse::ok(CandidatePromptListResponse { candidates, total })
}

/// 终止任务
///
/// POST /api/v1/tasks/{task_id}/terminate
/// 仅在 Running/Paused 状态下可操作
#[utoipa::path(
    post,
    path = "/api/v1/tasks/{task_id}/terminate",
    request_body = TerminateTaskRequest,
    params(
        ("task_id" = String, Path, description = "优化任务 ID")
    ),
    responses(
        (status = 200, description = "终止成功", body = ApiSuccess<TerminateTaskResponse>),
        (status = 400, description = "参数错误或状态不允许", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 403, description = "无权访问该任务", body = ApiError),
        (status = 404, description = "任务或迭代不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "iteration_control"
)]
pub(crate) async fn terminate_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
    Json(req): Json<TerminateTaskRequest>,
) -> ApiResponse<TerminateTaskResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;
    let timestamp = current_unix_ms();

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "terminate",
        selected_iteration_id = ?req.selected_iteration_id,
        timestamp = timestamp,
        "终止任务请求"
    );

    // 查询任务（权限校验）
    let task = match OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id).await {
        Ok(task) => task,
        Err(crate::infra::db::repositories::OptimizationTaskRepoError::NotFound) => {
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
                Err(e) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %e,
                        "检查任务存在性失败"
                    );
                    return ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询优化任务失败",
                    );
                }
            }
        }
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "查询优化任务失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询优化任务失败",
            );
        }
    };

    // 状态校验：仅 Running/Paused 状态可操作
    let prev_status = task.status;
    if !matches!(
        prev_status,
        crate::domain::models::OptimizationTaskStatus::Running
            | crate::domain::models::OptimizationTaskStatus::Paused
    ) {
        warn!(
            correlation_id = %correlation_id,
            user_id = %user_id,
            task_id = %task_id,
            current_status = ?prev_status,
            "终止任务状态校验失败：任务不在 Running/Paused 状态"
        );
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "仅在运行中或已暂停状态下可终止任务",
        );
    }

    // 触发引擎终止信号（运行中内存状态）
    let controller = global_pause_registry().get_or_create(&task_id).await;
    controller.request_stop(&correlation_id, user_id).await;

    // 如果指定了迭代 ID，获取对应的 Prompt
    let (final_prompt, selected_round) = if let Some(ref iteration_id) = req.selected_iteration_id {
        match IterationRepo::get_by_id(&state.db, user_id, &task_id, iteration_id).await {
            Ok(detail) => {
                // 从 artifacts 中提取最佳 Prompt
                let best_prompt = detail
                    .artifacts
                    .candidate_prompts
                    .iter()
                    .find(|p| p.is_best)
                    .or_else(|| detail.artifacts.candidate_prompts.first())
                    .map(|p| p.content.clone());
                (best_prompt, Some(detail.round))
            }
            Err(IterationRepoError::NotFound) => {
                return ApiResponse::err(
                    StatusCode::NOT_FOUND,
                    error_codes::NOT_FOUND,
                    "指定的迭代记录不存在",
                );
            }
            Err(IterationRepoError::TaskNotFoundOrForbidden) => {
                return ApiResponse::err(
                    StatusCode::FORBIDDEN,
                    error_codes::FORBIDDEN,
                    "任务不存在或无权访问",
                );
            }
            Err(e) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %e,
                    "查询迭代详情失败"
                );
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询迭代详情失败",
                );
            }
        }
    } else {
        (None, None)
    };

    // 更新任务状态为 terminated
    let terminated_at = timestamp;
    let terminated_at_iso = unix_ms_to_iso8601(terminated_at);

    match sqlx::query(
        r#"
        UPDATE optimization_tasks
        SET status = 'terminated',
            terminated_at = ?1,
            final_prompt = ?2,
            selected_iteration_id = ?3,
            updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(terminated_at)
    .bind(&final_prompt)
    .bind(&req.selected_iteration_id)
    .bind(terminated_at)
    .bind(&task_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let payload = TaskTerminatedPayload {
                task_id: task_id.clone(),
                terminated_at: terminated_at_iso.clone(),
                final_prompt: final_prompt.clone(),
                selected_iteration_id: req.selected_iteration_id.clone(),
            };
            let msg = WsMessage::new(EVT_TASK_TERMINATED, payload, correlation_id.clone());
            if let Ok(text) = serde_json::to_string(&msg) {
                global_ws_bus().publish(text);
            }

            info!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %task_id,
                action = "terminate",
                prev_state = ?prev_status,
                new_state = "terminated",
                iteration_state = ?selected_round,
                timestamp = current_unix_ms(),
                "任务终止成功"
            );

            ApiResponse::ok(TerminateTaskResponse {
                task_id: task_id.clone(),
                terminated_at: terminated_at_iso,
                final_prompt,
                selected_round,
            })
        }
        Err(e) => {
            warn!(
                correlation_id = %correlation_id,
                error = %e,
                "更新任务状态失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "终止任务失败",
            )
        }
    }
}

/// 创建迭代控制路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", patch(add_rounds))
        .route("/candidates", get(get_candidates))
        .route("/terminate", post(terminate_task))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::middleware::CurrentUser;
    use crate::api::middleware::{LoginAttemptStore, SessionStore};
    use crate::core::iteration_engine::pause_state::global_pause_registry;
    use crate::domain::models::optimization_task_config::{
        OptimizationTaskConfig, serialize_config_with_existing_extra,
    };
    use crate::domain::models::{
        ExecutionTargetType, OptimizationTaskMode, OptimizationTaskStatus,
    };
    use crate::domain::types::{ArtifactSource, CandidatePrompt, IterationArtifacts};
    use crate::infra::db::pool::create_pool;
    use crate::infra::db::repositories::{OptimizationTaskRepo, TestSetRepo, WorkspaceRepo};
    use crate::infra::external::api_key_manager::ApiKeyManager;
    use crate::shared::config::AppConfig;
    use crate::shared::ws::chrono_timestamp;
    use reqwest::Client;
    use sqlx::SqlitePool;
    use std::sync::Arc;

    async fn setup_state() -> AppState {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("创建测试数据库失败");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("运行 migrations 失败");

        let config = Arc::new(AppConfig {
            database_url: "sqlite::memory:".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            log_level: "info".to_string(),
            is_dev: true,
            cors_origins: Vec::new(),
            is_docker: false,
            allow_http_base_url: true,
            allow_localhost_base_url: true,
            allow_private_network_base_url: true,
        });

        AppState {
            db: pool,
            http_client: Client::new(),
            config,
            api_key_manager: Arc::new(ApiKeyManager::new(None)),
            session_store: SessionStore::new(24),
            login_attempt_store: LoginAttemptStore::default(),
        }
    }

    async fn insert_user(pool: &SqlitePool, id: &str, username: &str) {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind("test_hash")
        .bind(0_i64)
        .bind(0_i64)
        .execute(pool)
        .await
        .expect("插入测试用户失败");
    }

    async fn seed_task(pool: &SqlitePool, user_id: &str, status: &str) -> (String, String) {
        insert_user(pool, user_id, "user").await;

        let workspace = WorkspaceRepo::create(pool, user_id, "ws", None)
            .await
            .expect("创建工作区失败");

        let test_set = TestSetRepo::create(pool, &workspace.id, "ts", None, &[], None, None)
            .await
            .expect("创建测试集失败");

        let created = OptimizationTaskRepo::create_scoped(
            pool,
            crate::infra::db::repositories::CreateOptimizationTaskInput {
                user_id,
                workspace_id: &workspace.id,
                name: "task",
                description: Some("desc"),
                goal: "goal",
                execution_target_type: ExecutionTargetType::Generic,
                task_mode: OptimizationTaskMode::Fixed,
                test_set_ids: std::slice::from_ref(&test_set.id),
            },
        )
        .await
        .expect("创建任务失败");

        let config = OptimizationTaskConfig::default();
        let config_json =
            serialize_config_with_existing_extra(config, None).expect("序列化 config 失败");

        sqlx::query(
            r#"
            UPDATE optimization_tasks
            SET status = ?1,
                config_json = ?2,
                updated_at = ?3
            WHERE id = ?4
            "#,
        )
        .bind(status)
        .bind(String::from_utf8(config_json).expect("config json 转换失败"))
        .bind(1_i64)
        .bind(&created.task.id)
        .execute(pool)
        .await
        .expect("更新任务状态失败");

        (workspace.id, created.task.id)
    }

    async fn insert_iteration(
        pool: &SqlitePool,
        task_id: &str,
        prompt: &str,
        status: &str,
    ) -> String {
        let iteration_id = "iter-1".to_string();
        let artifacts = IterationArtifacts {
            candidate_prompts: vec![CandidatePrompt {
                id: "candidate:1".to_string(),
                content: prompt.to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: true,
            }],
            updated_at: chrono_timestamp(),
            ..IterationArtifacts::default()
        };
        let artifacts_json = serde_json::to_string(&artifacts).expect("序列化 artifacts 失败");

        sqlx::query(
            r#"
            INSERT INTO iterations (
                id, task_id, round, started_at, completed_at, status,
                artifacts, evaluation_results, reflection_summary,
                pass_rate, total_cases, passed_cases, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&iteration_id)
        .bind(task_id)
        .bind(1_i32)
        .bind(0_i64)
        .bind(1_i64)
        .bind(status)
        .bind(artifacts_json)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(0.9_f64)
        .bind(10_i32)
        .bind(9_i32)
        .bind(1_i64)
        .execute(pool)
        .await
        .expect("插入迭代记录失败");

        iteration_id
    }

    #[tokio::test]
    async fn add_rounds_updates_config_and_runtime_override() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "running").await;

        let response = add_rounds(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id.clone()),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
            Json(AddRoundsRequest {
                additional_rounds: 3,
            }),
        )
        .await;

        let ApiResponse::Success(success) = response else {
            panic!("expected success response");
        };

        assert_eq!(success.data.previous_max_iterations, 10);
        assert_eq!(success.data.new_max_iterations, 13);

        let task = OptimizationTaskRepo::find_by_id_for_user(&state.db, "u1", &task_id)
            .await
            .expect("查询任务失败");
        let config =
            OptimizationTaskConfig::normalized_from_config_json(task.config_json.as_deref());
        assert_eq!(config.max_iterations, 13);

        let controller = global_pause_registry().get_or_create(&task_id).await;
        assert_eq!(controller.get_max_iterations_override().await, Some(13));
    }

    #[tokio::test]
    async fn add_rounds_forbidden_for_other_user() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "running").await;
        insert_user(&state.db, "u2", "user2").await;

        let response = add_rounds(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id),
            CurrentUser {
                user_id: "u2".to_string(),
                unlock_context: None,
            },
            Json(AddRoundsRequest {
                additional_rounds: 1,
            }),
        )
        .await;

        let ApiResponse::Error(status, err) = response else {
            panic!("expected error response");
        };
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(err.error.code, error_codes::FORBIDDEN);
    }

    #[tokio::test]
    async fn add_rounds_rejects_invalid_status() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "draft").await;

        let response = add_rounds(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
            Json(AddRoundsRequest {
                additional_rounds: 1,
            }),
        )
        .await;

        let ApiResponse::Error(status, err) = response else {
            panic!("expected error response");
        };
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(err.error.code, error_codes::VALIDATION_ERROR);
    }

    #[tokio::test]
    async fn terminate_task_updates_state_and_sets_stop_flag() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "running").await;

        let iteration_id = insert_iteration(&state.db, &task_id, "Final Prompt", "completed").await;

        let response = terminate_task(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id.clone()),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
            Json(TerminateTaskRequest {
                selected_iteration_id: Some(iteration_id.clone()),
            }),
        )
        .await;

        let ApiResponse::Success(success) = response else {
            panic!("expected success response");
        };

        assert_eq!(success.data.task_id, task_id);
        assert_eq!(success.data.final_prompt.as_deref(), Some("Final Prompt"));
        assert_eq!(success.data.selected_round, Some(1));

        let task = OptimizationTaskRepo::find_by_id_for_user(&state.db, "u1", &task_id)
            .await
            .expect("查询任务失败");
        assert_eq!(task.status, OptimizationTaskStatus::Terminated);
        assert_eq!(task.final_prompt.as_deref(), Some("Final Prompt"));
        assert_eq!(
            task.selected_iteration_id.as_deref(),
            Some(iteration_id.as_str())
        );
        assert!(task.terminated_at.is_some());

        let controller = global_pause_registry().get_or_create(&task_id).await;
        assert!(controller.is_stop_requested());
    }

    #[tokio::test]
    async fn terminate_task_forbidden_for_other_user() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "running").await;
        insert_user(&state.db, "u2", "user2").await;

        let response = terminate_task(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id),
            CurrentUser {
                user_id: "u2".to_string(),
                unlock_context: None,
            },
            Json(TerminateTaskRequest {
                selected_iteration_id: None,
            }),
        )
        .await;

        let ApiResponse::Error(status, err) = response else {
            panic!("expected error response");
        };
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(err.error.code, error_codes::FORBIDDEN);
    }

    #[tokio::test]
    async fn terminate_task_rejects_invalid_status() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "completed").await;

        let response = terminate_task(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
            Json(TerminateTaskRequest {
                selected_iteration_id: None,
            }),
        )
        .await;

        let ApiResponse::Error(status, err) = response else {
            panic!("expected error response");
        };
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(err.error.code, error_codes::VALIDATION_ERROR);
    }

    #[tokio::test]
    async fn get_candidates_returns_only_completed_iterations() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "running").await;

        let _ = insert_iteration(&state.db, &task_id, "Final Prompt", "completed").await;

        // 插入一条 running 迭代（即使有候选，也应被过滤）
        let running_artifacts = IterationArtifacts {
            candidate_prompts: vec![CandidatePrompt {
                id: "candidate:running".to_string(),
                content: "Should not appear".to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: true,
            }],
            updated_at: chrono_timestamp(),
            ..IterationArtifacts::default()
        };
        let running_artifacts_json =
            serde_json::to_string(&running_artifacts).expect("序列化 running artifacts 失败");
        sqlx::query(
            r#"
            INSERT INTO iterations (
                id, task_id, round, started_at, completed_at, status,
                artifacts, evaluation_results, reflection_summary,
                pass_rate, total_cases, passed_cases, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind("iter-running")
        .bind(&task_id)
        .bind(2_i32)
        .bind(0_i64)
        .bind(1_i64)
        .bind("running")
        .bind(running_artifacts_json)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(0.5_f64)
        .bind(10_i32)
        .bind(5_i32)
        .bind(1_i64)
        .execute(&state.db)
        .await
        .expect("插入 running 迭代失败");

        let response = get_candidates(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id.clone()),
            Query(CandidatesQuery {
                limit: None,
                offset: None,
            }),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
        )
        .await;

        let ApiResponse::Success(success) = response else {
            panic!("expected success response");
        };

        assert_eq!(success.data.candidates.len(), 1);
        assert_eq!(success.data.candidates[0].prompt, "Final Prompt");
    }

    #[tokio::test]
    async fn get_candidates_returns_404_when_task_missing() {
        let state = setup_state().await;

        let response = get_candidates(
            State(state.clone()),
            HeaderMap::new(),
            Path("missing-task".to_string()),
            Query(CandidatesQuery {
                limit: None,
                offset: None,
            }),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
        )
        .await;

        let ApiResponse::Error(status, err) = response else {
            panic!("expected error response");
        };
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(err.error.code, error_codes::OPTIMIZATION_TASK_NOT_FOUND);
    }

    #[tokio::test]
    async fn get_candidates_forbidden_for_other_user() {
        let state = setup_state().await;
        let (_workspace_id, task_id) = seed_task(&state.db, "u1", "running").await;
        insert_user(&state.db, "u2", "user2").await;

        let response = get_candidates(
            State(state.clone()),
            HeaderMap::new(),
            Path(task_id),
            Query(CandidatesQuery {
                limit: None,
                offset: None,
            }),
            CurrentUser {
                user_id: "u2".to_string(),
                unlock_context: None,
            },
        )
        .await;

        let ApiResponse::Error(status, err) = response else {
            panic!("expected error response");
        };
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(err.error.code, error_codes::FORBIDDEN);
    }
}
