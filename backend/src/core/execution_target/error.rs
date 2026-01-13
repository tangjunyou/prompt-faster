use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    #[error("network error (test_case_id={test_case_id}): {message}")]
    Network {
        test_case_id: String,
        message: String,
    },

    #[error("invalid request (test_case_id={test_case_id}): {message}")]
    InvalidRequest {
        test_case_id: String,
        message: String,
    },

    #[error("upstream error (test_case_id={test_case_id}): {message}")]
    UpstreamError {
        test_case_id: String,
        message: String,
    },

    #[error("invalid credentials (test_case_id={test_case_id}): {message}")]
    InvalidCredentials {
        test_case_id: String,
        message: String,
    },

    #[error("parse error (test_case_id={test_case_id}): {message}")]
    ParseError {
        test_case_id: String,
        message: String,
    },

    #[error("timeout (test_case_id={test_case_id}): {message}")]
    Timeout {
        test_case_id: String,
        message: String,
    },

    #[error("not implemented (test_case_id={test_case_id}): {message}")]
    NotImplemented {
        test_case_id: String,
        message: String,
    },

    #[error("internal error (test_case_id={test_case_id}): {message}")]
    Internal {
        test_case_id: String,
        message: String,
    },
}
