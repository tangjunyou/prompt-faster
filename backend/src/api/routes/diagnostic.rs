//! 诊断报告 API

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Router, routing::get};
use serde::Deserialize;
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::core::diagnostic_service::{
    DiagnosticServiceError, FAILED_CASES_DEFAULT_LIMIT, FAILED_CASES_MAX_LIMIT,
    generate_diagnostic_report, get_failed_case_detail,
};
use crate::domain::models::{DiagnosticReport, FailedCaseDetail};
use crate::infra::db::repositories::{OptimizationTaskRepo, OptimizationTaskRepoError};
use crate::shared::error_codes;
use crate::shared::time::now_millis;

#[derive(Debug, Deserialize, IntoParams)]
pub struct DiagnosticReportQuery {
    /// 失败用例最大返回条数（默认 50，最大 100）
    pub failed_cases_limit: Option<usize>,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

async fn task_exists(pool: &sqlx::SqlitePool, task_id: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM optimization_tasks WHERE id = ?1")
        .bind(task_id)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}

fn normalize_failed_cases_limit(value: Option<usize>) -> Result<usize, String> {
    let limit = value.unwrap_or(FAILED_CASES_DEFAULT_LIMIT);
    if limit == 0 || limit > FAILED_CASES_MAX_LIMIT {
        return Err(format!(
            "failed_cases_limit 必须在 1-{} 范围内",
            FAILED_CASES_MAX_LIMIT
        ));
    }
    Ok(limit)
}

/// 获取诊断报告
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/diagnostic",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        DiagnosticReportQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<DiagnosticReport>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 404, description = "任务不存在"),
        (status = 409, description = "任务尚未完成"),
        (status = 500, description = "服务器错误")
    ),
    tag = "diagnostic"
)]
pub(crate) async fn get_diagnostic_report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<DiagnosticReportQuery>,
    current_user: CurrentUser,
) -> ApiResponse<DiagnosticReport> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "diagnostic:report",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = now_millis(),
        "查询诊断报告"
    );

    let failed_cases_limit = match normalize_failed_cases_limit(query.failed_cases_limit) {
        Ok(value) => value,
        Err(msg) => {
            return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg);
        }
    };

    if let Err(err) = OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id).await
    {
        return match err {
            OptimizationTaskRepoError::NotFound => match task_exists(&state.db, &task_id).await {
                Ok(true) => ApiResponse::err(
                    StatusCode::FORBIDDEN,
                    error_codes::FORBIDDEN,
                    "无权访问该任务",
                ),
                Ok(false) => ApiResponse::err(
                    StatusCode::NOT_FOUND,
                    error_codes::OPTIMIZATION_TASK_NOT_FOUND,
                    "任务不存在",
                ),
                Err(err) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %err,
                        "检查任务存在性失败"
                    );
                    ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询任务失败",
                    )
                }
            },
            OptimizationTaskRepoError::DatabaseError(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询任务失败"
                );
                ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询任务失败",
                )
            }
            other => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %other,
                    "查询任务失败"
                );
                ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询任务失败",
                )
            }
        };
    }

    match generate_diagnostic_report(&state.db, user_id, &task_id, failed_cases_limit).await {
        Ok(report) => ApiResponse::ok(report),
        Err(err) => map_service_error(&correlation_id, err),
    }
}

