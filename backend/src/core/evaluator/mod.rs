mod default_impl;
mod error;

pub use default_impl::{
    CandidateRankingEntry, DefaultEvaluator, EvaluationStats, SplitFilter,
    build_evaluations_by_test_case_id, rank_candidates, split_filter_for_stats,
    summarize_for_stats,
};
pub use error::EvaluatorError;

/// `ctx.extensions` 中用于注入“任务级评估器选择配置”的 key。
///
/// - 类型：`domain::models::EvaluatorConfig`（序列化为 JSON）
/// - 写入方：编排层（Orchestrator）
/// - 何时必需：每次调用 `Evaluator.evaluate()` / `Evaluator.evaluate_batch()`
pub const EXT_TASK_EVALUATOR_CONFIG: &str = "task_evaluator_config";

pub const EXTRA_SELECTED_EVALUATORS: &str = "selected_evaluators";
pub const EXTRA_THRESHOLDS: &str = "thresholds";
pub const EXTRA_EVALUATOR_FALLBACK_REASON: &str = "evaluator_fallback_reason";
