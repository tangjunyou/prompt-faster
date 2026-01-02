use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::infra::db::repositories::{WorkspaceRepo, WorkspaceRepoError};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 创建工作区请求类型（前端类型别名）
#[allow(dead_code)]
#[derive(TS)]
#[ts(export_to = "api/")]
#[ts(type = "CreateWorkspaceRequest")]
pub struct WorkspaceCreateRequest;

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct WorkspaceResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

/// 工作区列表响应类型（前端类型别名）
#[allow(dead_code)]
#[derive(TS)]
#[ts(export_to = "api/")]
#[ts(type = "Array<WorkspaceResponse>")]
pub struct WorkspaceListResponse;

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct DeleteWorkspaceResponse {
    pub message: String,
}

fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn workspace_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::WORKSPACE_NOT_FOUND,
        "工作区不存在",
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/workspaces",
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 200, description = "创建成功", body = ApiSuccess<WorkspaceResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "workspaces"
)]
pub(crate) async fn create_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Json(req): Json<CreateWorkspaceRequest>,
) -> ApiResponse<WorkspaceResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    if req.name.trim().is_empty() {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "工作区名称不能为空",
        );
    }

    if req.name.len() > 128 {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "工作区名称不能超过 128 个字符",
        );
    }

    info!(correlation_id = %correlation_id, user_id = %user_id, "创建工作区");

    let workspace = match WorkspaceRepo::create(
        &state.db,
        user_id,
        &req.name,
        req.description.as_deref(),
    )
    .await
    {
        Ok(w) => w,
        Err(e) => {
            warn!(error = %e, "创建工作区失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "创建工作区失败",
            );
        }
    };

    ApiResponse::ok(WorkspaceResponse {
        id: workspace.id,
        name: workspace.name,
        description: workspace.description,
        created_at: workspace.created_at,
        updated_at: workspace.updated_at,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces",
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<Vec<WorkspaceResponse>>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "workspaces"
)]
pub(crate) async fn list_workspaces(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
) -> ApiResponse<Vec<WorkspaceResponse>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, "列出工作区");

    let workspaces = match WorkspaceRepo::find_all_by_user(&state.db, user_id).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!(error = %e, "查询工作区列表失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询工作区列表失败",
            );
        }
    };

    ApiResponse::ok(
        workspaces
            .into_iter()
            .map(|w| WorkspaceResponse {
                id: w.id,
                name: w.name,
                description: w.description,
                created_at: w.created_at,
                updated_at: w.updated_at,
            })
            .collect(),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{id}",
    params(
        ("id" = String, Path, description = "工作区 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<WorkspaceResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "workspaces"
)]
pub(crate) async fn get_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<WorkspaceResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "获取工作区");

    let workspace = match WorkspaceRepo::find_by_id(&state.db, &workspace_id, user_id).await {
        Ok(w) => w,
        Err(WorkspaceRepoError::NotFound) => return workspace_not_found(),
        Err(e) => {
            warn!(error = %e, "查询工作区失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询工作区失败",
            );
        }
    };

    ApiResponse::ok(WorkspaceResponse {
        id: workspace.id,
        name: workspace.name,
        description: workspace.description,
        created_at: workspace.created_at,
        updated_at: workspace.updated_at,
    })
}

#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{id}",
    params(
        ("id" = String, Path, description = "工作区 ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiSuccess<DeleteWorkspaceResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "workspaces"
)]
pub(crate) async fn delete_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<DeleteWorkspaceResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "删除工作区");

    match WorkspaceRepo::delete(&state.db, &workspace_id, user_id).await {
        Ok(true) => ApiResponse::ok(DeleteWorkspaceResponse {
            message: "删除成功".to_string(),
        }),
        Ok(false) => workspace_not_found(),
        Err(e) => {
            warn!(error = %e, "删除工作区失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "删除工作区失败",
            )
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_workspace).get(list_workspaces))
        .route("/{id}", get(get_workspace).delete(delete_workspace))
}
