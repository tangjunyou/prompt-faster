//! Dify API 连接测试客户端
//! 用于验证 Dify API Key 的有效性

use std::collections::{HashMap, HashSet};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
#[serde(rename_all = "lowercase")]
pub enum DifyValueType {
    String,
    Number,
    Bool,
    Object,
    Array,
    Unknown,
}

/// Dify 输入变量（用于前端渲染与变量绑定配置）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct DifyInputVariable {
    pub name: String,
    pub component: String,
    pub r#type: DifyValueType,
    pub required: bool,
    /// required 字段是否由上游明确给出（用于 UI 展示“未知”）
    pub required_known: bool,
    pub default_value: Option<serde_json::Value>,
    /// 原始片段（仅供 debug；前端 UI 禁止直接展示）
    pub raw: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct DifyVariablesResponse {
    pub variables: Vec<DifyInputVariable>,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
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

#[derive(Debug, Deserialize)]
struct DifyParametersRaw {
    #[serde(default)]
    user_input_form: Vec<HashMap<String, DifyUserInputFieldRaw>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DifyUserInputFieldRaw {
    #[serde(default)]
    variable: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    default: Option<serde_json::Value>,
    #[serde(flatten)]
    _extra: HashMap<String, serde_json::Value>,
}

fn normalize_base_url_for_parameters(base_url: &str) -> String {
    let mut base = base_url.trim_end_matches('/').to_string();
    if base.ends_with("/v1") {
        base.truncate(base.len().saturating_sub(3));
        base = base.trim_end_matches('/').to_string();
    }
    base
}

fn infer_value_type(component: &str, default_value: Option<&serde_json::Value>) -> DifyValueType {
    if let Some(v) = default_value {
        match v {
            serde_json::Value::String(_) => return DifyValueType::String,
            serde_json::Value::Number(_) => return DifyValueType::Number,
            serde_json::Value::Bool(_) => return DifyValueType::Bool,
            serde_json::Value::Array(_) => return DifyValueType::Array,
            serde_json::Value::Object(_) => return DifyValueType::Object,
            serde_json::Value::Null => return DifyValueType::Unknown,
        }
    }

    match component {
        "text-input" | "paragraph" | "select" | "multi-select" | "radio" => DifyValueType::String,
        "number" | "slider" => DifyValueType::Number,
        "checkbox" | "switch" => DifyValueType::Bool,
        _ => DifyValueType::Unknown,
    }
}

fn parse_parameters_variables(
    raw: DifyParametersRaw,
) -> Result<Vec<DifyInputVariable>, ConnectionError> {
    let mut out: Vec<DifyInputVariable> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for group in raw.user_input_form {
        for (component, field) in group {
            let name = field
                .variable
                .clone()
                .or(field.name.clone())
                .ok_or_else(|| ConnectionError::ParseError("缺少 variable/name 字段".to_string()))?
                .trim()
                .to_string();

            if name.is_empty() {
                return Err(ConnectionError::ParseError(
                    "variable/name 不能为空".to_string(),
                ));
            }

            if !seen.insert(name.clone()) {
                return Err(ConnectionError::ParseError(format!(
                    "重复的变量名: {}",
                    name
                )));
            }

            let default_value = field.default.clone();
            let value_type = infer_value_type(&component, default_value.as_ref());

            out.push(DifyInputVariable {
                name,
                component: component.clone(),
                r#type: value_type,
                required: field.required.unwrap_or(false),
                required_known: field.required.is_some(),
                default_value,
                raw: Some(serde_json::json!({ component: field })),
            });
        }
    }

    Ok(out)
}

/// 获取 Dify 应用的输入变量结构
///
/// GET /v1/parameters
pub async fn get_parameters(
    client: &Client,
    base_url: &str,
    api_key: &str,
    correlation_id: &str,
) -> Result<DifyVariablesResponse, ConnectionError> {
    let base_url = normalize_base_url_for_parameters(base_url);
    let url = format!("{}/v1/parameters", base_url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-Correlation-Id", correlation_id)
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
        200..=299 => {
            let json = response.json::<DifyParametersRaw>().await.map_err(|e| {
                ConnectionError::ParseError(format!("解析 /parameters 响应失败: {}", e))
            })?;

            let variables = parse_parameters_variables(json)?;
            Ok(DifyVariablesResponse { variables })
        }
        401 => Err(ConnectionError::InvalidCredentials),
        403 => Err(ConnectionError::Forbidden),
        status => Err(ConnectionError::UpstreamError(format!("HTTP {}", status))),
    }
}

/// 测试 Dify API 连接
///
/// 使用 GET /v1/parameters 端点验证 API Key 有效性
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
    let base_url = normalize_base_url_for_parameters(base_url);
    let url = format!("{}/v1/parameters", base_url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-Correlation-Id", correlation_id)
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
        status => Err(ConnectionError::UpstreamError(format!("HTTP {}", status))),
    }
}
