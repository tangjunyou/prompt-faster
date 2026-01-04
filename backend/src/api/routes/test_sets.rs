use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::domain::models::TestCase;
use crate::infra::db::repositories::{
    TestSetRepo, TestSetRepoError, WorkspaceRepo, WorkspaceRepoError,
};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct CreateTestSetRequest {
    pub name: String,
    pub description: Option<String>,
    pub cases: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct UpdateTestSetRequest {
    pub name: String,
    pub description: Option<String>,
    pub cases: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct TestSetResponse {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub cases: Vec<TestCase>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct TestSetListItemResponse {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    #[ts(type = "number")]
    pub cases_count: u32,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct DeleteTestSetResponse {
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

fn test_set_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::TEST_SET_NOT_FOUND,
        "测试集不存在",
    )
}

fn validate_name<T: Serialize>(name: &str) -> Result<(), ApiResponse<T>> {
    let name = name.trim();

    if name.is_empty() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "测试集名称不能为空",
        ));
    }

    if name.chars().count() > 128 {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "测试集名称不能超过 128 个字符",
        ));
    }

    Ok(())
}

fn parse_cases<T: Serialize>(cases: serde_json::Value) -> Result<Vec<TestCase>, ApiResponse<T>> {
    serde_json::from_value::<Vec<TestCase>>(cases).map_err(|e| {
        ApiResponse::err_with_details(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "cases 格式错误：必须是 TestCase 数组",
            serde_json::json!({ "error": e.to_string() }),
        )
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/test-sets",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<Vec<TestSetListItemResponse>>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_sets"
)]
pub(crate) async fn list_test_sets(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<Vec<TestSetListItemResponse>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "列出测试集");

    // Ensure workspace exists and belongs to user; otherwise return WORKSPACE_NOT_FOUND.
    match WorkspaceRepo::find_by_id(&state.db, &workspace_id, user_id).await {
        Ok(_) => {}
        Err(WorkspaceRepoError::NotFound) => return workspace_not_found(),
        Err(e) => {
            warn!(error = %e, "查询工作区失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询工作区失败",
            );
        }
    }

    let list = match TestSetRepo::list_summaries_by_workspace(&state.db, &workspace_id).await {
        Ok(list) => list,
        Err(e) => {
            warn!(error = %e, "查询测试集列表失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询测试集列表失败",
            );
        }
    };

    ApiResponse::ok(
        list.into_iter()
            .map(|ts| TestSetListItemResponse {
                id: ts.id,
                workspace_id: ts.workspace_id,
                name: ts.name,
                description: ts.description,
                cases_count: ts.cases_count,
                created_at: ts.created_at,
                updated_at: ts.updated_at,
            })
            .collect(),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/test-sets",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID")
    ),
    request_body = CreateTestSetRequest,
    responses(
        (status = 200, description = "创建成功", body = ApiSuccess<TestSetResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_sets"
)]
pub(crate) async fn create_test_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
    Json(req): Json<CreateTestSetRequest>,
) -> ApiResponse<TestSetResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    if let Err(err) = validate_name::<TestSetResponse>(&req.name) {
        return err;
    }

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "创建测试集");

    let cases = match parse_cases::<TestSetResponse>(req.cases) {
        Ok(c) => c,
        Err(e) => return e,
    };

    match WorkspaceRepo::find_by_id(&state.db, &workspace_id, user_id).await {
        Ok(_) => {}
        Err(WorkspaceRepoError::NotFound) => return workspace_not_found(),
        Err(e) => {
            warn!(error = %e, "查询工作区失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询工作区失败",
            );
        }
    }

    let created = match TestSetRepo::create(
        &state.db,
        &workspace_id,
        req.name.trim(),
        req.description.as_deref(),
        &cases,
    )
    .await
    {
        Ok(ts) => ts,
        Err(e) => {
            warn!(error = %e, "创建测试集失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "创建测试集失败",
            );
        }
    };

    ApiResponse::ok(TestSetResponse {
        id: created.id,
        workspace_id: created.workspace_id,
        name: created.name,
        description: created.description,
        cases: created.cases,
        created_at: created.created_at,
        updated_at: created.updated_at,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<TestSetResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_sets"
)]
pub(crate) async fn get_test_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<TestSetResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "获取测试集");

    let loaded =
        match TestSetRepo::find_by_id_scoped(&state.db, user_id, &workspace_id, &test_set_id).await
        {
            Ok(ts) => ts,
            Err(TestSetRepoError::NotFound) => return test_set_not_found(),
            Err(e) => {
                warn!(error = %e, "查询测试集失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询测试集失败",
                );
            }
        };

    ApiResponse::ok(TestSetResponse {
        id: loaded.id,
        workspace_id: loaded.workspace_id,
        name: loaded.name,
        description: loaded.description,
        cases: loaded.cases,
        created_at: loaded.created_at,
        updated_at: loaded.updated_at,
    })
}

#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    request_body = UpdateTestSetRequest,
    responses(
        (status = 200, description = "更新成功", body = ApiSuccess<TestSetResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_sets"
)]
pub(crate) async fn update_test_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
    Json(req): Json<UpdateTestSetRequest>,
) -> ApiResponse<TestSetResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    if let Err(err) = validate_name::<TestSetResponse>(&req.name) {
        return err;
    }

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "更新测试集");

    let cases = match parse_cases::<TestSetResponse>(req.cases) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let updated = match TestSetRepo::update_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &test_set_id,
        req.name.trim(),
        req.description.as_deref(),
        &cases,
    )
    .await
    {
        Ok(ts) => ts,
        Err(TestSetRepoError::NotFound) => return test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "更新测试集失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "更新测试集失败",
            );
        }
    };

    ApiResponse::ok(TestSetResponse {
        id: updated.id,
        workspace_id: updated.workspace_id,
        name: updated.name,
        description: updated.description,
        cases: updated.cases,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    })
}

#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiSuccess<DeleteTestSetResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_sets"
)]
pub(crate) async fn delete_test_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<DeleteTestSetResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "删除测试集");

    match TestSetRepo::delete_scoped(&state.db, user_id, &workspace_id, &test_set_id).await {
        Ok(true) => ApiResponse::ok(DeleteTestSetResponse {
            message: "删除成功".to_string(),
        }),
        Ok(false) => test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "删除测试集失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "删除测试集失败",
            )
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_test_sets).post(create_test_set))
        .route(
            "/{test_set_id}",
            get(get_test_set)
                .put(update_test_set)
                .delete(delete_test_set),
        )
}
