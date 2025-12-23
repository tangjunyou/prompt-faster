//! HTTP 客户端封装
//! 统一的 HTTP 客户端配置，用于外部 API 调用

use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

/// HTTP 客户端创建错误
#[derive(Debug, Error)]
#[error("创建 HTTP 客户端失败: {0}")]
pub struct HttpClientError(#[from] reqwest::Error);

/// 创建配置好的 HTTP 客户端
///
/// 配置：
/// - 总超时 60 秒 (NFR23)
/// - 连接超时 10 秒
///
/// # Returns
/// * `Ok(Client)` - 成功创建的客户端
/// * `Err(HttpClientError)` - 创建失败（如 TLS 初始化失败）
pub fn create_http_client() -> Result<Client, HttpClientError> {
    Client::builder()
        .timeout(Duration::from_secs(60)) // NFR23: 总超时 60s
        .connect_timeout(Duration::from_secs(10)) // 连接超时 10s
        .build()
        .map_err(HttpClientError)
}

/// 截断错误消息体
///
/// 防止上游错误信息过长或泄露敏感信息
/// 最大保留 1KB
pub fn truncate_error_body(body: &str) -> String {
    const MAX_LEN: usize = 1024;
    if body.len() <= MAX_LEN {
        body.to_string()
    } else {
        format!("{}... (截断，原长度 {} 字节)", &body[..MAX_LEN], body.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_http_client_success() {
        let client = create_http_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_truncate_error_body_short() {
        let short = "短错误消息";
        assert_eq!(truncate_error_body(short), short);
    }

    #[test]
    fn test_truncate_error_body_long() {
        let long = "x".repeat(2000);
        let result = truncate_error_body(&long);
        assert!(result.len() < 2000);
        assert!(result.contains("截断"));
    }
}
