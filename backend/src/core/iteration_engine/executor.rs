use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::core::execution_target::ExecutionError;
use crate::core::traits::ExecutionTarget;
use crate::domain::models::{ExecutionResult, TestCase};
use crate::domain::types::ExecutionTargetConfig;

pub async fn serial_execute(
    execution_target: &dyn ExecutionTarget,
    execution_target_config: &ExecutionTargetConfig,
    prompt: &str,
    batch: &[TestCase],
) -> Result<Vec<ExecutionResult>, ExecutionError> {
    let mut results = Vec::with_capacity(batch.len());
    for test_case in batch {
        results.push(
            execution_target
                .execute(
                    execution_target_config,
                    prompt,
                    &test_case.input,
                    &test_case.id,
                )
                .await?,
        );
    }
    Ok(results)
}

pub async fn parallel_execute(
    execution_target: Arc<dyn ExecutionTarget>,
    execution_target_config: &ExecutionTargetConfig,
    prompt: &str,
    batch: &[TestCase],
    max_concurrency: u32,
) -> Result<Vec<ExecutionResult>, ExecutionError> {
    if max_concurrency < 1 {
        return Err(ExecutionError::InvalidRequest {
            test_case_id: "unknown".to_string(),
            message: "max_concurrency must be >= 1".to_string(),
        });
    }
    let max_concurrency = max_concurrency as usize;
    if batch.is_empty() {
        return Ok(vec![]);
    }

    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let prompt = Arc::new(prompt.to_string());
    let execution_target_config = Arc::new(execution_target_config.clone());

    let mut join_set = JoinSet::new();
    for (index, test_case) in batch.iter().enumerate() {
        let execution_target = Arc::clone(&execution_target);
        let semaphore = Arc::clone(&semaphore);
        let prompt = Arc::clone(&prompt);
        let execution_target_config = Arc::clone(&execution_target_config);
        let input: HashMap<String, serde_json::Value> = test_case.input.clone();
        let test_case_id = test_case.id.clone();

        join_set.spawn(async move {
            let _permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => {
                    return (
                        index,
                        Err(ExecutionError::Internal {
                            test_case_id: test_case_id.clone(),
                            message: "semaphore closed".to_string(),
                        }),
                    );
                }
            };

            let result = execution_target
                .execute(&execution_target_config, &prompt, &input, &test_case_id)
                .await;
            (index, result)
        });
    }

    let mut out: Vec<Option<ExecutionResult>> = vec![None; batch.len()];
    while let Some(joined) = join_set.join_next().await {
        let (index, result) = joined.map_err(|e| ExecutionError::Internal {
            test_case_id: "unknown".to_string(),
            message: format!("join error: {e}"),
        })?;

        match result {
            Ok(exec_result) => {
                out[index] = Some(exec_result);
            }
            Err(err) => {
                join_set.abort_all();
                while join_set.join_next().await.is_some() {}
                return Err(err);
            }
        }
    }

    out.into_iter()
        .map(|v| {
            v.ok_or_else(|| ExecutionError::Internal {
                test_case_id: "unknown".to_string(),
                message: "missing execution result".to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::time::{Duration, Instant};

    use super::*;
    use crate::core::traits::ExecutionTarget;

    #[derive(Debug)]
    struct MockExecutionTarget {
        current_in_flight: AtomicUsize,
        max_in_flight: AtomicUsize,
        fail_test_case_id: Option<String>,
        per_call_latency_ms: u64,
    }

    impl MockExecutionTarget {
        fn new(per_call_latency_ms: u64) -> Self {
            Self {
                current_in_flight: AtomicUsize::new(0),
                max_in_flight: AtomicUsize::new(0),
                fail_test_case_id: None,
                per_call_latency_ms,
            }
        }

        fn with_failure(mut self, test_case_id: &str) -> Self {
            self.fail_test_case_id = Some(test_case_id.to_string());
            self
        }
    }

    #[async_trait::async_trait]
    impl ExecutionTarget for MockExecutionTarget {
        async fn execute(
            &self,
            _execution_target_config: &crate::domain::types::ExecutionTargetConfig,
            prompt: &str,
            input: &HashMap<String, serde_json::Value>,
            test_case_id: &str,
        ) -> Result<ExecutionResult, ExecutionError> {
            let in_flight = self.current_in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_in_flight.fetch_max(in_flight, Ordering::SeqCst);

            let start = Instant::now();
            tokio::time::sleep(Duration::from_millis(self.per_call_latency_ms)).await;

            self.current_in_flight.fetch_sub(1, Ordering::SeqCst);

            if self
                .fail_test_case_id
                .as_ref()
                .is_some_and(|id| id == test_case_id)
            {
                return Err(ExecutionError::UpstreamError {
                    test_case_id: test_case_id.to_string(),
                    message: "mock upstream error".to_string(),
                });
            }

            Ok(ExecutionResult {
                test_case_id: test_case_id.to_string(),
                output: format!("prompt_len={},input_keys={}", prompt.len(), input.len()),
                latency_ms: start.elapsed().as_millis() as u64,
                token_usage: None,
                raw_response: None,
            })
        }

        fn name(&self) -> &str {
            "mock"
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

    #[tokio::test]
    async fn serial_execute_preserves_order() {
        let target = MockExecutionTarget::new(1);
        let batch = vec![test_case("a"), test_case("b"), test_case("c")];

        let results = serial_execute(&target, &ExecutionTargetConfig::default(), "hello", &batch)
            .await
            .unwrap();
        let ids: Vec<_> = results.iter().map(|r| r.test_case_id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn parallel_execute_empty_batch_returns_empty() {
        let target = Arc::new(MockExecutionTarget::new(1));
        let batch: Vec<TestCase> = vec![];

        let execution_target: Arc<dyn ExecutionTarget> = target;
        let results = parallel_execute(
            execution_target,
            &ExecutionTargetConfig::default(),
            "p",
            &batch,
            4,
        )
        .await
        .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn parallel_execute_rejects_zero_max_concurrency() {
        let target = Arc::new(MockExecutionTarget::new(1));
        let batch = vec![test_case("a")];

        let execution_target: Arc<dyn ExecutionTarget> = target;
        let err = parallel_execute(
            execution_target,
            &ExecutionTargetConfig::default(),
            "p",
            &batch,
            0,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidRequest { .. }));
    }

    #[tokio::test]
    async fn parallel_execute_preserves_order_when_completions_out_of_order() {
        #[derive(Debug)]
        struct DelayByIdTarget;

        #[async_trait::async_trait]
        impl ExecutionTarget for DelayByIdTarget {
            async fn execute(
                &self,
                _execution_target_config: &crate::domain::types::ExecutionTargetConfig,
                _prompt: &str,
                _input: &HashMap<String, serde_json::Value>,
                test_case_id: &str,
            ) -> Result<ExecutionResult, ExecutionError> {
                // reverse delay so later ids finish sooner
                let delay_ms = match test_case_id {
                    "0" => 30,
                    "1" => 25,
                    "2" => 20,
                    "3" => 15,
                    "4" => 10,
                    _ => 5,
                };
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                Ok(ExecutionResult {
                    test_case_id: test_case_id.to_string(),
                    output: test_case_id.to_string(),
                    latency_ms: delay_ms,
                    token_usage: None,
                    raw_response: None,
                })
            }

            fn name(&self) -> &str {
                "delay_by_id"
            }
        }

        let target = Arc::new(DelayByIdTarget);
        let batch = (0..5)
            .map(|i| test_case(&i.to_string()))
            .collect::<Vec<_>>();

        let results = parallel_execute(target, &ExecutionTargetConfig::default(), "p", &batch, 2)
            .await
            .unwrap();
        let ids: Vec<_> = results.iter().map(|r| r.test_case_id.as_str()).collect();
        assert_eq!(ids, vec!["0", "1", "2", "3", "4"]);
    }

    #[tokio::test]
    async fn parallel_execute_respects_max_concurrency() {
        let target = Arc::new(MockExecutionTarget::new(20));
        let batch = (0..20)
            .map(|i| test_case(&i.to_string()))
            .collect::<Vec<_>>();

        let execution_target: Arc<dyn ExecutionTarget> = target.clone();
        let _ = parallel_execute(
            execution_target,
            &ExecutionTargetConfig::default(),
            "p",
            &batch,
            4,
        )
        .await
        .unwrap();

        let max_seen = target.max_in_flight.load(Ordering::SeqCst);
        assert!(max_seen <= 4, "max_seen={max_seen}");
        assert!(max_seen >= 2, "max_seen={max_seen}"); // sanity: should overlap
    }

    #[tokio::test]
    async fn parallel_execute_allows_max_concurrency_greater_than_batch_size() {
        let target = Arc::new(MockExecutionTarget::new(5));
        let batch = (0..3)
            .map(|i| test_case(&i.to_string()))
            .collect::<Vec<_>>();

        let execution_target: Arc<dyn ExecutionTarget> = target.clone();
        let _ = parallel_execute(
            execution_target,
            &ExecutionTargetConfig::default(),
            "p",
            &batch,
            64,
        )
        .await
        .unwrap();

        let max_seen = target.max_in_flight.load(Ordering::SeqCst);
        assert!(max_seen <= batch.len(), "max_seen={max_seen}");
        assert!(max_seen >= 2, "max_seen={max_seen}"); // sanity: should overlap
    }

    #[tokio::test]
    async fn parallel_execute_is_all_or_nothing_on_error() {
        let target = Arc::new(MockExecutionTarget::new(5).with_failure("b"));
        let batch = vec![test_case("a"), test_case("b"), test_case("c")];

        let execution_target: Arc<dyn ExecutionTarget> = target;
        let err = parallel_execute(
            execution_target,
            &ExecutionTargetConfig::default(),
            "p",
            &batch,
            2,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, ExecutionError::UpstreamError { .. }));
        assert!(err.to_string().contains("test_case_id=b"));
    }
}
