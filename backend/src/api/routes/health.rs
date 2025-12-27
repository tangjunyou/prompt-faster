//! 健康检查路由

use axum::{Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::response::{ApiResponse, ApiSuccess};
use crate::shared::time::now_millis;

/// 健康检查响应
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    #[serde(rename = "timestampMs")]
    pub timestamp_ms: i64,
}

/// 健康检查处理器
/// 必须使用 ApiResponse<HealthResponse> 返回（Task 5.4）
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "服务正常", body = ApiSuccess<HealthResponse>)
    )
)]
pub(crate) async fn health_check() -> ApiResponse<HealthResponse> {
    ApiResponse::ok(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp_ms: now_millis(),
    })
}

/// 创建健康检查路由
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::<S>::new().route("/health", get(health_check))
}
