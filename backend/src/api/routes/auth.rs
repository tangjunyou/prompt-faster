//! 认证相关路由
//! 包含 API 连接测试端点

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::post};
use serde::Deserialize;
use tracing::{info, warn};

use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::ApiResponse;
use crate::api::state::AppState;
use crate::infra::external::dify_client::{self, ConnectionError, TestConnectionResult};
use crate::infra::external::llm_client::{self, LlmConnectionError};
use crate::shared::log_sanitizer::sanitize_api_key;
use crate::shared::url_validator::{validate_api_key, validate_base_url};

/// Dify 连接测试请求
#[derive(Debug, Deserialize)]
pub struct TestDifyConnectionRequest {
    pub base_url: String,
    pub api_key: String,
}

/// 通用大模型连接测试请求
#[derive(Debug, Deserialize)]
pub struct TestGenericLlmConnectionRequest {
    pub base_url: String,
    pub api_key: String,
    pub provider: String, // "siliconflow" | "modelscope"
}

/// 从请求头提取 correlation_id
fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// 测试 Dify API 连接
///
/// POST /api/v1/auth/test-connection/dify
async fn test_dify_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TestDifyConnectionRequest>,
) -> ApiResponse<TestConnectionResult> {
    let correlation_id = extract_correlation_id(&headers);

    info!(
        base_url = %req.base_url,
        api_key = %sanitize_api_key(&req.api_key),
        correlation_id = %correlation_id,
        "测试 Dify 连接"
    );

    // 输入验证：SSRF 防护（开发环境允许 HTTP）
    #[cfg(debug_assertions)]
    let allow_http = true;
    #[cfg(not(debug_assertions))]
    let allow_http = false;

    if let Err(e) = validate_base_url(&req.base_url, allow_http) {
        warn!(error = %e, "URL 验证失败");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            "AUTH_VALIDATION_ERROR",
            e.to_string(),
        );
    }

    // 输入验证：API Key 非空
    if let Err(e) = validate_api_key(&req.api_key) {
        warn!(error = %e, "API Key 验证失败");
        return ApiResponse::err(StatusCode::BAD_REQUEST, "AUTH_VALIDATION_ERROR", e);
    }

    match dify_client::test_connection(
        &state.http_client,
        &req.base_url,
        &req.api_key,
        &correlation_id,
    )
    .await
    {
        Ok(result) => {
            info!("Dify 连接测试成功");
            ApiResponse::ok(result)
        }
        Err(e) => {
            warn!(error = %e, "Dify 连接测试失败");
            map_connection_error(e)
        }
    }
}

/// 测试通用大模型 API 连接
///
/// POST /api/v1/auth/test-connection/generic-llm
async fn test_generic_llm_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TestGenericLlmConnectionRequest>,
) -> ApiResponse<TestConnectionResult> {
    let correlation_id = extract_correlation_id(&headers);

    info!(
        base_url = %req.base_url,
        api_key = %sanitize_api_key(&req.api_key),
        provider = %req.provider,
        correlation_id = %correlation_id,
        "测试通用大模型连接"
    );

    // 输入验证：SSRF 防护（开发环境允许 HTTP）
    #[cfg(debug_assertions)]
    let allow_http = true;
    #[cfg(not(debug_assertions))]
    let allow_http = false;

    if let Err(e) = validate_base_url(&req.base_url, allow_http) {
        warn!(error = %e, "URL 验证失败");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            "AUTH_VALIDATION_ERROR",
            e.to_string(),
        );
    }

    // 输入验证：API Key 非空
    if let Err(e) = validate_api_key(&req.api_key) {
        warn!(error = %e, "API Key 验证失败");
        return ApiResponse::err(StatusCode::BAD_REQUEST, "AUTH_VALIDATION_ERROR", e);
    }

    match llm_client::test_connection(
        &state.http_client,
        &req.base_url,
        &req.api_key,
        &req.provider,
        &correlation_id,
    )
    .await
    {
        Ok(result) => {
            info!("通用大模型连接测试成功");
            ApiResponse::ok(result)
        }
        Err(e) => {
            warn!(error = %e, "通用大模型连接测试失败");
            map_llm_connection_error(e)
        }
    }
}

