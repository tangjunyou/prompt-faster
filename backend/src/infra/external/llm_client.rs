//! LLM API 调用客户端
//! 唯一的外部 LLM 调用点
//! 支持 OpenAI 兼容 API（硅基流动、魔搭社区等）

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::dify_client::TestConnectionResult;
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

/// OpenAI 兼容 Chat Completions - 消息
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// OpenAI 兼容 Chat Completions - 请求
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: Option<ChatChoiceMessage>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    #[serde(default)]
    content: Option<String>,
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
    let model_ids = list_models(client, base_url, api_key, provider, correlation_id).await?;
    let model_count = model_ids.len();

    Ok(TestConnectionResult {
        message: format!("连接成功，可用模型: {}", model_count),
        models: Some(model_ids),
    })
}

/// 获取通用大模型可用模型列表
///
/// 调用 OpenAI 兼容的 `/v1/models` 端点。
pub async fn list_models(
    client: &Client,
    base_url: &str,
    api_key: &str,
    provider: &str,
    correlation_id: &str,
) -> Result<Vec<String>, LlmConnectionError> {
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

            Ok(models.data.into_iter().map(|m| m.id).collect())
        }
        401 => Err(LlmConnectionError::InvalidCredentials),
        403 => Err(LlmConnectionError::Forbidden),
        status => {
            // 不回显上游 body（可能包含敏感信息，如 prompt/testcase/token）。
            Err(LlmConnectionError::UpstreamError(format!(
                "HTTP {}",
                status
            )))
        }
    }
}

/// 调用 OpenAI 兼容的 `/v1/chat/completions` 获取输出文本（阻塞模式）。
///
/// 重要：错误信息不得回显 prompt/input 原文或上游 body（可能包含敏感信息）。
pub async fn chat_completions(
    client: &Client,
    base_url: &str,
    api_key: &str,
    correlation_id: &str,
    req: &ChatCompletionsRequest,
) -> Result<String, LlmConnectionError> {
    let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-Correlation-Id", correlation_id)
        .json(req)
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
            let json = response
                .json::<ChatCompletionsResponse>()
                .await
                .map_err(|e| {
                    LlmConnectionError::ParseError(format!("解析 chat/completions 响应失败: {}", e))
                })?;
            let choice = json.choices.into_iter().next().ok_or_else(|| {
                LlmConnectionError::ParseError("chat/completions 缺少 choices[0]".to_string())
            })?;

            if let Some(s) = choice
                .message
                .and_then(|m| m.content)
                .filter(|s| !s.trim().is_empty())
            {
                return Ok(s);
            }
            if let Some(s) = choice.text.filter(|s| !s.trim().is_empty()) {
                return Ok(s);
            }

            Err(LlmConnectionError::ParseError(
                "chat/completions 缺少 message.content/text".to_string(),
            ))
        }
        401 => Err(LlmConnectionError::InvalidCredentials),
        403 => Err(LlmConnectionError::Forbidden),
        status => {
            // 不读取/拼接 body，避免上游回显敏感内容。
            Err(LlmConnectionError::UpstreamError(format!(
                "HTTP {}",
                status
            )))
        }
    }
}
