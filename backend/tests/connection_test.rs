//! 连接测试集成测试
//! 使用 wiremock 模拟外部 API 响应

use prompt_faster::infra::external::dify_client;
use prompt_faster::infra::external::http_client::create_http_client;
use prompt_faster::infra::external::llm_client;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// 测试 Dify 连接成功（200 响应）
#[tokio::test]
async fn test_dify_connection_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/completion-messages"))
        .and(header("Authorization", "Bearer test-api-key"))
        .and(header("X-Correlation-Id", "test-correlation-id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "answer": "test response",
            "conversation_id": "test-conv-id"
        })))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = dify_client::test_connection(
        &client,
        &mock_server.uri(),
        "test-api-key",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.message, "连接成功");
    assert!(result.models.is_none());
}

/// 测试 Dify 连接 401 未授权
#[tokio::test]
async fn test_dify_connection_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/completion-messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code": "unauthorized",
            "message": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = dify_client::test_connection(
        &client,
        &mock_server.uri(),
        "invalid-key",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        dify_client::ConnectionError::InvalidCredentials
    ));
}

/// 测试 Dify 连接 403 禁止访问
#[tokio::test]
async fn test_dify_connection_forbidden() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/completion-messages"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "code": "forbidden",
            "message": "Access denied"
        })))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = dify_client::test_connection(
        &client,
        &mock_server.uri(),
        "test-key",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, dify_client::ConnectionError::Forbidden));
}

/// 测试 Dify 连接 500 服务器错误
#[tokio::test]
async fn test_dify_connection_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/completion-messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = dify_client::test_connection(
        &client,
        &mock_server.uri(),
        "test-key",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        dify_client::ConnectionError::UpstreamError(_)
    ));
}

/// 测试 LLM 连接成功（200 响应 + 模型列表）
#[tokio::test]
async fn test_llm_connection_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("Authorization", "Bearer test-api-key"))
        .and(header("X-Correlation-Id", "test-correlation-id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                { "id": "gpt-4" },
                { "id": "gpt-3.5-turbo" },
                { "id": "claude-3" }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = llm_client::test_connection(
        &client,
        &mock_server.uri(),
        "test-api-key",
        "siliconflow",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.message.contains("连接成功"));
    assert!(result.message.contains("3"));
    assert!(result.models.is_some());
    let models = result.models.unwrap();
    assert_eq!(models.len(), 3);
    assert!(models.contains(&"gpt-4".to_string()));
}

/// 测试 LLM 连接 401 未授权
#[tokio::test]
async fn test_llm_connection_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "message": "Invalid API key",
                "type": "invalid_request_error"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = llm_client::test_connection(
        &client,
        &mock_server.uri(),
        "invalid-key",
        "siliconflow",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        llm_client::LlmConnectionError::InvalidCredentials
    ));
}

/// 测试 LLM 连接无效 provider
#[tokio::test]
async fn test_llm_connection_invalid_provider() {
    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = llm_client::test_connection(
        &client,
        "https://api.example.com",
        "test-key",
        "invalid-provider",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        llm_client::LlmConnectionError::ValidationError(_)
    ));
}

/// 测试 provider 验证函数
#[test]
fn test_validate_provider() {
    assert!(llm_client::validate_provider("siliconflow").is_ok());
    assert!(llm_client::validate_provider("modelscope").is_ok());
    assert!(llm_client::validate_provider("invalid").is_err());
    assert!(llm_client::validate_provider("").is_err());
}

/// 测试连接超时处理（AC #4: NFR23 60s 超时）
///
/// 注意：此测试使用短超时的自定义客户端来验证超时逻辑
/// 实际生产环境使用 60s 超时
#[tokio::test]
async fn test_connection_timeout() {
    use reqwest::Client;
    use std::time::Duration;

    let mock_server = MockServer::start().await;

    // 配置 mock 延迟响应（超过客户端超时时间）
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(
            ResponseTemplate::new(200).set_delay(Duration::from_secs(5)), // 延迟 5 秒
        )
        .mount(&mock_server)
        .await;

    // 创建短超时客户端用于测试（1秒超时）
    let short_timeout_client = Client::builder()
        .timeout(Duration::from_secs(1))
        .connect_timeout(Duration::from_secs(1))
        .build()
        .expect("创建测试客户端失败");

    let result = llm_client::test_connection(
        &short_timeout_client,
        &mock_server.uri(),
        "test-key",
        "siliconflow",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, llm_client::LlmConnectionError::Timeout),
        "预期超时错误，实际: {:?}",
        err
    );
}

/// 测试错误消息截断（防止信息泄露）
#[tokio::test]
async fn test_error_body_truncation() {
    let mock_server = MockServer::start().await;

    // 创建一个很长的错误响应
    let long_error = "x".repeat(3000);

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(500).set_body_string(&long_error))
        .mount(&mock_server)
        .await;

    let client = create_http_client().expect("创建 HTTP 客户端失败");
    let result = llm_client::test_connection(
        &client,
        &mock_server.uri(),
        "test-key",
        "siliconflow",
        "test-correlation-id",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        llm_client::LlmConnectionError::UpstreamError(msg) => {
            // 确保错误消息被截断
            assert!(msg.len() < 2000, "错误消息应被截断");
            assert!(msg.contains("截断"), "应包含截断提示");
        }
        _ => panic!("预期 UpstreamError"),
    }
}
