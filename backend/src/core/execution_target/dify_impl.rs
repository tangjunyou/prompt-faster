use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};
use tokio::time::Instant;
use url::Url;

use crate::core::execution_target::ExecutionError;
use crate::core::traits::ExecutionTarget;
use crate::domain::models::ExecutionResult;
use crate::domain::types::ExecutionTargetConfig;
use crate::infra::external::http_client::create_http_client;

#[derive(Debug, Clone)]
pub struct DifyExecutionTarget {
    client: Client,
}

impl Default for DifyExecutionTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl DifyExecutionTarget {
    pub fn new() -> Self {
        let client = create_http_client().unwrap_or_else(|_| Client::new());
        Self { client }
    }
}

fn normalize_base_url_for_v1(base_url: &str) -> String {
    let trimmed = base_url.trim();
    if let Ok(mut url) = Url::parse(trimmed) {
        let path = url.path().trim_end_matches('/');
        if path == "/v1" {
            url.set_path("/");
        }
        url.set_query(None);
        url.set_fragment(None);
        return url.to_string().trim_end_matches('/').to_string();
    }

    // Fallback：保持容错（避免因 URL 非法导致功能不可用）
    let mut base = trimmed.trim_end_matches('/').to_string();
    if base.ends_with("/v1") {
        base.truncate(base.len().saturating_sub(3));
        base = base.trim_end_matches('/').to_string();
    }
    base
}

fn build_workflow_run_url(base_url: &str, workflow_id: &str) -> String {
    let base_url = normalize_base_url_for_v1(base_url);
    let wf = workflow_id.trim();
    if wf.is_empty() {
        format!("{base_url}/v1/workflows/run")
    } else {
        format!("{base_url}/v1/workflows/{wf}/run")
    }
}

fn build_inputs_with_prompt(
    input: &HashMap<String, Value>,
    prompt_variable: &str,
    prompt: &str,
) -> serde_json::Map<String, Value> {
    let mut out: serde_json::Map<String, Value> = serde_json::Map::new();
    for (k, v) in input {
        out.insert(k.clone(), v.clone());
    }
    out.insert(
        prompt_variable.to_string(),
        Value::String(prompt.to_string()),
    );
    out
}

