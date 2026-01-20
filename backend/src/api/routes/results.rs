//! 任务结果查看与导出 API

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Router, routing::get};
use serde::Deserialize;
use sqlx::SqlitePool;
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::core::result_formatter::{format_as_json, format_as_markdown, format_as_xml};
use crate::domain::models::{
    ExportResultResponse, IterationSummaryEntry, OptimizationTaskStatus, ResultExportFormat,
    TaskResultView,
};
use crate::domain::types::unix_ms_to_iso8601;
use crate::infra::db::repositories::{
    IterationRepo, IterationRepoError, OptimizationTaskRepo, OptimizationTaskRepoError,
};
use crate::shared::error_codes;
use crate::shared::time::now_millis;

#[derive(Debug)]
struct ResultError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ResultError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    fn to_response<T: serde::Serialize>(&self) -> ApiResponse<T> {
        ApiResponse::err(self.status, self.code, &self.message)
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ExportResultQuery {
    /// 导出格式：markdown | json | xml
    pub format: Option<String>,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn task_status_label(status: OptimizationTaskStatus) -> &'static str {
    match status {
        OptimizationTaskStatus::Draft => "draft",
        OptimizationTaskStatus::Running => "running",
        OptimizationTaskStatus::Paused => "paused",
        OptimizationTaskStatus::Completed => "completed",
        OptimizationTaskStatus::Terminated => "terminated",
    }
}

async fn task_exists(pool: &SqlitePool, task_id: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM optimization_tasks WHERE id = ?1")
        .bind(task_id)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}

fn parse_export_format(value: &str) -> Option<ResultExportFormat> {
    match value.trim().to_ascii_lowercase().as_str() {
        "markdown" => Some(ResultExportFormat::Markdown),
        "json" => Some(ResultExportFormat::Json),
        "xml" => Some(ResultExportFormat::Xml),
        _ => None,
    }
}

fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_underscore = false;
    for ch in input.chars() {
        let mapped = match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c if c.is_whitespace() => '_',
            _ => ch,
        };
        if mapped == '_' {
            if prev_underscore {
                continue;
            }
            prev_underscore = true;
        } else {
            prev_underscore = false;
        }
        out.push(mapped);
    }
    out.trim_matches('_').to_string()
}

fn filename_timestamp() -> String {
    unix_ms_to_iso8601(now_millis()).replace([':', '.'], "-")
}

fn select_best_prompt(artifacts: &crate::domain::types::IterationArtifacts) -> Option<String> {
    artifacts
        .candidate_prompts
        .iter()
        .find(|prompt| prompt.is_best)
        .or_else(|| artifacts.candidate_prompts.first())
        .map(|prompt| prompt.content.clone())
}

