//! 统一错误处理
//! 使用 thiserror 定义类型安全错误
//! 与 api/response.rs 的 ApiError 结构保持一致 (AR1)

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("未授权")]
    Unauthorized,

    #[error("禁止访问")]
    Forbidden,

    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
}

/// 错误详情（与 api/response.rs 的 ErrorDetail 结构一致）
#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// API 错误响应结构（与 api/response.rs 的 ApiError 结构一致）
/// 统一使用 { error: { code, message, details? } } 格式
#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub error: ErrorDetail,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                format!("数据库操作失败: {}", e),
            ),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            AppError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("资源不存在: {}", resource),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "请先登录".to_string(),
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "无权访问".to_string()),
            AppError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                format!("内部错误: {}", e),
            ),
        };

        let body = Json(ApiErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                details: None,
            },
        });

        (status, body).into_response()
    }
}

/// Result 类型别名
pub type AppResult<T> = Result<T, AppError>;
