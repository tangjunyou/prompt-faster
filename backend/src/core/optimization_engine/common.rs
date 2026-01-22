use std::collections::HashMap;
use std::sync::Arc;

use crate::core::evaluator::EXT_TASK_EVALUATOR_CONFIG;
use crate::core::evaluator::{SplitFilter, build_evaluations_by_test_case_id, summarize_for_stats};
use crate::core::iteration_engine::checkpoint::save_checkpoint;
use crate::core::iteration_engine::events::record_event_async;
use crate::core::iteration_engine::orchestrator::{IterationEngine, record_evaluation_completed};
use crate::core::diversity_analyzer::{DefaultDiversityAnalyzer, DiversityAnalyzer};
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::core::traits::{Evaluator, ExecutionTarget};
use crate::domain::models::{
    Actor, CandidateSource, Checkpoint, DiversityConfig, EvaluationResult, EventType,
    ExecutionResult, FailureArchiveEntry, IterationState, OptimizationResult, OptimizationTaskConfig,
    PromptCandidate, TerminationReason, TestCase,
};
use crate::domain::types::{
    ArtifactSource, CandidatePrompt, CandidateStats, EXT_BEST_CANDIDATE_INDEX,
    EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS, EXT_CANDIDATE_RANKING,
    EXT_CURRENT_PROMPT_STATS, EXT_DIVERSITY_ANALYSIS, EXT_EVALUATIONS_BY_TEST_CASE_ID,
    EXT_FAILURE_ARCHIVE, EXT_PREV_ITERATION_STATE, EXT_TASK_MODE, EXT_USER_GUIDANCE,
    IterationArtifacts, OptimizationContext, PatternHypothesis, RunControlState,
};
use crate::infra::db::repositories::{IterationRepo, IterationRepoError};
use crate::shared::ws::chrono_timestamp;
use crate::shared::ws::{EVT_GUIDANCE_APPLIED, GuidanceAppliedPayload, WsMessage};
use crate::shared::ws_bus::global_ws_bus;
use serde_json::json;
use tokio::time::{sleep, timeout, Duration};

use super::OptimizationEngineError;

pub struct RunTestsAndEvaluateOutput {
    pub batch: Vec<TestCase>,
    pub evaluations: Vec<EvaluationResult>,
    #[cfg_attr(not(feature = "alt-optimization-engine"), allow(dead_code))]
    pub stats: crate::core::evaluator::EvaluationStats,
}

#[derive(Clone)]
struct DiversityAnalysisContext {
    task_id: String,
    iteration: u32,
    correlation_id: Option<String>,
}

pub fn validate_ctx_for_run(ctx: &OptimizationContext) -> Result<(), OptimizationEngineError> {
    if ctx.task_id.trim().is_empty() {
        return Err(OptimizationEngineError::InvalidRequest(
            "ctx.task_id 不能为空".to_string(),
        ));
    }
    if ctx.current_prompt.trim().is_empty() {
        return Err(OptimizationEngineError::InvalidRequest(
            "ctx.current_prompt 不能为空".to_string(),
        ));
    }
    if ctx.test_cases.is_empty() {
        return Err(OptimizationEngineError::InvalidRequest(
            "ctx.test_cases 为空（至少需要 1 个测试用例）".to_string(),
        ));
    }
    Ok(())
}

fn build_diversity_context(ctx: &OptimizationContext) -> DiversityAnalysisContext {
    DiversityAnalysisContext {
        task_id: ctx.task_id.clone(),
        iteration: ctx.iteration,
        correlation_id: read_optional_string(ctx, "correlation_id"),
    }
}

pub fn apply_checkpoint(checkpoint: Checkpoint, ctx: &mut OptimizationContext) {
    ctx.task_id = checkpoint.task_id;
    ctx.iteration = checkpoint.iteration;
    set_iteration_state(ctx, checkpoint.state);
    ctx.current_prompt = checkpoint.prompt;
    ctx.rule_system = checkpoint.rule_system;
}

