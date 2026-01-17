use std::collections::HashMap;
use std::sync::Arc;

use crate::core::evaluator::EvaluatorError;
use crate::core::execution_target::ExecutionError;
use crate::core::feedback_aggregator::AggregatorError;
use crate::core::iteration_engine::executor::{parallel_execute, serial_execute};
use crate::core::prompt_generator::{EXT_CANDIDATE_INDEX, GeneratorError, TEMPLATE_VARIANT_COUNT};
use crate::core::traits::Evaluator;
use crate::core::traits::ExecutionTarget;
use crate::core::traits::FeedbackAggregator;
use crate::core::traits::PromptGenerator;
use crate::domain::models::{
    EvaluationResult, ExecutionMode, ExecutionResult, FailureArchiveEntry, FailureType,
    IterationState, OptimizationTaskConfig, RecommendedAction, ReflectionResult, Suggestion,
    SuggestionType, TestCase, UnifiedReflection,
};
use crate::domain::types::{
    CandidateStats, EXT_BEST_CANDIDATE_STATS, EXT_CONSECUTIVE_NO_IMPROVEMENT,
    EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID, EXT_FAILURE_ARCHIVE,
    FAILURE_ARCHIVE_MAX_ENTRIES, METRIC_EPS, OptimizationContext,
};
use serde_json::json;
use thiserror::Error;

#[derive(Clone)]
pub struct IterationEngine {
    execution_target: Arc<dyn ExecutionTarget>,
}

impl IterationEngine {
    pub fn new(execution_target: Arc<dyn ExecutionTarget>) -> Self {
        Self { execution_target }
    }

    pub async fn run_tests(
        &self,
        ctx: &mut OptimizationContext,
        prompt: &str,
        batch: &[TestCase],
        task_config: &OptimizationTaskConfig,
    ) -> Result<Vec<ExecutionResult>, ExecutionError> {
        ctx.state = IterationState::RunningTests;

        let results = match task_config.execution_mode {
            ExecutionMode::Serial => {
                serial_execute(
                    self.execution_target.as_ref(),
                    &ctx.execution_target_config,
                    prompt,
                    batch,
                )
                .await?
            }
            ExecutionMode::Parallel => {
                parallel_execute(
                    Arc::clone(&self.execution_target),
                    &ctx.execution_target_config,
                    prompt,
                    batch,
                    task_config.max_concurrency,
                )
                .await?
            }
        };

        // Hard contract: results must align with input batch order AND be self-identifying.
        for (idx, (tc, r)) in batch.iter().zip(results.iter()).enumerate() {
            if tc.id != r.test_case_id {
                return Err(ExecutionError::Internal {
                    test_case_id: r.test_case_id.clone(),
                    message: format!(
                        "execution result test_case_id mismatch at index={idx}: expected={}, actual={}",
                        tc.id, r.test_case_id
                    ),
                });
            }
        }

        Ok(results)
    }

    /// Build `Evaluator.evaluate_batch(ctx, results)` input pairs in stable order.
    ///
    /// Hard contracts:
    /// - output length equals `batch.len()`
    /// - `(TestCase, output)` pairs preserve `batch` order
    /// - `ExecutionResult.test_case_id` must match `TestCase.id` at the same index
    pub fn build_evaluation_pairs(
        batch: &[TestCase],
        exec_results: &[ExecutionResult],
    ) -> Result<Vec<(TestCase, String)>, ExecutionError> {
        if batch.len() != exec_results.len() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: "unknown".to_string(),
                message: format!(
                    "batch/exec_results length mismatch: {} vs {}",
                    batch.len(),
                    exec_results.len()
                ),
            });
        }

        let mut out = Vec::with_capacity(batch.len());
        for (idx, (tc, r)) in batch.iter().zip(exec_results.iter()).enumerate() {
            if tc.id != r.test_case_id {
                return Err(ExecutionError::Internal {
                    test_case_id: r.test_case_id.clone(),
                    message: format!(
                        "execution result test_case_id mismatch at index={idx}: expected={}, actual={}",
                        tc.id, r.test_case_id
                    ),
                });
            }
            out.push((tc.clone(), r.output.clone()));
        }
        Ok(out)
    }
}

/// 编排层契约：维护连续无提升计数（只写 extensions；Layer 4 只读使用）。
///
/// - best_is_better=true → 重置为 0
/// - best_is_better=false → +1
pub fn update_consecutive_no_improvement(
    ctx: &mut OptimizationContext,
    best_is_better: bool,
) -> u32 {
    let prev = ctx
        .extensions
        .get(EXT_CONSECUTIVE_NO_IMPROVEMENT)
        .and_then(|v| v.as_u64())
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(0);

    let next = if best_is_better {
        0
    } else {
        prev.saturating_add(1)
    };

    ctx.extensions.insert(
        EXT_CONSECUTIVE_NO_IMPROVEMENT.to_string(),
        serde_json::Value::Number(serde_json::Number::from(next as u64)),
    );
    next
}

