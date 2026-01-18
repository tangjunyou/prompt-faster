use axum::Router;
use axum::routing::get;
use serde::Serialize;
use utoipa::ToSchema;

use crate::api::response::ApiResponse;
use crate::api::state::AppState;
use crate::core::iteration_engine::checkpoint::{checkpoint_cache_defaults, checkpoint_cache_metrics};
use crate::domain::models::{IterationStageDescriptor, all_iteration_stages};

/// 获取 IterationState 的阶段/口径映射（权威入口）。
///
/// GET /api/v1/meta/iteration-stages
pub(crate) async fn get_iteration_stages() -> ApiResponse<Vec<IterationStageDescriptor>> {
    ApiResponse::ok(all_iteration_stages())
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointCacheMetricsResponse {
    pub total_tasks: usize,
    pub total_cached: usize,
    pub cache_limit: usize,
    pub alert_threshold: usize,
}

/// 获取 Checkpoint 缓存监控指标
///
/// GET /api/v1/meta/checkpoint-metrics
pub(crate) async fn get_checkpoint_metrics() -> ApiResponse<CheckpointCacheMetricsResponse> {
    let metrics = checkpoint_cache_metrics().await;
    let (cache_limit, alert_threshold) = checkpoint_cache_defaults();
    ApiResponse::ok(CheckpointCacheMetricsResponse {
        total_tasks: metrics.total_tasks,
        total_cached: metrics.total_cached,
        cache_limit,
        alert_threshold,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/iteration-stages", get(get_iteration_stages))
        .route("/checkpoint-metrics", get(get_checkpoint_metrics))
}
