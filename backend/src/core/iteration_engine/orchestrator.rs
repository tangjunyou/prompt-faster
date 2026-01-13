use std::sync::Arc;

use crate::core::execution_target::ExecutionError;
use crate::core::iteration_engine::executor::{parallel_execute, serial_execute};
use crate::core::traits::ExecutionTarget;
use crate::domain::models::{
    ExecutionMode, ExecutionResult, IterationState, OptimizationTaskConfig, TestCase,
};
use crate::domain::types::OptimizationContext;

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
                serial_execute(self.execution_target.as_ref(), prompt, batch).await?
            }
            ExecutionMode::Parallel => {
                parallel_execute(
                    Arc::clone(&self.execution_target),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

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
            reference: crate::domain::models::TaskReference::Exact {
                expected: "x".to_string(),
            },
            split: None,
            metadata: None,
        }
    }

    fn base_ctx(test_cases: Vec<TestCase>) -> OptimizationContext {
        OptimizationContext {
            task_id: "t".to_string(),
            execution_target_config: crate::domain::types::ExecutionTargetConfig::default(),
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
            test_cases,
            config: crate::domain::types::OptimizationConfig::default(),
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
}
