//! 统一 API 响应结构
//! ApiResponse<T> - data 与 error 互斥 (AR1)

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

/// API 成功响应
#[derive(Serialize)]
pub struct ApiSuccess<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

/// 分页元信息
#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

/// API 错误响应
#[derive(Serialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}

/// 错误详情
#[derive(Serialize)]
pub struct ErrorDetail {
    /// 格式：DOMAIN_ACTION_REASON
    pub code: String,
    /// 用户可见消息
    pub message: String,
    /// 仅开发环境显示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// 统一响应类型 - data 与 error 互斥
pub enum ApiResponse<T: Serialize> {
    Success(ApiSuccess<T>),
    Error(StatusCode, ApiError),
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    pub fn ok(data: T) -> Self {
        ApiResponse::Success(ApiSuccess { data, meta: None })
    }

    /// 创建带分页的成功响应
    pub fn ok_with_meta(data: T, meta: PaginationMeta) -> Self {
        ApiResponse::Success(ApiSuccess {
            data,
            meta: Some(meta),
        })
    }

    /// 创建错误响应
    pub fn err(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        ApiResponse::Error(
            status,
            ApiError {
                error: ErrorDetail {
                    code: code.into(),
                    message: message.into(),
                    details: None,
                },
            },
        )
    }

    /// 创建带详情的错误响应（仅开发环境）
    pub fn err_with_details(
        status: StatusCode,
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        ApiResponse::Error(
            status,
            ApiError {
                error: ErrorDetail {
                    code: code.into(),
                    message: message.into(),
                    details: Some(details),
                },
            },
        )
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiResponse::Success(success) => (StatusCode::OK, Json(success)).into_response(),
            ApiResponse::Error(status, error) => (status, Json(error)).into_response(),
        }
    }
}

/// 快捷函数：创建成功响应
pub fn ok<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse::ok(data)
}

/// 快捷函数：创建 404 错误
pub fn not_found<T: Serialize>(resource: &str) -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        "RESOURCE_NOT_FOUND",
        format!("资源不存在: {}", resource),
    )
}

/// 快捷函数：创建 400 错误
pub fn bad_request<T: Serialize>(message: &str) -> ApiResponse<T> {
    ApiResponse::err(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message)
}

/// 快捷函数：创建 401 错误
pub fn unauthorized<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "请先登录")
}

/// 快捷函数：创建 500 错误
pub fn internal_error<T: Serialize>(message: &str) -> ApiResponse<T> {
    ApiResponse::err(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
}
