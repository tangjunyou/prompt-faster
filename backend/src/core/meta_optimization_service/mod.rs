//! 元优化服务

use sqlx::SqlitePool;
use thiserror::Error;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::core::evaluator::{EXT_TASK_EVALUATOR_CONFIG, create_evaluator_for_task_config};
use crate::core::execution_target::{ExecutionError, create_execution_target};
use crate::core::iteration_engine::orchestrator::IterationEngine;
use crate::core::teacher_model::{TeacherModelType, create_teacher_model};
use crate::domain::models::{
    CaseComparisonResult, CompareSummary, CreateTeacherPromptInput, ExecutionTargetType,
    IterationState, MetaOptimizationOverview, MetaOptimizationTaskSummary, OptimizationTaskConfig,
    PromptCompareRequest, PromptCompareResponse, PromptPreviewRequest, PromptPreviewResponse,
    PromptPreviewResult, PromptValidationRequest, PromptValidationResult, RuleSystem, TeacherPrompt,
    TeacherPromptStats, TeacherPromptVersion, TestCase, VersionCompareResult,
};
use crate::domain::types::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, unix_ms_to_iso8601,
};
use crate::infra::db::repositories::{
    CreateTeacherPromptRecordInput, CredentialRepo, CredentialRepoError, CredentialType,
    OptimizationTaskRepo, OptimizationTaskRepoError, TeacherPromptRecord, TeacherPromptRepo,
    TeacherPromptRepoError, TeacherPromptVersionWithStatsRecord, TestSetRepo, TestSetRepoError,
};
use crate::infra::external::api_key_manager::{ApiKeyManager, EncryptedApiKey};