/// 编排层契约：写入失败档案（Failure Archive）。
///
/// 写入时机：某个候选 Prompt 完成评估后，对所有失败用例生成条目并写入：
/// - key: `EXT_FAILURE_ARCHIVE`
/// - 去重键：`(failure_fingerprint, test_case_id)`
/// - 上限策略：全局上限 `FAILURE_ARCHIVE_MAX_ENTRIES`；超限 FIFO 丢弃最旧
pub fn append_failure_archive_from_evaluations(
    ctx: &mut OptimizationContext,
    prompt: &str,
    evaluations_by_test_case_id: &HashMap<String, EvaluationResult>,
) -> Result<usize, OrchestrationError> {
    let mut archive = read_failure_archive(ctx)?;
    let mut existing = std::collections::HashSet::<(String, String)>::new();
    for e in &archive {
        existing.insert((e.failure_fingerprint.clone(), e.test_case_id.clone()));
    }

    let mut appended = 0usize;
    // HashMap iteration order is nondeterministic; sort by test_case_id for stable FIFO behavior.
    let mut ordered: Vec<(&String, &EvaluationResult)> =
        evaluations_by_test_case_id.iter().collect();
    ordered.sort_by(|a, b| a.0.cmp(b.0));
    for (test_case_id, ev) in ordered {
        if ev.passed {
            continue;
        }
        let entry = FailureArchiveEntry::new(prompt, test_case_id.clone(), summarize_failure(ev));
        let key = (
            entry.failure_fingerprint.clone(),
            entry.test_case_id.clone(),
        );
        if existing.insert(key) {
            archive.push(entry);
            appended += 1;
        }
    }

    if archive.len() > FAILURE_ARCHIVE_MAX_ENTRIES {
        let overflow = archive.len() - FAILURE_ARCHIVE_MAX_ENTRIES;
        archive.drain(0..overflow);
    }

    ctx.extensions.insert(
        EXT_FAILURE_ARCHIVE.to_string(),
        serde_json::to_value(&archive)
            .map_err(|e| OrchestrationError::SerializeArchive { source: e })?,
    );

    Ok(appended)
}

fn read_failure_archive(
    ctx: &OptimizationContext,
) -> Result<Vec<FailureArchiveEntry>, OrchestrationError> {
    let Some(v) = ctx.extensions.get(EXT_FAILURE_ARCHIVE) else {
        return Ok(Vec::new());
    };
    serde_json::from_value(v.clone())
        .map_err(|e| OrchestrationError::DeserializeArchive { source: e })
}

fn summarize_failure(ev: &EvaluationResult) -> String {
    if ev.failure_points.is_empty() {
        return "passed=false (no failure_points)".to_string();
    }

    // Hard safety: do NOT include fp.description because it may contain input/output excerpts.
    // Keep only structured, non-raw identifiers for diagnostics.
    let total = ev.failure_points.len();
    let mut top = Vec::new();
    for fp in ev.failure_points.iter().take(5) {
        top.push(format!("{}:{:?}", fp.dimension, fp.severity));
    }
    let mut s = format!("failure_points_total={total}; top=[{}]", top.join(", "));
    if s.chars().count() > 400 {
        s = s.chars().take(400).collect();
    }
    s
}

#[derive(Debug, Error)]
pub enum OrchestrationError {
    #[error("layer4.failure_archive 反序列化失败：{source}")]
    DeserializeArchive {
        #[source]
        source: serde_json::Error,
    },

    #[error("失败档案序列化失败：{source}")]
    SerializeArchive {
        #[source]
        source: serde_json::Error,
    },
}

/// 编排层策略：遇到 `DuplicateCandidate` 时，继续尝试下一个 candidate_index。
///
/// 当候选空间耗尽（0..TEMPLATE_VARIANT_COUNT 全部不可用）时返回 `CandidateSpaceExhausted`，
/// 上层应记录结构化原因并转入 `InjectDiversity`（避免“重复候选→重复失败”死循环）。
pub async fn generate_candidate_with_retry<G: PromptGenerator>(
    generator: &G,
    ctx: &mut OptimizationContext,
    start_candidate_index: u32,
) -> Result<(u32, String), CandidateGenerationError> {
    let mut tried = 0u32;
    for candidate_index in start_candidate_index..TEMPLATE_VARIANT_COUNT {
        ctx.extensions.insert(
            EXT_CANDIDATE_INDEX.to_string(),
            serde_json::Value::Number(serde_json::Number::from(candidate_index as u64)),
        );
        match generator.generate(ctx).await {
            Ok(prompt) => return Ok((candidate_index, prompt)),
            Err(GeneratorError::DuplicateCandidate { .. }) => {
                tried += 1;
                continue;
            }
            Err(other) => return Err(CandidateGenerationError::Generator(other)),
        }
    }
    Err(CandidateGenerationError::CandidateSpaceExhausted { tried })
}

