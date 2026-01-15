use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::evaluator::EXT_TASK_EVALUATOR_CONFIG;
use crate::core::evaluator::{SplitFilter, build_evaluations_by_test_case_id, summarize_for_stats};
use crate::core::iteration_engine::orchestrator::IterationEngine;
use crate::core::traits::{
    Evaluator, ExecutionTarget, FeedbackAggregator, Optimizer, RuleEngine, TeacherModel,
};
use crate::domain::models::{
    CandidateSource, Checkpoint, FailureType, IterationState, OptimizationResult,
    OptimizationTaskConfig, PromptCandidate, ReflectionResult, Suggestion, TerminationReason,
};
use crate::domain::types::{
    CandidateStats, EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_CANDIDATE_RANKING, EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID,
    EXTRA_ADOPT_BEST_CANDIDATE, METRIC_EPS, OptimizationContext,
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

        let prompt = ctx.current_prompt.clone();
        let batch = ctx.test_cases.clone();
        let engine = IterationEngine::new(Arc::clone(&self.execution_target));
        let exec_results = engine
            .run_tests(ctx, &prompt, &batch, &self.task_config)
            .await?;

        let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results)?;
        ctx.state = IterationState::Evaluating;

        // DefaultEvaluator 依赖 task 级 evaluator_config（写入方约定为编排层）。
        // Alternate 作为替代门面实现，同样必须补齐该上下文，保证路径不腐烂。
        let evaluator_cfg_value = serde_json::to_value(&self.task_config.evaluator_config)
            .map_err(|_| {
                OptimizationEngineError::Internal("evaluator_config 序列化失败".to_string())
            })?;
        ctx.extensions
            .insert(EXT_TASK_EVALUATOR_CONFIG.to_string(), evaluator_cfg_value);

        let evaluations = self.evaluator.evaluate_batch(ctx, &pairs).await?;
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

            return Ok(OptimizationResult {
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
            });
        }

        // === full pipeline（与默认实现不同点：仅在 fast-path 未命中时才进入）===
        ctx.state = IterationState::ExtractingRules;
        let rules = self.rule_engine.extract_rules(ctx, &ctx.test_cases).await?;
        ctx.rule_system.rules = rules;
        ctx.rule_system.version = ctx.rule_system.version.saturating_add(1);

        let failed_ids: Vec<String> = evaluations
            .iter()
            .zip(batch.iter())
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

        ctx.state = IterationState::Optimizing;
        let mut out = self
            .optimizer
            .optimize_step(ctx, &unified_reflection)
            .await?;

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
        Ok(out)
    }
}

#[async_trait]
impl OptimizationEngine for AlternateOptimizationEngine {
    async fn run(
        &self,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError> {
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

        let max_iters = ctx.config.iteration.max_iterations.max(1);
        let mut last: Option<OptimizationResult> = None;

        while ctx.iteration < max_iters {
            ctx.iteration = ctx.iteration.saturating_add(1);
            let out = self.run_one_iteration(ctx).await?;
            last = Some(out.clone());
            if out.should_terminate {
                ctx.state = IterationState::Completed;
                return Ok(out);
            }
        }

        last.ok_or_else(|| {
            OptimizationEngineError::Internal(
                "no iterations executed (unexpected state)".to_string(),
            )
        })
    }

    async fn resume(
        &self,
        checkpoint: Checkpoint,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError> {
        ctx.task_id = checkpoint.task_id;
        ctx.iteration = checkpoint.iteration;
        ctx.state = checkpoint.state;
        ctx.current_prompt = checkpoint.prompt;
        ctx.rule_system = checkpoint.rule_system;

        self.run(ctx).await
    }

    fn name(&self) -> &str {
        "alternate_optimization_engine"
    }
}
