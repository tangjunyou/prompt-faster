//! correlationId 中间件
//! 确保全链路透传 (AR2)

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use tracing::Span;

/// correlationId 请求头名称
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

/// correlationId 中间件
pub async fn correlation_id_middleware(mut request: Request, next: Next) -> Response {
    // 从请求头获取或生成新的 correlationId
    let correlation_id = request
        .headers()
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 添加到 tracing span
    Span::current().record("correlation_id", &correlation_id);

    // 确保请求头包含 correlationId
    request.headers_mut().insert(
        CORRELATION_ID_HEADER,
        HeaderValue::from_str(&correlation_id).unwrap(),
    );

    // 执行后续处理
    let mut response = next.run(request).await;

    // 在响应头中也添加 correlationId
    response.headers_mut().insert(
        CORRELATION_ID_HEADER,
        HeaderValue::from_str(&correlation_id).unwrap(),
    );

    response
}
