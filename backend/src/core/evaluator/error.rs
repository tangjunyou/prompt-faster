use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EvaluatorError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("evaluation timeout: {0}")]
    Timeout(String),

    #[error("model failure: {0}")]
    ModelFailure(String),

    #[error("internal error: {0}")]
    Internal(String),
}
