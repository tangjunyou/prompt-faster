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
use crate::api::routes::dify::DifyConfig;
use crate::api::state::AppState;
use crate::domain::models::TestCase;
use crate::infra::db::repositories::{
    TestSetRepo, TestSetRepoError, WorkspaceRepo, WorkspaceRepoError,
};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct SaveAsTemplateRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct TestSetTemplateResponse {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub cases: Vec<TestCase>,
    pub dify_config: Option<DifyConfig>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct TestSetTemplateListItemResponse {
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

fn template_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::TEST_SET_NOT_FOUND,
        "模板不存在",
    )
}

fn test_set_not_found<T: Serialize>() -> ApiResponse<T> {
    ApiResponse::err(
        StatusCode::NOT_FOUND,
        error_codes::TEST_SET_NOT_FOUND,
        "测试集不存在",
    )
}

fn parse_dify_config(dify_config_json: Option<String>) -> Result<Option<DifyConfig>, String> {
    let Some(json) = dify_config_json else {
        return Ok(None);
    };

    let config: DifyConfig = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(Some(config))
}

fn validate_template_name<T: Serialize>(name: &str) -> Result<(), ApiResponse<T>> {
    let name = name.trim();

    if name.is_empty() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "模板名称不能为空",
        ));
    }

    if name.chars().count() > 128 {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "模板名称不能超过 128 个字符",
        ));
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{workspace_id}/test-set-templates",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<Vec<TestSetTemplateListItemResponse>>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_set_templates"
)]
pub(crate) async fn list_test_set_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(workspace_id): Path<String>,
    current_user: CurrentUser,
) -> ApiResponse<Vec<TestSetTemplateListItemResponse>> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, "列出测试集模板");

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

    let list = match TestSetRepo::list_template_summaries_by_workspace_scoped(
        &state.db,
        user_id,
        &workspace_id,
    )
    .await
    {
        Ok(list) => list,
        Err(e) => {
            warn!(error = %e, "查询模板列表失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询模板列表失败",
            );
        }
    };

    ApiResponse::ok(
        list.into_iter()
            .map(|ts| TestSetTemplateListItemResponse {
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
    get,
    path = "/api/v1/workspaces/{workspace_id}/test-set-templates/{template_id}",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("template_id" = String, Path, description = "模板 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<TestSetTemplateResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区或模板不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_set_templates"
)]
pub(crate) async fn get_test_set_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, template_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<TestSetTemplateResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, template_id = %template_id, "获取测试集模板");

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

    let loaded = match TestSetRepo::find_template_by_id_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &template_id,
    )
    .await
    {
        Ok(ts) => ts,
        Err(TestSetRepoError::NotFound) => return template_not_found(),
        Err(e) => {
            warn!(error = %e, "查询模板失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询模板失败",
            );
        }
    };

    ApiResponse::ok(TestSetTemplateResponse {
        id: loaded.id,
        workspace_id: loaded.workspace_id,
        name: loaded.name,
        description: loaded.description,
        cases: loaded.cases,
        dify_config: match parse_dify_config(loaded.dify_config_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "解析 dify_config_json 失败");
                None
            }
        },
        created_at: loaded.created_at,
        updated_at: loaded.updated_at,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/save-as-template",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    request_body = SaveAsTemplateRequest,
    responses(
        (status = 200, description = "创建成功", body = ApiSuccess<TestSetTemplateResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "工作区或测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "test_set_templates"
)]
pub(crate) async fn save_as_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
    Json(req): Json<SaveAsTemplateRequest>,
) -> ApiResponse<TestSetTemplateResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    if let Err(err) = validate_template_name::<TestSetTemplateResponse>(&req.name) {
        return err;
    }

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "保存测试集为模板");

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

    let created = match TestSetRepo::create_template_from_test_set_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &test_set_id,
        req.name.trim(),
        req.description
            .as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty()),
    )
    .await
    {
        Ok(ts) => ts,
        Err(TestSetRepoError::NotFound) => return test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "创建模板失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "创建模板失败",
            );
        }
    };

    ApiResponse::ok(TestSetTemplateResponse {
        id: created.id,
        workspace_id: created.workspace_id,
        name: created.name,
        description: created.description,
        cases: created.cases,
        dify_config: match parse_dify_config(created.dify_config_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "解析 dify_config_json 失败");
                None
            }
        },
        created_at: created.created_at,
        updated_at: created.updated_at,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_test_set_templates))
        .route("/{template_id}", get(get_test_set_template))
}
