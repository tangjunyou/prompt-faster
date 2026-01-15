use axum::Router;
use axum::routing::get;

use crate::api::response::ApiResponse;
use crate::api::state::AppState;
use crate::domain::models::{IterationStageDescriptor, all_iteration_stages};

/// 获取 IterationState 的阶段/口径映射（权威入口）。
///
/// GET /api/v1/meta/iteration-stages
pub(crate) async fn get_iteration_stages() -> ApiResponse<Vec<IterationStageDescriptor>> {
    ApiResponse::ok(all_iteration_stages())
}

pub fn router() -> Router<AppState> {
    Router::new().route("/iteration-stages", get(get_iteration_stages))
}