fn extract_output_from_dify_response(v: &Value) -> Option<String> {
    // Prefer common shapes:
    // - { answer: "..." } (chat-style)
    // - { data: { outputs: { text: "..." } } } (workflow-style)
    // - { data: { outputs: { <single_key>: "..." } } }
    // - { data: { text: "..." } } / { output: "..." }
    if let Some(answer) = v.get("answer").and_then(|x| x.as_str()) {
        return Some(answer.to_string());
    }
    if let Some(answer) = v
        .get("data")
        .and_then(|d| d.get("answer"))
        .and_then(|x| x.as_str())
    {
        return Some(answer.to_string());
    }
    if let Some(outputs) = v
        .get("data")
        .and_then(|d| d.get("outputs"))
        .or_else(|| v.get("outputs"))
    {
        if let Some(s) = outputs.as_str() {
            return Some(s.to_string());
        }
        if let Some(obj) = outputs.as_object() {
            for k in ["text", "output", "result", "answer"] {
                if let Some(s) = obj.get(k).and_then(|x| x.as_str()) {
                    return Some(s.to_string());
                }
            }
            let string_fields = obj
                .iter()
                .filter_map(|(_k, x)| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            if string_fields.len() == 1 {
                return Some(string_fields[0].clone());
            }
        }
    }
    if let Some(text) = v
        .get("data")
        .and_then(|d| d.get("text"))
        .and_then(|x| x.as_str())
    {
        return Some(text.to_string());
    }
    if let Some(output) = v.get("output").and_then(|x| x.as_str()) {
        return Some(output.to_string());
    }
    None
}

#[async_trait]
impl ExecutionTarget for DifyExecutionTarget {
    async fn execute(
        &self,
        execution_target_config: &ExecutionTargetConfig,
        prompt: &str,
        input: &HashMap<String, Value>,
        test_case_id: &str,
    ) -> Result<ExecutionResult, ExecutionError> {
        let test_case_id = test_case_id.trim();
        if test_case_id.is_empty() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: "unknown".to_string(),
                message: "test_case_id 不能为空".to_string(),
            });
        }

        let (api_url, workflow_id, prompt_variable, api_key) = match execution_target_config {
            ExecutionTargetConfig::Dify {
                api_url,
                workflow_id,
                prompt_variable,
                api_key,
            } => (api_url, workflow_id, prompt_variable, api_key.as_deref()),
            _ => {
                return Err(ExecutionError::InvalidRequest {
                    test_case_id: test_case_id.to_string(),
                    message: "execution_target_config 不是 Dify 配置".to_string(),
                });
            }
        };

        let Some(api_key) = api_key else {
            return Err(ExecutionError::InvalidCredentials {
                test_case_id: test_case_id.to_string(),
                message: "缺少 Dify API Key（运行时注入字段 api_key 为空）".to_string(),
            });
        };

        let url = build_workflow_run_url(api_url, workflow_id);
        let inputs = build_inputs_with_prompt(input, prompt_variable, prompt);

        let body = json!({
            "inputs": inputs,
            "response_mode": "blocking",
            "user": "prompt-faster",
        });

        let start = Instant::now();
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("X-Correlation-Id", test_case_id)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ExecutionError::Timeout {
                        test_case_id: test_case_id.to_string(),
                        message: "Dify 请求超时".to_string(),
                    }
                } else {
                    ExecutionError::Network {
                        test_case_id: test_case_id.to_string(),
                        message: "Dify 网络请求失败".to_string(),
                    }
                }
            })?;

        let status = response.status().as_u16();
        if !(200..=299).contains(&status) {
            // 重要：错误消息不得包含 prompt/input 原文或上游 body（可能回显敏感内容）。
            return Err(match status {
                401 | 403 => ExecutionError::InvalidCredentials {
                    test_case_id: test_case_id.to_string(),
                    message: format!("Dify 鉴权失败（HTTP {status}）"),
                },
                400 => ExecutionError::InvalidRequest {
                    test_case_id: test_case_id.to_string(),
                    message: "Dify 请求参数错误（HTTP 400）".to_string(),
                },
                _ => ExecutionError::UpstreamError {
                    test_case_id: test_case_id.to_string(),
                    message: format!("Dify 上游错误（HTTP {status}）"),
                },
            });
        }

        let v = response
            .json::<Value>()
            .await
            .map_err(|_e| ExecutionError::ParseError {
                test_case_id: test_case_id.to_string(),
                message: "解析 Dify 响应失败".to_string(),
            })?;

        let output =
            extract_output_from_dify_response(&v).ok_or_else(|| ExecutionError::ParseError {
                test_case_id: test_case_id.to_string(),
                message: "Dify 响应缺少可识别的输出字段".to_string(),
            })?;

        Ok(ExecutionResult {
            test_case_id: test_case_id.to_string(),
            output,
            latency_ms: start.elapsed().as_millis() as u64,
            token_usage: None,
            raw_response: None,
        })
    }

    fn name(&self) -> &str {
        "dify"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn dify_config(server: &MockServer) -> ExecutionTargetConfig {
        ExecutionTargetConfig::Dify {
            api_url: server.uri(),
            workflow_id: "wf-1".to_string(),
            prompt_variable: "prompt".to_string(),
            api_key: Some("dify_test_key".to_string()),
        }
    }

    #[tokio::test]
    async fn dify_execution_target_parses_output_and_injects_prompt_variable() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/workflows/wf-1/run"))
            .and(header("Authorization", "Bearer dify_test_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": { "outputs": { "text": "OK" } }
            })))
            .mount(&server)
            .await;

        let target = DifyExecutionTarget::new();
        let mut input = HashMap::new();
        input.insert("x".to_string(), Value::String("y".to_string()));

        let r = target
            .execute(&dify_config(&server), "PROMPT_VALUE", &input, "tc-1")
            .await
            .unwrap();
        assert_eq!(r.test_case_id, "tc-1");
        assert_eq!(r.output, "OK");
    }

    #[tokio::test]
    async fn dify_execution_target_maps_unauthorized_to_invalid_credentials() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/workflows/wf-1/run"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let target = DifyExecutionTarget::new();
        let input = HashMap::new();
        let err = target
            .execute(&dify_config(&server), "p", &input, "tc-401")
            .await
            .unwrap_err();
        assert!(matches!(err, ExecutionError::InvalidCredentials { .. }));
    }

    #[tokio::test]
    async fn dify_execution_target_error_message_does_not_leak_prompt_or_input() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/workflows/wf-1/run"))
            .respond_with(ResponseTemplate::new(500).set_body_string("TOPSECRET_DO_NOT_ECHO"))
            .mount(&server)
            .await;

        let target = DifyExecutionTarget::new();
        let mut input = HashMap::new();
        input.insert(
            "secret".to_string(),
            Value::String("TOPSECRET_DO_NOT_ECHO".to_string()),
        );

        let err = target
            .execute(
                &dify_config(&server),
                "TOPSECRET_DO_NOT_ECHO",
                &input,
                "tc-500",
            )
            .await
            .unwrap_err();

        let s = err.to_string();
        assert!(!s.contains("TOPSECRET_DO_NOT_ECHO"), "err={s}");
    }
}