/// 获取失败用例详情
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/diagnostic/cases/{case_id}",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        ("case_id" = String, Path, description = "失败用例 ID")
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<FailedCaseDetail>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 404, description = "失败用例不存在"),
        (status = 409, description = "任务尚未完成"),
        (status = 500, description = "服务器错误")
    ),
    tag = "diagnostic"
)]
pub(crate) async fn get_case_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((task_id, case_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<FailedCaseDetail> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "diagnostic:case_detail",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = now_millis(),
        "查询诊断失败用例详情"
    );

    if let Err(err) = OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id).await
    {
        return match err {
            OptimizationTaskRepoError::NotFound => match task_exists(&state.db, &task_id).await {
                Ok(true) => ApiResponse::err(
                    StatusCode::FORBIDDEN,
                    error_codes::FORBIDDEN,
                    "无权访问该任务",
                ),
                Ok(false) => ApiResponse::err(
                    StatusCode::NOT_FOUND,
                    error_codes::OPTIMIZATION_TASK_NOT_FOUND,
                    "任务不存在",
                ),
                Err(err) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %err,
                        "检查任务存在性失败"
                    );
                    ApiResponse::err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_codes::DATABASE_ERROR,
                        "查询任务失败",
                    )
                }
            },
            OptimizationTaskRepoError::DatabaseError(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询任务失败"
                );
                ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询任务失败",
                )
            }
            other => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %other,
                    "查询任务失败"
                );
                ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询任务失败",
                )
            }
        };
    }

    match get_failed_case_detail(&state.db, user_id, &task_id, &case_id).await {
        Ok(detail) => ApiResponse::ok(detail),
        Err(err) => map_service_error(&correlation_id, err),
    }
}

fn map_service_error<T: serde::Serialize>(
    correlation_id: &str,
    err: DiagnosticServiceError,
) -> ApiResponse<T> {
    match err {
        DiagnosticServiceError::TaskNotFoundOrForbidden => ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        ),
        DiagnosticServiceError::TaskNotStarted => {
            ApiResponse::err(StatusCode::NOT_FOUND, error_codes::NOT_FOUND, "任务未开始")
        }
        DiagnosticServiceError::TaskNotCompleted => ApiResponse::err(
            StatusCode::CONFLICT,
            error_codes::VALIDATION_ERROR,
            "任务尚未完成",
        ),
        DiagnosticServiceError::FailedCaseNotFound | DiagnosticServiceError::IterationNotFound => {
            ApiResponse::err(
                StatusCode::NOT_FOUND,
                error_codes::NOT_FOUND,
                "失败用例不存在",
            )
        }
        DiagnosticServiceError::InvalidCaseId => ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "case_id 格式错误",
        ),
        DiagnosticServiceError::Database(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "诊断报告查询失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询诊断报告失败",
            )
        }
        DiagnosticServiceError::Repo(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "诊断报告查询失败"
            );
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询诊断报告失败",
            )
        }
    }
}

