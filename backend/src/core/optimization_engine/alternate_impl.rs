use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::traits::{
    Evaluator, ExecutionTarget, FeedbackAggregator, Optimizer, RuleEngine, TeacherModel,
};
use crate::domain::models::{
    CandidateSource, Checkpoint, FailureType, IterationState, OptimizationResult,
    OptimizationTaskConfig, PromptCandidate, ReflectionResult, Suggestion, TerminationReason,
};
use crate::domain::types::{
    EXT_BEST_CANDIDATE_PROMPT, EXTRA_ADOPT_BEST_CANDIDATE, METRIC_EPS, OptimizationContext,
    RunControlState,
};

use super::common::{
    apply_checkpoint, checkpoint_pause_if_requested, clear_user_guidance_from_context,
    run_tests_and_evaluate, validate_ctx_for_run,
};
use super::{OptimizationEngine, OptimizationEngineError};

pub struct AlternateOptimizationEngineParts {
    pub rule_engine: Arc<dyn RuleEngine>,
    pub evaluator: Arc<dyn Evaluator>,
    pub feedback_aggregator: Arc<dyn FeedbackAggregator>,
    pub optimizer: Arc<dyn Optimizer>,
    pub teacher_model: Arc<dyn TeacherModel>,
    pub execution_target: Arc<dyn ExecutionTarget>,
    pub task_config: OptimizationTaskConfig,
}

pub struct AlternateOptimizationEngine {
    rule_engine: Arc<dyn RuleEngine>,
    evaluator: Arc<dyn Evaluator>,
    feedback_aggregator: Arc<dyn FeedbackAggregator>,
    optimizer: Arc<dyn Optimizer>,
    teacher_model: Arc<dyn TeacherModel>,
    execution_target: Arc<dyn ExecutionTarget>,
    task_config: OptimizationTaskConfig,
}

impl AlternateOptimizationEngine {
    pub fn new(parts: AlternateOptimizationEngineParts) -> Self {
        Self {
            rule_engine: parts.rule_engine,
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
        checkpoint_pause_if_requested(ctx).await?;
        let stats = &run.stats;

        // === Alternate 中等差异：fast-path ===
        // 若已达到通过率阈值：直接终止，不执行 rule_engine/feedback_aggregator/optimizer。
        let pass_threshold = ctx.config.iteration.pass_threshold;
        if stats.pass_rate + METRIC_EPS >= pass_threshold {
            ctx.state = IterationState::Completed;
            let mut extra = HashMap::new();
            extra.insert("engine_variant".to_string(), serde_json::json!("alternate"));
            extra.insert("alternate_path".to_string(), serde_json::json!("fast_path"));
            extra.insert("pass_rate".to_string(), serde_json::json!(stats.pass_rate));
            extra.insert(
                "pass_threshold".to_string(),
                serde_json::json!(pass_threshold),
            );

            let termination_reason = if stats.pass_rate + METRIC_EPS >= 1.0 {
                Some(TerminationReason::AllTestsPassed)
            } else {
                Some(TerminationReason::PassThresholdReached {
                    threshold: pass_threshold,
                    actual: stats.pass_rate,
                })
            };

            let result = OptimizationResult {
                primary: PromptCandidate {
                    id: "current".to_string(),
                    content: ctx.current_prompt.clone(),
                    score: stats.pass_rate,
                    source: CandidateSource::ExpressionRefinement,
                    failure_fingerprints: Vec::new(),
                },
                alternatives: Vec::new(),
                should_terminate: true,
                termination_reason,
                iteration: ctx.iteration,
                improvement_summary: Some(
                    "alternate fast-path: pass_rate 达标，跳过 rule/reflect/optimize".to_string(),
                ),
                extra,
            };
            clear_user_guidance_from_context(ctx);
            return Ok(result);
        }

        // === full pipeline（与默认实现不同点：仅在 fast-path 未命中时才进入）===
        ctx.state = IterationState::ExtractingRules;
        let rules = self.rule_engine.extract_rules(ctx, &ctx.test_cases).await?;
        ctx.rule_system.rules = rules;
        ctx.rule_system.version = ctx.rule_system.version.saturating_add(1);
        checkpoint_pause_if_requested(ctx).await?;

        let failed_ids: Vec<String> = run
            .evaluations
            .iter()
            .zip(run.batch.iter())
            .filter_map(|(ev, tc)| (!ev.passed).then_some(tc.id.clone()))
            .collect();

        ctx.state = IterationState::Reflecting;
        let rr = ReflectionResult {
            failure_type: FailureType::ExpressionIssue,
            analysis: "alternate deterministic reflection (no prompt/input echo)".to_string(),
            root_cause: "derived from evaluation pass/fail only".to_string(),
            suggestions: Vec::<Suggestion>::new(),
            failed_test_case_ids: failed_ids,
            related_rule_ids: Vec::new(),
            evaluation_ref: None,
            extra: HashMap::new(),
        };
        let unified_reflection = self.feedback_aggregator.aggregate(ctx, &[rr]).await?;
        checkpoint_pause_if_requested(ctx).await?;

        ctx.state = IterationState::Optimizing;
        let mut out = self
            .optimizer
            .optimize_step(ctx, &unified_reflection)
            .await?;
        checkpoint_pause_if_requested(ctx).await?;

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

        out.extra
            .insert("engine_variant".to_string(), serde_json::json!("alternate"));
        out.extra.insert(
            "alternate_path".to_string(),
            serde_json::json!("full_pipeline"),
        );
        clear_user_guidance_from_context(ctx);
        Ok(out)
    }
}

#[async_trait]
impl OptimizationEngine for AlternateOptimizationEngine {
    async fn run(
        &self,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError> {
        validate_ctx_for_run(ctx)?;
        ctx.run_control_state
            .try_transition_to(RunControlState::Running)
            .map_err(|err| OptimizationEngineError::Internal(format!("{err}")))?;

        let max_iters = ctx.config.iteration.max_iterations.max(1);
        let mut last: Option<OptimizationResult> = None;

        while ctx.iteration < max_iters {
            ctx.iteration = ctx.iteration.saturating_add(1);
            let out = self.run_one_iteration(ctx).await?;
            last = Some(out.clone());
            if out.should_terminate {
                ctx.state = IterationState::Completed;
                let _ = ctx
                    .run_control_state
                    .try_transition_to(RunControlState::Idle);
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
        "alternate_optimization_engine"
    }
}