async fn build_task_result_view(
    state: &AppState,
    user_id: &str,
    task_id: &str,
    correlation_id: &str,
) -> Result<TaskResultView, ResultError> {
    let task = match OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, task_id).await {
        Ok(task) => task,
        Err(OptimizationTaskRepoError::NotFound) => match task_exists(&state.db, task_id).await {
            Ok(true) => {
                return Err(ResultError::new(
                    StatusCode::FORBIDDEN,
                    error_codes::FORBIDDEN,
                    "无权访问该任务",
                ));
            }
            Ok(false) => {
                return Err(ResultError::new(
                    StatusCode::NOT_FOUND,
                    error_codes::OPTIMIZATION_TASK_NOT_FOUND,
                    "任务不存在",
                ));
            }
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "检查任务存在性失败"
                );
                return Err(ResultError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询任务失败",
                ));
            }
        },
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询任务失败"
            );
            return Err(ResultError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询任务失败",
            ));
        }
    };

    if task.status == OptimizationTaskStatus::Draft {
        return Err(ResultError::new(
            StatusCode::NOT_FOUND,
            error_codes::NOT_FOUND,
            "任务未开始",
        ));
    }

    let completed_iterations = match IterationRepo::list_with_artifacts_by_task_id(
        &state.db,
        user_id,
        task_id,
        Some(100),
        Some(0),
        Some("completed"),
    )
    .await
    {
        Ok(list) => list,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            return Err(ResultError::new(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "任务不存在或无权访问",
            ));
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询迭代摘要失败"
            );
            return Err(ResultError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询迭代摘要失败",
            ));
        }
    };

    let iteration_summary: Vec<IterationSummaryEntry> = completed_iterations
        .iter()
        .map(|item| IterationSummaryEntry {
            round: item.summary.round.max(0) as u32,
            pass_rate: Some(item.summary.pass_rate),
            status: item.summary.status.as_str().to_string(),
        })
        .collect();

    let completed_at = match task.status {
        OptimizationTaskStatus::Completed => completed_iterations
            .first()
            .and_then(|item| item.summary.completed_at.clone()),
        OptimizationTaskStatus::Terminated => task.terminated_at.map(unix_ms_to_iso8601),
        _ => None,
    };

    let total_iterations = match IterationRepo::count_by_task_id(&state.db, user_id, task_id).await
    {
        Ok(count) => count.max(0) as u32,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            return Err(ResultError::new(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "任务不存在或无权访问",
            ));
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "统计迭代轮次失败"
            );
            return Err(ResultError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "统计迭代轮次失败",
            ));
        }
    };

    let mut best_prompt: Option<String> = None;
    let mut pass_rate: Option<f64> = None;

    if let Some(selected_iteration_id) = task.selected_iteration_id.as_deref() {
        match IterationRepo::get_by_id(&state.db, user_id, task_id, selected_iteration_id).await {
            Ok(detail) => {
                if let Some(content) = select_best_prompt(&detail.artifacts) {
                    best_prompt = Some(content);
                    pass_rate = Some(detail.pass_rate);
                }
            }
            Err(IterationRepoError::NotFound) => {}
            Err(IterationRepoError::TaskNotFoundOrForbidden) => {
                return Err(ResultError::new(
                    StatusCode::FORBIDDEN,
                    error_codes::FORBIDDEN,
                    "任务不存在或无权访问",
                ));
            }
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询选定迭代失败"
                );
                return Err(ResultError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询选定迭代失败",
                ));
            }
        }
    }

    if best_prompt.is_none() {
        if let Some(final_prompt) = task.final_prompt.clone() {
            best_prompt = Some(final_prompt);
        }
    }

    let completed_best = match IterationRepo::find_best_completed_with_artifacts_by_task_id(
        &state.db, user_id, task_id,
    )
    .await
    {
        Ok(Some(best)) => {
            select_best_prompt(&best.artifacts).map(|prompt| (prompt, best.summary.pass_rate))
        }
        Ok(None) => None,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            return Err(ResultError::new(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "任务不存在或无权访问",
            ));
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询最佳迭代失败"
            );
            return Err(ResultError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询最佳迭代失败",
            ));
        }
    };
    if best_prompt.is_none() {
        best_prompt = completed_best.as_ref().map(|(prompt, _)| prompt.clone());
    }
    if pass_rate.is_none() {
        pass_rate = completed_best.map(|(_, rate)| rate);
    }

    Ok(TaskResultView {
        task_id: task.id,
        task_name: task.name,
        status: task_status_label(task.status).to_string(),
        best_prompt,
        pass_rate,
        total_iterations,
        completed_at,
        created_at: unix_ms_to_iso8601(task.created_at),
        iteration_summary,
    })
}

/// 获取优化结果
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/result",
    params(
        ("task_id" = String, Path, description = "优化任务 ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<TaskResultView>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 404, description = "任务未开始"),
        (status = 500, description = "服务器错误")
    ),
    tag = "results"
)]
pub(crate) async fn get_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<TaskResultView> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "result:view",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = now_millis(),
        "查询任务结果"
    );

    match build_task_result_view(&state, user_id, &task_id, &correlation_id).await {
        Ok(result) => ApiResponse::ok(result),
        Err(err) => err.to_response(),
    }
}

