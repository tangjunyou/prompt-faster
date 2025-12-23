//! URL 验证工具
//! 防止 SSRF 攻击，确保 base_url 安全

use std::net::IpAddr;

/// URL 验证错误
#[derive(Debug, Clone, PartialEq)]
pub enum UrlValidationError {
    /// URL 为空
    Empty,
    /// URL 格式无效
    InvalidFormat,
    /// 必须使用 HTTPS
    HttpsRequired,
    /// 禁止访问私有网络
    PrivateNetworkForbidden,
    /// 禁止访问本地地址
    LocalhostForbidden,
}

impl std::fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "URL 不能为空"),
            Self::InvalidFormat => write!(f, "URL 格式无效"),
            Self::HttpsRequired => write!(f, "必须使用 HTTPS 协议"),
            Self::PrivateNetworkForbidden => write!(f, "禁止访问私有网络地址"),
            Self::LocalhostForbidden => write!(f, "禁止访问本地地址"),
        }
    }
}

impl std::error::Error for UrlValidationError {}

/// 验证 base_url 是否安全（防 SSRF）
///
/// 规则：
/// - 不能为空
/// - 必须是有效 URL 格式
/// - 必须使用 HTTPS（生产环境）
/// - 禁止 localhost、127.0.0.1、::1
/// - 禁止私有网络地址（10.x.x.x、172.16-31.x.x、192.168.x.x）
/// - 禁止 link-local 地址（169.254.x.x）
///
/// # Arguments
/// * `base_url` - 要验证的 URL
/// * `allow_http` - 是否允许 HTTP（仅开发环境）
pub fn validate_base_url(base_url: &str, allow_http: bool) -> Result<(), UrlValidationError> {
    let trimmed = base_url.trim();

    // 检查空值
    if trimmed.is_empty() {
        return Err(UrlValidationError::Empty);
    }

    // 解析 URL
    let url = url::Url::parse(trimmed).map_err(|_| UrlValidationError::InvalidFormat)?;

    // 检查协议
    let scheme = url.scheme();
    if scheme != "https" && !(allow_http && scheme == "http") {
        return Err(UrlValidationError::HttpsRequired);
    }

    // 获取 host
    let host = url.host_str().ok_or(UrlValidationError::InvalidFormat)?;

    // 检查 localhost
    if is_localhost(host) {
        return Err(UrlValidationError::LocalhostForbidden);
    }

    // 检查私有网络
    if is_private_network(host) {
        return Err(UrlValidationError::PrivateNetworkForbidden);
    }

    Ok(())
}

/// 检查是否为 localhost
fn is_localhost(host: &str) -> bool {
    let lower = host.to_lowercase();
    lower == "localhost"
        || lower == "127.0.0.1"
        || lower == "::1"
        || lower == "[::1]"
        || lower.ends_with(".localhost")
        || lower.ends_with(".local")
}

/// 检查是否为私有网络地址
fn is_private_network(host: &str) -> bool {
    // 尝试解析为 IP 地址
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // 10.0.0.0/8
                octets[0] == 10
                // 172.16.0.0/12
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                // 192.168.0.0/16
                || (octets[0] == 192 && octets[1] == 168)
                // 169.254.0.0/16 (link-local)
                || (octets[0] == 169 && octets[1] == 254)
                // 0.0.0.0
                || octets == [0, 0, 0, 0]
            }
            IpAddr::V6(ipv6) => {
                // fe80::/10 (link-local)
                ipv6.segments()[0] & 0xffc0 == 0xfe80
                // fc00::/7 (unique local)
                || ipv6.segments()[0] & 0xfe00 == 0xfc00
            }
        };
    }

    false
}

/// 验证 API Key 是否有效（非空）
pub fn validate_api_key(api_key: &str) -> Result<(), &'static str> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err("API Key 不能为空");
    }
    if trimmed.len() < 8 {
        return Err("API Key 长度过短");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        assert!(validate_base_url("https://api.example.com", false).is_ok());
        assert!(validate_base_url("https://api.siliconflow.cn", false).is_ok());
        assert!(validate_base_url("https://api.dify.ai/v1", false).is_ok());
    }

    #[test]
    fn test_http_rejected_in_production() {
        assert_eq!(
            validate_base_url("http://api.example.com", false),
            Err(UrlValidationError::HttpsRequired)
        );
    }

    #[test]
    fn test_http_allowed_in_dev() {
        assert!(validate_base_url("http://api.example.com", true).is_ok());
    }

    #[test]
    fn test_localhost_rejected() {
        assert_eq!(
            validate_base_url("https://localhost:8080", false),
            Err(UrlValidationError::LocalhostForbidden)
        );
        assert_eq!(
            validate_base_url("https://127.0.0.1:8080", false),
            Err(UrlValidationError::LocalhostForbidden)
        );
        assert_eq!(
            validate_base_url("https://[::1]:8080", false),
            Err(UrlValidationError::LocalhostForbidden)
        );
    }

    #[test]
    fn test_private_network_rejected() {
        assert_eq!(
            validate_base_url("https://10.0.0.1:8080", false),
            Err(UrlValidationError::PrivateNetworkForbidden)
        );
        assert_eq!(
            validate_base_url("https://172.16.0.1:8080", false),
            Err(UrlValidationError::PrivateNetworkForbidden)
        );
        assert_eq!(
            validate_base_url("https://192.168.1.1:8080", false),
            Err(UrlValidationError::PrivateNetworkForbidden)
        );
    }

    #[test]
    fn test_empty_url() {
        assert_eq!(validate_base_url("", false), Err(UrlValidationError::Empty));
        assert_eq!(
            validate_base_url("   ", false),
            Err(UrlValidationError::Empty)
        );
    }

    #[test]
    fn test_invalid_format() {
        assert_eq!(
            validate_base_url("not-a-url", false),
            Err(UrlValidationError::InvalidFormat)
        );
    }

    #[test]
    fn test_validate_api_key() {
        assert!(validate_api_key("sk-1234567890").is_ok());
        assert!(validate_api_key("").is_err());
        assert!(validate_api_key("   ").is_err());
        assert!(validate_api_key("short").is_err());
    }
}
