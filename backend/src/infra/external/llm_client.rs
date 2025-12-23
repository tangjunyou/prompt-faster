//! LLM API 调用客户端
//! 唯一的外部 LLM 调用点
//! 支持 OpenAI 兼容 API（硅基流动、魔搭社区等）

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use super::dify_client::TestConnectionResult;
use super::http_client::truncate_error_body;

/// Provider 允许列表（白名单验证）
pub const ALLOWED_PROVIDERS: &[&str] = &["siliconflow", "modelscope"];

/// Provider 默认 Base URL
pub fn get_default_base_url(provider: &str) -> Option<&'static str> {
    match provider {
        "siliconflow" => Some("https://api.siliconflow.cn"),
        "modelscope" => Some("https://dashscope.aliyuncs.com/compatible-mode"),
        _ => None,
    }
}

/// OpenAI 兼容 API 模型列表响应
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}

/// 模型信息
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub id: String,
}

/// LLM 连接错误类型
#[derive(Debug, Error)]
pub enum LlmConnectionError {
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

/// 验证 provider 是否在允许列表中
pub fn validate_provider(provider: &str) -> Result<(), LlmConnectionError> {
    if !ALLOWED_PROVIDERS.contains(&provider) {
        return Err(LlmConnectionError::ValidationError(format!(
            "不支持的 Provider: {}，支持的 Provider: {:?}",
            provider, ALLOWED_PROVIDERS
        )));
    }
    Ok(())
}

/// 测试通用大模型 API 连接
///
/// 调用 OpenAI 兼容的 /v1/models 端点获取模型列表验证凭证
///
/// # Arguments
/// * `client` - 共享的 HTTP 客户端（复用连接池）
/// * `base_url` - API 基础 URL（已通过 SSRF 验证）
/// * `api_key` - API Key（已通过非空验证）
/// * `provider` - Provider 标识 ("siliconflow" | "modelscope")
/// * `correlation_id` - 请求追踪 ID（透传到上游）
///
/// # Returns
/// * `Ok(TestConnectionResult)` - 连接成功，包含可用模型列表
/// * `Err(LlmConnectionError)` - 连接失败
pub async fn test_connection(
    client: &Client,
    base_url: &str,
    api_key: &str,
    provider: &str,
    correlation_id: &str,
) -> Result<TestConnectionResult, LlmConnectionError> {
    // 验证 provider
    validate_provider(provider)?;

    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-Correlation-Id", correlation_id)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                LlmConnectionError::Timeout
            } else {
                LlmConnectionError::RequestFailed(e)
            }
        })?;

    match response.status().as_u16() {
        200..=299 => {
            let models: ModelsResponse = response
                .json()
                .await
                .map_err(|e| LlmConnectionError::ParseError(format!("解析模型列表失败: {}", e)))?;

            let model_ids: Vec<String> = models.data.iter().map(|m| m.id.clone()).collect();
            let model_count = model_ids.len();

            Ok(TestConnectionResult {
                message: format!("连接成功，可用模型: {}", model_count),
                models: Some(model_ids),
            })
        }
        401 => Err(LlmConnectionError::InvalidCredentials),
        403 => Err(LlmConnectionError::Forbidden),
        status => {
            let body = response.text().await.unwrap_or_default();
            // 截断错误消息，防止泄露敏感信息
            Err(LlmConnectionError::UpstreamError(format!(
                "HTTP {} - {}",
                status,
                truncate_error_body(&body)
            )))
        }
    }
}