fn read_optional_string(ctx: &OptimizationContext, key: &str) -> Option<String> {
    ctx.extensions
        .get(key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn record_error_event(ctx: &OptimizationContext, stage: &str, message: &str) {
    let correlation_id = read_optional_string(ctx, "correlation_id");
    record_event_async(
        ctx.task_id.clone(),
        EventType::ErrorOccurred,
        Actor::System,
        Some(json!({
            "stage": stage,
            "message": message,
        })),
        Some(ctx.iteration),
        correlation_id,
    );
}

fn read_optional_usize(ctx: &OptimizationContext, key: &str) -> Option<usize> {
    ctx.extensions
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
}

fn build_iteration_artifacts(ctx: &OptimizationContext) -> IterationArtifacts {
    let patterns: Vec<PatternHypothesis> = ctx
        .rule_system
        .rules
        .iter()
        .map(|rule| PatternHypothesis {
            id: rule.id.clone(),
            pattern: rule.description.clone(),
            source: ArtifactSource::System,
            confidence: Some(rule.verification_score),
        })
        .collect();

    let mut candidate_prompts: Vec<CandidatePrompt> = Vec::new();
    if !ctx.current_prompt.trim().is_empty() {
        candidate_prompts.push(CandidatePrompt {
            id: "current".to_string(),
            content: ctx.current_prompt.clone(),
            source: ArtifactSource::System,
            score: None,
            is_best: false,
        });
    }

    let best_prompt = read_optional_string(ctx, EXT_BEST_CANDIDATE_PROMPT);
    let best_index = read_optional_usize(ctx, EXT_BEST_CANDIDATE_INDEX);

    if let Some(best_prompt) = best_prompt {
        if candidate_prompts
            .first()
            .map(|p| p.content == best_prompt)
            .unwrap_or(false)
        {
            if let Some(current) = candidate_prompts.first_mut() {
                current.is_best = true;
            }
        } else {
            let id = best_index
                .map(|idx| format!("candidate:{idx}"))
                .unwrap_or_else(|| "best".to_string());
            candidate_prompts.push(CandidatePrompt {
                id,
                content: best_prompt,
                source: ArtifactSource::System,
                score: None,
                is_best: true,
            });
        }
    } else if let Some(current) = candidate_prompts.first_mut() {
        current.is_best = true;
    }

    let failure_archive = ctx
        .extensions
        .get(EXT_FAILURE_ARCHIVE)
        .and_then(|value| serde_json::from_value::<Vec<FailureArchiveEntry>>(value.clone()).ok())
        .filter(|entries| !entries.is_empty());

    let diversity_analysis = ctx
        .extensions
        .get(EXT_DIVERSITY_ANALYSIS)
        .and_then(|value| serde_json::from_value(value.clone()).ok());

    IterationArtifacts {
        patterns,
        candidate_prompts,
        user_guidance: None,
        failure_archive,
        diversity_analysis,
        updated_at: chrono_timestamp(),
    }
}

fn apply_artifacts_to_context(ctx: &mut OptimizationContext, artifacts: &IterationArtifacts) {
    let pattern_map: std::collections::HashMap<&str, &PatternHypothesis> = artifacts
        .patterns
        .iter()
        .map(|pattern| (pattern.id.as_str(), pattern))
        .collect();

    let mut updated_rules = Vec::new();
    for rule in &ctx.rule_system.rules {
        if let Some(pattern) = pattern_map.get(rule.id.as_str()) {
            let mut updated = rule.clone();
            updated.description = pattern.pattern.clone();
            updated_rules.push(updated);
        }
    }
    ctx.rule_system.rules = updated_rules;

    let preferred = artifacts
        .candidate_prompts
        .iter()
        .find(|prompt| prompt.is_best)
        .or_else(|| artifacts.candidate_prompts.first());

    if let Some(prompt) = preferred {
        if !prompt.content.trim().is_empty() {
            ctx.current_prompt = prompt.content.clone();
            ctx.extensions.insert(
                EXT_BEST_CANDIDATE_PROMPT.to_string(),
                serde_json::Value::String(prompt.content.clone()),
            );
            if let Some(idx_str) = prompt.id.strip_prefix("candidate:") {
                if let Ok(idx) = idx_str.parse::<u64>() {
                    ctx.extensions.insert(
                        EXT_BEST_CANDIDATE_INDEX.to_string(),
                        serde_json::Value::Number(idx.into()),
                    );
                }
            }
        }
    }
}

fn build_context_snapshot(
    ctx: &OptimizationContext,
    run_control_state: RunControlState,
) -> serde_json::Value {
    let artifacts = build_iteration_artifacts(ctx);
    serde_json::json!({
        "taskId": ctx.task_id,
        "iteration": ctx.iteration,
        "iterationState": ctx.state,
        "runControlState": run_control_state,
        "artifacts": artifacts,
    })
}

fn iteration_state_label(state: IterationState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) fn set_iteration_state(ctx: &mut OptimizationContext, next: IterationState) {
    let prev_label = iteration_state_label(ctx.state);
    ctx.extensions.insert(
        EXT_PREV_ITERATION_STATE.to_string(),
        serde_json::Value::String(prev_label),
    );
    ctx.state = next;
}

fn transition_run_control_state(
    ctx: &mut OptimizationContext,
    target: RunControlState,
) -> Result<(), OptimizationEngineError> {
    ctx.run_control_state
        .try_transition_to(target)
        .map_err(|err| OptimizationEngineError::Internal(format!("{err}")))
}

/// 应用用户引导并推送 guidance:applied 事件
async fn apply_user_guidance_if_present(
    ctx: &mut OptimizationContext,
    controller: &crate::core::iteration_engine::pause_state::PauseController,
) {
    if let Some(mut guidance) = controller.get_guidance().await {
        // 标记为已应用
        guidance.mark_applied();

        // 注入到 extensions
        if let Ok(guidance_value) = serde_json::to_value(&guidance) {
            ctx.extensions
                .insert(EXT_USER_GUIDANCE.to_string(), guidance_value);
        }

        // 推送 guidance:applied 事件
        let payload = GuidanceAppliedPayload {
            task_id: ctx.task_id.clone(),
            guidance_id: guidance.id.clone(),
            applied_at: guidance.applied_at.clone().unwrap_or_else(chrono_timestamp),
            iteration: ctx.iteration,
        };
        let correlation_id = controller
            .get_snapshot()
            .await
            .map(|s| s.correlation_id)
            .unwrap_or_else(|| format!("guidance-applied-{}", ctx.task_id));
        let msg = WsMessage::new(EVT_GUIDANCE_APPLIED, payload, correlation_id);
        if let Ok(text) = serde_json::to_string(&msg) {
            global_ws_bus().publish(text);
        }

        tracing::info!(
            task_id = %ctx.task_id,
            guidance_id = %guidance.id,
            iteration = ctx.iteration,
            "user guidance applied to iteration context"
        );

        // 清理 pause_state 中的引导（已应用）
        let _ = controller.clear_guidance().await;
    }
}

/// 清理本轮引导信息（确保单轮生效）
pub fn clear_user_guidance_from_context(ctx: &mut OptimizationContext) {
    ctx.extensions.remove(EXT_USER_GUIDANCE);
}

pub async fn checkpoint_pause_if_requested(
    ctx: &mut OptimizationContext,
) -> Result<bool, OptimizationEngineError> {
    let registry = global_pause_registry();
    let controller = registry.get_or_create(&ctx.task_id).await;

    if controller.is_paused() {
        transition_run_control_state(ctx, RunControlState::Paused)?;
        controller.wait_for_resume().await;
        if let Some(artifacts) = controller.get_artifacts().await {
            apply_artifacts_to_context(ctx, &artifacts);
        }
        // 应用用户引导（在 Layer 1 开始前）
        apply_user_guidance_if_present(ctx, &controller).await;
        controller.clear_snapshot().await;
        transition_run_control_state(ctx, RunControlState::Running)?;
        return Ok(true);
    }

    if !controller.is_pause_requested() {
        return Ok(false);
    }

    let stage = iteration_state_label(ctx.state);
    if controller
        .checkpoint_pause(
            ctx.iteration,
            &stage,
            None,
            build_context_snapshot(ctx, RunControlState::Paused),
        )
        .await?
    {
        transition_run_control_state(ctx, RunControlState::Paused)?;
        if let Err(err) = save_checkpoint(ctx, "", "").await {
            record_error_event(ctx, "checkpoint_pause_save", &err.to_string());
            let correlation_id = read_optional_string(ctx, "correlation_id")
                .unwrap_or_else(|| format!("checkpoint-{}", ctx.task_id));
            let user_id =
                read_optional_string(ctx, "user_id").unwrap_or_else(|| "system".to_string());
            tracing::error!(
                correlation_id = %correlation_id,
                user_id = %user_id,
                task_id = %ctx.task_id,
                action = "checkpoint_save_failed",
                prev_state = ?ctx.state,
                new_state = ?ctx.state,
                iteration_state = ?ctx.state,
                timestamp = crate::shared::time::now_millis(),
                error = %err,
                "Checkpoint 保存失败（已降级，继续执行）"
            );
        }
        controller.wait_for_resume().await;
        if let Some(artifacts) = controller.get_artifacts().await {
            apply_artifacts_to_context(ctx, &artifacts);
        }
        // 应用用户引导（在 Layer 1 开始前）
        apply_user_guidance_if_present(ctx, &controller).await;
        controller.clear_snapshot().await;
        transition_run_control_state(ctx, RunControlState::Running)?;
        return Ok(true);
    }

    Ok(false)
}

/// Layer 级别自动保存 Checkpoint（失败时降级不阻塞）
pub async fn save_checkpoint_after_layer(ctx: &OptimizationContext) {
    if let Err(err) = save_checkpoint(ctx, "", "").await {
        record_error_event(ctx, "checkpoint_layer_save", &err.to_string());
        let correlation_id = read_optional_string(ctx, "correlation_id")
            .unwrap_or_else(|| format!("checkpoint-{}", ctx.task_id));
        let user_id = read_optional_string(ctx, "user_id").unwrap_or_else(|| "system".to_string());
        tracing::error!(
            correlation_id = %correlation_id,
            user_id = %user_id,
            task_id = %ctx.task_id,
            action = "checkpoint_save_failed",
            prev_state = ?ctx.state,
            new_state = ?ctx.state,
            iteration_state = ?ctx.state,
            timestamp = crate::shared::time::now_millis(),
            error = %err,
            "Layer Checkpoint 保存失败（已降级，继续执行）"
        );
    }
}

/// 同步运行中迭代上限（用于增加轮数）
pub async fn sync_max_iterations(
    ctx: &mut OptimizationContext,
) -> Result<u32, OptimizationEngineError> {
    let registry = global_pause_registry();
    let controller = registry.get_or_create(&ctx.task_id).await;
    if let Some(max_iterations) = controller.get_max_iterations_override().await {
        ctx.config.iteration.max_iterations = max_iterations.max(1);
    }

    Ok(ctx.config.iteration.max_iterations.max(1))
}

/// 如果已请求终止，则返回终止结果
pub async fn stop_if_requested(
    ctx: &mut OptimizationContext,
    last: Option<OptimizationResult>,
) -> Result<Option<OptimizationResult>, OptimizationEngineError> {
    let registry = global_pause_registry();
    let controller = registry.get_or_create(&ctx.task_id).await;
    if !controller.is_stop_requested() {
        return Ok(None);
    }

    transition_run_control_state(ctx, RunControlState::Stopped)?;

    let result = if let Some(mut last) = last {
        last.should_terminate = true;
        last.termination_reason = Some(TerminationReason::UserStopped);
        last
    } else {
        OptimizationResult {
            primary: PromptCandidate {
                id: "current".to_string(),
                content: ctx.current_prompt.clone(),
                score: 0.0,
                source: CandidateSource::InitialGeneration,
                failure_fingerprints: Vec::new(),
            },
            alternatives: Vec::new(),
            should_terminate: true,
            termination_reason: Some(TerminationReason::UserStopped),
            iteration: ctx.iteration,
            improvement_summary: None,
            extra: HashMap::new(),
        }
    };

    Ok(Some(result))
}

pub async fn run_tests_and_evaluate(
    ctx: &mut OptimizationContext,
    execution_target: Arc<dyn ExecutionTarget>,
    evaluator: Arc<dyn Evaluator>,
    task_config: &OptimizationTaskConfig,
) -> Result<RunTestsAndEvaluateOutput, OptimizationEngineError> {
    set_iteration_state(ctx, IterationState::RunningTests);

    let prompt = ctx.current_prompt.clone();
    let batch = ctx.test_cases.clone();
    let engine = IterationEngine::new(execution_target);
    let exec_results = engine
        .run_tests(ctx, &prompt, &batch, task_config)
        .await
        .map_err(|err| {
            record_error_event(ctx, "run_tests", &err.to_string());
            OptimizationEngineError::from(err)
        })?;

    let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results).map_err(|err| {
        record_error_event(ctx, "build_evaluation_pairs", &err.to_string());
        OptimizationEngineError::from(err)
    })?;

    set_iteration_state(ctx, IterationState::Evaluating);
    // DefaultEvaluator 依赖 task 级 evaluator_config（写入方约定为编排层）。
    // OptimizationEngine 作为门面/编排入口，必须补齐该上下文，避免默认评估路径“隐式失败”。
    let evaluator_cfg_value =
        serde_json::to_value(&task_config.evaluator_config).map_err(|_| {
            OptimizationEngineError::Internal("evaluator_config 序列化失败".to_string())
        })?;
    ctx.extensions
        .insert(EXT_TASK_EVALUATOR_CONFIG.to_string(), evaluator_cfg_value);

    let evaluations = evaluator.evaluate_batch(ctx, &pairs).await.map_err(|err| {
        record_error_event(ctx, "evaluate_batch", &err.to_string());
        OptimizationEngineError::from(err)
    })?;

    let evaluations_by_id = build_evaluations_by_test_case_id(&pairs, &evaluations)?;
    let executions_by_id: HashMap<String, _> = exec_results
        .iter()
        .map(|r| (r.test_case_id.clone(), r.clone()))
        .collect();

    // Layer 1 追溯契约：仅暴露 ID/结构化信息，不回显 input/prompt 原文。
    let evaluations_value =
        serde_json::to_value(&evaluations_by_id).unwrap_or(serde_json::Value::Null);
    let executions_value =
        serde_json::to_value(&executions_by_id).unwrap_or(serde_json::Value::Null);
    ctx.extensions.insert(
        EXT_EVALUATIONS_BY_TEST_CASE_ID.to_string(),
        evaluations_value.clone(),
    );
    ctx.extensions.insert(
        "layer1_test_results".to_string(),
        serde_json::json!({
            "evaluations_by_test_case_id": evaluations_value,
            "executions_by_test_case_id": executions_value,
        }),
    );

    let stats = summarize_for_stats(SplitFilter::All, &pairs, &evaluations)?;
    let candidate_stats = CandidateStats {
        pass_rate: stats.pass_rate,
        mean_score: stats.mean_score,
    };

    record_evaluation_completed(ctx, stats.pass_rate, stats.total_count, stats.passed_count);

    ensure_task_mode(ctx).await;
    if should_compute_diversity(ctx, task_config) {
        spawn_diversity_analysis(ctx, task_config, &exec_results);
    }

    // 统一写入 Layer 4 约定的候选/最佳候选口径，确保与 Optimizer 的接口契约一致。
    ctx.extensions.insert(
        EXT_CURRENT_PROMPT_STATS.to_string(),
        serde_json::to_value(candidate_stats).unwrap_or(serde_json::Value::Null),
    );
    ctx.extensions.insert(
        EXT_BEST_CANDIDATE_STATS.to_string(),
        serde_json::to_value(candidate_stats).unwrap_or(serde_json::Value::Null),
    );
    ctx.extensions.insert(
        EXT_CANDIDATE_RANKING.to_string(),
        serde_json::json!([{
            "candidate_index": 0,
            "pass_rate": stats.pass_rate,
            "mean_score": stats.mean_score,
        }]),
    );
    ctx.extensions
        .insert(EXT_BEST_CANDIDATE_INDEX.to_string(), serde_json::json!(0));
    ctx.extensions.insert(
        EXT_BEST_CANDIDATE_PROMPT.to_string(),
        serde_json::json!(ctx.current_prompt.clone()),
    );

    Ok(RunTestsAndEvaluateOutput {
        batch,
        evaluations,
        stats,
    })
}

async fn ensure_task_mode(ctx: &mut OptimizationContext) {
    if ctx.extensions.contains_key(EXT_TASK_MODE) {
        return;
    }
    let Some(pool) = crate::infra::db::pool::global_db_pool() else {
        tracing::warn!(
            task_id = %ctx.task_id,
            "task_mode 缺失且数据库连接池不可用，无法判定任务模式"
        );
        return;
    };
    match sqlx::query_as::<_, (String,)>(
        "SELECT task_mode FROM optimization_tasks WHERE id = ?1",
    )
    .bind(&ctx.task_id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some((mode,))) => {
            ctx.extensions
                .insert(EXT_TASK_MODE.to_string(), serde_json::Value::String(mode));
        }
        Ok(None) => {
            tracing::warn!(
                task_id = %ctx.task_id,
                "task_mode 缺失且未找到任务记录，无法判定任务模式"
            );
        }
        Err(err) => {
            tracing::warn!(
                error = %err,
                task_id = %ctx.task_id,
                "查询 task_mode 失败，跳过多样性检测"
            );
        }
    }
}

