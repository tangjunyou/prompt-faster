//! 日志脱敏工具
//! 确保敏感信息（如 API Key）不会出现在日志中 (NFR11)

/// 脱敏 API Key
///
/// 规则：
/// - 长度 ≤ 8 时返回 `****`
/// - 长度 > 8 时返回 `前4位****后4位`
///
/// # Examples
/// ```
/// use prompt_faster::shared::log_sanitizer::sanitize_api_key;
///
/// assert_eq!(sanitize_api_key("short"), "****");
/// assert_eq!(sanitize_api_key("sk-1234567890abcdef"), "sk-1****cdef");
/// ```
pub fn sanitize_api_key(api_key: &str) -> String {
    let len = api_key.len();
    if len <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &api_key[..4], &api_key[len - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_short_key() {
        assert_eq!(sanitize_api_key(""), "****");
        assert_eq!(sanitize_api_key("abc"), "****");
        assert_eq!(sanitize_api_key("12345678"), "****");
    }

    #[test]
    fn test_sanitize_long_key() {
        assert_eq!(sanitize_api_key("123456789"), "1234****6789");
        assert_eq!(sanitize_api_key("sk-1234567890abcdef"), "sk-1****cdef");
        assert_eq!(
            sanitize_api_key("abcdefghijklmnopqrstuvwxyz"),
            "abcd****wxyz"
        );
    }

    #[test]
    fn test_sanitize_boundary() {
        // 恰好 9 个字符
        assert_eq!(sanitize_api_key("123456789"), "1234****6789");
    }
}
