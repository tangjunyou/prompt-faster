use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::traits::{
    Evaluator, ExecutionTarget, FeedbackAggregator, Optimizer, PromptGenerator, RuleEngine,
    TeacherModel,
};
use crate::domain::models::{
    Checkpoint, FailureType, IterationState, OptimizationResult, OptimizationTaskConfig,
    RecommendedAction, ReflectionResult, Suggestion, TerminationReason,
};
use crate::domain::types::{
    EXT_BEST_CANDIDATE_PROMPT, EXTRA_ADOPT_BEST_CANDIDATE, OptimizationContext, RunControlState,
};

use super::common::{
    apply_checkpoint, checkpoint_pause_if_requested, clear_user_guidance_from_context,
    run_tests_and_evaluate, save_checkpoint_after_layer, set_iteration_state, stop_if_requested,
    sync_max_iterations, validate_ctx_for_run,
};
use super::{OptimizationEngine, OptimizationEngineError};

pub struct DefaultOptimizationEngine {
    rule_engine: Arc<dyn RuleEngine>,
    prompt_generator: Arc<dyn PromptGenerator>,
    evaluator: Arc<dyn Evaluator>,
    feedback_aggregator: Arc<dyn FeedbackAggregator>,
    optimizer: Arc<dyn Optimizer>,
    teacher_model: Arc<dyn TeacherModel>,
    execution_target: Arc<dyn ExecutionTarget>,
    task_config: OptimizationTaskConfig,
}

pub struct DefaultOptimizationEngineParts {
    pub rule_engine: Arc<dyn RuleEngine>,
    pub prompt_generator: Arc<dyn PromptGenerator>,
    pub evaluator: Arc<dyn Evaluator>,
    pub feedback_aggregator: Arc<dyn FeedbackAggregator>,
    pub optimizer: Arc<dyn Optimizer>,
    pub teacher_model: Arc<dyn TeacherModel>,
    pub execution_target: Arc<dyn ExecutionTarget>,
    pub task_config: OptimizationTaskConfig,
}

impl DefaultOptimizationEngine {
    pub fn new(parts: DefaultOptimizationEngineParts) -> Self {
        Self {
            rule_engine: parts.rule_engine,
            prompt_generator: parts.prompt_generator,
            evaluator: parts.evaluator,
            feedback_aggregator: parts.feedback_aggregator,
            optimizer: parts.optimizer,
            teacher_model: parts.teacher_model,
            execution_target: parts.execution_target,
            task_config: parts.task_config,
        }
    }

