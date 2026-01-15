mod dify_impl;
mod direct_api_impl;
mod error;
mod example_impl;

use std::sync::Arc;

use crate::core::traits::ExecutionTarget;
use crate::domain::models::ExecutionTargetType;

pub use dify_impl::DifyExecutionTarget;
pub use direct_api_impl::DirectApiExecutionTarget;
pub use error::ExecutionError;
pub use example_impl::ExampleExecutionTarget;

pub fn create_execution_target(
    execution_target_type: ExecutionTargetType,
) -> Arc<dyn ExecutionTarget> {
    match execution_target_type {
        ExecutionTargetType::Dify => Arc::new(DifyExecutionTarget::new()),
        ExecutionTargetType::Generic => Arc::new(DirectApiExecutionTarget::new()),
        ExecutionTargetType::Example => Arc::new(ExampleExecutionTarget::new()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::core::evaluator::{create_evaluator_for_task_config, SplitFilter, summarize_for_stats};
    use crate::core::iteration_engine::orchestrator::IterationEngine;
    use crate::domain::models::{
        EvaluatorType, ExecutionMode, ExecutionTargetType, OptimizationTaskConfig, TaskReference, TestCase,
    };
    use crate::domain::types::{ExecutionTargetConfig, OptimizationConfig, OptimizationContext};

    #[tokio::test]
    async fn example_execution_target_can_run_minimal_end_to_end_loop() {
        let prompt = "p";
        let prompt_len = prompt.chars().count();

        let mut input = HashMap::new();
        input.insert("x".to_string(), serde_json::Value::String("y".to_string()));

        let tc1 = TestCase {
            id: "tc-1".to_string(),
            input: input.clone(),
            reference: TaskReference::Exact {
                expected: format!(
                    "example_execution_target: test_case_id=tc-1 prompt_len={prompt_len} input_keys_count=1"
                ),
            },
            split: None,
            metadata: None,
        };
        let tc2 = TestCase {
            id: "tc-2".to_string(),
            input,
            reference: TaskReference::Exact {
                expected: format!(
                    "example_execution_target: test_case_id=tc-2 prompt_len={prompt_len} input_keys_count=1"
                ),
            },
            split: None,
            metadata: None,
        };

        let mut ctx = OptimizationContext {
            task_id: "t1".to_string(),
            execution_target_config: ExecutionTargetConfig::default(),
            current_prompt: prompt.to_string(),
            rule_system: crate::domain::models::RuleSystem {
                rules: vec![],
                conflict_resolution_log: vec![],
                merge_log: vec![],
                coverage_map: HashMap::new(),
                version: 1,
            },
            iteration: 0,
            state: crate::domain::models::IterationState::RunningTests,
            test_cases: vec![tc1.clone(), tc2.clone()],
            config: OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        };

        let task_config = OptimizationTaskConfig {
            execution_mode: ExecutionMode::Serial,
            ..OptimizationTaskConfig::default()
        };
        let mut task_config = task_config;
        task_config.evaluator_config.evaluator_type = EvaluatorType::Example;

        let target = create_execution_target(ExecutionTargetType::Example);
        let engine = IterationEngine::new(target);
        let batch = ctx.test_cases.clone();
        let exec_results = engine
            .run_tests(&mut ctx, prompt, &batch, &task_config)
            .await
            .unwrap();

        let pairs = IterationEngine::build_evaluation_pairs(&batch, &exec_results).unwrap();

        let evaluator = create_evaluator_for_task_config(&task_config, None);
        let evals = evaluator.evaluate_batch(&ctx, &pairs).await.unwrap();

        let stats = summarize_for_stats(SplitFilter::All, &pairs, &evals).unwrap();
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.passed_count, 2);
        assert_eq!(stats.pass_rate, 1.0);
    }
}
