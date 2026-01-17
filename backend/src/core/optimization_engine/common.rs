use std::collections::HashMap;
use std::sync::Arc;

use crate::core::evaluator::EXT_TASK_EVALUATOR_CONFIG;
use crate::core::evaluator::{SplitFilter, build_evaluations_by_test_case_id, summarize_for_stats};
use crate::core::iteration_engine::orchestrator::IterationEngine;
use crate::core::iteration_engine::pause_state::global_pause_registry;
use crate::core::traits::{Evaluator, ExecutionTarget};
use crate::domain::models::{
    Checkpoint, EvaluationResult, IterationState, OptimizationTaskConfig, TestCase,
};
use crate::domain::types::{
    ArtifactSource, CandidatePrompt, CandidateStats, IterationArtifacts, PatternHypothesis,
    EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_CANDIDATE_RANKING, EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID,
    OptimizationContext, RunControlState,
};
use crate::shared::ws::chrono_timestamp;

use super::OptimizationEngineError;

pub struct RunTestsAndEvaluateOutput {
    pub batch: Vec<TestCase>,
    pub evaluations: Vec<EvaluationResult>,
    #[cfg_attr(not(feature = "alt-optimization-engine"), allow(dead_code))]
    pub stats: crate::core::evaluator::EvaluationStats,
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

pub fn apply_checkpoint(checkpoint: Checkpoint, ctx: &mut OptimizationContext) {
    ctx.task_id = checkpoint.task_id;
    ctx.iteration = checkpoint.iteration;
    ctx.state = checkpoint.state;
    ctx.current_prompt = checkpoint.prompt;
    ctx.rule_system = checkpoint.rule_system;
}

fn read_optional_string(ctx: &OptimizationContext, key: &str) -> Option<String> {
    ctx.extensions
        .get(key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn read_optional_usize(ctx: &OptimizationContext, key: &str) -> Option<usize> {
    ctx.extensions.get(key).and_then(|v| v.as_u64()).map(|n| n as usize)
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

    IterationArtifacts {
        patterns,
        candidate_prompts,
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

fn build_context_snapshot(ctx: &OptimizationContext) -> serde_json::Value {
    let artifacts = build_iteration_artifacts(ctx);
    serde_json::json!({
        "taskId": ctx.task_id,
        "iteration": ctx.iteration,
        "iterationState": ctx.state,
        "runControlState": ctx.run_control_state,
        "artifacts": artifacts,
    })
}

fn iteration_state_label(state: IterationState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn transition_run_control_state(
    ctx: &mut OptimizationContext,
    target: RunControlState,
) -> Result<(), OptimizationEngineError> {
    ctx.run_control_state
        .try_transition_to(target)
        .map_err(|err| OptimizationEngineError::Internal(format!("{err}")))
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
        controller.clear_snapshot().await;
        transition_run_control_state(ctx, RunControlState::Running)?;
        return Ok(true);
    }

    if !controller.is_pause_requested() {
        return Ok(false);
    }

    let stage = iteration_state_label(ctx.state);
    if controller
        .checkpoint_pause(ctx.iteration, &stage, None, build_context_snapshot(ctx))
        .await?
    {
        transition_run_control_state(ctx, RunControlState::Paused)?;
        controller.wait_for_resume().await;
        if let Some(artifacts) = controller.get_artifacts().await {
            apply_artifacts_to_context(ctx, &artifacts);
        }
        controller.clear_snapshot().await;
        transition_run_control_state(ctx, RunControlState::Running)?;
        return Ok(true);
    }

    Ok(false)
}

pub async fn run_tests_and_evaluate(
    ctx: &mut OptimizationContext,
    execution_target: Arc<dyn ExecutionTarget>,
    evaluator: Arc<dyn Evaluator>,
    task_config: &OptimizationTaskConfig,
) -> Result<RunTestsAndEvaluateOutput, OptimizationEngineError> {
    ctx.state = IterationState::RunningTests;

    let prompt = ctx.current_prompt.clone();
    let batch = ctx.test_cases.clone();
    let engine = IterationEngine::new(execution_target);
    let exec_results = engine.run_tests(ctx, &prompt, &batch, task_config).await?;

    let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results)?;

    ctx.state = IterationState::Evaluating;
    // DefaultEvaluator 依赖 task 级 evaluator_config（写入方约定为编排层）。
    // OptimizationEngine 作为门面/编排入口，必须补齐该上下文，避免默认评估路径“隐式失败”。
    let evaluator_cfg_value =
        serde_json::to_value(&task_config.evaluator_config).map_err(|_| {
            OptimizationEngineError::Internal("evaluator_config 序列化失败".to_string())
        })?;
    ctx.extensions
        .insert(EXT_TASK_EVALUATOR_CONFIG.to_string(), evaluator_cfg_value);

    let evaluations = evaluator.evaluate_batch(ctx, &pairs).await?;

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
