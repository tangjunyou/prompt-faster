//! 离线状态检查中间件

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::api::response::ApiResponse;
use crate::infra::external::connectivity::check_connectivity_status;
use crate::domain::models::ConnectivityStatus;
use crate::shared::error_codes;

/// 离线状态拦截（仅对需要网络的接口使用）
pub async fn connectivity_middleware(request: Request, next: Next) -> Response {
    let status = check_connectivity_status().await;
    if matches!(status, ConnectivityStatus::Offline) {
        return ApiResponse::<()>::err(
            StatusCode::SERVICE_UNAVAILABLE,
            error_codes::CONNECTIVITY_OFFLINE,
            "当前离线，部分功能受限",
        )
        .into_response();
    }
    next.run(request).await
}