/// 创建诊断报告路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_diagnostic_report))
        .route("/cases/{case_id}", get(get_case_detail))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::middleware::{LoginAttemptStore, SessionStore};
    use crate::domain::models::{
        ExecutionTargetType, OptimizationTaskMode, TaskReference, TestCase,
    };
    use crate::infra::db::pool::create_pool;
    use crate::infra::db::repositories::{OptimizationTaskRepo, TestSetRepo, WorkspaceRepo};
    use crate::shared::config::AppConfig;
    use reqwest::Client;
    use sqlx::SqlitePool;
    use std::collections::HashMap;
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

    async fn seed_task_with_case(pool: &SqlitePool, user_id: &str) -> (String, String) {
        insert_user(pool, user_id, "user").await;
        let workspace = WorkspaceRepo::create(pool, user_id, "ws", None)
            .await
            .expect("创建工作区失败");

        let test_case_id = "case-1".to_string();
        let cases = vec![TestCase {
            id: test_case_id.clone(),
            input: HashMap::from([(
                "prompt".to_string(),
                serde_json::Value::String("input".to_string()),
            )]),
            reference: TaskReference::Exact {
                expected: "expected".to_string(),
            },
            split: None,
            metadata: None,
        }];

        let test_set = TestSetRepo::create(pool, &workspace.id, "ts", None, &cases, None, None)
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
            teacher_prompt_version_id: None,
        };
        let created = OptimizationTaskRepo::create_scoped(pool, input)
            .await
            .expect("创建任务失败");

        sqlx::query("UPDATE optimization_tasks SET status = ?1 WHERE id = ?2")
            .bind("completed")
            .bind(&created.task.id)
            .execute(pool)
            .await
            .expect("更新任务状态失败");

        (created.task.id, test_case_id)
    }

    async fn insert_iteration(
        pool: &SqlitePool,
        task_id: &str,
        round: i32,
        evaluation_results: serde_json::Value,
    ) {
        let now = 1_700_000_000_000_i64 + (round as i64) * 1000;
        let artifacts = serde_json::json!({
            "candidatePrompts": [
                {"id": "p1", "content": "prompt", "isBest": true}
            ],
            "patterns": [],
            "updatedAt": "2025-01-01T00:00:00Z"
        })
        .to_string();

        sqlx::query(
            r#"
            INSERT INTO iterations (
                id, task_id, round, started_at, completed_at, status, artifacts,
                evaluation_results, reflection_summary, pass_rate, total_cases, passed_cases, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, 1, 0, ?10)
            "#,
        )
        .bind(format!("iter-{round}"))
        .bind(task_id)
        .bind(round)
        .bind(now)
        .bind(now + 1000)
        .bind("completed")
        .bind(artifacts)
        .bind(evaluation_results.to_string())
        .bind(0.0_f64)
        .bind(now)
        .execute(pool)
        .await
        .expect("插入迭代失败");
    }

    #[tokio::test]
    async fn test_get_diagnostic_report_success() {
        let state = setup_state().await;
        let (task_id, test_case_id) = seed_task_with_case(&state.db, "u1").await;

        let evaluation_results = serde_json::json!([
            {
                "testCaseId": test_case_id,
                "passed": false,
                "failureReason": "format",
                "actualOutput": "bad"
            }
        ]);
        insert_iteration(&state.db, &task_id, 1, evaluation_results).await;

        let response = get_diagnostic_report(
            State(state),
            HeaderMap::new(),
            Path(task_id),
            Query(DiagnosticReportQuery {
                failed_cases_limit: Some(10),
            }),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
        )
        .await;

        match response {
            ApiResponse::Success(success) => {
                assert_eq!(success.data.failed_cases.len(), 1);
            }
            _ => panic!("期望成功响应"),
        }
    }

    #[tokio::test]
    async fn test_get_diagnostic_report_not_found() {
        let state = setup_state().await;
        insert_user(&state.db, "u1", "user").await;

        let response = get_diagnostic_report(
            State(state),
            HeaderMap::new(),
            Path("missing".to_string()),
            Query(DiagnosticReportQuery {
                failed_cases_limit: None,
            }),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
        )
        .await;

        match response {
            ApiResponse::Error(status, _) => assert_eq!(status, StatusCode::NOT_FOUND),
            _ => panic!("期望 NOT_FOUND"),
        }
    }

    #[tokio::test]
    async fn test_get_diagnostic_report_forbidden() {
        let state = setup_state().await;
        let (task_id, _) = seed_task_with_case(&state.db, "u1").await;

        let response = get_diagnostic_report(
            State(state),
            HeaderMap::new(),
            Path(task_id),
            Query(DiagnosticReportQuery {
                failed_cases_limit: None,
            }),
            CurrentUser {
                user_id: "u2".to_string(),
                unlock_context: None,
            },
        )
        .await;

        match response {
            ApiResponse::Error(status, _) => assert_eq!(status, StatusCode::FORBIDDEN),
            _ => panic!("期望 FORBIDDEN"),
        }
    }

    #[tokio::test]
    async fn test_get_case_detail_returns_diff_segments() {
        let state = setup_state().await;
        let (task_id, test_case_id) = seed_task_with_case(&state.db, "u1").await;

        let evaluation_results = serde_json::json!([
            {
                "testCaseId": test_case_id,
                "passed": false,
                "failureReason": "format",
                "actualOutput": "actual"
            }
        ]);
        insert_iteration(&state.db, &task_id, 1, evaluation_results).await;

        let case_id = format!("iter-1:{test_case_id}");
        let response = get_case_detail(
            State(state),
            HeaderMap::new(),
            Path((task_id, case_id)),
            CurrentUser {
                user_id: "u1".to_string(),
                unlock_context: None,
            },
        )
        .await;

        match response {
            ApiResponse::Success(success) => {
                assert!(!success.data.diff_segments.is_empty());
            }
            _ => panic!("期望成功响应"),
        }
    }
}
