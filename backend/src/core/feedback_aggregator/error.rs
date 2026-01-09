use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AggregatorError {
    #[error("invalid reflections: {0}")]
    InvalidReflections(String),

    #[error("arbitration failed: {0}")]
    ArbitrationFailed(String),

    #[error("model failure: {0}")]
    ModelFailure(String),

    #[error("internal error: {0}")]
    Internal(String),
}
