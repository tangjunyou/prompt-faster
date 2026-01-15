#[cfg(feature = "alt-optimization-engine")]
mod alternate_impl;
#[cfg_attr(feature = "alt-optimization-engine", allow(dead_code))]
mod default_impl;
mod error;

use std::sync::Arc;

use async_trait::async_trait;

use crate::core::evaluator::create_evaluator_for_task_config;
use crate::core::execution_target::create_execution_target;
use crate::core::feedback_aggregator::DefaultFeedbackAggregator;
use crate::core::optimizer::DefaultOptimizer;
use crate::core::rule_engine::DefaultRuleEngine;
use crate::core::teacher_model::{TeacherModelType, create_teacher_model};
use crate::domain::models::{
    Checkpoint, ExecutionTargetType, OptimizationResult, OptimizationTaskConfig,
};
use crate::domain::types::OptimizationContext;

pub use error::OptimizationEngineError;

#[async_trait]
pub trait OptimizationEngine: Send + Sync {
    async fn run(
        &self,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError>;

    async fn resume(
        &self,
        checkpoint: Checkpoint,
        ctx: &mut OptimizationContext,
    ) -> Result<OptimizationResult, OptimizationEngineError>;

    fn name(&self) -> &str;
}

/// OptimizationEngine 工厂：算法替换的**单一入口点**（feature gate）。
pub fn create_optimization_engine(
    execution_target_type: ExecutionTargetType,
    task_config: OptimizationTaskConfig,
) -> Arc<dyn OptimizationEngine> {
    // 默认使用确定性 TeacherModel，保证本地/CI 可复现（不出网）。
    let teacher_model = create_teacher_model(TeacherModelType::Example);
    let execution_target = create_execution_target(execution_target_type);
    let evaluator =
        create_evaluator_for_task_config(&task_config, Some(Arc::clone(&teacher_model)));

    #[cfg(feature = "alt-optimization-engine")]
    return Arc::new(alternate_impl::AlternateOptimizationEngine::new(
        alternate_impl::AlternateOptimizationEngineParts {
            rule_engine: Arc::new(DefaultRuleEngine::new()),
            evaluator,
            feedback_aggregator: Arc::new(DefaultFeedbackAggregator),
            optimizer: Arc::new(DefaultOptimizer),
            teacher_model,
            execution_target,
            task_config,
        },
    ));

    #[cfg(not(feature = "alt-optimization-engine"))]
    {
        use crate::core::prompt_generator::DefaultPromptGenerator;
        Arc::new(default_impl::DefaultOptimizationEngine::new(
            default_impl::DefaultOptimizationEngineParts {
                rule_engine: Arc::new(DefaultRuleEngine::new()),
                prompt_generator: Arc::new(DefaultPromptGenerator::new()),
                evaluator,
                feedback_aggregator: Arc::new(DefaultFeedbackAggregator),
                optimizer: Arc::new(DefaultOptimizer),
                teacher_model,
                execution_target,
                task_config,
            },
        ))
    }
}
