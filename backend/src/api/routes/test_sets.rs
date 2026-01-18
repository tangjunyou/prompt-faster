use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router, middleware,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::instrument;
use tracing::{info, warn};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::api::middleware::{CurrentUser, connectivity_middleware};
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::routes::dify::{
    DifyBindingSource, DifyConfig, SaveDifyConfigRequest, SaveDifyConfigResponse,
};
use crate::api::routes::generic::{
    DeleteGenericConfigResponse, GenericConfig, GenericInputVariable, GenericValueType,
    SaveGenericConfigRequest, SaveGenericConfigResponse,
};
use crate::api::routes::test_set_templates;
use crate::api::state::AppState;
use crate::domain::models::TestCase;
use crate::infra::db::repositories::{CredentialRepo, CredentialRepoError, CredentialType};
use crate::infra::db::repositories::{
    TestSetRepo, TestSetRepoError, WorkspaceRepo, WorkspaceRepoError,
};
use crate::infra::external::api_key_manager::EncryptedApiKey;
use crate::infra::external::dify_client::{self, DifyVariablesResponse};
use crate::shared::error_codes;

#[derive(Debug, Deserialize, ToSchema, TS)]
#[ts(export_to = "api/")]
pub struct CreateTestSetRequest {
    pub name: String,
    pub description: Option<String>,
    pub cases: serde_json::Value,
    /// 可选：创建测试集时一并写入 Dify 配置（用于模板复制/创建态一次性落盘）
    pub dify_config: Option<SaveDifyConfigRequest>,
    /// 可选：创建测试集时一并写入通用变量配置（用于模板复制/创建态一次性落盘）
    pub generic_config: Option<SaveGenericConfigRequest>,
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
    pub dify_config: Option<DifyConfig>,
    pub generic_config: Option<GenericConfig>,
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

async fn ensure_workspace_exists<T: Serialize>(
    state: &AppState,
    workspace_id: &str,
    user_id: &str,
) -> Result<(), ApiResponse<T>> {
    match WorkspaceRepo::find_by_id(&state.db, workspace_id, user_id).await {
        Ok(_) => Ok(()),
        Err(WorkspaceRepoError::NotFound) => Err(workspace_not_found()),
        Err(e) => {
            warn!(error = %e, "查询工作区失败");
            Err(ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询工作区失败",
            ))
        }
    }
}

fn parse_dify_config(dify_config_json: Option<String>) -> Result<Option<DifyConfig>, String> {
    let Some(json) = dify_config_json else {
        return Ok(None);
    };

    let config: DifyConfig = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(Some(config))
}

fn parse_generic_config(
    generic_config_json: Option<String>,
) -> Result<Option<GenericConfig>, String> {
    let Some(json) = generic_config_json else {
        return Ok(None);
    };

    let json = json.trim();
    if json.is_empty() {
        return Ok(None);
    }
    let config: GenericConfig = serde_json::from_str(json).map_err(|e| e.to_string())?;
    Ok(Some(config))
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

fn validate_dify_config_request<T: Serialize>(
    req: &SaveDifyConfigRequest,
    snapshot: Option<&DifyConfig>,
) -> Result<(), ApiResponse<T>> {
    let target = req.target_prompt_variable.trim();
    if target.is_empty() {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "targetPromptVariable 不能为空",
        ));
    }
    if target.chars().count() > 128 {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "targetPromptVariable 不能超过 128 个字符",
        ));
    }

    if req.bindings.contains_key(target) {
        return Err(ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "targetPromptVariable 不能出现在 bindings 中",
        ));
    }

    for (var_name, binding) in &req.bindings {
        if var_name.trim().is_empty() {
            return Err(ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "bindings 中存在空变量名",
            ));
        }

        match binding.source {
            DifyBindingSource::Fixed => {
                if binding.value.is_none() {
                    return Err(ApiResponse::err(
                        StatusCode::BAD_REQUEST,
                        error_codes::VALIDATION_ERROR,
                        format!("变量 {} 的固定值缺少 value 字段", var_name),
                    ));
                }
            }
            DifyBindingSource::TestCaseInput => {
                let key = binding.input_key.as_deref().map(|s| s.trim()).unwrap_or("");
                if key.is_empty() {
                    return Err(ApiResponse::err(
                        StatusCode::BAD_REQUEST,
                        error_codes::VALIDATION_ERROR,
                        format!("变量 {} 的 testCaseInput 缺少 inputKey", var_name),
                    ));
                }
                if key.chars().count() > 128 {
                    return Err(ApiResponse::err(
                        StatusCode::BAD_REQUEST,
                        error_codes::VALIDATION_ERROR,
                        format!("变量 {} 的 inputKey 不能超过 128 个字符", var_name),
                    ));
                }
            }
        }
    }

    // 必填变量校验：仅在有 snapshot 时启用（用户可通过 refresh 更新 snapshot）
    if let Some(existing) = snapshot {
        if let Some(vars) = existing.parameters_snapshot.as_ref() {
            let names: HashSet<&str> = vars.iter().map(|v| v.name.as_str()).collect();

            if !names.contains(target) {
                return Err(ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "targetPromptVariable 不存在于当前 variables 快照中，请先刷新变量列表",
                ));
            }

            for name in req.bindings.keys() {
                if !names.contains(name.as_str()) {
                    return Err(ApiResponse::err(
                        StatusCode::BAD_REQUEST,
                        error_codes::VALIDATION_ERROR,
                        format!(
                            "bindings 中的变量 {} 不存在于当前 variables 快照中，请先刷新变量列表",
                            name
                        ),
                    ));
                }
            }

            for v in vars {
                if !v.required_known || !v.required {
                    continue;
                }
                if v.default_value.is_some() {
                    continue;
                }
                if v.name == target {
                    continue;
                }
                if !req.bindings.contains_key(&v.name) {
                    return Err(ApiResponse::err(
                        StatusCode::BAD_REQUEST,
                        error_codes::VALIDATION_ERROR,
                        format!("必填变量 {} 未配置来源且无默认值", v.name),
                    ));
                }
            }
        }
    }

    Ok(())
}