/// 映射 Dify 连接错误到 API 响应
fn map_connection_error(error: ConnectionError) -> ApiResponse<TestConnectionResult> {
    match error {
        ConnectionError::InvalidCredentials => ApiResponse::err(
            StatusCode::UNAUTHORIZED,
            "AUTH_INVALID_CREDENTIALS",
            "无效的 API Key",
        ),
        ConnectionError::Forbidden => {
            ApiResponse::err(StatusCode::FORBIDDEN, "AUTH_FORBIDDEN", "访问被拒绝")
        }
        ConnectionError::Timeout => ApiResponse::err(
            StatusCode::REQUEST_TIMEOUT,
            "AUTH_CONNECTION_TIMEOUT",
            "连接超时",
        ),
        ConnectionError::UpstreamError(msg) => ApiResponse::err(
            StatusCode::BAD_GATEWAY,
            "AUTH_UPSTREAM_ERROR",
            format!("上游服务不可用: {}", msg),
        ),
        ConnectionError::RequestFailed(e) => ApiResponse::err(
            StatusCode::BAD_GATEWAY,
            "AUTH_UPSTREAM_ERROR",
            format!("请求失败: {}", e),
        ),
        ConnectionError::ParseError(msg) => ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUTH_INTERNAL_ERROR",
            format!("响应解析失败: {}", msg),
        ),
        ConnectionError::ValidationError(msg) => {
            ApiResponse::err(StatusCode::BAD_REQUEST, "AUTH_VALIDATION_ERROR", msg)
        }
        ConnectionError::ClientError(msg) => ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUTH_INTERNAL_ERROR",
            format!("HTTP 客户端错误: {}", msg),
        ),
    }
}

/// 映射 LLM 连接错误到 API 响应
fn map_llm_connection_error(error: LlmConnectionError) -> ApiResponse<TestConnectionResult> {
    match error {
        LlmConnectionError::InvalidCredentials => ApiResponse::err(
            StatusCode::UNAUTHORIZED,
            "AUTH_INVALID_CREDENTIALS",
            "无效的 API Key",
        ),
        LlmConnectionError::Forbidden => {
            ApiResponse::err(StatusCode::FORBIDDEN, "AUTH_FORBIDDEN", "访问被拒绝")
        }
        LlmConnectionError::Timeout => ApiResponse::err(
            StatusCode::REQUEST_TIMEOUT,
            "AUTH_CONNECTION_TIMEOUT",
            "连接超时",
        ),
        LlmConnectionError::UpstreamError(msg) => ApiResponse::err(
            StatusCode::BAD_GATEWAY,
            "AUTH_UPSTREAM_ERROR",
            format!("上游服务不可用: {}", msg),
        ),
        LlmConnectionError::RequestFailed(e) => ApiResponse::err(
            StatusCode::BAD_GATEWAY,
            "AUTH_UPSTREAM_ERROR",
            format!("请求失败: {}", e),
        ),
        LlmConnectionError::ParseError(msg) => ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUTH_INTERNAL_ERROR",
            format!("响应解析失败: {}", msg),
        ),
        LlmConnectionError::ValidationError(msg) => {
            ApiResponse::err(StatusCode::BAD_REQUEST, "AUTH_VALIDATION_ERROR", msg)
        }
        LlmConnectionError::ClientError(msg) => ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "AUTH_INTERNAL_ERROR",
            format!("HTTP 客户端错误: {}", msg),
        ),
    }
}

/// 创建认证路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/test-connection/dify", post(test_dify_connection))
        .route(
            "/test-connection/generic-llm",
            post(test_generic_llm_connection),
        )
}