#[derive(Debug, Error)]
pub enum MetaOptimizationServiceError {
    #[error("Prompt 版本不存在或无权访问")]
    NotFoundOrForbidden(Option<String>),
    #[error("请求参数错误: {0}")]
    InvalidRequest(String),
    #[error("预览执行失败: {0}")]
    ExecutionFailed(String),
    #[error("预览执行超时")]
    Timeout,
    #[error("API Key 解密失败: {0}")]
    Encryption(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    #[error("仓库错误: {0}")]
    Repo(String),
}

fn record_to_prompt(record: TeacherPromptRecord) -> TeacherPrompt {
    TeacherPrompt {
        id: record.id,
        user_id: record.user_id,
        version: record.version,
        content: record.content,
        description: record.description,
        is_active: record.is_active,
        created_at: unix_ms_to_iso8601(record.created_at),
        updated_at: unix_ms_to_iso8601(record.updated_at),
    }
}

fn version_with_stats_record_to_version(
    record: TeacherPromptVersionWithStatsRecord,
) -> (TeacherPromptVersion, TeacherPromptStats) {
    let success_rate = if record.total_tasks == 0 {
        None
    } else {
        Some(record.successful_tasks as f64 / record.total_tasks as f64)
    };

    let version = TeacherPromptVersion {
        id: record.id.clone(),
        version: record.version,
        description: record.description.clone(),
        is_active: record.is_active,
        success_rate,
        task_count: record.total_tasks,
        created_at: unix_ms_to_iso8601(record.created_at),
    };

    let stats = TeacherPromptStats {
        version_id: record.id,
        version: record.version,
        total_tasks: record.total_tasks,
        successful_tasks: record.successful_tasks,
        success_rate,
        average_pass_rate: record.average_pass_rate,
    };

    (version, stats)
}

fn map_repo_error(err: TeacherPromptRepoError) -> MetaOptimizationServiceError {
    match err {
        TeacherPromptRepoError::NotFound => MetaOptimizationServiceError::NotFoundOrForbidden(None),
        TeacherPromptRepoError::DatabaseError(e) => MetaOptimizationServiceError::Database(e),
    }
}

const MAX_PROMPT_BYTES: usize = 100 * 1024;
const PREVIEW_TIMEOUT_SECS: u64 = 30;
const COMPARE_TIMEOUT_SECS: u64 = 60;

fn preview_timeout_secs() -> u64 {
    std::env::var("PROMPT_FASTER_PREVIEW_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(PREVIEW_TIMEOUT_SECS)
}

fn compare_timeout_secs() -> u64 {
    std::env::var("PROMPT_FASTER_COMPARE_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(COMPARE_TIMEOUT_SECS)
}

fn validate_prompt_content(content: &str) -> PromptValidationResult {
    let mut errors = Vec::new();
    let trimmed = content.trim();
    if trimmed.is_empty() {
        errors.push("Prompt 内容不能为空".to_string());
    }
    if content.len() > MAX_PROMPT_BYTES {
        errors.push("Prompt 内容不能超过 100KB".to_string());
    }

    PromptValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings: Vec::new(),
    }
}

fn map_execution_error(err: ExecutionError) -> MetaOptimizationServiceError {
    match err {
        ExecutionError::InvalidRequest { message, .. } => {
            MetaOptimizationServiceError::InvalidRequest(message)
        }
        ExecutionError::InvalidCredentials { message, .. } => {
            MetaOptimizationServiceError::InvalidRequest(message)
        }
        ExecutionError::Timeout { .. } => MetaOptimizationServiceError::Timeout,
        ExecutionError::Network { message, .. }
        | ExecutionError::UpstreamError { message, .. }
        | ExecutionError::ParseError { message, .. }
        | ExecutionError::NotImplemented { message, .. }
        | ExecutionError::Internal { message, .. } => {
            MetaOptimizationServiceError::ExecutionFailed(message)
        }
    }
}

fn map_task_repo_error(err: OptimizationTaskRepoError) -> MetaOptimizationServiceError {
    match err {
        OptimizationTaskRepoError::NotFound => {
            MetaOptimizationServiceError::InvalidRequest("历史任务不存在或无权访问".to_string())
        }
        OptimizationTaskRepoError::WorkspaceNotFound => {
            MetaOptimizationServiceError::InvalidRequest("任务工作区不存在".to_string())
        }
        OptimizationTaskRepoError::TestSetNotFound => {
            MetaOptimizationServiceError::InvalidRequest("任务关联的测试集不存在".to_string())
        }
        OptimizationTaskRepoError::InvalidConfig(msg) => {
            MetaOptimizationServiceError::InvalidRequest(msg)
        }
        OptimizationTaskRepoError::DatabaseError(err) => {
            MetaOptimizationServiceError::Database(err)
        }
    }
}

fn map_test_set_repo_error(err: TestSetRepoError) -> MetaOptimizationServiceError {
    match err {
        TestSetRepoError::NotFound => {
            MetaOptimizationServiceError::InvalidRequest("测试集不存在或无权访问".to_string())
        }
        TestSetRepoError::JsonError(err) => MetaOptimizationServiceError::Repo(err.to_string()),
        TestSetRepoError::DatabaseError(err) => MetaOptimizationServiceError::Database(err),
    }
}

fn map_credential_repo_error(err: CredentialRepoError) -> MetaOptimizationServiceError {
    match err {
        CredentialRepoError::NotFound { .. } => {
            MetaOptimizationServiceError::InvalidRequest("缺少执行所需的 API Key 配置".to_string())
        }
        CredentialRepoError::DatabaseError(err) => MetaOptimizationServiceError::Database(err),
    }
}

fn build_preview_error_message(
    passed: bool,
    failure_points: &[crate::domain::models::FailurePoint],
    dimensions: &HashMap<String, crate::domain::models::DimensionScore>,
) -> Option<String> {
    if passed {
        return None;
    }

    if !failure_points.is_empty() {
        let details: Vec<String> = failure_points
            .iter()
            .take(2)
            .map(|point| format!("{}: {}", point.dimension, point.description))
            .collect();
        return Some(details.join("；"));
    }

    let mut detail_parts = Vec::new();
    for (dimension, score) in dimensions {
        if !score.passed {
            if let Some(details) = &score.details {
                detail_parts.push(format!("{}: {}", dimension, details));
            }
        }
    }

    if !detail_parts.is_empty() {
        return Some(detail_parts.join("；"));
    }

    Some("评估未通过".to_string())
}

pub async fn create_prompt_version(
    pool: &SqlitePool,
    user_id: &str,
    input: CreateTeacherPromptInput,
) -> Result<TeacherPromptVersion, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::create(
        pool,
        user_id,
        CreateTeacherPromptRecordInput {
            content: input.content,
            description: input.description,
            activate: input.activate,
        },
    )
    .await
    .map_err(map_repo_error)?;

    let stats = TeacherPromptRepo::calculate_stats(pool, &record.id, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(TeacherPromptVersion {
        id: record.id,
        version: record.version,
        description: record.description,
        is_active: record.is_active,
        success_rate: stats.success_rate,
        task_count: stats.total_tasks,
        created_at: unix_ms_to_iso8601(record.created_at),
    })
}

pub async fn list_prompt_versions(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<TeacherPromptVersion>, MetaOptimizationServiceError> {
    let records = TeacherPromptRepo::list_with_stats_by_user(pool, user_id, limit, offset)
        .await
        .map_err(map_repo_error)?;

    let mut versions = Vec::with_capacity(records.len());
    for record in records {
        let (version, _stats) = version_with_stats_record_to_version(record);
        versions.push(version);
    }

    Ok(versions)
}

pub async fn get_active_prompt(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<TeacherPrompt>, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::find_active(pool, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(record.map(record_to_prompt))
}

pub async fn set_active_prompt(
    pool: &SqlitePool,
    user_id: &str,
    version_id: &str,
) -> Result<TeacherPrompt, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::set_active(pool, version_id, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(record_to_prompt(record))
}

pub async fn get_prompt_by_id(
    pool: &SqlitePool,
    user_id: &str,
    version_id: &str,
) -> Result<TeacherPrompt, MetaOptimizationServiceError> {
    let record = TeacherPromptRepo::find_by_id(pool, version_id, user_id)
        .await
        .map_err(map_repo_error)?;

    Ok(record_to_prompt(record))
}

pub async fn get_overview(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<MetaOptimizationOverview, MetaOptimizationServiceError> {
    let total_versions = TeacherPromptRepo::count_by_user(pool, user_id)
        .await
        .map_err(map_repo_error)?;

    if total_versions == 0 {
        return Ok(MetaOptimizationOverview {
            total_versions: 0,
            active_version: None,
            best_version: None,
            stats: Vec::new(),
        });
    }

    let records =
        TeacherPromptRepo::list_with_stats_by_user(pool, user_id, total_versions as i64, 0)
            .await
            .map_err(map_repo_error)?;

    let mut stats_list = Vec::with_capacity(records.len());
    let mut versions = Vec::with_capacity(records.len());
    for record in records {
        let (version, stats) = version_with_stats_record_to_version(record);
        stats_list.push(stats);
        versions.push(version);
    }

    let active_version = versions.iter().find(|v| v.is_active).cloned();
    let best_version = versions
        .iter()
        .filter(|v| v.success_rate.is_some())
        .max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned();

    Ok(MetaOptimizationOverview {
        total_versions,
        active_version,
        best_version,
        stats: stats_list,
    })
}

#[allow(dead_code)]
pub async fn get_historical_tasks_for_meta_optimization(
    pool: &SqlitePool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<MetaOptimizationTaskSummary>, MetaOptimizationServiceError> {
    let task_rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
        r#"
        SELECT ot.id, ot.workspace_id, ot.name, ot.status, ot.created_at
        FROM optimization_tasks ot
        JOIN workspaces w ON ot.workspace_id = w.id
        WHERE w.user_id = ?1
        ORDER BY ot.created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(MetaOptimizationServiceError::Database)?;

    let mut summaries = Vec::with_capacity(task_rows.len());
    for (task_id, workspace_id, name, status, created_at) in task_rows {
        let selected_iteration_id: Option<String> = sqlx::query_scalar(
            r#"
            SELECT selected_iteration_id
            FROM optimization_tasks
            WHERE id = ?1
            "#,
        )
        .bind(&task_id)
        .fetch_optional(pool)
        .await
        .map_err(MetaOptimizationServiceError::Database)?;

        let selected_pass_rate: Option<f64> = if let Some(iteration_id) = selected_iteration_id {
            sqlx::query_scalar("SELECT pass_rate FROM iterations WHERE id = ?1")
                .bind(iteration_id)
                .fetch_optional(pool)
                .await
                .map_err(MetaOptimizationServiceError::Database)?
        } else {
            None
        };

        let pass_rate = if selected_pass_rate.is_some() {
            selected_pass_rate
        } else {
            sqlx::query_scalar(
                r#"
                SELECT pass_rate
                FROM iterations
                WHERE task_id = ?1 AND status = 'completed'
                ORDER BY round DESC
                LIMIT 1
                "#,
            )
            .bind(&task_id)
            .fetch_optional(pool)
            .await
            .map_err(MetaOptimizationServiceError::Database)?
        };

        summaries.push(MetaOptimizationTaskSummary {
            id: task_id,
            workspace_id,
            name,
            status,
            pass_rate,
            created_at: unix_ms_to_iso8601(created_at),
        });
    }

    Ok(summaries)
}

pub fn validate_prompt(
    request: PromptValidationRequest,
) -> Result<PromptValidationResult, MetaOptimizationServiceError> {
    Ok(validate_prompt_content(&request.content))
}

struct PreviewContext {
    workspace_id: String,
    execution_target_type: ExecutionTargetType,
    task_config: OptimizationTaskConfig,
    test_set_ids: Vec<String>,
    selected_cases: Vec<TestCase>,
    task_id: String,
}

async fn resolve_preview_context(
    pool: &SqlitePool,
    user_id: &str,
    task_ids: &[String],
    test_case_ids: &[String],
    max_cases: usize,
) -> Result<PreviewContext, MetaOptimizationServiceError> {
    if task_ids.is_empty() {
        return Err(MetaOptimizationServiceError::InvalidRequest(
            "请先选择历史任务".to_string(),
        ));
    }

    let mut workspace_id: Option<String> = None;
    let mut execution_target_type: Option<ExecutionTargetType> = None;
    let mut task_config: Option<OptimizationTaskConfig> = None;
    let mut test_set_ids: Vec<String> = Vec::new();
    let mut first_task_id: Option<String> = None;
    let mut seen_test_sets = HashSet::new();

    for task_id in task_ids {
        let task = OptimizationTaskRepo::find_by_id_for_user(pool, user_id, task_id)
            .await
            .map_err(map_task_repo_error)?;

        if first_task_id.is_none() {
            first_task_id = Some(task.id.clone());
        }

        let current_workspace = task.workspace_id.clone();
        if let Some(existing) = &workspace_id {
            if existing != &current_workspace {
                return Err(MetaOptimizationServiceError::InvalidRequest(
                    "预览任务需属于同一工作区".to_string(),
                ));
            }
        } else {
            workspace_id = Some(current_workspace.clone());
        }

        if let Some(existing) = execution_target_type {
            if existing != task.execution_target_type {
                return Err(MetaOptimizationServiceError::InvalidRequest(
                    "预览任务执行目标类型不一致".to_string(),
                ));
            }
        } else {
            execution_target_type = Some(task.execution_target_type);
        }

        if task_config.is_none() {
            task_config = Some(OptimizationTaskConfig::normalized_from_config_json(
                task.config_json.as_deref(),
            ));
        }

        let scoped =
            OptimizationTaskRepo::find_by_id_scoped(pool, user_id, &current_workspace, task_id)
                .await
                .map_err(map_task_repo_error)?;

        for id in scoped.test_set_ids {
            if seen_test_sets.insert(id.clone()) {
                test_set_ids.push(id);
            }
        }
    }

    let workspace_id = workspace_id.ok_or_else(|| {
        MetaOptimizationServiceError::InvalidRequest("任务工作区不存在".to_string())
    })?;

    if test_set_ids.is_empty() {
        return Err(MetaOptimizationServiceError::InvalidRequest(
            "历史任务未绑定测试集，无法预览".to_string(),
        ));
    }

    let mut selected_cases: Vec<TestCase> = Vec::new();
    if !test_case_ids.is_empty() {
        if test_case_ids.len() > max_cases {
            return Err(MetaOptimizationServiceError::InvalidRequest(format!(
                "最多只能选择 {} 条测试用例",
                max_cases
            )));
        }

        for test_case_id in test_case_ids {
            let case =
                TestSetRepo::find_case_by_id(pool, &workspace_id, &test_set_ids, test_case_id)
                    .await
                    .map_err(map_test_set_repo_error)?;

            let Some(case) = case else {
                return Err(MetaOptimizationServiceError::InvalidRequest(format!(
                    "测试用例不存在或无权访问: {}",
                    test_case_id
                )));
            };
            selected_cases.push(case);
        }
    } else {
        for test_set_id in &test_set_ids {
            let test_set =
                TestSetRepo::find_by_id_scoped(pool, user_id, &workspace_id, test_set_id)
                    .await
                    .map_err(map_test_set_repo_error)?;

            for case in test_set.cases {
                selected_cases.push(case);
                if selected_cases.len() >= max_cases {
                    break;
                }
            }
            if selected_cases.len() >= max_cases {
                break;
            }
        }
    }

    if selected_cases.is_empty() {
        return Err(MetaOptimizationServiceError::InvalidRequest(
            "没有可用的测试用例可预览".to_string(),
        ));
    }

    Ok(PreviewContext {
        workspace_id,
        execution_target_type: execution_target_type.unwrap_or(ExecutionTargetType::Example),
        task_config: task_config.unwrap_or_default(),
        test_set_ids,
        selected_cases,
        task_id: first_task_id.unwrap_or_else(|| "preview".to_string()),
    })
}

async fn execute_prompt_preview(
    pool: &SqlitePool,
    api_key_manager: &ApiKeyManager,
    user_id: &str,
    user_password: &[u8],
    ctx: &PreviewContext,
    prompt: &str,
    correlation_id: Option<String>,
    timeout: Duration,
) -> Result<PromptPreviewResponse, MetaOptimizationServiceError> {
    let execution_target_context = ExecutionTargetContext {
        pool,
        api_key_manager,
        user_id,
        user_password,
        workspace_id: &ctx.workspace_id,
    };
    let execution_target_config = build_execution_target_config(
        execution_target_context,
        ctx.execution_target_type,
        &ctx.task_config,
        &ctx.test_set_ids,
    )
    .await?;

    let mut context = OptimizationContext {
        task_id: ctx.task_id.clone(),
        execution_target_config,
        current_prompt: prompt.to_string(),
        rule_system: RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: HashMap::new(),
            version: 1,
        },
        iteration: 0,
        state: IterationState::RunningTests,
        run_control_state: Default::default(),
        test_cases: ctx.selected_cases.clone(),
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions: HashMap::new(),
    };

    if let Some(cid) = correlation_id {
        context
            .extensions
            .insert("correlation_id".to_string(), serde_json::Value::String(cid));
    }

    let evaluator_cfg_value =
        serde_json::to_value(&ctx.task_config.evaluator_config).map_err(|_| {
            MetaOptimizationServiceError::ExecutionFailed("evaluator_config 序列化失败".to_string())
        })?;
    context
        .extensions
        .insert(EXT_TASK_EVALUATOR_CONFIG.to_string(), evaluator_cfg_value);

    let teacher_model = create_teacher_model(TeacherModelType::Example);
    let evaluator = create_evaluator_for_task_config(&ctx.task_config, Some(teacher_model));
    let execution_target = create_execution_target(ctx.execution_target_type);
    let engine = IterationEngine::new(execution_target);
    let batch = context.test_cases.clone();

    let output = tokio::time::timeout(timeout, async {
        let exec_results = engine
            .run_tests(&mut context, prompt, &batch, &ctx.task_config)
            .await
            .map_err(map_execution_error)?;

        let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results)
            .map_err(map_execution_error)?;

        let evals = evaluator
            .evaluate_batch(&context, &pairs)
            .await
            .map_err(|err| match err {
                crate::core::evaluator::EvaluatorError::InvalidInput(msg) => {
                    MetaOptimizationServiceError::InvalidRequest(msg)
                }
                crate::core::evaluator::EvaluatorError::Timeout(_) => {
                    MetaOptimizationServiceError::Timeout
                }
                crate::core::evaluator::EvaluatorError::ModelFailure(msg)
                | crate::core::evaluator::EvaluatorError::Internal(msg) => {
                    MetaOptimizationServiceError::ExecutionFailed(msg)
                }
            })?;

        let mut results = Vec::with_capacity(batch.len());
        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_time_ms = 0i64;

        for idx in 0..pairs.len() {
            let (test_case, _output) = &pairs[idx];
            let exec_result = &exec_results[idx];
            let eval = &evals[idx];
            if eval.passed {
                total_passed += 1;
            } else {
                total_failed += 1;
            }
            let exec_time = exec_result.latency_ms as i64;
            total_time_ms += exec_time;

            let error_message = build_preview_error_message(
                eval.passed,
                &eval.failure_points,
                &eval.dimensions,
            );

            results.push(PromptPreviewResult {
                test_case_id: test_case.id.clone(),
                input: test_case.input.clone(),
                reference: test_case.reference.clone(),
                actual_output: exec_result.output.clone(),
                passed: eval.passed,
                execution_time_ms: exec_time,
                error_message,
            });
        }

        Ok::<_, MetaOptimizationServiceError>(PromptPreviewResponse {
            results,
            total_passed,
            total_failed,
            total_execution_time_ms: total_time_ms,
        })
    })
    .await;

    match output {
        Ok(result) => result,
        Err(_) => Err(MetaOptimizationServiceError::Timeout),
    }
}

pub async fn preview_prompt(
    pool: &SqlitePool,
    api_key_manager: &ApiKeyManager,
    user_id: &str,
    user_password: &[u8],
    request: PromptPreviewRequest,
    correlation_id: Option<String>,
) -> Result<PromptPreviewResponse, MetaOptimizationServiceError> {
    let validation = validate_prompt_content(&request.content);
    if !validation.is_valid {
        let msg = validation.errors.join("; ");
        return Err(MetaOptimizationServiceError::InvalidRequest(msg));
    }

    let ctx =
        resolve_preview_context(pool, user_id, &request.task_ids, &request.test_case_ids, 3)
            .await?;

    execute_prompt_preview(
        pool,
        api_key_manager,
        user_id,
        user_password,
        &ctx,
        &request.content,
        correlation_id,
        Duration::from_secs(preview_timeout_secs()),
    )
    .await
}

async fn load_prompt_for_compare(
    pool: &SqlitePool,
    user_id: &str,
    version_id: &str,
) -> Result<TeacherPromptRecord, MetaOptimizationServiceError> {
    match TeacherPromptRepo::find_by_id(pool, version_id, user_id).await {
        Ok(record) => Ok(record),
        Err(TeacherPromptRepoError::NotFound) => Err(MetaOptimizationServiceError::NotFoundOrForbidden(
            Some(version_id.to_string()),
        )),
        Err(TeacherPromptRepoError::DatabaseError(err)) => {
            Err(MetaOptimizationServiceError::Database(err))
        }
    }
}

fn generate_difference_note(
    a_passed: bool,
    b_passed: bool,
    a_output: &str,
    b_output: &str,
    a_error: &Option<String>,
    b_error: &Option<String>,
) -> Option<String> {
    match (a_passed, b_passed) {
        (true, false) => Some(format!(
            "版本 B 在此用例退化：{}",
            b_error.as_deref().unwrap_or("失败原因未知")
        )),
        (false, true) => Some(format!(
            "版本 B 在此用例改进（A 失败：{}）",
            a_error.as_deref().unwrap_or("失败原因未知")
        )),
        (true, true) => {
            if a_output != b_output {
                Some("两版本均通过，但输出内容存在差异".to_string())
            } else {
                None
            }
        }
        (false, false) => {
            if a_error.is_some() || b_error.is_some() {
                Some(format!(
                    "两版本均失败：A={}；B={}",
                    a_error.as_deref().unwrap_or("失败原因未知"),
                    b_error.as_deref().unwrap_or("失败原因未知")
                ))
            } else {
                Some("两版本均失败，错误原因可能不同".to_string())
            }
        }
    }
}

pub async fn compare_prompts(
    pool: &SqlitePool,
    api_key_manager: &ApiKeyManager,
    user_id: &str,
    user_password: &[u8],
    request: PromptCompareRequest,
    correlation_id: Option<String>,
) -> Result<PromptCompareResponse, MetaOptimizationServiceError> {
    if request.version_id_a == request.version_id_b {
        return Err(MetaOptimizationServiceError::InvalidRequest(
            "不能选择相同版本进行对比".to_string(),
        ));
    }

    let version_a = load_prompt_for_compare(pool, user_id, &request.version_id_a).await?;
    let version_b = load_prompt_for_compare(pool, user_id, &request.version_id_b).await?;

    let ctx =
        resolve_preview_context(pool, user_id, &request.task_ids, &request.test_case_ids, 10)
            .await?;

    let compare_timeout = Duration::from_secs(compare_timeout_secs());
    let per_prompt_timeout = Duration::from_secs(preview_timeout_secs());
    let deadline = tokio::time::Instant::now() + compare_timeout;

    let remaining_a = deadline.saturating_duration_since(tokio::time::Instant::now());
    if remaining_a.is_zero() {
        return Err(MetaOptimizationServiceError::Timeout);
    }
    let timeout_a = remaining_a.min(per_prompt_timeout);
    let preview_a = execute_prompt_preview(
        pool,
        api_key_manager,
        user_id,
        user_password,
        &ctx,
        &version_a.content,
        correlation_id.clone(),
        timeout_a,
    )
    .await?;

    let remaining_b = deadline.saturating_duration_since(tokio::time::Instant::now());
    if remaining_b.is_zero() {
        return Err(MetaOptimizationServiceError::Timeout);
    }
    let timeout_b = remaining_b.min(per_prompt_timeout);
    let preview_b = execute_prompt_preview(
        pool,
        api_key_manager,
        user_id,
        user_password,
        &ctx,
        &version_b.content,
        correlation_id.clone(),
        timeout_b,
    )
    .await?;

    let total_cases = ctx.selected_cases.len() as f64;
    let version_a_pass_rate = if total_cases == 0.0 {
        0.0
    } else {
        preview_a.total_passed as f64 / total_cases
    };
    let version_b_pass_rate = if total_cases == 0.0 {
        0.0
    } else {
        preview_b.total_passed as f64 / total_cases
    };

    let version_a_summary = VersionCompareResult {
        version_id: version_a.id.clone(),
        version: version_a.version,
        total_passed: preview_a.total_passed,
        total_failed: preview_a.total_failed,
        pass_rate: version_a_pass_rate,
    };
    let version_b_summary = VersionCompareResult {
        version_id: version_b.id.clone(),
        version: version_b.version,
        total_passed: preview_b.total_passed,
        total_failed: preview_b.total_failed,
        pass_rate: version_b_pass_rate,
    };

    let mut preview_a_map = HashMap::new();
    for result in preview_a.results {
        preview_a_map.insert(result.test_case_id.clone(), result);
    }
    let mut preview_b_map = HashMap::new();
    for result in preview_b.results {
        preview_b_map.insert(result.test_case_id.clone(), result);
    }

    let mut case_comparisons = Vec::with_capacity(ctx.selected_cases.len());
    let mut improved_cases = 0;
    let mut regressed_cases = 0;
    let mut output_diff_cases = 0;
    let mut unchanged_cases = 0;

    for case in &ctx.selected_cases {
        let result_a = preview_a_map
            .get(&case.id)
            .ok_or_else(|| {
                MetaOptimizationServiceError::ExecutionFailed(
                    "版本 A 预览结果不完整".to_string(),
                )
            })?;
        let result_b = preview_b_map
            .get(&case.id)
            .ok_or_else(|| {
                MetaOptimizationServiceError::ExecutionFailed(
                    "版本 B 预览结果不完整".to_string(),
                )
            })?;

        let output_diff = result_a.actual_output != result_b.actual_output;
        let is_different = result_a.passed != result_b.passed || output_diff;

        if !result_a.passed && result_b.passed {
            improved_cases += 1;
        } else if result_a.passed && !result_b.passed {
            regressed_cases += 1;
        } else if result_a.passed && result_b.passed && output_diff {
            output_diff_cases += 1;
        } else {
            unchanged_cases += 1;
        }

        case_comparisons.push(CaseComparisonResult {
            test_case_id: case.id.clone(),
            input: case.input.clone(),
            reference: case.reference.clone(),
            version_a_output: result_a.actual_output.clone(),
            version_a_passed: result_a.passed,
            version_a_error: result_a.error_message.clone(),
            version_b_output: result_b.actual_output.clone(),
            version_b_passed: result_b.passed,
            version_b_error: result_b.error_message.clone(),
            is_different,
            difference_note: generate_difference_note(
                result_a.passed,
                result_b.passed,
                &result_a.actual_output,
                &result_b.actual_output,
                &result_a.error_message,
                &result_b.error_message,
            ),
        });
    }

    let summary = CompareSummary {
        pass_rate_diff: version_b_pass_rate - version_a_pass_rate,
        improved_cases,
        regressed_cases,
        output_diff_cases,
        unchanged_cases,
        total_execution_time_ms: preview_a.total_execution_time_ms
            + preview_b.total_execution_time_ms,
    };

    Ok(PromptCompareResponse {
        version_a: version_a_summary,
        version_b: version_b_summary,
        version_a_content: version_a.content,
        version_b_content: version_b.content,
        case_comparisons,
        summary,
    })
}

struct ExecutionTargetContext<'a> {
    pool: &'a SqlitePool,
    api_key_manager: &'a ApiKeyManager,
    user_id: &'a str,
    user_password: &'a [u8],
    workspace_id: &'a str,
}

async fn build_execution_target_config(
    ctx: ExecutionTargetContext<'_>,
    execution_target_type: ExecutionTargetType,
    task_config: &OptimizationTaskConfig,
    test_set_ids: &[String],
) -> Result<ExecutionTargetConfig, MetaOptimizationServiceError> {
    match execution_target_type {
        ExecutionTargetType::Dify => {
            let prompt_variable =
                extract_prompt_variable(ctx.pool, ctx.user_id, ctx.workspace_id, test_set_ids)
                    .await?;
            let credential =
                CredentialRepo::find_by_user_and_type(ctx.pool, ctx.user_id, CredentialType::Dify)
                    .await
                    .map_err(map_credential_repo_error)?;
            let api_key = decrypt_api_key(ctx.api_key_manager, ctx.user_password, &credential)
                .map_err(MetaOptimizationServiceError::Encryption)?;
            Ok(ExecutionTargetConfig::Dify {
                api_url: credential.base_url,
                workflow_id: String::new(),
                prompt_variable,
                api_key: Some(api_key),
            })
        }
        ExecutionTargetType::Generic => {
            let credential = CredentialRepo::find_by_user_and_type(
                ctx.pool,
                ctx.user_id,
                CredentialType::GenericLlm,
            )
            .await
            .map_err(map_credential_repo_error)?;
            let model_name = task_config
                .teacher_llm
                .model_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let api_key = decrypt_api_key(ctx.api_key_manager, ctx.user_password, &credential)
                .map_err(MetaOptimizationServiceError::Encryption)?;
            Ok(ExecutionTargetConfig::DirectModel {
                base_url: credential.base_url,
                model_name,
                user_prompt_template: "{input}".to_string(),
                api_key: Some(api_key),
            })
        }
        ExecutionTargetType::Example => Ok(ExecutionTargetConfig::default()),
    }
}

fn decrypt_api_key(
    api_key_manager: &ApiKeyManager,
    user_password: &[u8],
    credential: &crate::infra::db::repositories::CredentialRecord,
) -> Result<String, String> {
    let encrypted = EncryptedApiKey {
        ciphertext: credential.encrypted_api_key.clone(),
        nonce: credential.nonce.clone(),
        salt: credential.salt.clone(),
    };

    let api_key_bytes = api_key_manager
        .decrypt_bytes(user_password, &encrypted)
        .map_err(|_| "解密 API Key 失败".to_string())?;

    std::str::from_utf8(api_key_bytes.as_slice())
        .map(|s| s.to_string())
        .map_err(|_| "解密后的 API Key 非法".to_string())
}

async fn extract_prompt_variable(
    pool: &SqlitePool,
    user_id: &str,
    workspace_id: &str,
    test_set_ids: &[String],
) -> Result<String, MetaOptimizationServiceError> {
    for test_set_id in test_set_ids {
        let test_set = TestSetRepo::find_by_id_scoped(pool, user_id, workspace_id, test_set_id)
            .await
            .map_err(map_test_set_repo_error)?;
        if let Some(raw) = test_set.dify_config_json {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(variable) = value
                    .get("targetPromptVariable")
                    .and_then(|v| v.as_str())
                    .or_else(|| value.get("target_prompt_variable").and_then(|v| v.as_str()))
                {
                    return Ok(variable.to_string());
                }
            }
        }
    }
    Ok("prompt".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    use crate::infra::db::pool::create_pool;
    use crate::infra::db::repositories::{TeacherPromptRepo, WorkspaceRepo};
    use crate::shared::time::now_millis;

    async fn setup_db() -> SqlitePool {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("创建测试数据库失败");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("运行 migrations 失败");
        pool
    }

    async fn insert_user(pool: &SqlitePool, user_id: &str, username: &str) {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(user_id)
        .bind(username)
        .bind("hash")
        .bind(now_millis())
        .bind(now_millis())
        .execute(pool)
        .await
        .expect("插入用户失败");
    }

    #[test]
    fn test_generate_difference_note_variants() {
        let note = generate_difference_note(true, true, "a", "b", &None, &None);
        assert_eq!(note.as_deref(), Some("两版本均通过，但输出内容存在差异"));

        let note = generate_difference_note(true, false, "a", "a", &None, &Some("err".to_string()));
        assert_eq!(note.as_deref(), Some("版本 B 在此用例退化：err"));

        let note = generate_difference_note(false, true, "a", "a", &Some("bad".to_string()), &None);
        assert_eq!(note.as_deref(), Some("版本 B 在此用例改进（A 失败：bad）"));

        let note = generate_difference_note(false, false, "a", "b", &None, &None);
        assert_eq!(note.as_deref(), Some("两版本均失败，错误原因可能不同"));

        let note = generate_difference_note(
            false,
            false,
            "a",
            "b",
            &Some("err-a".to_string()),
            &Some("err-b".to_string()),
        );
        assert_eq!(
            note.as_deref(),
            Some("两版本均失败：A=err-a；B=err-b")
        );
    }

    async fn insert_task_with_iteration(
        pool: &SqlitePool,
        task_id: &str,
        workspace_id: &str,
        version_id: &str,
        selected_iteration_id: Option<&str>,
        pass_rate: f64,
        round: i32,
    ) {
        let now = now_millis();
        sqlx::query(
            r#"
            INSERT INTO optimization_tasks
              (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, teacher_prompt_version_id, selected_iteration_id, created_at, updated_at)
            VALUES
              (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, NULL, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(task_id)
        .bind(workspace_id)
        .bind("task")
        .bind("goal")
        .bind("example")
        .bind("fixed")
        .bind("completed")
        .bind(version_id)
        .bind(selected_iteration_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .expect("插入任务失败");

        sqlx::query(
            r#"
            INSERT INTO iterations
              (id, task_id, round, started_at, completed_at, status, artifacts, evaluation_results, reflection_summary, pass_rate, total_cases, passed_cases, created_at)
            VALUES
              (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, NULL, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(selected_iteration_id.unwrap_or(task_id))
        .bind(task_id)
        .bind(round)
        .bind(now)
        .bind(now)
        .bind("completed")
        .bind(pass_rate)
        .bind(10)
        .bind((pass_rate * 10.0).round() as i32)
        .bind(now)
        .execute(pool)
        .await
        .expect("插入迭代失败");
    }

    #[tokio::test]
    async fn test_create_and_list_versions() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let v1 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-1".to_string(),
                description: Some("first".to_string()),
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        let v2 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-2".to_string(),
                description: Some("second".to_string()),
                activate: false,
            },
        )
        .await
        .expect("创建版本失败");

        assert_eq!(v1.version, 1);
        assert_eq!(v2.version, 2);

        let list = list_prompt_versions(&pool, "u1", 50, 0)
            .await
            .expect("获取列表失败");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].version, 2);
        assert_eq!(list[1].version, 1);
    }

    #[tokio::test]
    async fn test_set_active_prompt() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let v1 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-1".to_string(),
                description: None,
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        let v2 = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-2".to_string(),
                description: None,
                activate: false,
            },
        )
        .await
        .expect("创建版本失败");

        let active_before = TeacherPromptRepo::find_active(&pool, "u1")
            .await
            .expect("查询失败")
            .expect("缺少 active");
        assert_eq!(active_before.id, v1.id);

        let active = set_active_prompt(&pool, "u1", &v2.id)
            .await
            .expect("设置 active 失败");
        assert_eq!(active.id, v2.id);

        let active_after = TeacherPromptRepo::find_active(&pool, "u1")
            .await
            .expect("查询失败")
            .expect("缺少 active");
        assert_eq!(active_after.id, v2.id);
    }