fn should_compute_diversity(ctx: &OptimizationContext, task_config: &OptimizationTaskConfig) -> bool {
    if !task_config.diversity_config.enabled {
        return false;
    }
    let Some(mode) = ctx
        .extensions
        .get(EXT_TASK_MODE)
        .and_then(|v| v.as_str())
    else {
        tracing::warn!(
            task_id = %ctx.task_id,
            "task_mode 缺失，已启用多样性检测但无法判定任务模式，默认跳过"
        );
        return false;
    };
    mode == "creative"
}

fn spawn_diversity_analysis(
    ctx: &OptimizationContext,
    task_config: &OptimizationTaskConfig,
    exec_results: &[ExecutionResult],
) {
    let context = build_diversity_context(ctx);
    let config = task_config.diversity_config.clone();
    let outputs: Vec<String> = exec_results.iter().map(|r| r.output.clone()).collect();
    tokio::spawn(async move {
        if let Some(analysis) = compute_diversity_analysis(context.clone(), config, outputs).await {
            persist_diversity_analysis(&context, &analysis).await;
        }
    });
}

async fn persist_diversity_analysis(
    context: &DiversityAnalysisContext,
    analysis: &crate::domain::models::DiversityAnalysisResult,
) {
    let Some(pool) = crate::infra::db::pool::global_db_pool() else {
        tracing::warn!(
            correlation_id = ?context.correlation_id,
            task_id = %context.task_id,
            "数据库未初始化，跳过多样性分析写入"
        );
        return;
    };

    let mut delay = Duration::from_millis(200);
    let max_delay = Duration::from_secs(5);
    let max_attempts = 6;
    for attempt in 0..max_attempts {
        match IterationRepo::update_diversity_analysis_for_round(
            &pool,
            &context.task_id,
            context.iteration,
            analysis,
        )
        .await
        {
            Ok(_) => return,
            Err(IterationRepoError::NotFound) if attempt + 1 < max_attempts => {
                sleep(delay).await;
                delay = delay.saturating_mul(2).min(max_delay);
            }
            Err(IterationRepoError::NotFound) => break,
            Err(err) => {
                tracing::warn!(
                    correlation_id = ?context.correlation_id,
                    task_id = %context.task_id,
                    iteration = context.iteration,
                    error = %err,
                    "写入多样性分析产物失败"
                );
                return;
            }
        }
    }

    tracing::warn!(
        correlation_id = ?context.correlation_id,
        task_id = %context.task_id,
        iteration = context.iteration,
        attempts = max_attempts,
        "迭代产物仍未就绪，多样性分析写入放弃"
    );
}

