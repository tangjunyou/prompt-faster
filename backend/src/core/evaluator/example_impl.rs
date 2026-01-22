use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::json;

use crate::core::evaluator::EvaluatorError;
use crate::core::traits::Evaluator;
use crate::domain::models::{
    DimensionScore, EvaluationResult, FailurePoint, Severity, TaskReference, TestCase,
};
use crate::domain::types::OptimizationContext;

fn task_reference_kind(reference: &TaskReference) -> &'static str {
    match reference {
        TaskReference::Exact { .. } => "exact",
        TaskReference::Constrained { .. } => "constrained",
        TaskReference::Hybrid { .. } => "hybrid",
    }
}

/// 示例评估器：用于演示“新增 Evaluator 扩展点”的最小闭环（确定性、不出网）。
///
/// 约束：
/// - 必须同序返回（`evaluate_batch` 不过滤/重排）
/// - 结果可追溯：通过同序契约 + `extra.test_case_id` 做映射
/// - 不回显 output / expected 原文（仅输出长度与结构化摘要）
#[derive(Debug, Default)]
pub struct ExampleEvaluator;

impl ExampleEvaluator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Evaluator for ExampleEvaluator {
    async fn evaluate(
        &self,
        _ctx: &OptimizationContext,
        test_case: &TestCase,
        output: &str,
    ) -> Result<EvaluationResult, EvaluatorError> {
        if test_case.id.trim().is_empty() {
            return Err(EvaluatorError::InvalidInput(
                "test_case.id 不能为空".to_string(),
            ));
        }

        let (passed, score, mut failure_points) = match &test_case.reference {
            TaskReference::Exact { expected } => {
                let passed = output.trim() == expected.trim();
                let score = if passed { 1.0 } else { 0.0 };
                let failure_points = if passed {
                    vec![]
                } else {
                    vec![FailurePoint {
                        dimension: "example_exact_match".to_string(),
                        description: "output 与 expected 不匹配（示例评估器不回显原文）"
                            .to_string(),
                        severity: Severity::Major,
                        expected: None,
                        actual: None,
                    }]
                };
                (passed, score, failure_points)
            }
            other => {
                return Err(EvaluatorError::InvalidInput(format!(
                    "ExampleEvaluator 仅支持 TaskReference::Exact，实际为：{}",
                    task_reference_kind(other)
                )));
            }
        };

        let mut dimensions = HashMap::new();
        dimensions.insert(
            "example_exact_match".to_string(),
            DimensionScore {
                score,
                passed,
                weight: Some(1.0),
                details: None,
            },
        );

        let mut extra = HashMap::new();
        extra.insert("test_case_id".to_string(), json!(test_case.id));
        extra.insert("output_len".to_string(), json!(output.chars().count()));
        extra.insert(
            "failure_points_count".to_string(),
            json!(failure_points.len()),
        );

        Ok(EvaluationResult {
            passed,
            score,
            dimensions,
            failure_points: std::mem::take(&mut failure_points),
            evaluator_type: "example".to_string(),
            confidence: Some(1.0),
            reasoning: Some("deterministic example evaluator".to_string()),
            diversity_analysis: None,
            extra,
        })
    }

    async fn evaluate_batch(
        &self,
        ctx: &OptimizationContext,
        results: &[(TestCase, String)],
    ) -> Result<Vec<EvaluationResult>, EvaluatorError> {
        if results.is_empty() {
            return Err(EvaluatorError::InvalidInput(
                "results 为空（需要至少 1 条 (TestCase, output)）".to_string(),
            ));
        }

        let mut out = Vec::with_capacity(results.len());
        for (tc, output) in results {
            out.push(self.evaluate(ctx, tc, output).await?);
        }
        Ok(out)
    }

    fn name(&self) -> &str {
        "example_evaluator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(test_cases: Vec<TestCase>) -> OptimizationContext {
        let rule_system = crate::domain::models::RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: HashMap::new(),
            version: 1,
        };
        OptimizationContext {
            task_id: "t1".to_string(),
            execution_target_config: crate::domain::types::ExecutionTargetConfig::default(),
            current_prompt: "p".to_string(),
            rule_system,
            iteration: 0,
            state: crate::domain::models::IterationState::RunningTests,
            run_control_state: Default::default(),
            test_cases,
            config: crate::domain::types::OptimizationConfig::default(),
            checkpoints: vec![],
            extensions: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn evaluate_batch_preserves_order_and_is_traceable() {
        let ev = ExampleEvaluator::new();
        let tc_a = TestCase {
            id: "a".to_string(),
            input: HashMap::new(),
            reference: TaskReference::Exact {
                expected: "ok-a".to_string(),
            },
            split: None,
            metadata: None,
        };
        let tc_b = TestCase {
            id: "b".to_string(),
            input: HashMap::new(),
            reference: TaskReference::Exact {
                expected: "ok-b".to_string(),
            },
            split: None,
            metadata: None,
        };

        let ctx = make_ctx(vec![tc_a.clone(), tc_b.clone()]);
        let results = vec![
            (tc_b.clone(), "ok-b".to_string()),
            (tc_a.clone(), "wrong".to_string()),
        ];

        let out = ev.evaluate_batch(&ctx, &results).await.unwrap();
        assert_eq!(out.len(), 2);
        assert!(out[0].passed);
        assert!(!out[1].passed);
        assert_eq!(out[0].extra.get("test_case_id").unwrap(), "b");
        assert_eq!(out[1].extra.get("test_case_id").unwrap(), "a");
    }
}
