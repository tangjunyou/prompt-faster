//! correlationId 中间件
//! 确保全链路透传 (AR2)

use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::{info_span, Instrument};

/// correlationId 请求头名称
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// correlationId 中间件
/// 创建包含 correlation_id 的 span，确保日志可追踪
pub async fn correlation_id_middleware(mut request: Request, next: Next) -> Response {
    // 从请求头获取或生成新的 correlationId
    let correlation_id = request
        .headers()
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 确保请求头包含 correlationId
    request.headers_mut().insert(
        CORRELATION_ID_HEADER,
        HeaderValue::from_str(&correlation_id).unwrap(),
    );

    // 创建包含 correlation_id 的 span 并在其中执行请求
    let span = info_span!(
        "request",
        correlation_id = %correlation_id,
        method = %request.method(),
        uri = %request.uri(),
    );

    // 在 span 中执行后续处理
    let mut response = async move {
        next.run(request).await
    }
    .instrument(span)
    .await;

    // 在响应头中也添加 correlationId
    response.headers_mut().insert(
        CORRELATION_ID_HEADER,
        HeaderValue::from_str(&correlation_id).unwrap(),
    );

    response
}