/// 导出优化结果
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/result/export",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        ExportResultQuery
    ),
    responses(
        (status = 200, description = "导出成功", body = ApiSuccess<ExportResultResponse>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 404, description = "任务未开始"),
        (status = 500, description = "服务器错误")
    ),
    tag = "results"
)]
pub(crate) async fn export_result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<ExportResultQuery>,
    current_user: CurrentUser,
) -> ApiResponse<ExportResultResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "result:export",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = now_millis(),
        "导出任务结果"
    );

    let Some(format_raw) = query.format.as_deref() else {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "format 必填（markdown/json/xml）",
        );
    };
    let Some(format) = parse_export_format(format_raw) else {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "format 必须为 markdown/json/xml",
        );
    };

    let result = match build_task_result_view(&state, user_id, &task_id, &correlation_id).await {
        Ok(result) => result,
        Err(err) => return err.to_response(),
    };

    let content = match format {
        ResultExportFormat::Markdown => format_as_markdown(&result),
        ResultExportFormat::Json => format_as_json(&result),
        ResultExportFormat::Xml => format_as_xml(&result),
    };

    let base_name = sanitize_filename(&result.task_name);
    let safe_name = if base_name.is_empty() {
        task_id.clone()
    } else {
        base_name
    };
    let ext = match format {
        ResultExportFormat::Markdown => "md",
        ResultExportFormat::Json => "json",
        ResultExportFormat::Xml => "xml",
    };
    let filename = format!("{safe_name}_prompt_{}.{}", filename_timestamp(), ext);

    ApiResponse::ok(ExportResultResponse {
        content,
        format,
        filename,
    })
}