const GENERIC_CONFIG_JSON_MAX_BYTES: usize = 32 * 1024;
const DIFY_CONFIG_JSON_MAX_BYTES: usize = 32 * 1024;

fn validate_generic_config_request<T: Serialize>(
    req: &SaveGenericConfigRequest,
) -> Result<Vec<GenericInputVariable>, ApiResponse<T>> {
    let mut seen = HashSet::new();
    let mut sanitized = Vec::with_capacity(req.variables.len());

    for v in &req.variables {
        let name = v.name.trim();
        if name.is_empty() {
            return Err(ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "变量名不能为空",
            ));
        }
        if name.chars().count() > 128 {
            return Err(ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "变量名不能超过 128 个字符",
            ));
        }
        if !seen.insert(name.to_string()) {
            return Err(ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "变量名必须唯一",
            ));
        }

        if let Some(default_value) = &v.default_value {
            let ok = match v.value_type {
                GenericValueType::String => default_value.is_string(),
                GenericValueType::Number => default_value.is_number(),
                GenericValueType::Boolean => default_value.is_boolean(),
                GenericValueType::Json => true,
            };
            if !ok {
                return Err(ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "defaultValue 与 valueType 不兼容",
                ));
            }
        }

        sanitized.push(GenericInputVariable {
            name: name.to_string(),
            value_type: v.value_type,
            default_value: v.default_value.clone(),
        });
    }

    Ok(sanitized)
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
    post,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/dify/variables/refresh",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<DifyVariablesResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 502, description = "上游错误", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "dify_variables"
)]
#[instrument(skip_all)]
pub(crate) async fn refresh_dify_variables(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<DifyVariablesResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    let user_password = match current_user.unlock_context.as_ref() {
        Some(ctx) => ctx.password_bytes(),
        None => {
            return ApiResponse::err(
                StatusCode::UNAUTHORIZED,
                error_codes::UNAUTHORIZED,
                "会话已过期，请重新登录",
            );
        }
    };

    if let Err(err) =
        ensure_workspace_exists::<DifyVariablesResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

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

    let dify_credential =
        match CredentialRepo::find_by_user_and_type(&state.db, user_id, CredentialType::Dify).await
        {
            Ok(cred) => cred,
            Err(CredentialRepoError::NotFound { .. }) => {
                return ApiResponse::err(
                    StatusCode::BAD_REQUEST,
                    error_codes::VALIDATION_ERROR,
                    "请先在“配置”中保存 Dify 凭证",
                );
            }
            Err(e) => {
                warn!(error = %e, "查询 Dify 凭证失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "查询 Dify 凭证失败",
                );
            }
        };

    let encrypted = EncryptedApiKey {
        ciphertext: dify_credential.encrypted_api_key,
        nonce: dify_credential.nonce,
        salt: dify_credential.salt,
    };

    let api_key_bytes = match state
        .api_key_manager
        .decrypt_bytes(user_password, &encrypted)
    {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "解密 Dify API Key 失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::ENCRYPTION_ERROR,
                "解密 Dify API Key 失败",
            );
        }
    };

    let api_key = match std::str::from_utf8(api_key_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => {
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::ENCRYPTION_ERROR,
                "解密后的 Dify API Key 非法",
            );
        }
    };

    let resp = match dify_client::get_parameters(
        &state.http_client,
        &dify_credential.base_url,
        api_key,
        &correlation_id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "调用 Dify /parameters 失败");
            let message = match e {
                dify_client::ConnectionError::InvalidCredentials => {
                    "Dify API Key 无效，请重新配置".to_string()
                }
                dify_client::ConnectionError::Forbidden => "Dify 拒绝访问，请检查权限".to_string(),
                dify_client::ConnectionError::Timeout => "Dify 请求超时，请重试".to_string(),
                dify_client::ConnectionError::ParseError(_) => {
                    "解析 Dify 参数结构失败，请重试".to_string()
                }
                _ => "Dify 服务异常，请稍后重试".to_string(),
            };
            return ApiResponse::err(
                StatusCode::BAD_GATEWAY,
                error_codes::UPSTREAM_ERROR,
                &message,
            );
        }
    };

    // 可选缓存：如果已存在 dify_config_json，则更新其中的 parametersSnapshot 用于回显/校验
    if let Ok(Some(mut cfg)) = parse_dify_config(loaded.dify_config_json) {
        cfg.parameters_snapshot = Some(
            resp.variables
                .iter()
                .cloned()
                .map(|mut v| {
                    v.raw = None;
                    v
                })
                .collect(),
        );
        if let Ok(cfg_json) = serde_json::to_string(&cfg) {
            if cfg_json.len() <= DIFY_CONFIG_JSON_MAX_BYTES {
                let _ = TestSetRepo::update_dify_config_json_scoped(
                    &state.db,
                    user_id,
                    &workspace_id,
                    &test_set_id,
                    Some(&cfg_json),
                )
                .await;
            } else {
                warn!(
                    correlation_id = %correlation_id,
                    user_id = %user_id,
                    workspace_id = %workspace_id,
                    test_set_id = %test_set_id,
                    size_bytes = cfg_json.len(),
                    "dify_config_json 过大，跳过 parametersSnapshot 缓存"
                );
            }
        }
    }

    ApiResponse::ok(resp)
}

