//! 任务历史聚合 API 路由

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use axum::{Router, routing::get};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::domain::models::{
    Actor, BranchInfo, CheckpointListResponse, CheckpointSummary, EventType, HistoryEvent,
    HistoryEventFilter, HistoryEventResponse, HistoryExportData, IterationExportEntry,
    TaskExportMeta, TaskHistoryResponse, TimelineResponse,
};
use crate::domain::types::{IterationArtifacts, unix_ms_to_iso8601};
use crate::infra::db::repositories::{
    CheckpointRepo, CheckpointRepoError, HistoryEventRepo, HistoryEventRepoError, IterationRepo,
    IterationRepoError, OptimizationTaskRepo,
};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, IntoParams)]
pub struct TaskHistoryQuery {
    /// 迭代历史最大返回条数（可选，默认 100，最大 100）
    pub iterations_limit: Option<i32>,
    /// Checkpoint 最大返回条数（可选，默认 20，最大 100）
    pub checkpoints_limit: Option<u32>,
    /// Checkpoint 偏移量（可选，默认 0）
    pub checkpoints_offset: Option<u32>,
    /// 是否包含已归档的 Checkpoint（默认 false）
    pub include_archived: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct HistoryEventQuery {
    /// 事件类型（逗号分隔，snake_case）
    pub event_types: Option<String>,
    /// 操作者（system/user）
    pub actor: Option<String>,
    /// 迭代范围下限（包含）
    pub iteration_min: Option<u32>,
    /// 迭代范围上限（包含）
    pub iteration_max: Option<u32>,
    /// 时间范围起点（Unix ms）
    pub time_start: Option<i64>,
    /// 时间范围终点（Unix ms）
    pub time_end: Option<i64>,
    /// 最大返回条数（默认 50，最大 100）
    pub limit: Option<usize>,
    /// 偏移量（默认 0，最大 10000）
    pub offset: Option<usize>,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn current_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn parse_event_types(raw: &Option<String>) -> Result<Option<Vec<EventType>>, String> {
    let Some(raw) = raw.as_ref() else {
        return Ok(None);
    };
    let mut out = Vec::new();
    for item in raw.split(',') {
        let value = item.trim();
        if value.is_empty() {
            continue;
        }
        let parsed = value
            .parse::<EventType>()
            .map_err(|_| format!("不支持的 event_types 值: {value}"))?;
        out.push(parsed);
    }
    if out.is_empty() {
        Ok(None)
    } else {
        Ok(Some(out))
    }
}

fn parse_actor(raw: &Option<String>) -> Result<Option<Actor>, String> {
    let Some(raw) = raw.as_ref() else {
        return Ok(None);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse::<Actor>()
        .map(Some)
        .map_err(|_| format!("不支持的 actor 值: {value}"))
}

fn build_history_event_filter(query: &HistoryEventQuery) -> Result<HistoryEventFilter, String> {
    if let (Some(min), Some(max)) = (query.iteration_min, query.iteration_max) {
        if min > max {
            return Err("iteration_min 不能大于 iteration_max".to_string());
        }
    }
    if let (Some(start), Some(end)) = (query.time_start, query.time_end) {
        if start > end {
            return Err("time_start 不能大于 time_end".to_string());
        }
    }

    Ok(HistoryEventFilter {
        event_types: parse_event_types(&query.event_types)?,
        actor: parse_actor(&query.actor)?,
        iteration_min: query.iteration_min,
        iteration_max: query.iteration_max,
        time_start: query.time_start,
        time_end: query.time_end,
    })
}

fn normalize_limit_offset(query: &HistoryEventQuery) -> Result<(usize, usize), String> {
    let limit = query.limit.unwrap_or(50);
    if limit == 0 || limit > 100 {
        return Err("limit 必须在 1-100 范围内".to_string());
    }
    let offset = query.offset.unwrap_or(0);
    if offset > 10_000 {
        return Err("offset 不能超过 10000".to_string());
    }
    Ok((limit, offset))
}

/// 获取任务历史聚合（迭代历史 + 回滚候选）
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/history",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        TaskHistoryQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<TaskHistoryResponse>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "history"
)]
pub(crate) async fn get_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<TaskHistoryQuery>,
    current_user: CurrentUser,
) -> ApiResponse<TaskHistoryResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "history:aggregate",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询任务历史聚合"
    );

    if OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id)
        .await
        .is_err()
    {
        return ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        );
    }

    let iterations_limit = query
        .iterations_limit
        .filter(|value| *value > 0)
        .map(|value| value.min(100));
    let checkpoints_limit = query.checkpoints_limit.unwrap_or(20).min(100) as usize;
    let checkpoints_offset = query.checkpoints_offset.unwrap_or(0) as usize;
    let include_archived = query.include_archived.unwrap_or(false);

    let iterations = match IterationRepo::list_by_task_id(
        &state.db,
        user_id,
        &task_id,
        iterations_limit,
    )
    .await
    {
        Ok(list) => list,
        Err(IterationRepoError::TaskNotFoundOrForbidden) => {
            return ApiResponse::err(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "任务不存在或无权访问",
            );
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询迭代历史失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询迭代历史失败",
            );
        }
    };

    let total = match CheckpointRepo::count_checkpoints_by_task_with_archived(
        &state.db,
        &task_id,
        include_archived,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 总数失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            );
        }
    };

    let current_branch_id =
        match CheckpointRepo::get_latest_active_branch_id(&state.db, &task_id).await {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询 checkpoint 分支失败"
                );
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询 checkpoint 失败",
                );
            }
        };

    let checkpoints = match CheckpointRepo::list_checkpoint_summaries(
        &state.db,
        &task_id,
        include_archived,
        checkpoints_limit,
        checkpoints_offset,
    )
    .await
    {
        Ok(list) => list,
        Err(CheckpointRepoError::Database(err)) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 列表失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            );
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 列表失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询 checkpoint 失败",
            );
        }
    };

    ApiResponse::ok(TaskHistoryResponse {
        iterations,
        checkpoints: CheckpointListResponse {
            checkpoints,
            total,
            current_branch_id,
        },
    })
}