/// 创建结果路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_result))
        .route("/export", get(export_result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::middleware::{LoginAttemptStore, SessionStore};
    use crate::domain::models::{ExecutionTargetType, OptimizationTaskMode};
    use crate::infra::db::pool::create_pool;
    use crate::infra::db::repositories::{OptimizationTaskRepo, TestSetRepo, WorkspaceRepo};
    use crate::shared::config::AppConfig;
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
            checkpoint_cache_limit: 10,
            checkpoint_memory_alert_threshold: 10,
        });

        AppState {
            db: pool,
            http_client: Client::new(),
            config,
            api_key_manager: Arc::new(crate::infra::external::api_key_manager::ApiKeyManager::new(
                None,
            )),
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

    async fn seed_task(pool: &SqlitePool, user_id: &str, status: &str) -> String {
        insert_user(pool, user_id, "user").await;
        let workspace = WorkspaceRepo::create(pool, user_id, "ws", None)
            .await
            .expect("创建工作区失败");
        let test_set = TestSetRepo::create(pool, &workspace.id, "ts", None, &[], None, None)
            .await
            .expect("创建测试集失败");

        let input = crate::infra::db::repositories::CreateOptimizationTaskInput {
            user_id,
            workspace_id: &workspace.id,
            name: "任务A",
            description: None,
            goal: "goal",
            execution_target_type: ExecutionTargetType::Example,
            task_mode: OptimizationTaskMode::Fixed,
            test_set_ids: &[test_set.id.clone()],
        };
        let created = OptimizationTaskRepo::create_scoped(pool, input)
            .await
            .expect("创建任务失败");

        sqlx::query("UPDATE optimization_tasks SET status = ?1 WHERE id = ?2")
            .bind(status)
            .bind(&created.task.id)
            .execute(pool)
            .await
            .expect("更新任务状态失败");

        created.task.id
    }

    async fn update_task_selected(
        pool: &SqlitePool,
        task_id: &str,
        selected_iteration_id: Option<&str>,
        final_prompt: Option<&str>,
    ) {
        sqlx::query(
            r#"
            UPDATE optimization_tasks
            SET selected_iteration_id = ?1, final_prompt = ?2
            WHERE id = ?3
            "#,
        )
        .bind(selected_iteration_id)
        .bind(final_prompt)
        .bind(task_id)
        .execute(pool)
        .await
        .expect("更新任务选定迭代失败");
    }

    async fn insert_iteration(
        pool: &SqlitePool,
        task_id: &str,
        round: i32,
        status: &str,
        pass_rate: f64,
        artifacts: Option<String>,
    ) {
        let now = 1_700_000_000_000_i64 + (round as i64) * 1000;
        sqlx::query(
            r#"
            INSERT INTO iterations (
                id, task_id, round, started_at, completed_at, status, artifacts,
                evaluation_results, reflection_summary, pass_rate, total_cases, passed_cases, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL, ?8, 10, 8, ?9)
            "#,
        )
        .bind(format!("iter-{round}"))
        .bind(task_id)
        .bind(round)
        .bind(now)
        .bind(now + 1000)
        .bind(status)
        .bind(artifacts)
        .bind(pass_rate)
        .bind(now)
            .execute(pool)
            .await
            .expect("插入迭代失败");
    }

    fn build_artifacts(prompt: &str) -> String {
        serde_json::json!({
            "candidatePrompts": [
                {"id": "p1", "content": prompt, "isBest": true}
            ],
            "patterns": [],
            "updatedAt": "2025-01-01T00:00:00Z"
        })
        .to_string()
    }

    #[tokio::test]
    async fn test_build_task_result_view_basic() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "completed").await;

        let artifacts = build_artifacts("best");
        insert_iteration(&state.db, &task_id, 1, "completed", 0.8, Some(artifacts)).await;

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.task_id, task_id);
        assert_eq!(view.best_prompt.as_deref(), Some("best"));
        assert_eq!(view.iteration_summary.len(), 1);
    }

    #[tokio::test]
    async fn test_build_task_result_view_draft_not_found() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "draft").await;

        let err = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect_err("应返回错误");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_build_task_result_view_task_not_found() {
        let state = setup_state().await;
        insert_user(&state.db, "u1", "user").await;

        let err = build_task_result_view(&state, "u1", "missing-task", "c1")
            .await
            .expect_err("应返回错误");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_build_task_result_view_selected_iteration_priority() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "completed").await;

        let artifacts_selected = build_artifacts("selected");
        let artifacts_fallback = build_artifacts("fallback");

        insert_iteration(
            &state.db,
            &task_id,
            1,
            "completed",
            0.6,
            Some(artifacts_fallback),
        )
        .await;
        insert_iteration(
            &state.db,
            &task_id,
            2,
            "completed",
            0.9,
            Some(artifacts_selected),
        )
        .await;
        update_task_selected(&state.db, &task_id, Some("iter-2"), Some("final")).await;

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.best_prompt.as_deref(), Some("selected"));
    }

    #[tokio::test]
    async fn test_build_task_result_view_final_prompt_fallback() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "terminated").await;
        update_task_selected(&state.db, &task_id, None, Some("final-prompt")).await;

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.best_prompt.as_deref(), Some("final-prompt"));
    }

    #[tokio::test]
    async fn test_build_task_result_view_forbidden() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "completed").await;

        let err = build_task_result_view(&state, "u2", &task_id, "c1")
            .await
            .expect_err("应返回错误");
        assert_eq!(err.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_build_task_result_view_running_status() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "running").await;
        let artifacts = build_artifacts("running-prompt");
        insert_iteration(&state.db, &task_id, 1, "completed", 0.7, Some(artifacts)).await;

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.status, "running");
        assert_eq!(view.iteration_summary.len(), 1);
    }

    #[tokio::test]
    async fn test_build_task_result_view_paused_status() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "paused").await;
        let artifacts = build_artifacts("paused-prompt");
        insert_iteration(&state.db, &task_id, 1, "completed", 0.6, Some(artifacts)).await;

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.status, "paused");
        assert_eq!(view.iteration_summary.len(), 1);
    }

    #[tokio::test]
    async fn test_build_task_result_view_total_iterations_count() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "completed").await;

        insert_iteration(
            &state.db,
            &task_id,
            1,
            "completed",
            0.4,
            Some(build_artifacts("p1")),
        )
        .await;
        insert_iteration(
            &state.db,
            &task_id,
            3,
            "completed",
            0.5,
            Some(build_artifacts("p3")),
        )
        .await;
        insert_iteration(
            &state.db,
            &task_id,
            5,
            "completed",
            0.6,
            Some(build_artifacts("p5")),
        )
        .await;

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.total_iterations, 3);
    }

    #[tokio::test]
    async fn test_build_task_result_view_best_prompt_ignores_limit() {
        let state = setup_state().await;
        let task_id = seed_task(&state.db, "u1", "completed").await;

        insert_iteration(
            &state.db,
            &task_id,
            1,
            "completed",
            0.99,
            Some(build_artifacts("best-old")),
        )
        .await;

        for round in 2..=101 {
            insert_iteration(
                &state.db,
                &task_id,
                round,
                "completed",
                0.5,
                Some(build_artifacts(&format!("p-{round}"))),
            )
            .await;
        }

        let view = build_task_result_view(&state, "u1", &task_id, "c1")
            .await
            .expect("构建结果失败");
        assert_eq!(view.best_prompt.as_deref(), Some("best-old"));
    }

    #[test]
    fn test_parse_export_format() {
        assert_eq!(
            parse_export_format("markdown"),
            Some(ResultExportFormat::Markdown)
        );
        assert_eq!(parse_export_format("JSON"), Some(ResultExportFormat::Json));
        assert_eq!(parse_export_format("xml"), Some(ResultExportFormat::Xml));
        assert_eq!(parse_export_format("yaml"), None);
    }
}
