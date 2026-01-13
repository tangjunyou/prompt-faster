use std::collections::HashMap;

use async_trait::async_trait;

use crate::core::execution_target::ExecutionError;
use crate::core::traits::ExecutionTarget;
use crate::domain::models::ExecutionResult;

#[derive(Debug, Default)]
pub struct DifyExecutionTarget;

impl DifyExecutionTarget {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExecutionTarget for DifyExecutionTarget {
    async fn execute(
        &self,
        _prompt: &str,
        _input: &HashMap<String, serde_json::Value>,
        test_case_id: &str,
    ) -> Result<ExecutionResult, ExecutionError> {
        Err(ExecutionError::NotImplemented {
            test_case_id: test_case_id.to_string(),
            message: "dify execution target not implemented".to_string(),
        })
    }

    fn name(&self) -> &str {
        "dify"
    }
}
