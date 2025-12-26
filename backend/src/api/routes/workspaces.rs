use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::ApiResponse;
use crate::api::state::AppState;
use crate::infra::db::repositories::{WorkspaceRepo, WorkspaceRepoError};

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
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
    ApiResponse::err(StatusCode::NOT_FOUND, "WORKSPACE_NOT_FOUND", "工作区不存在")
}

async fn create_workspace(
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
            "VALIDATION_ERROR",
            "工作区名称不能为空",
        );
    }

    if req.name.len() > 128 {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR",
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
                "DATABASE_ERROR",
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

async fn list_workspaces(
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
                "DATABASE_ERROR",
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

async fn get_workspace(
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
                "DATABASE_ERROR",
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

async fn delete_workspace(
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
                "DATABASE_ERROR",
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