async fn compute_diversity_analysis(
    ctx: DiversityAnalysisContext,
    config: DiversityConfig,
    outputs: Vec<String>,
) -> Option<crate::domain::models::DiversityAnalysisResult> {
    let normalized_outputs: Vec<String> = outputs
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let analyzer = DefaultDiversityAnalyzer::new(config);
    if normalized_outputs.len() < 2 {
        let mut analysis = analyzer.analyze(&normalized_outputs, None, None);
        analysis.baseline_comparison = None;
        analysis.warnings = Vec::new();
        analysis.suggestions = Vec::new();
        return Some(analysis);
    }
    let pool = crate::infra::db::pool::global_db_pool();
    let mut baseline = None;
    let mut baseline_known_missing = false;
    let mut baseline_read_failed = false;
    if let Some(pool) = pool.as_ref() {
        match timeout(
            Duration::from_millis(300),
            crate::infra::db::repositories::diversity_baseline_repo::DiversityBaselineRepo::get_by_task_id(
                pool,
                &ctx.task_id,
            ),
        )
        .await
        {
            Ok(Ok(found)) => {
                baseline_known_missing = found.is_none();
                baseline = found;
            }
            Ok(Err(err)) => {
                baseline_read_failed = true;
                tracing::warn!(
                    correlation_id = ?ctx.correlation_id,
                    error = %err,
                    task_id = %ctx.task_id,
                    "读取多样性基准线失败，跳过基准线对比"
                );
            }
            Err(_) => {
                baseline_read_failed = true;
                tracing::warn!(
                    correlation_id = ?ctx.correlation_id,
                    task_id = %ctx.task_id,
                    "读取多样性基准线超时，跳过基准线对比"
                );
            }
        }
    }

    let analysis = analyzer.analyze(&normalized_outputs, baseline.as_ref(), None);

    if baseline_known_missing || baseline_read_failed {
        if let Some(pool) = pool.as_ref() {
            let pool = pool.clone();
            let task_id = ctx.task_id.clone();
            let metrics = analysis.metrics.clone();
            let iteration = ctx.iteration;
            let correlation_id = ctx.correlation_id.clone();
            tokio::spawn(async move {
                if let Err(err) = crate::infra::db::repositories::diversity_baseline_repo::DiversityBaselineRepo::insert_if_absent(
                    &pool,
                    &task_id,
                    &metrics,
                    iteration,
                )
                .await
                {
                    tracing::warn!(
                        correlation_id = ?correlation_id,
                        error = %err,
                        task_id = %task_id,
                        "记录多样性基准线失败"
                    );
                }
            });
        }
    }

    Some(analysis)
}

