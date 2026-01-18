//! 断点恢复核心逻辑

use std::collections::HashMap;

use thiserror::Error;
use tracing::{error, info, warn};

use crate::core::iteration_engine::checkpoint::{compute_checksum, verify_checksum};
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::domain::models::{
    Checkpoint, CheckpointCreateRequest, CheckpointEntity, ExecutionTargetType, IterationState,
    LineageType, OptimizationTaskConfig, OptimizationTaskStatus, OutputLength, Rule, RuleSystem,
    RuleTags,
};
use crate::domain::models::recovery::UnfinishedTask;
use crate::domain::models::RecoveryMetrics;
use crate::domain::types::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, RunControlState,
    SplitStrategy, EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_USER_GUIDANCE,
    unix_ms_to_iso8601,
};
use crate::infra::db::pool::global_db_pool;
use crate::infra::db::repositories::{
    CheckpointRepo, CheckpointRepoError, CredentialRepo, CredentialRepoError, CredentialType,
    OptimizationTaskRepo, OptimizationTaskRepoError, RecoveryMetricsRepo,
    RecoveryMetricsRepoError, TestSetRepo, TestSetRepoError,
};
use crate::shared::time::now_millis;

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("数据库未初始化")]
    DatabaseNotInitialized,
    #[error("任务不存在或无权访问")]
    TaskNotFound,
    #[error("Checkpoint 未找到")]
    CheckpointNotFound,
    #[error("没有可用的 Checkpoint")]
    NoValidCheckpoint,
    #[error("暂停快照不存在")]
    PauseStateNotFound,
    #[error("仓库错误: {0}")]
    Repo(#[from] OptimizationTaskRepoError),
    #[error("Checkpoint 仓库错误: {0}")]
    CheckpointRepo(#[from] CheckpointRepoError),
    #[error("测试集仓库错误: {0}")]
    TestSetRepo(#[from] TestSetRepoError),
    #[error("凭证仓库错误: {0}")]
    CredentialRepo(#[from] CredentialRepoError),
    #[error("恢复上下文失败: {0}")]
    Context(String),
    #[error("恢复统计写入失败: {0}")]
    MetricsRepo(#[from] RecoveryMetricsRepoError),
}

async fn record_recovery_attempt(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<(), RecoveryError> {
    RecoveryMetricsRepo::record_attempt(pool, task_id).await?;
    Ok(())
}

async fn record_recovery_success(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<(), RecoveryError> {
    RecoveryMetricsRepo::record_success(pool, task_id).await?;
    Ok(())
}

pub async fn get_recovery_metrics(task_id: &str) -> Result<RecoveryMetrics, RecoveryError> {
    let pool = global_db_pool().ok_or(RecoveryError::DatabaseNotInitialized)?;
    get_recovery_metrics_with_pool(&pool, task_id).await
}

/// 检测未完成任务
pub async fn detect_unfinished_tasks(
    user_id: &str,
) -> Result<Vec<UnfinishedTask>, RecoveryError> {
    let pool = global_db_pool().ok_or(RecoveryError::DatabaseNotInitialized)?;
    detect_unfinished_tasks_with_pool(&pool, user_id).await
}

/// 按 Checkpoint ID 恢复
pub async fn recover_from_checkpoint(
    checkpoint_id: &str,
    user_id: &str,
    correlation_id: &str,
) -> Result<OptimizationContext, RecoveryError> {
    let pool = global_db_pool().ok_or(RecoveryError::DatabaseNotInitialized)?;
    recover_from_checkpoint_with_pool(&pool, checkpoint_id, user_id, correlation_id).await
}

/// 按任务恢复（可选指定 checkpoint_id）
pub async fn recover_task(
    task_id: &str,
    user_id: &str,
    correlation_id: &str,
    checkpoint_id: Option<&str>,
) -> Result<(OptimizationContext, CheckpointEntity), RecoveryError> {
    let pool = global_db_pool().ok_or(RecoveryError::DatabaseNotInitialized)?;
    recover_task_with_pool(&pool, task_id, user_id, correlation_id, checkpoint_id).await
}

/// 放弃恢复，标记任务为中止
pub async fn abort_task(
    task_id: &str,
    user_id: &str,
    correlation_id: &str,
) -> Result<(), RecoveryError> {
    let pool = global_db_pool().ok_or(RecoveryError::DatabaseNotInitialized)?;
    abort_task_with_pool(&pool, task_id, user_id, correlation_id).await
}

pub(crate) async fn detect_unfinished_tasks_with_pool(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<Vec<UnfinishedTask>, RecoveryError> {
    let tasks = OptimizationTaskRepo::find_unfinished_with_checkpoints(pool, user_id).await?;
    let mut valid_tasks = Vec::with_capacity(tasks.len());
    for task in tasks {
        let checkpoint = match CheckpointRepo::get_checkpoint_by_id(pool, &task.checkpoint_id).await?
        {
            Some(cp) => cp,
            None => {
                warn!(
                    task_id = %task.task_id,
                    checkpoint_id = %task.checkpoint_id,
                    "未找到任务的最新 checkpoint，跳过未完成任务提示"
                );
                continue;
            }
        };

        let mut selected_checkpoint = checkpoint;
        if !verify_checksum(&selected_checkpoint) {
            warn!(
                task_id = %task.task_id,
                checkpoint_id = %selected_checkpoint.id,
                "未完成任务的最新 checkpoint checksum 校验失败，尝试回退"
            );
            let total =
                CheckpointRepo::count_checkpoints_by_task(pool, &task.task_id).await?;
            let candidates =
                CheckpointRepo::list_checkpoints_by_task(pool, &task.task_id, total.max(1))
                    .await?;
            if let Some(valid) = candidates.into_iter().find(|cp| verify_checksum(cp)) {
                selected_checkpoint = valid;
            } else {
                warn!(
                    task_id = %task.task_id,
                    "未完成任务未找到可用 checkpoint，跳过提示"
                );
                continue;
            }
        }

        valid_tasks.push(UnfinishedTask {
            task_id: task.task_id,
            task_name: task.task_name,
            checkpoint_id: selected_checkpoint.id.clone(),
            last_checkpoint_at: unix_ms_to_iso8601(selected_checkpoint.created_at),
            iteration: selected_checkpoint.iteration,
            state: iteration_state_label(&selected_checkpoint.state),
            run_control_state: run_control_state_label(&selected_checkpoint.run_control_state),
        });
    }

    Ok(valid_tasks)
}

pub(crate) async fn recover_from_checkpoint_with_pool(
    pool: &sqlx::SqlitePool,
    checkpoint_id: &str,
    user_id: &str,
    correlation_id: &str,
) -> Result<OptimizationContext, RecoveryError> {
    let checkpoint = CheckpointRepo::get_checkpoint_for_user(pool, user_id, checkpoint_id)
        .await?
        .ok_or(RecoveryError::CheckpointNotFound)?;

    if let Err(err) = record_recovery_attempt(pool, &checkpoint.task_id).await {
        warn!(
            task_id = %checkpoint.task_id,
            error = %err,
            "记录恢复尝试失败"
        );
    }

    let (ctx, used_checkpoint) =
        recover_with_fallback(pool, user_id, correlation_id, &checkpoint, None).await?;

    if let Err(err) = record_recovery_success(pool, &checkpoint.task_id).await {
        warn!(
            task_id = %checkpoint.task_id,
            error = %err,
            "记录恢复成功失败"
        );
    }

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %checkpoint.task_id,
        action = "checkpoint_recovered",
        prev_state = ?used_checkpoint.state,
        new_state = ?used_checkpoint.state,
        iteration_state = ?used_checkpoint.state,
        timestamp = now_millis(),
        used_checkpoint_id = %used_checkpoint.id,
        "Checkpoint 恢复成功"
    );

    Ok(ctx)
}

pub(crate) async fn recover_task_with_pool(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    user_id: &str,
    correlation_id: &str,
    checkpoint_id: Option<&str>,
) -> Result<(OptimizationContext, CheckpointEntity), RecoveryError> {
    // 校验任务归属
    OptimizationTaskRepo::find_by_id_for_user(pool, user_id, task_id)
        .await
        .map_err(|_| RecoveryError::TaskNotFound)?;

    if let Err(err) = record_recovery_attempt(pool, task_id).await {
        warn!(
            task_id = %task_id,
            error = %err,
            "记录恢复尝试失败"
        );
    }

    let checkpoint = if let Some(id) = checkpoint_id {
        let checkpoint = CheckpointRepo::get_checkpoint_for_user(pool, user_id, id)
            .await?
            .ok_or(RecoveryError::CheckpointNotFound)?;
        Some(checkpoint)
    } else {
        None
    };

    let result = if let Some(cp) = checkpoint {
        recover_with_fallback(pool, user_id, correlation_id, &cp, None).await
    } else {
        // 使用最近的 checkpoint
        let total = CheckpointRepo::count_checkpoints_by_task(pool, task_id).await?;
        let candidates = CheckpointRepo::list_checkpoints_by_task(pool, task_id, total.max(1)).await?;
        if let Some(latest) = candidates.first().cloned() {
            recover_with_fallback(pool, user_id, correlation_id, &latest, Some(candidates))
                .await
        } else {
            recover_from_pause_state(pool, user_id, correlation_id, task_id).await
        }
    }?;

    if let Err(err) = record_recovery_success(pool, task_id).await {
        warn!(
            task_id = %task_id,
            error = %err,
            "记录恢复成功失败"
        );
    }

    Ok(result)
}

pub(crate) async fn abort_task_with_pool(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    user_id: &str,
    correlation_id: &str,
) -> Result<(), RecoveryError> {
    let task = OptimizationTaskRepo::find_by_id_for_user(pool, user_id, task_id)
        .await
        .map_err(|_| RecoveryError::TaskNotFound)?;
    OptimizationTaskRepo::update_status(pool, task_id, OptimizationTaskStatus::Terminated).await?;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "task_aborted",
        prev_state = ?task.status,
        new_state = ?OptimizationTaskStatus::Terminated,
        iteration_state = "N/A",
        timestamp = now_millis(),
        "任务已中止"
    );

    Ok(())
}

async fn recover_with_fallback(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    correlation_id: &str,
    preferred: &CheckpointEntity,
    preloaded: Option<Vec<CheckpointEntity>>,
) -> Result<(OptimizationContext, CheckpointEntity), RecoveryError> {
    let mut checkpoints = match preloaded {
        Some(list) => list,
        None => {
            let total = CheckpointRepo::count_checkpoints_by_task(pool, &preferred.task_id).await?;
            CheckpointRepo::list_checkpoints_by_task(pool, &preferred.task_id, total.max(1)).await?
        }
    };

    // 保证首选 checkpoint 优先
    if !checkpoints.iter().any(|c| c.id == preferred.id) {
        checkpoints.insert(0, preferred.clone());
    }

    for checkpoint in checkpoints {
        if !verify_checksum(&checkpoint) {
            warn!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %checkpoint.task_id,
                checkpoint_id = %checkpoint.id,
                action = "checkpoint_checksum_failed",
                prev_state = ?checkpoint.state,
                new_state = ?checkpoint.state,
                iteration_state = ?checkpoint.state,
                timestamp = now_millis(),
                "Checkpoint checksum 校验失败，尝试回退"
            );
            continue;
        }

        let ctx = rebuild_optimization_context(pool, user_id, &checkpoint).await?;
        return Ok((ctx, checkpoint));
    }

    error!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %preferred.task_id,
        action = "recovery_failed",
        prev_state = ?preferred.state,
        new_state = "N/A",
        iteration_state = ?preferred.state,
        error = "no_valid_checkpoint",
        timestamp = now_millis(),
        "所有 Checkpoint 均无法通过校验"
    );

    Err(RecoveryError::NoValidCheckpoint)
}

async fn recover_from_pause_state(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    correlation_id: &str,
    task_id: &str,
) -> Result<(OptimizationContext, CheckpointEntity), RecoveryError> {
    let registry = global_pause_registry();
    let controller = registry.get_or_create(task_id).await;
    let snapshot = controller
        .get_snapshot()
        .await
        .ok_or(RecoveryError::PauseStateNotFound)?;

    let artifacts = controller.get_artifacts().await;
    let guidance = controller.get_guidance().await;
    let checkpoint = build_compensation_checkpoint(task_id, &snapshot, artifacts, guidance)?;

    let checkpoint = CheckpointRepo::create_checkpoint(pool, checkpoint).await?;

    warn!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        checkpoint_id = %checkpoint.id,
        action = "compensation_checkpoint_created",
        prev_state = "N/A",
        new_state = ?checkpoint.state,
        iteration_state = ?checkpoint.state,
        timestamp = now_millis(),
        "未找到 Checkpoint，已生成补偿 Checkpoint"
    );

    let ctx = rebuild_optimization_context(pool, user_id, &checkpoint).await?;
    Ok((ctx, checkpoint))
}

fn build_compensation_checkpoint(
    task_id: &str,
    snapshot: &crate::core::iteration_engine::pause_state::PauseStateSnapshot,
    artifacts: Option<crate::domain::types::IterationArtifacts>,
    guidance: Option<crate::domain::types::UserGuidance>,
) -> Result<CheckpointEntity, RecoveryError> {
    let (prompt, rules) = derive_prompt_and_rules_from_artifacts(artifacts.as_ref());
    let rule_system = RuleSystem {
        rules,
        conflict_resolution_log: vec![],
        merge_log: vec![],
        coverage_map: HashMap::new(),
        version: 1,
    };

    let state = parse_iteration_state(&snapshot.stage);
    let req = CheckpointCreateRequest {
        task_id: task_id.to_string(),
        iteration: snapshot.iteration,
        state,
        run_control_state: RunControlState::Paused,
        prompt: prompt.clone(),
        rule_system: rule_system.clone(),
        artifacts: artifacts.clone(),
        user_guidance: guidance.clone(),
        branch_id: task_id.to_string(),
        parent_id: None,
        lineage_type: LineageType::Restored,
        branch_description: Some("pause_state_compensation".to_string()),
    };

    let checksum = compute_checksum(&req);

    Ok(CheckpointEntity {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: req.task_id,
        iteration: req.iteration,
        state: req.state,
        run_control_state: req.run_control_state,
        prompt: req.prompt,
        rule_system: req.rule_system,
        artifacts: req.artifacts,
        user_guidance: req.user_guidance,
        branch_id: req.branch_id,
        parent_id: req.parent_id,
        lineage_type: req.lineage_type,
        branch_description: req.branch_description,
        checksum,
        created_at: now_millis(),
    })
}

fn derive_prompt_and_rules_from_artifacts(
    artifacts: Option<&crate::domain::types::IterationArtifacts>,
) -> (String, Vec<Rule>) {
    let mut prompt = String::new();
    let mut rules = Vec::new();

    if let Some(artifacts) = artifacts {
        if let Some(best) = artifacts
            .candidate_prompts
            .iter()
            .find(|p| p.is_best)
            .or_else(|| artifacts.candidate_prompts.first())
        {
            prompt = best.content.clone();
        }

        for pattern in &artifacts.patterns {
            let rule = Rule {
                id: pattern.id.clone(),
                description: pattern.pattern.clone(),
                tags: RuleTags {
                    output_format: vec![],
                    output_structure: vec![],
                    output_length: OutputLength::Flexible,
                    semantic_focus: vec![],
                    key_concepts: vec![],
                    must_include: vec![],
                    must_exclude: vec![],
                    tone: None,
                    extra: HashMap::new(),
                },
                source_test_cases: vec![],
                abstraction_level: 0,
                parent_rules: vec![],
                verified: false,
                verification_score: pattern.confidence.unwrap_or(0.0),
                ir: None,
            };
            rules.push(rule);
        }
    }

    (prompt, rules)
}

fn parse_iteration_state(raw: &str) -> crate::domain::models::IterationState {
    serde_json::from_value::<crate::domain::models::IterationState>(serde_json::Value::String(
        raw.to_string(),
    ))
    .unwrap_or(crate::domain::models::IterationState::WaitingUser)
}

fn iteration_state_label(state: &IterationState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn run_control_state_label(state: &RunControlState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) async fn get_recovery_metrics_with_pool(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<RecoveryMetrics, RecoveryError> {
    Ok(RecoveryMetricsRepo::get_metrics(pool, task_id).await?)
}

async fn rebuild_optimization_context(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    checkpoint: &CheckpointEntity,
) -> Result<OptimizationContext, RecoveryError> {
    let task = OptimizationTaskRepo::find_by_id_for_user(pool, user_id, &checkpoint.task_id)
        .await
        .map_err(|_| RecoveryError::TaskNotFound)?;

    let task_with_sets =
        OptimizationTaskRepo::find_by_id_scoped(pool, user_id, &task.workspace_id, &task.id)
            .await
            .map_err(|_| RecoveryError::TaskNotFound)?;

    let task_config = OptimizationTaskConfig::normalized_from_config_json(
        task.config_json.as_deref(),
    );

    let execution_target_config = build_execution_target_config(
        pool,
        user_id,
        &task.workspace_id,
        task.execution_target_type,
        &task_config,
        &task_with_sets.test_set_ids,
    )
    .await?;

    let config = build_runtime_config(&task_config);

    let mut test_cases = Vec::new();
    for test_set_id in &task_with_sets.test_set_ids {
        let test_set = TestSetRepo::find_by_id(pool, &task.workspace_id, test_set_id).await?;
        test_cases.extend(test_set.cases);
    }

    if test_cases.is_empty() {
        return Err(RecoveryError::Context("测试集为空，无法恢复".to_string()));
    }

    let checkpoints = CheckpointRepo::list_checkpoints_by_task(pool, &checkpoint.task_id, 20)
        .await?
        .into_iter()
        .map(to_checkpoint_model)
        .collect::<Vec<_>>();

    let mut extensions: HashMap<String, serde_json::Value> = HashMap::new();
    if let Some(guidance) = checkpoint.user_guidance.clone() {
        if let Ok(value) = serde_json::to_value(guidance) {
            extensions.insert(EXT_USER_GUIDANCE.to_string(), value);
        }
    }
    if let Some(artifacts) = checkpoint.artifacts.clone() {
        if let Some(best) = artifacts
            .candidate_prompts
            .iter()
            .find(|p| p.is_best)
            .or_else(|| artifacts.candidate_prompts.first())
        {
            extensions.insert(
                EXT_BEST_CANDIDATE_PROMPT.to_string(),
                serde_json::Value::String(best.content.clone()),
            );
            let idx = best
                .id
                .strip_prefix("candidate:")
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| best.id.parse::<u64>().ok());
            if let Some(idx) = idx {
                extensions.insert(
                    EXT_BEST_CANDIDATE_INDEX.to_string(),
                    serde_json::Value::Number(idx.into()),
                );
            }
        }
        if let Ok(value) = serde_json::to_value(artifacts) {
            extensions.insert("artifacts".to_string(), value);
        }
    }

    Ok(OptimizationContext {
        task_id: checkpoint.task_id.clone(),
        execution_target_config,
        current_prompt: checkpoint.prompt.clone(),
        rule_system: checkpoint.rule_system.clone(),
        iteration: checkpoint.iteration,
        state: checkpoint.state,
        run_control_state: checkpoint.run_control_state,
        test_cases,
        config,
        checkpoints,
        extensions,
    })
}

fn to_checkpoint_model(entity: CheckpointEntity) -> Checkpoint {
    Checkpoint {
        id: entity.id,
        task_id: entity.task_id,
        iteration: entity.iteration,
        state: entity.state,
        prompt: entity.prompt,
        rule_system: entity.rule_system,
        created_at: entity.created_at,
        branch_id: entity.branch_id,
        parent_id: entity.parent_id,
        lineage_type: entity.lineage_type,
        branch_description: entity.branch_description,
    }
}

fn build_runtime_config(task_config: &OptimizationTaskConfig) -> OptimizationConfig {
    let mut cfg = OptimizationConfig::default();
    cfg.iteration.max_iterations = task_config.max_iterations.max(1);
    cfg.iteration.pass_threshold = (task_config.pass_threshold_percent as f64 / 100.0)
        .clamp(0.0, 1.0);
    cfg.iteration.diversity_inject_after = task_config.diversity_injection_threshold.max(1);

    cfg.output.strategy = match task_config.output_config.strategy {
        crate::domain::models::OutputStrategy::Single => crate::domain::types::OutputStrategy::Single,
        crate::domain::models::OutputStrategy::Adaptive => {
            crate::domain::types::OutputStrategy::Adaptive
        }
        crate::domain::models::OutputStrategy::Multi => crate::domain::types::OutputStrategy::Multi,
    };
    cfg.output.conflict_alert_threshold = task_config.output_config.conflict_alert_threshold;
    cfg.output.auto_recommend = task_config.output_config.auto_recommend;

    cfg.evaluator.llm_judge_samples = task_config
        .evaluator_config
        .teacher_model
        .llm_judge_samples
        .max(1);

    let train = task_config.data_split.train_percent as f64 / 100.0;
    let validation = task_config.data_split.validation_percent as f64 / 100.0;
    cfg.data_split.enabled = validation > 0.0;
    cfg.data_split.train_ratio = train;
    cfg.data_split.validation_ratio = validation;
    cfg.data_split.strategy = match task_config.advanced_data_split.sampling_strategy {
        crate::domain::models::SamplingStrategy::Random => SplitStrategy::Random,
        crate::domain::models::SamplingStrategy::Stratified => SplitStrategy::Stratified,
    };

    cfg
}

async fn build_execution_target_config(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    workspace_id: &str,
    execution_target_type: ExecutionTargetType,
    task_config: &OptimizationTaskConfig,
    test_set_ids: &[String],
) -> Result<ExecutionTargetConfig, RecoveryError> {
    match execution_target_type {
        ExecutionTargetType::Dify => {
            let prompt_variable =
                extract_prompt_variable(pool, workspace_id, test_set_ids).await?;
            let credential =
                CredentialRepo::find_by_user_and_type(pool, user_id, CredentialType::Dify).await?;
            Ok(ExecutionTargetConfig::Dify {
                api_url: credential.base_url,
                workflow_id: String::new(),
                prompt_variable,
                api_key: None,
            })
        }
        ExecutionTargetType::Generic => {
            let credential = CredentialRepo::find_by_user_and_type(
                pool,
                user_id,
                CredentialType::GenericLlm,
            )
            .await?;
            let model_name = task_config
                .teacher_llm
                .model_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            Ok(ExecutionTargetConfig::DirectModel {
                base_url: credential.base_url,
                model_name,
                user_prompt_template: "{input}".to_string(),
                api_key: None,
            })
        }
        ExecutionTargetType::Example => Ok(ExecutionTargetConfig::DirectModel {
            base_url: "http://localhost".to_string(),
            model_name: "example".to_string(),
            user_prompt_template: "{input}".to_string(),
            api_key: None,
        }),
    }
}

async fn extract_prompt_variable(
    pool: &sqlx::SqlitePool,
    workspace_id: &str,
    test_set_ids: &[String],
) -> Result<String, RecoveryError> {
    for test_set_id in test_set_ids {
        let test_set = TestSetRepo::find_by_id(pool, workspace_id, test_set_id).await?;
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
    use std::collections::HashMap;
    use crate::infra::db::pool::{create_pool, init_global_db_pool};
    use crate::infra::db::repositories::{TestSetRepo, WorkspaceRepo};
    use crate::domain::models::{TaskReference, TestCase};
    use crate::domain::types::{ArtifactSource, CandidatePrompt, IterationArtifacts};
    use serde_json::json;

    async fn setup_db() -> sqlx::SqlitePool {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("创建测试数据库失败");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("运行 migrations 失败");
        init_global_db_pool(pool.clone());
        pool
    }

    async fn insert_user(pool: &sqlx::SqlitePool, id: &str, username: &str) {
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

    fn sample_case() -> TestCase {
        TestCase {
            id: "case-1".to_string(),
            input: HashMap::new(),
            reference: TaskReference::Exact {
                expected: "ok".to_string(),
            },
            split: None,
            metadata: None,
        }
    }

    fn build_checkpoint(
        task_id: &str,
        iteration: u32,
        checksum_override: Option<String>,
    ) -> CheckpointEntity {
        let req = CheckpointCreateRequest {
            task_id: task_id.to_string(),
            iteration,
            state: crate::domain::models::IterationState::RunningTests,
            run_control_state: RunControlState::Running,
            prompt: format!("prompt-{iteration}"),
            rule_system: RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 1,
            },
            artifacts: None,
            user_guidance: None,
            branch_id: task_id.to_string(),
            parent_id: None,
            lineage_type: LineageType::Automatic,
            branch_description: None,
        };
        let checksum = checksum_override.unwrap_or_else(|| compute_checksum(&req));

        CheckpointEntity {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: req.task_id,
            iteration: req.iteration,
            state: req.state,
            run_control_state: req.run_control_state,
            prompt: req.prompt,
            rule_system: req.rule_system,
            artifacts: req.artifacts,
            user_guidance: req.user_guidance,
            branch_id: req.branch_id,
            parent_id: req.parent_id,
            lineage_type: req.lineage_type,
            branch_description: req.branch_description,
            checksum,
            created_at: now_millis(),
        }
    }

    #[tokio::test]
    async fn recover_from_checkpoint_falls_back_on_bad_checksum() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;
        let workspace = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");
        let test_set = TestSetRepo::create(
            &pool,
            &workspace.id,
            "ts",
            None,
            &[sample_case()],
            None,
            None,
        )
        .await
        .expect("创建测试集失败");

        let created = OptimizationTaskRepo::create_scoped(
            &pool,
            crate::infra::db::repositories::CreateOptimizationTaskInput {
                user_id: "u1",
                workspace_id: &workspace.id,
                name: "task",
                description: None,
                goal: "goal",
                execution_target_type: ExecutionTargetType::Example,
                task_mode: crate::domain::models::OptimizationTaskMode::Fixed,
                test_set_ids: std::slice::from_ref(&test_set.id),
            },
        )
        .await
        .expect("创建任务失败");

        let valid = build_checkpoint(&created.task.id, 1, None);
        CheckpointRepo::create_checkpoint(&pool, valid.clone())
            .await
            .expect("创建 checkpoint 失败");

        let invalid = build_checkpoint(&created.task.id, 2, Some("bad".to_string()));
        CheckpointRepo::create_checkpoint(&pool, invalid.clone())
            .await
            .expect("创建 checkpoint 失败");

        let ctx = recover_from_checkpoint_with_pool(&pool, &invalid.id, "u1", "cid-1")
            .await
            .expect("恢复失败");
        assert_eq!(ctx.iteration, valid.iteration);
    }

    #[tokio::test]
    async fn recover_task_uses_pause_state_compensation() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;
        let workspace = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");
        let test_set = TestSetRepo::create(
            &pool,
            &workspace.id,
            "ts",
            None,
            &[sample_case()],
            None,
            None,
        )
        .await
        .expect("创建测试集失败");

        let created = OptimizationTaskRepo::create_scoped(
            &pool,
            crate::infra::db::repositories::CreateOptimizationTaskInput {
                user_id: "u1",
                workspace_id: &workspace.id,
                name: "task",
                description: None,
                goal: "goal",
                execution_target_type: ExecutionTargetType::Example,
                task_mode: crate::domain::models::OptimizationTaskMode::Fixed,
                test_set_ids: std::slice::from_ref(&test_set.id),
            },
        )
        .await
        .expect("创建任务失败");

        let controller = global_pause_registry().get_or_create(&created.task.id).await;
        controller.request_pause("cid-1", "u1").await;
        let artifacts = IterationArtifacts {
            patterns: vec![],
            candidate_prompts: vec![CandidatePrompt {
                id: "candidate:0".to_string(),
                content: "recover-prompt".to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: true,
            }],
            user_guidance: None,
            updated_at: "now".to_string(),
        };
        let snapshot = json!({ "artifacts": artifacts });
        controller
            .checkpoint_pause(1, "running_tests", Some("cid-1"), snapshot)
            .await
            .expect("checkpoint pause");

        let (ctx, checkpoint) =
            recover_task_with_pool(&pool, &created.task.id, "u1", "cid-1", None)
                .await
                .expect("恢复失败");

        assert_eq!(ctx.current_prompt, "recover-prompt");
        assert!(!checkpoint.id.is_empty());
    }

    #[tokio::test]
    async fn abort_task_updates_status() {
        let pool = setup_db().await;
        insert_user(&pool, "u1", "user1").await;
        let workspace = WorkspaceRepo::create(&pool, "u1", "ws", None)
            .await
            .expect("创建工作区失败");
        let test_set = TestSetRepo::create(
            &pool,
            &workspace.id,
            "ts",
            None,
            &[sample_case()],
            None,
            None,
        )
        .await
        .expect("创建测试集失败");

        let created = OptimizationTaskRepo::create_scoped(
            &pool,
            crate::infra::db::repositories::CreateOptimizationTaskInput {
                user_id: "u1",
                workspace_id: &workspace.id,
                name: "task",
                description: None,
                goal: "goal",
                execution_target_type: ExecutionTargetType::Example,
                task_mode: crate::domain::models::OptimizationTaskMode::Fixed,
                test_set_ids: std::slice::from_ref(&test_set.id),
            },
        )
        .await
        .expect("创建任务失败");

        OptimizationTaskRepo::update_status(
            &pool,
            &created.task.id,
            OptimizationTaskStatus::Running,
        )
        .await
        .expect("更新状态失败");

        abort_task_with_pool(&pool, &created.task.id, "u1", "cid-1")
            .await
            .expect("中止失败");

        let loaded = OptimizationTaskRepo::find_by_id_for_user(&pool, "u1", &created.task.id)
            .await
            .expect("查询任务失败");
        assert_eq!(loaded.status, OptimizationTaskStatus::Terminated);
    }
}