/// 获取历史事件列表
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/history/events",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        HistoryEventQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<HistoryEventResponse>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "history"
)]
pub(crate) async fn list_history_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<HistoryEventQuery>,
    current_user: CurrentUser,
) -> ApiResponse<HistoryEventResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "history:events",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询历史事件列表"
    );

    if OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id)
        .await
        .is_err()
    {
        return ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        );
    }

    let filter = match build_history_event_filter(&query) {
        Ok(filter) => filter,
        Err(msg) => {
            return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg);
        }
    };
    let (limit, offset) = match normalize_limit_offset(&query) {
        Ok(value) => value,
        Err(msg) => {
            return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg);
        }
    };

    let events =
        match HistoryEventRepo::list_events(&state.db, &task_id, &filter, limit, offset).await {
            Ok(list) => list,
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询历史事件失败"
                );
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询历史事件失败",
                );
            }
        };

    let total = match HistoryEventRepo::count_events(&state.db, &task_id, &filter).await {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询历史事件总数失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询历史事件失败",
            );
        }
    };

    let has_more = offset + events.len() < total as usize;

    ApiResponse::ok(HistoryEventResponse {
        events,
        total,
        has_more,
    })
}

/// 获取时间线视图
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/history/timeline",
    params(
        ("task_id" = String, Path, description = "优化任务 ID"),
        HistoryEventQuery
    ),
    responses(
        (status = 200, description = "查询成功", body = ApiSuccess<TimelineResponse>),
        (status = 400, description = "参数错误"),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "history"
)]
pub(crate) async fn get_history_timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Query(query): Query<HistoryEventQuery>,
    current_user: CurrentUser,
) -> ApiResponse<TimelineResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "history:timeline",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "查询时间线视图"
    );

    if OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id)
        .await
        .is_err()
    {
        return ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::FORBIDDEN,
            "任务不存在或无权访问",
        );
    }

    let filter = match build_history_event_filter(&query) {
        Ok(filter) => filter,
        Err(msg) => {
            return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg);
        }
    };
    let (limit, offset) = match normalize_limit_offset(&query) {
        Ok(value) => value,
        Err(msg) => {
            return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, msg);
        }
    };

    let entries =
        match HistoryEventRepo::list_timeline_entries(&state.db, &task_id, &filter, limit, offset)
            .await
        {
            Ok(list) => list,
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "查询时间线失败"
                );
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询时间线失败",
                );
            }
        };

    let total = match HistoryEventRepo::count_timeline_entries(&state.db, &task_id, &filter).await {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询时间线总数失败"
            );
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询时间线失败",
            );
        }
    };

    let has_more = offset + entries.len() < total as usize;

    ApiResponse::ok(TimelineResponse {
        entries,
        total,
        has_more,
    })
}

