use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::json;

use crate::core::execution_target::ExecutionError;
use crate::core::traits::ExecutionTarget;
use crate::domain::models::ExecutionResult;
use crate::domain::types::ExecutionTargetConfig;

/// 示例执行目标：用于演示“新增 ExecutionTarget 扩展点”的最小闭环（确定性、不出网）。
///
/// 设计约束：
/// - 不回显 prompt / input 原文（避免日志/错误/结果泄露敏感信息）
/// - 输出稳定可测（可用于单测与文档示例）
#[derive(Debug, Default)]
pub struct ExampleExecutionTarget;

impl ExampleExecutionTarget {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExecutionTarget for ExampleExecutionTarget {
    async fn execute(
        &self,
        _execution_target_config: &ExecutionTargetConfig,
        prompt: &str,
        input: &HashMap<String, serde_json::Value>,
        test_case_id: &str,
    ) -> Result<ExecutionResult, ExecutionError> {
        let test_case_id = test_case_id.trim();
        if test_case_id.is_empty() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: "unknown".to_string(),
                message: "test_case_id 不能为空".to_string(),
            });
        }

        let prompt_len = u32::try_from(prompt.chars().count()).unwrap_or(u32::MAX);
        let input_keys_count = u32::try_from(input.len()).unwrap_or(u32::MAX);

        // 输出仅包含结构化摘要（避免泄露 prompt/input 原文）。
        let output = format!(
            "example_execution_target: test_case_id={test_case_id} prompt_len={prompt_len} input_keys_count={input_keys_count}"
        );

        Ok(ExecutionResult {
            test_case_id: test_case_id.to_string(),
            output,
            latency_ms: 0,
            token_usage: None,
            raw_response: Some(json!({
                "kind": "example_execution_target",
                "prompt_len": prompt_len,
                "input_keys_count": input_keys_count,
            })),
        })
    }

    fn name(&self) -> &str {
        "example_execution_target"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn example_execution_target_is_deterministic_and_sanitized() {
        let target = ExampleExecutionTarget::new();

        let mut input = HashMap::new();
        input.insert(
            "secret_key".to_string(),
            serde_json::Value::String("should_not_leak".into()),
        );

        let r = target
            .execute(
                &ExecutionTargetConfig::default(),
                "PROMPT_SHOULD_NOT_LEAK",
                &input,
                "tc-1",
            )
            .await
            .unwrap();

        assert_eq!(r.test_case_id, "tc-1");
        assert!(r.output.contains("test_case_id=tc-1"));
        assert!(r.output.contains("prompt_len="));
        assert!(r.output.contains("input_keys_count=1"));
        assert!(!r.output.contains("PROMPT_SHOULD_NOT_LEAK"));
        assert!(!r.output.contains("should_not_leak"));
    }

    #[tokio::test]
    async fn example_execution_target_execute_batch_preserves_order_and_alignment() {
        let target = ExampleExecutionTarget::new();

        let mut input_a = HashMap::new();
        input_a.insert(
            "k1".to_string(),
            serde_json::Value::String("v1_should_not_leak".into()),
        );
        let mut input_b = HashMap::new();
        input_b.insert(
            "k2".to_string(),
            serde_json::Value::String("v2_should_not_leak".into()),
        );

        let inputs = vec![input_a, input_b];
        let test_case_ids = vec!["tc-a".to_string(), "tc-b".to_string()];

        let results = target
            .execute_batch(
                &ExecutionTargetConfig::default(),
                "PROMPT_SHOULD_NOT_LEAK",
                &inputs,
                &test_case_ids,
            )
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].test_case_id, "tc-a");
        assert_eq!(results[1].test_case_id, "tc-b");
        assert!(results[0].output.contains("test_case_id=tc-a"));
        assert!(results[1].output.contains("test_case_id=tc-b"));

        // 不泄露 prompt 或 input 原文
        for r in results {
            assert!(!r.output.contains("PROMPT_SHOULD_NOT_LEAK"));
            assert!(!r.output.contains("v1_should_not_leak"));
            assert!(!r.output.contains("v2_should_not_leak"));
        }
    }
}