#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/dify/config",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    request_body = SaveDifyConfigRequest,
    responses(
        (status = 200, description = "保存成功", body = ApiSuccess<SaveDifyConfigResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "dify_variables"
)]
#[instrument(skip_all)]
pub(crate) async fn save_dify_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
    Json(req): Json<SaveDifyConfigRequest>,
) -> ApiResponse<SaveDifyConfigResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "保存 Dify 配置");

    if let Err(err) =
        ensure_workspace_exists::<SaveDifyConfigResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

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

    let existing = match parse_dify_config(loaded.dify_config_json.clone()) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "解析 dify_config_json 失败");
            None
        }
    };

    if let Err(err) =
        validate_dify_config_request::<SaveDifyConfigResponse>(&req, existing.as_ref())
    {
        return err;
    }

    let dify_config = DifyConfig {
        target_prompt_variable: req.target_prompt_variable.trim().to_string(),
        bindings: req.bindings.clone(),
        parameters_snapshot: existing.and_then(|c| c.parameters_snapshot),
    };

    let cfg_json = match serde_json::to_string(&dify_config) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "序列化 dify_config 失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "保存失败：序列化错误",
            );
        }
    };

    match TestSetRepo::update_dify_config_json_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &test_set_id,
        Some(&cfg_json),
    )
    .await
    {
        Ok(()) => {}
        Err(TestSetRepoError::NotFound) => return test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "保存 dify_config_json 失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "保存 Dify 配置失败",
            );
        }
    }

    ApiResponse::ok(SaveDifyConfigResponse { dify_config })
}

#[utoipa::path(
    put,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/generic/config",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    request_body = SaveGenericConfigRequest,
    responses(
        (status = 200, description = "保存成功", body = ApiSuccess<SaveGenericConfigResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "generic_config"
)]
#[instrument(skip_all)]
pub(crate) async fn save_generic_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
    Json(req): Json<SaveGenericConfigRequest>,
) -> ApiResponse<SaveGenericConfigResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "保存通用 API 自定义变量配置");

    if let Err(err) =
        ensure_workspace_exists::<SaveGenericConfigResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

    match TestSetRepo::find_by_id_scoped(&state.db, user_id, &workspace_id, &test_set_id).await {
        Ok(_) => {}
        Err(TestSetRepoError::NotFound) => return test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "查询测试集失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "查询测试集失败",
            );
        }
    }

    let variables = match validate_generic_config_request::<SaveGenericConfigResponse>(&req) {
        Ok(v) => v,
        Err(e) => return e,
    };

    let generic_config = GenericConfig { variables };
    let cfg_json = match serde_json::to_string(&generic_config) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "序列化 generic_config 失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "保存通用变量配置失败",
            );
        }
    };

    if cfg_json.len() > GENERIC_CONFIG_JSON_MAX_BYTES {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "通用变量配置不能超过 32KB",
        );
    }

    match TestSetRepo::update_generic_config_json_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &test_set_id,
        Some(&cfg_json),
    )
    .await
    {
        Ok(_) => {}
        Err(TestSetRepoError::NotFound) => return test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "保存 generic_config_json 失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "保存通用变量配置失败",
            );
        }
    }

    ApiResponse::ok(SaveGenericConfigResponse { generic_config })
}

