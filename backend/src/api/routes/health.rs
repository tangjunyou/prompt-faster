//! 健康检查路由

use axum::{routing::get, Json, Router};
use serde::Serialize;

/// 健康检查响应
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp_ms: i64,
}

/// 健康检查处理器
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
    })
}

/// 创建健康检查路由
pub fn router() -> Router {
    Router::new().route("/health", get(health_check))
}