#[cfg(test)]
mod diversity_gate_tests {
    use super::*;
    use crate::domain::models::RuleSystem;
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig};
    use crate::infra::db::pool::{create_pool, global_db_pool, init_global_db_pool};
    use sqlx::SqlitePool;

    fn base_ctx() -> OptimizationContext {
        OptimizationContext {
            task_id: "t1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "p".to_string(),
            rule_system: RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 1,
            },
            iteration: 0,
            state: IterationState::Idle,
            run_control_state: Default::default(),
            test_cases: vec![],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        }
    }

    async fn ensure_task_tables(pool: &SqlitePool) {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("create users");
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("create workspaces");
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS optimization_tasks (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                goal TEXT NOT NULL,
                execution_target_type TEXT NOT NULL,
                task_mode TEXT NOT NULL,
                status TEXT NOT NULL,
                config_json TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("create optimization_tasks");
    }

    async fn setup_task_mode_pool() -> SqlitePool {
        if let Some(pool) = global_db_pool() {
            ensure_task_tables(&pool).await;
            return pool;
        }
        let pool = create_pool("sqlite::memory:").await.expect("create pool");
        ensure_task_tables(&pool).await;
        init_global_db_pool(pool.clone());
        pool
    }

    #[test]
    fn diversity_requires_creative_mode() {
        let mut ctx = base_ctx();
        ctx.extensions.insert(
            EXT_TASK_MODE.to_string(),
            serde_json::Value::String("fixed".to_string()),
        );
        let mut cfg = OptimizationTaskConfig::default();
        cfg.diversity_config.enabled = true;
        assert!(!should_compute_diversity(&ctx, &cfg));
    }

    #[test]
    fn diversity_requires_enabled_flag() {
        let mut ctx = base_ctx();
        ctx.extensions.insert(
            EXT_TASK_MODE.to_string(),
            serde_json::Value::String("creative".to_string()),
        );
        let cfg = OptimizationTaskConfig::default();
        assert!(!should_compute_diversity(&ctx, &cfg));
    }

    #[test]
    fn diversity_runs_for_creative_enabled() {
        let mut ctx = base_ctx();
        ctx.extensions.insert(
            EXT_TASK_MODE.to_string(),
            serde_json::Value::String("creative".to_string()),
        );
        let mut cfg = OptimizationTaskConfig::default();
        cfg.diversity_config.enabled = true;
        assert!(should_compute_diversity(&ctx, &cfg));
    }

    #[test]
    fn diversity_skips_when_task_mode_missing() {
        let ctx = base_ctx();
        let mut cfg = OptimizationTaskConfig::default();
        cfg.diversity_config.enabled = true;
        assert!(!should_compute_diversity(&ctx, &cfg));
    }

    #[tokio::test]
    async fn diversity_reads_task_mode_from_db_when_missing() {
        let pool = setup_task_mode_pool().await;
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO users (id, username, password_hash, created_at, updated_at)
            VALUES ('u1', 'user', 'hash', 0, 0)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert user");
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO workspaces (id, user_id, name, description, created_at, updated_at)
            VALUES ('w1', 'u1', 'ws', NULL, 0, 0)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert workspace");
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO optimization_tasks
              (id, workspace_id, name, description, goal, execution_target_type, task_mode, status, config_json, created_at, updated_at)
            VALUES
              ('task-creative', 'w1', 't', NULL, 'g', 'generic', 'creative', 'draft', NULL, 0, 0)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert task");

        let mut ctx = base_ctx();
        ctx.task_id = "task-creative".to_string();
        let mut cfg = OptimizationTaskConfig::default();
        cfg.diversity_config.enabled = true;

        ensure_task_mode(&mut ctx).await;
        assert!(should_compute_diversity(&ctx, &cfg));
    }

    #[tokio::test]
    async fn diversity_analysis_returns_zero_for_single_output() {
        let mut cfg = OptimizationTaskConfig::default();
        cfg.diversity_config.enabled = true;
        let ctx = base_ctx();
        let exec_results = vec![ExecutionResult {
            test_case_id: "case-1".to_string(),
            output: "only one".to_string(),
            latency_ms: 0,
            token_usage: None,
            raw_response: None,
        }];

        let outputs = exec_results.iter().map(|r| r.output.clone()).collect();
        let analysis = compute_diversity_analysis(
            build_diversity_context(&ctx),
            cfg.diversity_config.clone(),
            outputs,
        )
            .await
            .expect("analysis");
        assert_eq!(analysis.metrics.overall_score, 0.0);
        assert!(analysis.warnings.is_empty());
    }
}
