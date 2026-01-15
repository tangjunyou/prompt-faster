mod default_impl;
mod error;
mod example_impl;

pub use default_impl::{
    CandidateRankingEntry, DefaultEvaluator, EvaluationStats, SplitFilter,
    build_evaluations_by_test_case_id, rank_candidates, split_filter_for_stats,
    summarize_for_stats,
};
pub use error::EvaluatorError;
pub use example_impl::ExampleEvaluator;

use std::sync::Arc;

use crate::core::traits::{Evaluator, TeacherModel};
use crate::domain::models::{EvaluatorType, OptimizationTaskConfig};

/// Evaluator 工厂：根据任务配置选择可用实现（扩展点集中在此处）。
///
/// 注意：若返回 `DefaultEvaluator`，调用方仍需在 `ctx.extensions` 写入
/// `EXT_TASK_EVALUATOR_CONFIG`（`domain::models::EvaluatorConfig`）供其读取。
pub fn create_evaluator_for_task_config(
    task_config: &OptimizationTaskConfig,
    teacher_model: Option<Arc<dyn TeacherModel>>,
) -> Arc<dyn Evaluator> {
    match task_config.evaluator_config.evaluator_type {
        EvaluatorType::Example => Arc::new(ExampleEvaluator::new()),
        _ => Arc::new(DefaultEvaluator::new(teacher_model)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_evaluator_selects_example_implementation() {
        let mut cfg = OptimizationTaskConfig::default();
        cfg.evaluator_config.evaluator_type = EvaluatorType::Example;

        let ev = create_evaluator_for_task_config(&cfg, None);
        assert_eq!(ev.name(), "example_evaluator");
    }
}

/// `ctx.extensions` 中用于注入“任务级评估器选择配置”的 key。
///
/// - 类型：`domain::models::EvaluatorConfig`（序列化为 JSON）
/// - 写入方：编排层（Orchestrator）
/// - 何时必需：每次调用 `Evaluator.evaluate()` / `Evaluator.evaluate_batch()`
pub const EXT_TASK_EVALUATOR_CONFIG: &str = "task_evaluator_config";

pub const EXTRA_SELECTED_EVALUATORS: &str = "selected_evaluators";
pub const EXTRA_THRESHOLDS: &str = "thresholds";
pub const EXTRA_EVALUATOR_FALLBACK_REASON: &str = "evaluator_fallback_reason";
