//! 鉴权中间件
//! 从 Authorization 头解析 Bearer token 并验证会话

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::api::middleware::session::{SessionStore, UnlockContext};

/// 当前用户信息
/// 通过 request extensions 传递给后续 handler
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// 用户 ID
    pub user_id: String,
    /// 解锁上下文（用于 API Key 加解密）
    pub unlock_context: Option<UnlockContext>,
}

/// 从 Authorization 头提取 Bearer token
fn extract_bearer_token(auth_header: Option<&str>) -> Option<&str> {
    auth_header
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| &h[7..])
}

/// 鉴权中间件
///
/// 从 `Authorization: Bearer <token>` 头解析 token，
/// 验证会话有效性，并将 `CurrentUser` 注入 request extensions。
///
/// # 失败处理
/// - 缺少或无效的 Authorization 头返回 401
/// - 会话不存在或已过期返回 401
/// - 错误响应统一使用 `ApiResponse` 格式（AC #3）
pub async fn auth_middleware(
    State(session_store): State<SessionStore>,
    mut request: Request,
    next: Next,
) -> Response {
    // 提取 Authorization 头
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    // 解析 Bearer token
    let token = match extract_bearer_token(auth_header) {
        Some(t) => t,
        None => {
            return unauthorized_response();
        }
    };

    // 验证会话
    let session = match session_store.validate_session(token).await {
        Some(s) => s,
        None => {
            return unauthorized_response();
        }
    };

    // 注入 CurrentUser 到 request extensions
    let current_user = CurrentUser {
        user_id: session.user_id,
        unlock_context: session.unlock_context,
    };
    request.extensions_mut().insert(current_user);

    next.run(request).await
}

/// 返回统一的 401 错误响应
///
/// # 注意
/// 不暴露具体失败原因（AC #3）
fn unauthorized_response() -> Response {
    crate::api::response::unauthorized::<()>().into_response()
}

/// 从 request extensions 提取 CurrentUser 的提取器
pub mod extractor {
    use axum::{
        extract::FromRequestParts,
        http::request::Parts,
        response::{IntoResponse, Response},
    };

    use super::CurrentUser;

    impl<S> FromRequestParts<S> for CurrentUser
    where
        S: Send + Sync,
    {
        type Rejection = Response;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            parts
                .extensions
                .get::<CurrentUser>()
                .cloned()
                .ok_or_else(|| crate::api::response::unauthorized::<()>().into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[test]
    fn test_extract_bearer_token_valid() {
        let token = extract_bearer_token(Some("Bearer abc123"));
        assert_eq!(token, Some("abc123"));
    }

    #[test]
    fn test_extract_bearer_token_missing_prefix() {
        let token = extract_bearer_token(Some("abc123"));
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_none() {
        let token = extract_bearer_token(None);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let token = extract_bearer_token(Some("Basic abc123"));
        assert_eq!(token, None);
    }

    #[tokio::test]
    async fn test_unauthorized_response_is_apiresponse_error_shape() {
        let response = unauthorized_response();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("读取响应 body 失败")
            .to_bytes();

        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("解析 JSON 失败");
        assert!(json.get("data").is_none());
        assert!(json.get("error").is_some());
        assert_eq!(json["error"]["code"], "UNAUTHORIZED");
    }
}
