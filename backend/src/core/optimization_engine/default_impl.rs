use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::evaluator::EXT_TASK_EVALUATOR_CONFIG;
use crate::core::evaluator::{SplitFilter, build_evaluations_by_test_case_id, summarize_for_stats};
use crate::core::iteration_engine::orchestrator::IterationEngine;
use crate::core::traits::{
    Evaluator, ExecutionTarget, FeedbackAggregator, Optimizer, PromptGenerator, RuleEngine,
    TeacherModel,
};
use crate::domain::models::{
    Checkpoint, FailureType, IterationState, OptimizationResult, OptimizationTaskConfig,
    RecommendedAction, ReflectionResult, Suggestion, TerminationReason,
};
use crate::domain::types::{
    CandidateStats, EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_CANDIDATE_RANKING, EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID,
    EXTRA_ADOPT_BEST_CANDIDATE, OptimizationContext,
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

        ctx.state = IterationState::RunningTests;

        let prompt = ctx.current_prompt.clone();
        let batch = ctx.test_cases.clone();
        let engine = IterationEngine::new(Arc::clone(&self.execution_target));
        let exec_results = engine
            .run_tests(ctx, &prompt, &batch, &self.task_config)
            .await?;

        let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results)?;
        ctx.state = IterationState::Evaluating;
        // DefaultEvaluator 依赖 task 级 evaluator_config（写入方约定为编排层）。
        // OptimizationEngine 作为门面/编排入口，必须补齐该上下文，避免默认评估路径“隐式失败”。
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

        // Layer 1 追溯契约（RuleEngine/FeedbackAggregator 会消费；仅暴露 ID/结构化信息，不回显 input/prompt 原文）。
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

        // 最小闭环：将“当前 prompt”视作唯一候选（后续 Story 可扩展为多候选/多策略）。
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

        // RuleEngine：基于 Layer 1 结果更新规则体系（不要求本 Story 完整实现冲突处理/验证管线）。
        ctx.state = IterationState::ExtractingRules;
        let rules = self.rule_engine.extract_rules(ctx, &ctx.test_cases).await?;
        ctx.rule_system.rules = rules;
        ctx.rule_system.version = ctx.rule_system.version.saturating_add(1);

        let failed_ids: Vec<String> = evaluations
            .iter()
            .zip(batch.iter())
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
            ctx.state = IterationState::Reflecting;
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

        ctx.state = IterationState::Optimizing;
        let out = self
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

        Ok(out)
    }
}

#[async_trait]
impl OptimizationEngine for DefaultOptimizationEngine {
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
        "default_optimization_engine"
    }
}
