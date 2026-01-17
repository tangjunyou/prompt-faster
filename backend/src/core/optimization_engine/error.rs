use thiserror::Error;

use crate::core::evaluator::EvaluatorError;
use crate::core::execution_target::ExecutionError;
use crate::core::feedback_aggregator::AggregatorError;
use crate::core::iteration_engine::pause_state::PauseStateError;
use crate::core::optimizer::OptimizerError;
use crate::core::prompt_generator::GeneratorError;
use crate::core::rule_engine::RuleEngineError;

#[derive(Debug, Error)]
pub enum OptimizationEngineError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    // NOTE: 不在此处拼接 prompt/input 原文；错误信息必须保持脱敏与可诊断之间的平衡。
    #[error("execution failed")]
    Execution(#[from] ExecutionError),

    #[error("evaluation failed")]
    Evaluation(#[from] EvaluatorError),

    #[error("rule engine failed")]
    RuleEngine(#[from] RuleEngineError),

    #[error("prompt generation failed")]
    PromptGenerator(#[from] GeneratorError),

    #[error("feedback aggregation failed")]
    FeedbackAggregator(#[from] AggregatorError),

    #[error("pause state failed")]
    PauseState(#[from] PauseStateError),

    #[error("optimization step failed")]
    Optimizer(#[from] OptimizerError),

    #[error("internal error: {0}")]
    Internal(String),
}