/// 导出完整历史记录
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/history/export",
    params(
        ("task_id" = String, Path, description = "优化任务 ID")
    ),
    responses(
        (status = 200, description = "导出成功", body = ApiSuccess<HistoryExportData>),
        (status = 401, description = "未授权"),
        (status = 403, description = "无权访问该任务"),
        (status = 500, description = "服务器错误")
    ),
    tag = "history"
)]
pub(crate) async fn export_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    current_user: CurrentUser,
) -> impl IntoResponse {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %task_id,
        action = "history:export",
        iteration_state = "read_only",
        prev_state = "N/A",
        new_state = "N/A",
        timestamp = current_unix_ms(),
        "导出历史记录"
    );

    let task = match OptimizationTaskRepo::find_by_id_for_user(&state.db, user_id, &task_id).await {
        Ok(task) => task,
        Err(_) => {
            return ApiResponse::<HistoryExportData>::err(
                StatusCode::FORBIDDEN,
                error_codes::FORBIDDEN,
                "任务不存在或无权访问",
            )
            .into_response();
        }
    };

    let task_meta = TaskExportMeta {
        id: task.id.clone(),
        name: task.name.clone(),
        status: format!("{:?}", task.status).to_lowercase(),
        created_at: unix_ms_to_iso8601(task.created_at),
        updated_at: unix_ms_to_iso8601(task.updated_at),
    };

    let rule_system_map = match load_latest_rule_system_by_iteration(&state.db, &task_id).await {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 rule_system 失败"
            );
            return ApiResponse::<HistoryExportData>::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "导出历史失败",
            )
            .into_response();
        }
    };

    let iterations = match load_iteration_exports(&state.db, &task_id, &rule_system_map).await {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询迭代记录失败"
            );
            return ApiResponse::<HistoryExportData>::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "导出历史失败",
            )
            .into_response();
        }
    };

    let checkpoint_total =
        match CheckpointRepo::count_checkpoints_by_task_with_archived(&state.db, &task_id, true)
            .await
        {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "统计 checkpoint 总数失败"
                );
                return ApiResponse::<HistoryExportData>::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "导出历史失败",
                )
                .into_response();
            }
        };

    let checkpoints = match fetch_all_checkpoint_summaries(&state.db, &task_id).await {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询 checkpoint 列表失败"
            );
            return ApiResponse::<HistoryExportData>::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "导出历史失败",
            )
            .into_response();
        }
    };

    let event_total =
        match HistoryEventRepo::count_events(&state.db, &task_id, &HistoryEventFilter::default())
            .await
        {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    correlation_id = %correlation_id,
                    error = %err,
                    "统计历史事件总数失败"
                );
                return ApiResponse::<HistoryExportData>::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "导出历史失败",
                )
                .into_response();
            }
        };

    let events = match fetch_all_history_events(&state.db, &task_id).await {
        Ok(value) => value,
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "查询历史事件失败"
            );
            return ApiResponse::<HistoryExportData>::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "导出历史失败",
            )
            .into_response();
        }
    };

    let branches = summarize_branches_from_checkpoints(&checkpoints);
    let truncated = false;

    let export = HistoryExportData {
        task: task_meta,
        iterations,
        checkpoints,
        events,
        branches,
        truncated,
        event_total,
        checkpoint_total,
        export_limit: 0,
        exported_at: unix_ms_to_iso8601(current_unix_ms()),
    };

    let filename = format!(
        "{}_history_{}.json",
        task.name.replace(' ', "_"),
        current_unix_ms()
    );
    let mut export_headers = HeaderMap::new();
    export_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    export_headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).unwrap(),
    );

    (export_headers, ApiResponse::ok(export)).into_response()
}