    #[tokio::test]
    async fn test_success_rate_calculation() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let workspace = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");

        let version = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt".to_string(),
                description: None,
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        insert_task_with_iteration(
            &pool,
            "task-1",
            &workspace.id,
            &version.id,
            Some("iter-1"),
            1.0,
            1,
        )
        .await;

        insert_task_with_iteration(&pool, "task-2", &workspace.id, &version.id, None, 0.5, 2).await;

        let stats = TeacherPromptRepo::calculate_stats(&pool, &version.id, "u1")
            .await
            .expect("统计失败");

        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.successful_tasks, 1);
        assert_eq!(stats.success_rate.unwrap(), 0.5);
        assert_eq!(stats.average_pass_rate.unwrap(), 0.75);
    }

    #[tokio::test]
    async fn test_success_rate_empty_tasks_returns_null() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let version = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt".to_string(),
                description: None,
                activate: true,
            },
        )
        .await
        .expect("创建版本失败");

        let stats = TeacherPromptRepo::calculate_stats(&pool, &version.id, "u1")
            .await
            .expect("统计失败");

        assert_eq!(stats.total_tasks, 0);
        assert!(stats.success_rate.is_none());
        assert!(stats.average_pass_rate.is_none());
    }

    #[tokio::test]
    async fn test_list_prompt_versions_pagination() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        for idx in 0..3 {
            let _ = create_prompt_version(
                &pool,
                "u1",
                CreateTeacherPromptInput {
                    content: format!("prompt-{idx}"),
                    description: None,
                    activate: idx == 0,
                },
            )
            .await
            .expect("创建版本失败");
        }

        let page = list_prompt_versions(&pool, "u1", 1, 1)
            .await
            .expect("获取列表失败");
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].version, 2);
    }

    #[tokio::test]
    async fn test_concurrent_version_creation_unique_versions() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;

        let create_a = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-a".to_string(),
                description: None,
                activate: true,
            },
        );
        let create_b = create_prompt_version(
            &pool,
            "u1",
            CreateTeacherPromptInput {
                content: "prompt-b".to_string(),
                description: None,
                activate: false,
            },
        );

        let (a, b) = tokio::join!(create_a, create_b);
        let a = a.expect("创建版本失败");
        let b = b.expect("创建版本失败");

        let mut versions = vec![a.version, b.version];
        versions.sort_unstable();
        versions.dedup();
        assert_eq!(versions, vec![1, 2]);
    }
}