#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{workspace_id}/test-sets/{test_set_id}/generic/config",
    params(
        ("workspace_id" = String, Path, description = "工作区 ID"),
        ("test_set_id" = String, Path, description = "测试集 ID")
    ),
    responses(
        (status = 200, description = "清空成功", body = ApiSuccess<DeleteGenericConfigResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 404, description = "测试集不存在", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "generic_config"
)]
#[instrument(skip_all)]
pub(crate) async fn delete_generic_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((workspace_id, test_set_id)): Path<(String, String)>,
    current_user: CurrentUser,
) -> ApiResponse<DeleteGenericConfigResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;

    info!(correlation_id = %correlation_id, user_id = %user_id, workspace_id = %workspace_id, test_set_id = %test_set_id, "清空通用 API 自定义变量配置");

    if let Err(err) =
        ensure_workspace_exists::<DeleteGenericConfigResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

    match TestSetRepo::update_generic_config_json_scoped(
        &state.db,
        user_id,
        &workspace_id,
        &test_set_id,
        None,
    )
    .await
    {
        Ok(_) => {}
        Err(TestSetRepoError::NotFound) => return test_set_not_found(),
        Err(e) => {
            warn!(error = %e, "清空 generic_config_json 失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "清空通用变量配置失败",
            );
        }
    }

    ApiResponse::ok(DeleteGenericConfigResponse {
        message: "已清空".to_string(),
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

    let dify_config_json = if let Some(dify_req) = req.dify_config.as_ref() {
        if let Err(err) = validate_dify_config_request::<TestSetResponse>(dify_req, None) {
            return err;
        }

        let dify_config = DifyConfig {
            target_prompt_variable: dify_req.target_prompt_variable.trim().to_string(),
            bindings: dify_req.bindings.clone(),
            parameters_snapshot: None,
        };

        match serde_json::to_string(&dify_config) {
            Ok(s) => Some(s),
            Err(e) => {
                warn!(error = %e, "序列化 dify_config 失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::INTERNAL_ERROR,
                    "创建失败：序列化 Dify 配置错误",
                );
            }
        }
    } else {
        None
    };

    let generic_config_json = if let Some(generic_req) = req.generic_config.as_ref() {
        let variables = match validate_generic_config_request::<TestSetResponse>(generic_req) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let generic_config = GenericConfig { variables };
        let cfg_json = match serde_json::to_string(&generic_config) {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, "序列化 generic_config 失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::INTERNAL_ERROR,
                    "创建失败：序列化通用变量配置错误",
                );
            }
        };

        if cfg_json.len() > GENERIC_CONFIG_JSON_MAX_BYTES {
            return ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                "通用变量配置不能超过 32KB",
            );
        }

        Some(cfg_json)
    } else {
        None
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
        dify_config_json.as_deref(),
        generic_config_json.as_deref(),
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
        dify_config: None,
        generic_config: None,
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

    if let Err(err) =
        ensure_workspace_exists::<TestSetResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

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
        dify_config: match parse_dify_config(loaded.dify_config_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "解析 dify_config_json 失败");
                None
            }
        },
        generic_config: match parse_generic_config(loaded.generic_config_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "解析 generic_config_json 失败");
                None
            }
        },
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

    if let Err(err) =
        ensure_workspace_exists::<TestSetResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

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
        dify_config: match parse_dify_config(updated.dify_config_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "解析 dify_config_json 失败");
                None
            }
        },
        generic_config: match parse_generic_config(updated.generic_config_json) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "解析 generic_config_json 失败");
                None
            }
        },
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

    if let Err(err) =
        ensure_workspace_exists::<DeleteTestSetResponse>(&state, &workspace_id, user_id).await
    {
        return err;
    }

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
            "/{test_set_id}/save-as-template",
            post(test_set_templates::save_as_template),
        )
        .route(
            "/{test_set_id}/dify/variables/refresh",
            post(refresh_dify_variables)
                .route_layer(middleware::from_fn(connectivity_middleware)),
        )
        .route("/{test_set_id}/dify/config", put(save_dify_config))
        .route(
            "/{test_set_id}/generic/config",
            put(save_generic_config).delete(delete_generic_config),
        )
        .route(
            "/{test_set_id}",
            get(get_test_set)
                .put(update_test_set)
                .delete(delete_test_set),
        )
}