#[derive(Debug, sqlx::FromRow)]
struct IterationExportRow {
    round: i32,
    status: String,
    pass_rate: f64,
    created_at: i64,
    artifacts: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct CheckpointRuleRow {
    iteration: i64,
    rule_system: String,
}

async fn load_latest_rule_system_by_iteration(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<HashMap<u32, String>, sqlx::Error> {
    let rows: Vec<CheckpointRuleRow> = sqlx::query_as(
        r#"
        SELECT iteration, rule_system
        FROM checkpoints
        WHERE task_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    let mut out = HashMap::new();
    for row in rows {
        let iteration = row.iteration.max(0) as u32;
        out.entry(iteration).or_insert(row.rule_system);
    }
    Ok(out)
}

async fn load_iteration_exports(
    pool: &sqlx::SqlitePool,
    task_id: &str,
    rule_system_map: &HashMap<u32, String>,
) -> Result<Vec<IterationExportEntry>, sqlx::Error> {
    let rows: Vec<IterationExportRow> = sqlx::query_as(
        r#"
        SELECT round, status, pass_rate, created_at, artifacts
        FROM iterations
        WHERE task_id = ?
        ORDER BY round ASC
        "#,
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let artifacts = parse_iteration_artifacts(&row.artifacts);
        let prompt = extract_prompt_from_artifacts(&artifacts);
        let iteration = row.round.max(0) as u32;
        out.push(IterationExportEntry {
            iteration,
            prompt,
            rule_system: rule_system_map.get(&iteration).cloned(),
            pass_rate: Some(row.pass_rate),
            status: row.status,
            created_at: unix_ms_to_iso8601(row.created_at),
        });
    }
    Ok(out)
}

async fn fetch_all_checkpoint_summaries(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<Vec<CheckpointSummary>, CheckpointRepoError> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    let limit = 200usize;
    loop {
        let batch =
            CheckpointRepo::list_checkpoint_summaries(pool, task_id, true, limit, offset).await?;
        if batch.is_empty() {
            break;
        }
        offset += batch.len();
        out.extend(batch);
    }
    Ok(out)
}

async fn fetch_all_history_events(
    pool: &sqlx::SqlitePool,
    task_id: &str,
) -> Result<Vec<HistoryEvent>, HistoryEventRepoError> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    let limit = 200usize;
    let filter = HistoryEventFilter::default();
    loop {
        let batch = HistoryEventRepo::list_events(pool, task_id, &filter, limit, offset).await?;
        if batch.is_empty() {
            break;
        }
        offset += batch.len();
        out.extend(batch);
    }
    Ok(out)
}

fn summarize_branches_from_checkpoints(checkpoints: &[CheckpointSummary]) -> Vec<BranchInfo> {
    let mut branch_map: HashMap<String, BranchInfo> = HashMap::new();
    let mut checkpoint_to_branch: HashMap<String, String> = HashMap::new();

    for checkpoint in checkpoints {
        checkpoint_to_branch.insert(checkpoint.id.clone(), checkpoint.branch_id.clone());
    }

    for checkpoint in checkpoints {
        let entry = branch_map
            .entry(checkpoint.branch_id.clone())
            .or_insert(BranchInfo {
                branch_id: checkpoint.branch_id.clone(),
                parent_branch_id: None,
                created_at: checkpoint.created_at.clone(),
                checkpoint_count: 0,
            });
        entry.checkpoint_count = entry.checkpoint_count.saturating_add(1);
        if checkpoint.created_at < entry.created_at {
            entry.created_at = checkpoint.created_at.clone();
        }

        if entry.parent_branch_id.is_none() {
            if let Some(parent_id) = &checkpoint.parent_id {
                if let Some(parent_branch) = checkpoint_to_branch.get(parent_id) {
                    if parent_branch != &checkpoint.branch_id {
                        entry.parent_branch_id = Some(parent_branch.clone());
                    }
                }
            }
        }
    }

    let mut out: Vec<BranchInfo> = branch_map.into_values().collect();
    out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    out
}

fn parse_iteration_artifacts(raw: &Option<String>) -> IterationArtifacts {
    match raw.as_ref() {
        Some(value) => serde_json::from_str(value).unwrap_or_default(),
        None => IterationArtifacts::default(),
    }
}

fn extract_prompt_from_artifacts(artifacts: &IterationArtifacts) -> Option<String> {
    if let Some(best) = artifacts.candidate_prompts.iter().find(|p| p.is_best) {
        return Some(best.content.clone());
    }
    artifacts
        .candidate_prompts
        .iter()
        .find(|p| p.id == "current")
        .map(|p| p.content.clone())
}

/// 创建任务历史聚合路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_history))
        .route("/events", get(list_history_events))
        .route("/timeline", get(get_history_timeline))
        .route("/export", get(export_history))
}