#[derive(Debug, Error)]
pub enum CandidateGenerationError {
    #[error("候选空间耗尽（reason=candidate_space_exhausted tried={tried}）")]
    CandidateSpaceExhausted { tried: u32 },

    #[error(transparent)]
    Generator(#[from] GeneratorError),
}

#[derive(Debug)]
pub struct FailureArchiveAndDiversityOutcome {
    pub unified_reflection: UnifiedReflection,
    pub candidate_index: Option<u32>,
    pub candidate_prompt: Option<String>,
    pub failure_archive_appended: usize,
    pub consecutive_no_improvement: Option<u32>,
}

#[derive(Debug, Error)]
pub enum IterationOrchestrationError {
    #[error(transparent)]
    CandidateGeneration(#[from] CandidateGenerationError),

    #[error(transparent)]
    Execution(#[from] ExecutionError),

    #[error(transparent)]
    Evaluator(#[from] EvaluatorError),

    #[error(transparent)]
    Archive(#[from] OrchestrationError),

    #[error(transparent)]
    Aggregator(#[from] AggregatorError),

    #[error("无法构造 ReflectionResult：所有测试用例都通过（无需进入反思/聚合阶段）")]
    NoFailuresToReflect,
}

/// 最小“端到端”编排入口（纯内存可测）：
///
/// - 生成候选（含 DuplicateCandidate 重试；耗尽→结构化 reason + InjectDiversity）
/// - 评估候选 + current_prompt，写入失败档案（仅对候选失败用例）
/// - 维护连续无提升计数（只写 extensions；Layer 4 只读）
/// - 调用 FeedbackAggregator 产出 UnifiedReflection（含 InjectDiversity 与结构化 extra）
///
/// 注意：该入口不依赖真实 LLM/网络，适合 CI 下确定性验证。
pub async fn run_failure_archive_and_diversity_injection_step<
    G: PromptGenerator,
    E: Evaluator,
    A: FeedbackAggregator,
>(
    ctx: &mut OptimizationContext,
    iteration_engine: &IterationEngine,
    prompt_generator: &G,
    evaluator: &E,
    feedback_aggregator: &A,
    task_config: &OptimizationTaskConfig,
    start_candidate_index: u32,
) -> Result<FailureArchiveAndDiversityOutcome, IterationOrchestrationError> {
    // Avoid borrow conflicts: `run_tests` needs `&mut ctx`, so we clone the batch once per step.
    let batch = ctx.test_cases.clone();

    let (candidate_index, candidate_prompt) =
        match generate_candidate_with_retry(prompt_generator, ctx, start_candidate_index).await {
            Ok(v) => v,
            Err(CandidateGenerationError::CandidateSpaceExhausted { tried }) => {
                let mut extra = HashMap::new();
                extra.insert(
                    "strategy_reason".to_string(),
                    json!("candidate_space_exhausted"),
                );
                extra.insert(
                    "candidate_space_exhausted".to_string(),
                    json!({
                        "tried": tried,
                        "variant_count": TEMPLATE_VARIANT_COUNT,
                    }),
                );

                let unified = UnifiedReflection {
                    primary_failure_type: FailureType::ExpressionIssue,
                    unified_suggestions: vec![],
                    has_conflicts: false,
                    conflicts: vec![],
                    arbitration_result: None,
                    source_count: 0,
                    failure_type_distribution: HashMap::new(),
                    recommended_action: RecommendedAction::InjectDiversity,
                    extra,
                };

                return Ok(FailureArchiveAndDiversityOutcome {
                    unified_reflection: unified,
                    candidate_index: None,
                    candidate_prompt: None,
                    failure_archive_appended: 0,
                    consecutive_no_improvement: None,
                });
            }
            Err(other) => return Err(IterationOrchestrationError::CandidateGeneration(other)),
        };

    // 评估 current_prompt（用于“无提升”判定口径）
    let current_prompt = ctx.current_prompt.clone();
    let current_exec = iteration_engine
        .run_tests(ctx, &current_prompt, &batch, task_config)
        .await?;
    let current_pairs = IterationEngine::build_evaluation_pairs(&batch, &current_exec)?;
    let current_evals = evaluator.evaluate_batch(ctx, &current_pairs).await?;

    // 评估候选（用于失败档案 + best_candidate_stats）
    let candidate_exec = iteration_engine
        .run_tests(ctx, &candidate_prompt, &batch, task_config)
        .await?;
    let candidate_pairs = IterationEngine::build_evaluation_pairs(&batch, &candidate_exec)?;
    let candidate_evals = evaluator.evaluate_batch(ctx, &candidate_pairs).await?;

    let current_stats = summarize_stats(&current_evals);
    let candidate_stats = summarize_stats(&candidate_evals);

    ctx.extensions.insert(
        EXT_CURRENT_PROMPT_STATS.to_string(),
        serde_json::to_value(current_stats).unwrap_or_default(),
    );
    ctx.extensions.insert(
        EXT_BEST_CANDIDATE_STATS.to_string(),
        serde_json::to_value(candidate_stats).unwrap_or_default(),
    );

    let best_is_better = is_better_stats(candidate_stats, current_stats);
    let consecutive = update_consecutive_no_improvement(ctx, best_is_better);

    // 写入失败档案（仅对候选失败用例）
    let candidate_map = evaluations_map(&batch, &candidate_evals);
    let appended = append_failure_archive_from_evaluations(ctx, &candidate_prompt, &candidate_map)?;

    // 供 Aggregator 使用的逐用例评估映射（避免其依赖外部注入）
    ctx.extensions.insert(
        EXT_EVALUATIONS_BY_TEST_CASE_ID.to_string(),
        serde_json::to_value(&candidate_map).unwrap_or_default(),
    );

    // 进入反思阶段（便于上层观测；Aggregator 本身不依赖 state）
    ctx.state = IterationState::Reflecting;

    let reflections = build_minimal_reflections(&candidate_map)?;
    let unified = feedback_aggregator.aggregate(ctx, &reflections).await?;

    Ok(FailureArchiveAndDiversityOutcome {
        unified_reflection: unified,
        candidate_index: Some(candidate_index),
        candidate_prompt: Some(candidate_prompt),
        failure_archive_appended: appended,
        consecutive_no_improvement: Some(consecutive),
    })
}

fn summarize_stats(evals: &[EvaluationResult]) -> CandidateStats {
    if evals.is_empty() {
        return CandidateStats {
            pass_rate: 0.0,
            mean_score: 0.0,
        };
    }
    let total = evals.len() as f64;
    let passed = evals.iter().filter(|e| e.passed).count() as f64;
    let sum_score = evals.iter().map(|e| e.score).sum::<f64>();
    CandidateStats {
        pass_rate: (passed / total).clamp(0.0, 1.0),
        mean_score: (sum_score / total).clamp(0.0, 1.0),
    }
}

fn is_better_stats(best: CandidateStats, current: CandidateStats) -> bool {
    if best.pass_rate > current.pass_rate + METRIC_EPS {
        return true;
    }
    (best.pass_rate - current.pass_rate).abs() <= METRIC_EPS
        && best.mean_score > current.mean_score + METRIC_EPS
}

fn evaluations_map(
    batch: &[TestCase],
    evals: &[EvaluationResult],
) -> HashMap<String, EvaluationResult> {
    let mut out = HashMap::new();
    for (tc, ev) in batch.iter().zip(evals.iter()) {
        out.insert(tc.id.clone(), ev.clone());
    }
    out
}

fn build_minimal_reflections(
    evaluations_by_test_case_id: &HashMap<String, EvaluationResult>,
) -> Result<Vec<ReflectionResult>, IterationOrchestrationError> {
    let mut failed = Vec::new();
    for (id, ev) in evaluations_by_test_case_id {
        if !ev.passed {
            failed.push(id.clone());
        }
    }
    failed.sort();
    failed.dedup();

    if failed.is_empty() {
        return Err(IterationOrchestrationError::NoFailuresToReflect);
    }

    // Minimal deterministic reflection (no prompt/testcase input raw text).
    let rr = ReflectionResult {
        failure_type: FailureType::ExpressionIssue,
        analysis: "auto-generated reflection placeholder (deterministic)".to_string(),
        root_cause: "candidate did not improve under pass_rate/mean_score ordering".to_string(),
        suggestions: vec![Suggestion {
            suggestion_type: SuggestionType::Rephrase,
            content: "rephrase to address failed dimensions".to_string(),
            confidence: 0.9,
            expected_impact: Some(failed.len() as u32),
        }],
        failed_test_case_ids: failed,
        related_rule_ids: vec![],
        evaluation_ref: None,
        extra: HashMap::new(),
    };
    Ok(vec![rr])
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::core::feedback_aggregator::DefaultFeedbackAggregator;
    use crate::core::prompt_generator::DefaultPromptGenerator;
    use crate::core::traits::Evaluator as EvaluatorTrait;
    use crate::domain::models::TaskReference;
    use crate::domain::models::{FailurePoint, Severity};
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig};

    #[derive(Debug)]
    struct InFlightTarget {
        current_in_flight: AtomicUsize,
        max_in_flight: AtomicUsize,
        per_call_latency_ms: u64,
    }

    impl InFlightTarget {
        fn new(per_call_latency_ms: u64) -> Self {
            Self {
                current_in_flight: AtomicUsize::new(0),
                max_in_flight: AtomicUsize::new(0),
                per_call_latency_ms,
            }
        }

        fn max_in_flight(&self) -> usize {
            self.max_in_flight.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl ExecutionTarget for InFlightTarget {
        async fn execute(
            &self,
            _execution_target_config: &ExecutionTargetConfig,
            _prompt: &str,
            _input: &HashMap<String, serde_json::Value>,
            test_case_id: &str,
        ) -> Result<ExecutionResult, ExecutionError> {
            let in_flight = self.current_in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_in_flight.fetch_max(in_flight, Ordering::SeqCst);

            tokio::time::sleep(tokio::time::Duration::from_millis(self.per_call_latency_ms)).await;
            self.current_in_flight.fetch_sub(1, Ordering::SeqCst);

            Ok(ExecutionResult {
                test_case_id: test_case_id.to_string(),
                output: "ok".to_string(),
                latency_ms: self.per_call_latency_ms,
                token_usage: None,
                raw_response: None,
            })
        }

        fn name(&self) -> &str {
            "in_flight"
        }
    }

    #[derive(Debug)]
    struct WrongIdTarget;

    #[async_trait::async_trait]
    impl ExecutionTarget for WrongIdTarget {
        async fn execute(
            &self,
            _execution_target_config: &ExecutionTargetConfig,
            _prompt: &str,
            _input: &HashMap<String, serde_json::Value>,
            _test_case_id: &str,
        ) -> Result<ExecutionResult, ExecutionError> {
            Ok(ExecutionResult {
                test_case_id: "WRONG".to_string(),
                output: "ok".to_string(),
                latency_ms: 0,
                token_usage: None,
                raw_response: None,
            })
        }

        fn name(&self) -> &str {
            "wrong_id"
        }
    }

    fn test_case(id: &str) -> TestCase {
        TestCase {
            id: id.to_string(),
            input: HashMap::new(),
            reference: TaskReference::Exact {
                expected: "x".to_string(),
            },
            split: None,
            metadata: None,
        }
    }

    fn base_ctx(test_cases: Vec<TestCase>) -> OptimizationContext {
        OptimizationContext {
            task_id: "t".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "p".to_string(),
            rule_system: crate::domain::models::RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 1,
            },
            iteration: 1,
            state: IterationState::Idle,
            run_control_state: Default::default(),
            test_cases,
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn run_tests_serial_does_not_overlap() {
        let target = Arc::new(InFlightTarget::new(10));
        let engine = IterationEngine::new(target.clone());

        let batch = (0..10)
            .map(|i| test_case(&format!("tc-{i}")))
            .collect::<Vec<_>>();
        let mut ctx = base_ctx(batch.clone());

        let config = OptimizationTaskConfig {
            execution_mode: ExecutionMode::Serial,
            max_concurrency: 8,
            ..OptimizationTaskConfig::default()
        };

        let _ = engine
            .run_tests(&mut ctx, "p", &batch, &config)
            .await
            .unwrap();
        assert_eq!(ctx.state, IterationState::RunningTests);
        assert_eq!(target.max_in_flight(), 1);
    }

    #[tokio::test]
    async fn run_tests_parallel_respects_max_concurrency() {
        let target = Arc::new(InFlightTarget::new(20));
        let engine = IterationEngine::new(target.clone());

        let batch = (0..20)
            .map(|i| test_case(&format!("tc-{i}")))
            .collect::<Vec<_>>();
        let mut ctx = base_ctx(batch.clone());

        let config = OptimizationTaskConfig {
            execution_mode: ExecutionMode::Parallel,
            max_concurrency: 4,
            ..OptimizationTaskConfig::default()
        };

        let _ = engine
            .run_tests(&mut ctx, "p", &batch, &config)
            .await
            .unwrap();
        let max_seen = target.max_in_flight();
        assert!(max_seen <= 4, "max_seen={max_seen}");
        assert!(max_seen >= 2, "max_seen={max_seen}"); // sanity: should overlap
    }

    #[tokio::test]
    async fn run_tests_rejects_mismatched_test_case_id() {
        let target = Arc::new(WrongIdTarget);
        let engine = IterationEngine::new(target);

        let batch = vec![test_case("a")];
        let mut ctx = base_ctx(batch.clone());
        let config = OptimizationTaskConfig::default();

        let err = engine
            .run_tests(&mut ctx, "p", &batch, &config)
            .await
            .unwrap_err();
        assert!(matches!(err, ExecutionError::Internal { .. }));
        assert!(err.to_string().contains("mismatch"));
    }

    #[test]
    fn build_evaluation_pairs_preserves_order_and_validates_alignment() {
        let batch = vec![test_case("a"), test_case("b")];
        let exec_results = vec![
            ExecutionResult {
                test_case_id: "a".to_string(),
                output: "oa".to_string(),
                latency_ms: 1,
                token_usage: None,
                raw_response: None,
            },
            ExecutionResult {
                test_case_id: "b".to_string(),
                output: "ob".to_string(),
                latency_ms: 1,
                token_usage: None,
                raw_response: None,
            },
        ];

        let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results).unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0.id, "a");
        assert_eq!(pairs[0].1, "oa");
        assert_eq!(pairs[1].0.id, "b");
        assert_eq!(pairs[1].1, "ob");
    }

    #[test]
    fn consecutive_no_improvement_counter_resets_and_increments() {
        let mut ctx = base_ctx(vec![]);
        assert_eq!(update_consecutive_no_improvement(&mut ctx, false), 1);
        assert_eq!(update_consecutive_no_improvement(&mut ctx, false), 2);
        assert_eq!(update_consecutive_no_improvement(&mut ctx, true), 0);
        assert_eq!(update_consecutive_no_improvement(&mut ctx, false), 1);
    }

    #[test]
    fn failure_archive_is_deduped_by_fingerprint_and_test_case_id_and_is_bounded_fifo() {
        let mut ctx = base_ctx(vec![]);

        let mut evals = HashMap::new();
        evals.insert(
            "tc1".to_string(),
            EvaluationResult {
                passed: false,
                score: 0.0,
                dimensions: HashMap::new(),
                failure_points: vec![crate::domain::models::FailurePoint {
                    dimension: "format".to_string(),
                    description: "bad".to_string(),
                    severity: crate::domain::models::Severity::Major,
                    expected: None,
                    actual: None,
                }],
                evaluator_type: "e".to_string(),
                confidence: None,
                reasoning: None,
                extra: HashMap::new(),
            },
        );

        // same prompt + same testcase should not duplicate
        let p = "prompt v1";
        let a1 = append_failure_archive_from_evaluations(&mut ctx, p, &evals).unwrap();
        let a2 = append_failure_archive_from_evaluations(&mut ctx, p, &evals).unwrap();
        assert_eq!(a1, 1);
        assert_eq!(a2, 0);

        // exceed max entries should drop oldest (FIFO)
        for i in 0..(FAILURE_ARCHIVE_MAX_ENTRIES + 5) {
            let prompt = format!("p-{i}");
            let mut one = HashMap::new();
            one.insert(
                format!("tc-{i}"),
                EvaluationResult {
                    passed: false,
                    score: 0.0,
                    dimensions: HashMap::new(),
                    failure_points: vec![],
                    evaluator_type: "e".to_string(),
                    confidence: None,
                    reasoning: None,
                    extra: HashMap::new(),
                },
            );
            append_failure_archive_from_evaluations(&mut ctx, &prompt, &one).unwrap();
        }

        let archive: Vec<FailureArchiveEntry> =
            serde_json::from_value(ctx.extensions.get(EXT_FAILURE_ARCHIVE).cloned().unwrap())
                .unwrap();
        assert_eq!(archive.len(), FAILURE_ARCHIVE_MAX_ENTRIES);

        // oldest 5 should be dropped
        assert!(archive.iter().all(|e| e.test_case_id != "tc-0"));
        assert!(archive.iter().all(|e| e.test_case_id != "tc-1"));
        assert!(archive.iter().all(|e| e.test_case_id != "tc-2"));
        assert!(archive.iter().all(|e| e.test_case_id != "tc-3"));
        assert!(archive.iter().all(|e| e.test_case_id != "tc-4"));
    }

    #[tokio::test]
    async fn generate_candidate_with_retry_exhausts_when_all_variants_hit_failure_archive() {
        use crate::core::prompt_generator::DefaultPromptGenerator;
        use crate::domain::models::{OutputLength, Rule, RuleSystem, RuleTags};
        use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig};
        use serde_json::json;

        fn rule_with_polarity(id: &str, polarity: &str) -> Rule {
            let mut extra = HashMap::new();
            extra.insert("polarity".to_string(), json!(polarity));
            Rule {
                id: id.to_string(),
                description: format!("{polarity} rule"),
                tags: RuleTags {
                    output_format: vec![],
                    output_structure: vec![],
                    output_length: OutputLength::Flexible,
                    semantic_focus: vec![],
                    key_concepts: vec![],
                    must_include: vec![],
                    must_exclude: vec![],
                    tone: None,
                    extra,
                },
                source_test_cases: vec!["tc1".to_string()],
                abstraction_level: 0,
                parent_rules: vec![],
                verified: false,
                verification_score: 0.0,
                ir: None,
            }
        }

        let generator = DefaultPromptGenerator::new();
        let mut ctx = OptimizationContext {
            task_id: "task-1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: "prompt".to_string(),
            rule_system: RuleSystem {
                rules: vec![
                    rule_with_polarity("r1", "success"),
                    rule_with_polarity("r2", "failure"),
                ],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 0,
            },
            iteration: 0,
            state: crate::domain::models::IterationState::Idle,
            run_control_state: Default::default(),
            test_cases: vec![],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        };

        // prefill archive with all variant fingerprints so every generate() hits DuplicateCandidate
        let mut archive = Vec::new();
        for i in 0..TEMPLATE_VARIANT_COUNT {
            ctx.extensions
                .insert(EXT_CANDIDATE_INDEX.to_string(), json!(i));
            let p = generator.generate(&ctx).await.unwrap();
            archive.push(FailureArchiveEntry::new(&p, format!("tc-{i}"), "x"));
        }
        ctx.extensions.insert(
            EXT_FAILURE_ARCHIVE.to_string(),
            serde_json::to_value(archive).unwrap(),
        );

        let err = generate_candidate_with_retry(&generator, &mut ctx, 0)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            CandidateGenerationError::CandidateSpaceExhausted { .. }
        ));
    }

    #[derive(Debug)]
    struct EchoTarget;

    #[async_trait::async_trait]
    impl ExecutionTarget for EchoTarget {
        async fn execute(
            &self,
            _execution_target_config: &ExecutionTargetConfig,
            prompt: &str,
            _input: &HashMap<String, serde_json::Value>,
            test_case_id: &str,
        ) -> Result<ExecutionResult, ExecutionError> {
            Ok(ExecutionResult {
                test_case_id: test_case_id.to_string(),
                output: prompt.to_string(),
                latency_ms: 0,
                token_usage: None,
                raw_response: None,
            })
        }

        fn name(&self) -> &str {
            "echo"
        }
    }

    #[derive(Debug)]
    struct DeterministicEvaluator;

    #[async_trait::async_trait]
    impl EvaluatorTrait for DeterministicEvaluator {
        async fn evaluate(
            &self,
            _ctx: &OptimizationContext,
            _test_case: &TestCase,
            output: &str,
        ) -> Result<EvaluationResult, EvaluatorError> {
            if output.contains("CANDIDATE_FAIL") {
                return Ok(EvaluationResult {
                    passed: false,
                    score: 0.0,
                    dimensions: HashMap::new(),
                    failure_points: vec![FailurePoint {
                        dimension: "format".to_string(),
                        description: "SENSITIVE_INPUT=TOPSECRET".to_string(),
                        severity: Severity::Major,
                        expected: None,
                        actual: None,
                    }],
                    evaluator_type: "deterministic".to_string(),
                    confidence: Some(0.9),
                    reasoning: None,
                    extra: HashMap::new(),
                });
            }
            Ok(EvaluationResult {
                passed: true,
                score: 1.0,
                dimensions: HashMap::new(),
                failure_points: vec![],
                evaluator_type: "deterministic".to_string(),
                confidence: Some(0.9),
                reasoning: None,
                extra: HashMap::new(),
            })
        }

        async fn evaluate_batch(
            &self,
            ctx: &OptimizationContext,
            results: &[(TestCase, String)],
        ) -> Result<Vec<EvaluationResult>, EvaluatorError> {
            let mut out = Vec::with_capacity(results.len());
            for (tc, output) in results {
                out.push(self.evaluate(ctx, tc, output).await?);
            }
            Ok(out)
        }

        fn name(&self) -> &str {
            "deterministic"
        }
    }

    #[tokio::test]
    async fn orchestration_exhausted_candidate_space_maps_to_inject_diversity_with_reason() {
        let generator = DefaultPromptGenerator::new();
        let target = Arc::new(EchoTarget);
        let engine = IterationEngine::new(target);
        let evaluator = DeterministicEvaluator;
        let agg = DefaultFeedbackAggregator;

        // make a ctx that triggers DefaultPromptGenerator non-initial prompt path
        let mut ctx = base_ctx(vec![test_case("tc1")]);
        ctx.current_prompt = "prompt".to_string();
        ctx.rule_system.rules = vec![crate::domain::models::Rule {
            id: "r1".to_string(),
            description: "success rule".to_string(),
            tags: crate::domain::models::RuleTags {
                output_format: vec![],
                output_structure: vec![],
                output_length: crate::domain::models::OutputLength::Flexible,
                semantic_focus: vec![],
                key_concepts: vec![],
                must_include: vec![],
                must_exclude: vec![],
                tone: None,
                extra: {
                    let mut m = HashMap::new();
                    m.insert("polarity".to_string(), json!("success"));
                    m
                },
            },
            source_test_cases: vec!["tc1".to_string()],
            abstraction_level: 0,
            parent_rules: vec![],
            verified: false,
            verification_score: 0.0,
            ir: None,
        }];
        ctx.rule_system.rules.push(crate::domain::models::Rule {
            id: "r2".to_string(),
            description: "failure rule".to_string(),
            tags: crate::domain::models::RuleTags {
                output_format: vec![],
                output_structure: vec![],
                output_length: crate::domain::models::OutputLength::Flexible,
                semantic_focus: vec![],
                key_concepts: vec![],
                must_include: vec![],
                must_exclude: vec![],
                tone: None,
                extra: {
                    let mut m = HashMap::new();
                    m.insert("polarity".to_string(), json!("failure"));
                    m
                },
            },
            source_test_cases: vec!["tc1".to_string()],
            abstraction_level: 0,
            parent_rules: vec![],
            verified: false,
            verification_score: 0.0,
            ir: None,
        });

        // prefill archive with all variant fingerprints so every generate() hits DuplicateCandidate
        let mut archive = Vec::new();
        for i in 0..TEMPLATE_VARIANT_COUNT {
            ctx.extensions
                .insert(EXT_CANDIDATE_INDEX.to_string(), json!(i));
            let p = generator.generate(&ctx).await.unwrap();
            archive.push(FailureArchiveEntry::new(&p, format!("tc-{i}"), "x"));
        }
        ctx.extensions.insert(
            EXT_FAILURE_ARCHIVE.to_string(),
            serde_json::to_value(archive).unwrap(),
        );

        let mut task_config = OptimizationTaskConfig::default();
        task_config.execution_mode = ExecutionMode::Serial;
        task_config.max_concurrency = 1;

        let out = run_failure_archive_and_diversity_injection_step(
            &mut ctx,
            &engine,
            &generator,
            &evaluator,
            &agg,
            &task_config,
            0,
        )
        .await
        .unwrap();

        assert!(matches!(
            out.unified_reflection.recommended_action,
            RecommendedAction::InjectDiversity
        ));
        assert_eq!(
            out.unified_reflection
                .extra
                .get("strategy_reason")
                .and_then(|v| v.as_str()),
            Some("candidate_space_exhausted")
        );
    }

    #[tokio::test]
    async fn orchestration_writes_failure_archive_updates_consecutive_and_triggers_inject_diversity()
     {
        #[derive(Debug)]
        struct FixedGenerator;

        #[async_trait::async_trait]
        impl PromptGenerator for FixedGenerator {
            async fn generate(&self, _ctx: &OptimizationContext) -> Result<String, GeneratorError> {
                Ok("CANDIDATE_FAIL Bearer sk-abcdefghijklmnopqrstuvwxyz0123456789".to_string())
            }

            fn name(&self) -> &str {
                "fixed"
            }
        }

        let generator = FixedGenerator;
        let target = Arc::new(EchoTarget);
        let engine = IterationEngine::new(target);
        let evaluator = DeterministicEvaluator;
        let agg = DefaultFeedbackAggregator;

        let mut ctx = base_ctx(vec![test_case("tc1"), test_case("tc2")]);
        ctx.current_prompt = "CURRENT_OK".to_string();
        ctx.config.iteration.diversity_inject_after = 2;
        ctx.extensions
            .insert(EXT_CONSECUTIVE_NO_IMPROVEMENT.to_string(), json!(1));

        let mut task_config = OptimizationTaskConfig::default();
        task_config.execution_mode = ExecutionMode::Serial;
        task_config.max_concurrency = 1;

        let out = run_failure_archive_and_diversity_injection_step(
            &mut ctx,
            &engine,
            &generator,
            &evaluator,
            &agg,
            &task_config,
            0,
        )
        .await
        .unwrap();

        // AC2: 连续无提升达到阈值 → InjectDiversity，并写入结构化依据（extra）
        assert!(matches!(
            out.unified_reflection.recommended_action,
            RecommendedAction::InjectDiversity
        ));
        assert_eq!(
            out.unified_reflection
                .extra
                .get("strategy_reason")
                .and_then(|v| v.as_str()),
            Some("no_improvement_and_consecutive_threshold_reached")
        );
        let gate = out
            .unified_reflection
            .extra
            .get("diversity_injection_gate")
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            gate.get("source").and_then(|v| v.as_str()),
            Some("extensions")
        );
        assert_eq!(gate.get("threshold").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(gate.get("current").and_then(|v| v.as_u64()), Some(2));

        // AC1: 失败档案写入（仅候选失败用例），且不泄露 failure_points.description 原文
        assert!(out.failure_archive_appended > 0);
        let archive: Vec<FailureArchiveEntry> =
            serde_json::from_value(ctx.extensions.get(EXT_FAILURE_ARCHIVE).cloned().unwrap())
                .unwrap();
        assert!(archive.iter().any(|e| e.test_case_id == "tc1"));
        assert!(archive.iter().any(|e| e.test_case_id == "tc2"));
        for e in &archive {
            assert!(e.prompt_excerpt.len() <= 200);
            assert!(!e.failure_reason.contains("SENSITIVE_INPUT=TOPSECRET"));
            assert!(
                !e.prompt_excerpt
                    .contains("sk-abcdefghijklmnopqrstuvwxyz0123456789")
            );
        }
    }
}
