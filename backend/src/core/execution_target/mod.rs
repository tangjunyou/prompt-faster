mod dify_impl;
mod direct_api_impl;
mod error;

use std::sync::Arc;

use crate::core::traits::ExecutionTarget;
use crate::domain::models::ExecutionTargetType;

pub use dify_impl::DifyExecutionTarget;
pub use direct_api_impl::DirectApiExecutionTarget;
pub use error::ExecutionError;

pub fn create_execution_target(
    execution_target_type: ExecutionTargetType,
) -> Arc<dyn ExecutionTarget> {
    match execution_target_type {
        ExecutionTargetType::Dify => Arc::new(DifyExecutionTarget::new()),
        ExecutionTargetType::Generic => Arc::new(DirectApiExecutionTarget::new()),
    }
}