    async fn run_one_iteration(
        &self,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError> {
        let _prompt_generator_name = self.prompt_generator.name();
        if cfg!(test) {
            let _ = self.teacher_model.generate("ping").await.map_err(|_| {
                OptimizationEngineError::Internal("teacher model ping failed".to_string())
            })?;
        }

        let run = run_tests_and_evaluate(
            ctx,
            Arc::clone(&self.execution_target),
            Arc::clone(&self.evaluator),
            &self.task_config,
        )
        .await?;
        save_checkpoint_after_layer(ctx).await;
        checkpoint_pause_if_requested(ctx).await?;
        if let Some(stopped) = stop_if_requested(ctx, None).await? {
            return Ok(stopped);
        }

        // RuleEngine：基于 Layer 1 结果更新规则体系（不要求本 Story 完整实现冲突处理/验证管线）。
        set_iteration_state(ctx, IterationState::ExtractingRules);
        let rules = self.rule_engine.extract_rules(ctx, &ctx.test_cases).await?;
        ctx.rule_system.rules = rules;
        ctx.rule_system.version = ctx.rule_system.version.saturating_add(1);
        save_checkpoint_after_layer(ctx).await;
        checkpoint_pause_if_requested(ctx).await?;
        if let Some(stopped) = stop_if_requested(ctx, None).await? {
            return Ok(stopped);
        }

        let failed_ids: Vec<String> = run
            .evaluations
            .iter()
            .zip(run.batch.iter())
            .filter_map(|(ev, tc)| (!ev.passed).then_some(tc.id.clone()))
            .collect();

        let unified_reflection = if failed_ids.is_empty() {
            crate::domain::models::UnifiedReflection {
                primary_failure_type: FailureType::ExpressionIssue,
                unified_suggestions: Vec::new(),
                has_conflicts: false,
                conflicts: Vec::new(),
                arbitration_result: None,
                source_count: 0,
                failure_type_distribution: HashMap::new(),
                recommended_action: RecommendedAction::Terminate {
                    reason: TerminationReason::AllTestsPassed,
                },
                extra: HashMap::new(),
            }
        } else {
            set_iteration_state(ctx, IterationState::Reflecting);
            let rr = ReflectionResult {
                failure_type: FailureType::ExpressionIssue,
                analysis: "minimal deterministic reflection (no prompt/input echo)".to_string(),
                root_cause: "derived from evaluation pass/fail only".to_string(),
                suggestions: Vec::<Suggestion>::new(),
                failed_test_case_ids: failed_ids,
                related_rule_ids: Vec::new(),
                evaluation_ref: None,
                extra: HashMap::new(),
            };
            self.feedback_aggregator.aggregate(ctx, &[rr]).await?
        };
        save_checkpoint_after_layer(ctx).await;
        checkpoint_pause_if_requested(ctx).await?;
        if let Some(stopped) = stop_if_requested(ctx, None).await? {
            return Ok(stopped);
        }

        set_iteration_state(ctx, IterationState::Optimizing);
        let out = self
            .optimizer
            .optimize_step(ctx, &unified_reflection)
            .await?;
        save_checkpoint_after_layer(ctx).await;
        checkpoint_pause_if_requested(ctx).await?;
        if let Some(stopped) = stop_if_requested(ctx, None).await? {
            return Ok(stopped);
        }

        if out
            .extra
            .get(EXTRA_ADOPT_BEST_CANDIDATE)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(p) = ctx
                .extensions
                .get(EXT_BEST_CANDIDATE_PROMPT)
                .and_then(|v| v.as_str())
            {
                ctx.current_prompt = p.to_string();
            }
        }

        clear_user_guidance_from_context(ctx);
        Ok(out)
    }
}

#[async_trait]
impl OptimizationEngine for DefaultOptimizationEngine {
    async fn run(
        &self,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError> {
        validate_ctx_for_run(ctx)?;
        ctx.run_control_state
            .try_transition_to(RunControlState::Running)
            .map_err(|err| OptimizationEngineError::Internal(format!("{err}")))?;

        let mut last: Option<OptimizationResult> = None;

        loop {
            if let Some(stopped) = stop_if_requested(ctx, last.clone()).await? {
                return Ok(stopped);
            }

            let max_iters = sync_max_iterations(ctx).await?;
            if ctx.iteration >= max_iters {
                break;
            }

            ctx.iteration = ctx.iteration.saturating_add(1);
            let out = self.run_one_iteration(ctx).await?;
            last = Some(out.clone());
            if out.should_terminate {
                set_iteration_state(ctx, IterationState::Completed);
                if !matches!(out.termination_reason, Some(TerminationReason::UserStopped)) {
                    let _ = ctx
                        .run_control_state
                        .try_transition_to(RunControlState::Idle);
                }
                return Ok(out);
            }
        }

        let Some(last) = last else {
            return Err(OptimizationEngineError::Internal(
                "no iterations executed (unexpected state)".to_string(),
            ));
        };

        let _ = ctx
            .run_control_state
            .try_transition_to(RunControlState::Idle);
        Ok(last)
    }

    async fn resume(
        &self,
        checkpoint: Checkpoint,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError> {
        apply_checkpoint(checkpoint, ctx);
        ctx.run_control_state
            .try_transition_to(RunControlState::Running)
            .map_err(|err| OptimizationEngineError::Internal(format!("{err}")))?;
        self.run(ctx).await
    }

    fn name(&self) -> &str {
        "default_optimization_engine"
    }
}
