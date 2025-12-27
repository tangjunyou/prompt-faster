//! Dify API 连接测试客户端
//! 用于验证 Dify API Key 的有效性

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use super::http_client::truncate_error_body;

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestConnectionResult {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
}

/// 连接错误类型
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("无效的 API Key")]
    InvalidCredentials,

    #[error("访问被拒绝")]
    Forbidden,

    #[error("连接超时")]
    Timeout,

    #[error("上游服务不可用: {0}")]
    UpstreamError(String),

    #[error("请求失败: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("响应解析失败: {0}")]
    ParseError(String),

    #[error("输入验证失败: {0}")]
    ValidationError(String),

    #[error("HTTP 客户端错误: {0}")]
    ClientError(String),
}

/// 测试 Dify API 连接
///
/// 使用 POST /v1/completion-messages 端点验证 API Key 有效性
///
/// # Arguments
/// * `client` - 共享的 HTTP 客户端（复用连接池）
/// * `base_url` - Dify API 基础 URL（已通过 SSRF 验证）
/// * `api_key` - Dify API Key（已通过非空验证）
/// * `correlation_id` - 请求追踪 ID（透传到上游）
///
/// # Returns
/// * `Ok(TestConnectionResult)` - 连接成功
/// * `Err(ConnectionError)` - 连接失败
pub async fn test_connection(
    client: &Client,
    base_url: &str,
    api_key: &str,
    correlation_id: &str,
) -> Result<TestConnectionResult, ConnectionError> {
    let url = format!("{}/v1/completion-messages", base_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("X-Correlation-Id", correlation_id)
        .json(&serde_json::json!({
            "inputs": {},
            "response_mode": "blocking",
            "user": "connection-test"
        }))
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                ConnectionError::Timeout
            } else {
                ConnectionError::RequestFailed(e)
            }
        })?;

    match response.status().as_u16() {
        200..=299 => Ok(TestConnectionResult {
            message: "连接成功".to_string(),
            models: None,
        }),
        401 => Err(ConnectionError::InvalidCredentials),
        403 => Err(ConnectionError::Forbidden),
        status => {
            let body = response.text().await.unwrap_or_default();
            // 截断错误消息，防止泄露敏感信息
            Err(ConnectionError::UpstreamError(format!(
                "HTTP {} - {}",
                status,
                truncate_error_body(&body)
            )))
        }
    }
}
