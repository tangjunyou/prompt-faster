use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OptimizerError {
    #[error("optimization step failed: {0}")]
    StepFailed(String),

    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("model failure: {0}")]
    ModelFailure(String),

    #[error("internal error: {0}")]
    Internal(String),
}
