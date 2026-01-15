use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;
use reqwest::Client;
use tokio::time::Instant;

use crate::core::execution_target::ExecutionError;
use crate::core::traits::ExecutionTarget;
use crate::domain::models::ExecutionResult;
use crate::domain::types::ExecutionTargetConfig;
use crate::infra::external::http_client::create_http_client;
use crate::infra::external::llm_client::{ChatCompletionsRequest, ChatMessage, LlmConnectionError};

#[derive(Debug, Clone)]
pub struct DirectApiExecutionTarget {
    client: Client,
}

impl Default for DirectApiExecutionTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectApiExecutionTarget {
    pub fn new() -> Self {
        let client = create_http_client().unwrap_or_else(|_| Client::new());
        Self { client }
    }
}

fn stable_json_object_string(map: &HashMap<String, serde_json::Value>) -> String {
    let mut ordered: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (k, v) in map {
        ordered.insert(k.clone(), v.clone());
    }
    serde_json::to_string(&ordered).unwrap_or_else(|_| "{}".to_string())
}

fn render_user_prompt(
    template: &str,
    prompt: &str,
    input: &HashMap<String, serde_json::Value>,
    test_case_id: &str,
) -> String {
    template
        .replace("{input}", &stable_json_object_string(input))
        .replace("{prompt}", prompt)
        .replace("{test_case_id}", test_case_id)
}

fn map_llm_error(test_case_id: &str, e: LlmConnectionError) -> ExecutionError {
    match e {
        LlmConnectionError::InvalidCredentials | LlmConnectionError::Forbidden => {
            ExecutionError::InvalidCredentials {
                test_case_id: test_case_id.to_string(),
                message: "LLM 鉴权失败".to_string(),
            }
        }
        LlmConnectionError::Timeout => ExecutionError::Timeout {
            test_case_id: test_case_id.to_string(),
            message: "LLM 请求超时".to_string(),
        },
        LlmConnectionError::RequestFailed(_e) => ExecutionError::Network {
            test_case_id: test_case_id.to_string(),
            message: "LLM 网络请求失败".to_string(),
        },
        LlmConnectionError::ParseError(_e) => ExecutionError::ParseError {
            test_case_id: test_case_id.to_string(),
            message: "解析 LLM 响应失败".to_string(),
        },
        LlmConnectionError::ValidationError(_e) => ExecutionError::InvalidRequest {
            test_case_id: test_case_id.to_string(),
            message: "LLM 请求参数无效".to_string(),
        },
        LlmConnectionError::ClientError(_e) => ExecutionError::Internal {
            test_case_id: test_case_id.to_string(),
            message: "LLM 客户端内部错误".to_string(),
        },
        LlmConnectionError::UpstreamError(msg) => ExecutionError::UpstreamError {
            test_case_id: test_case_id.to_string(),
            message: format!("LLM 上游错误: {msg}"),
        },
    }
}

#[async_trait]
impl ExecutionTarget for DirectApiExecutionTarget {
    async fn execute(
        &self,
        execution_target_config: &ExecutionTargetConfig,
        prompt: &str,
        input: &HashMap<String, serde_json::Value>,
        test_case_id: &str,
    ) -> Result<ExecutionResult, ExecutionError> {
        let test_case_id = test_case_id.trim();
        if test_case_id.is_empty() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: "unknown".to_string(),
                message: "test_case_id 不能为空".to_string(),
            });
        }

        let (base_url, model_name, user_prompt_template, api_key) = match execution_target_config {
            ExecutionTargetConfig::DirectModel {
                base_url,
                model_name,
                user_prompt_template,
                api_key,
            } => (
                base_url.as_str(),
                model_name.as_str(),
                user_prompt_template.as_str(),
                api_key.as_deref(),
            ),
            _ => {
                return Err(ExecutionError::InvalidRequest {
                    test_case_id: test_case_id.to_string(),
                    message: "execution_target_config 不是 DirectModel 配置".to_string(),
                });
            }
        };

        let Some(api_key) = api_key else {
            return Err(ExecutionError::InvalidCredentials {
                test_case_id: test_case_id.to_string(),
                message: "缺少 LLM API Key（运行时注入字段 api_key 为空）".to_string(),
            });
        };
        if base_url.trim().is_empty() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: test_case_id.to_string(),
                message: "base_url 不能为空".to_string(),
            });
        }
        if model_name.trim().is_empty() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: test_case_id.to_string(),
                message: "model_name 不能为空".to_string(),
            });
        }

        let user_prompt = render_user_prompt(user_prompt_template, prompt, input, test_case_id);

        let req = ChatCompletionsRequest {
            model: model_name.to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
        };

        let start = Instant::now();
        let output = crate::infra::external::llm_client::chat_completions(
            &self.client,
            base_url,
            api_key,
            test_case_id,
            &req,
        )
        .await
        .map_err(|e| map_llm_error(test_case_id, e))?;

        Ok(ExecutionResult {
            test_case_id: test_case_id.to_string(),
            output,
            latency_ms: start.elapsed().as_millis() as u64,
            token_usage: None,
            raw_response: None,
        })
    }

    fn name(&self) -> &str {
        "direct_api"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn direct_config(server: &MockServer) -> ExecutionTargetConfig {
        ExecutionTargetConfig::DirectModel {
            base_url: server.uri(),
            model_name: "m1".to_string(),
            user_prompt_template: "INPUT={input}".to_string(),
            api_key: Some("sk-test".to_string()),
        }
    }

    #[tokio::test]
    async fn direct_api_execution_target_parses_chat_completions_content() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer sk-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices": [{ "message": { "content": "RESULT" } }]
            })))
            .mount(&server)
            .await;

        let target = DirectApiExecutionTarget::new();
        let mut input = HashMap::new();
        input.insert("b".to_string(), json!(2));
        input.insert("a".to_string(), json!(1));

        let r = target
            .execute(&direct_config(&server), "SYS", &input, "tc-1")
            .await
            .unwrap();
        assert_eq!(r.test_case_id, "tc-1");
        assert_eq!(r.output, "RESULT");
    }

    #[tokio::test]
    async fn direct_api_execution_target_error_message_does_not_leak_prompt_or_input() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(429).set_body_string("TOPSECRET_DO_NOT_ECHO"))
            .mount(&server)
            .await;

        let target = DirectApiExecutionTarget::new();
        let mut input = HashMap::new();
        input.insert(
            "secret".to_string(),
            serde_json::Value::String("TOPSECRET_DO_NOT_ECHO".to_string()),
        );

        let err = target
            .execute(
                &direct_config(&server),
                "TOPSECRET_DO_NOT_ECHO",
                &input,
                "tc-429",
            )
            .await
            .unwrap_err();

        let s = err.to_string();
        assert!(!s.contains("TOPSECRET_DO_NOT_ECHO"), "err={s}");
    }
}
