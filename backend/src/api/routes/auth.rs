//! 认证相关路由
//! 包含 API 连接测试端点和凭证配置管理

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::infra::db::repositories::{
    CredentialRepo, CredentialType, TeacherSettingsRepo, UpsertCredentialInput,
    UpsertTeacherSettingsInput,
};
use crate::infra::external::api_key_manager::EncryptedApiKey;
use crate::infra::external::dify_client::{self, ConnectionError, TestConnectionResult};
use crate::infra::external::llm_client::{self, LlmConnectionError};
use crate::shared::error_codes;
use crate::shared::log_sanitizer::sanitize_api_key;
use crate::shared::url_validator::{validate_api_key, validate_base_url};

/// Dify 连接测试请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct TestDifyConnectionRequest {
    pub base_url: String,
    pub api_key: String,
}

/// 通用大模型连接测试请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct TestGenericLlmConnectionRequest {
    pub base_url: String,
    pub api_key: String,
    pub provider: String, // "siliconflow" | "modelscope"
}

/// 从请求头提取 correlation_id
fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// 测试 Dify API 连接
///
/// POST /api/v1/auth/test-connection/dify
#[utoipa::path(
    post,
    path = "/api/v1/auth/test-connection/dify",
    request_body = TestDifyConnectionRequest,
    responses(
        (status = 200, description = "连接成功", body = ApiSuccess<TestConnectionResult>),
        (status = 400, description = "请求参数错误", body = ApiError),
        (status = 401, description = "无效凭证", body = ApiError),
        (status = 403, description = "访问被拒绝", body = ApiError),
        (status = 502, description = "上游错误", body = ApiError)
    ),
    tag = "auth"
)]
pub(crate) async fn test_dify_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TestDifyConnectionRequest>,
) -> ApiResponse<TestConnectionResult> {
    let correlation_id = extract_correlation_id(&headers);

    info!(
        base_url = %req.base_url,
        api_key = %sanitize_api_key(&req.api_key),
        correlation_id = %correlation_id,
        "测试 Dify 连接"
    );

    // 输入验证：SSRF 防护（开发环境允许 HTTP）
    #[cfg(debug_assertions)]
    let allow_http = true;
    #[cfg(not(debug_assertions))]
    let allow_http = false;

    if let Err(e) = validate_base_url(&req.base_url, allow_http) {
        warn!(error = %e, "URL 验证失败");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::AUTH_VALIDATION_ERROR,
            e.to_string(),
        );
    }

    // 输入验证：API Key 非空
    if let Err(e) = validate_api_key(&req.api_key) {
        warn!(error = %e, "API Key 验证失败");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::AUTH_VALIDATION_ERROR,
            e,
        );
    }

    match dify_client::test_connection(
        &state.http_client,
        &req.base_url,
        &req.api_key,
        &correlation_id,
    )
    .await
    {
        Ok(result) => {
            info!("Dify 连接测试成功");
            ApiResponse::ok(result)
        }
        Err(e) => {
            warn!(error = %e, "Dify 连接测试失败");
            map_connection_error(e)
        }
    }
}

/// 测试通用大模型 API 连接
///
/// POST /api/v1/auth/test-connection/generic-llm
#[utoipa::path(
    post,
    path = "/api/v1/auth/test-connection/generic-llm",
    request_body = TestGenericLlmConnectionRequest,
    responses(
        (status = 200, description = "连接成功", body = ApiSuccess<TestConnectionResult>),
        (status = 400, description = "请求参数错误", body = ApiError),
        (status = 401, description = "无效凭证", body = ApiError),
        (status = 403, description = "访问被拒绝", body = ApiError),
        (status = 502, description = "上游错误", body = ApiError)
    ),
    tag = "auth"
)]
pub(crate) async fn test_generic_llm_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TestGenericLlmConnectionRequest>,
) -> ApiResponse<TestConnectionResult> {
    let correlation_id = extract_correlation_id(&headers);

    info!(
        base_url = %req.base_url,
        api_key = %sanitize_api_key(&req.api_key),
        provider = %req.provider,
        correlation_id = %correlation_id,
        "测试通用大模型连接"
    );

    // 输入验证：SSRF 防护（开发环境允许 HTTP）
    #[cfg(debug_assertions)]
    let allow_http = true;
    #[cfg(not(debug_assertions))]
    let allow_http = false;

    if let Err(e) = validate_base_url(&req.base_url, allow_http) {
        warn!(error = %e, "URL 验证失败");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::AUTH_VALIDATION_ERROR,
            e.to_string(),
        );
    }

    // 输入验证：API Key 非空
    if let Err(e) = validate_api_key(&req.api_key) {
        warn!(error = %e, "API Key 验证失败");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::AUTH_VALIDATION_ERROR,
            e,
        );
    }

    match llm_client::test_connection(
        &state.http_client,
        &req.base_url,
        &req.api_key,
        &req.provider,
        &correlation_id,
    )
    .await
    {
        Ok(result) => {
            info!("通用大模型连接测试成功");
            ApiResponse::ok(result)
        }
        Err(e) => {
            warn!(error = %e, "通用大模型连接测试失败");
            map_llm_connection_error(e)
        }
    }
}

// ============================================================================
// 连接错误映射（统一 trait 处理，消除代码重复）
// ============================================================================

/// 连接错误类型的统一抽象
/// 用于消除 ConnectionError 和 LlmConnectionError 的重复映射代码
trait ConnectionErrorMapping {
    fn is_invalid_credentials(&self) -> bool;
    fn is_forbidden(&self) -> bool;
    fn is_timeout(&self) -> bool;
    fn upstream_error_msg(&self) -> Option<String>;
    fn request_failed_msg(&self) -> Option<String>;
    fn parse_error_msg(&self) -> Option<String>;
    fn validation_error_msg(&self) -> Option<String>;
    fn client_error_msg(&self) -> Option<String>;
}

impl ConnectionErrorMapping for ConnectionError {
    fn is_invalid_credentials(&self) -> bool {
        matches!(self, ConnectionError::InvalidCredentials)
    }
    fn is_forbidden(&self) -> bool {
        matches!(self, ConnectionError::Forbidden)
    }
    fn is_timeout(&self) -> bool {
        matches!(self, ConnectionError::Timeout)
    }
    fn upstream_error_msg(&self) -> Option<String> {
        if let ConnectionError::UpstreamError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
    fn request_failed_msg(&self) -> Option<String> {
        if let ConnectionError::RequestFailed(e) = self {
            Some(e.to_string())
        } else {
            None
        }
    }
    fn parse_error_msg(&self) -> Option<String> {
        if let ConnectionError::ParseError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
    fn validation_error_msg(&self) -> Option<String> {
        if let ConnectionError::ValidationError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
    fn client_error_msg(&self) -> Option<String> {
        if let ConnectionError::ClientError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
}

impl ConnectionErrorMapping for LlmConnectionError {
    fn is_invalid_credentials(&self) -> bool {
        matches!(self, LlmConnectionError::InvalidCredentials)
    }
    fn is_forbidden(&self) -> bool {
        matches!(self, LlmConnectionError::Forbidden)
    }
    fn is_timeout(&self) -> bool {
        matches!(self, LlmConnectionError::Timeout)
    }
    fn upstream_error_msg(&self) -> Option<String> {
        if let LlmConnectionError::UpstreamError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
    fn request_failed_msg(&self) -> Option<String> {
        if let LlmConnectionError::RequestFailed(e) = self {
            Some(e.to_string())
        } else {
            None
        }
    }
    fn parse_error_msg(&self) -> Option<String> {
        if let LlmConnectionError::ParseError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
    fn validation_error_msg(&self) -> Option<String> {
        if let LlmConnectionError::ValidationError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
    fn client_error_msg(&self) -> Option<String> {
        if let LlmConnectionError::ClientError(msg) = self {
            Some(msg.clone())
        } else {
            None
        }
    }
}

/// 统一的连接错误映射函数
fn map_connection_error_impl<E: ConnectionErrorMapping>(
    error: E,
) -> ApiResponse<TestConnectionResult> {
    if error.is_invalid_credentials() {
        return ApiResponse::err(
            StatusCode::UNAUTHORIZED,
            error_codes::AUTH_INVALID_CREDENTIALS,
            "无效的 API Key",
        );
    }
    if error.is_forbidden() {
        return ApiResponse::err(
            StatusCode::FORBIDDEN,
            error_codes::AUTH_FORBIDDEN,
            "访问被拒绝",
        );
    }
    if error.is_timeout() {
        return ApiResponse::err(
            StatusCode::REQUEST_TIMEOUT,
            error_codes::AUTH_CONNECTION_TIMEOUT,
            "连接超时",
        );
    }
    if let Some(msg) = error.upstream_error_msg() {
        return ApiResponse::err(
            StatusCode::BAD_GATEWAY,
            error_codes::AUTH_UPSTREAM_ERROR,
            format!("上游服务不可用: {}", msg),
        );
    }
    if let Some(msg) = error.request_failed_msg() {
        return ApiResponse::err(
            StatusCode::BAD_GATEWAY,
            error_codes::AUTH_UPSTREAM_ERROR,
            format!("请求失败: {}", msg),
        );
    }
    if let Some(msg) = error.parse_error_msg() {
        return ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            error_codes::AUTH_INTERNAL_ERROR,
            format!("响应解析失败: {}", msg),
        );
    }
    if let Some(msg) = error.validation_error_msg() {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::AUTH_VALIDATION_ERROR,
            msg,
        );
    }
    if let Some(msg) = error.client_error_msg() {
        return ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            error_codes::AUTH_INTERNAL_ERROR,
            format!("HTTP 客户端错误: {}", msg),
        );
    }
    // 兜底：不应该执行到这里
    ApiResponse::err(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_codes::AUTH_INTERNAL_ERROR,
        "未知错误",
    )
}

/// 映射 Dify 连接错误到 API 响应
fn map_connection_error(error: ConnectionError) -> ApiResponse<TestConnectionResult> {
    map_connection_error_impl(error)
}

/// 映射 LLM 连接错误到 API 响应
fn map_llm_connection_error(error: LlmConnectionError) -> ApiResponse<TestConnectionResult> {
    map_connection_error_impl(error)
}

// ============================================================================
// 凭证配置管理 API
// ============================================================================

/// 保存配置请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveConfigRequest {
    /// Dify 凭证
    pub dify: Option<CredentialInput>,
    /// 通用大模型凭证
    pub generic_llm: Option<GenericLlmCredentialInput>,
    /// 老师模型参数
    pub teacher_settings: TeacherSettingsInput,
}

/// 凭证输入（Dify）
#[derive(Debug, Deserialize, ToSchema)]
pub struct CredentialInput {
    pub base_url: String,
    pub api_key: String,
}

/// 凭证输入（通用大模型）
#[derive(Debug, Deserialize, ToSchema)]
pub struct GenericLlmCredentialInput {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
}

/// 老师模型参数输入
#[derive(Debug, Deserialize, ToSchema)]
pub struct TeacherSettingsInput {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i32,
}

/// 配置响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ConfigResponse {
    /// 是否已配置 Dify API Key
    pub has_dify_key: bool,
    /// 是否已配置通用大模型 API Key
    pub has_generic_llm_key: bool,
    /// Dify Base URL（如果已配置）
    pub dify_base_url: Option<String>,
    /// 通用大模型 Base URL（如果已配置）
    pub generic_llm_base_url: Option<String>,
    /// 通用大模型 Provider（如果已配置）
    pub generic_llm_provider: Option<String>,
    /// 脱敏后的 Dify API Key（如 sk-****xxxx）
    pub masked_dify_key: Option<String>,
    /// 脱敏后的通用大模型 API Key
    pub masked_generic_llm_key: Option<String>,
    /// 老师模型参数
    pub teacher_settings: TeacherSettingsResponse,
}

/// 老师模型参数响应
#[derive(Debug, Serialize, ToSchema)]
pub struct TeacherSettingsResponse {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: i32,
}

/// 保存成功响应
#[derive(Debug, Serialize, ToSchema)]
pub struct SaveConfigResponse {
    pub message: String,
}

/// 验证老师模型参数
fn validate_teacher_settings(settings: &TeacherSettingsInput) -> Result<(), String> {
    if settings.temperature < 0.0 || settings.temperature > 2.0 {
        return Err(format!(
            "temperature 必须在 0.0 ~ 2.0 之间，当前值: {}",
            settings.temperature
        ));
    }
    if settings.top_p < 0.0 || settings.top_p > 1.0 {
        return Err(format!(
            "top_p 必须在 0.0 ~ 1.0 之间，当前值: {}",
            settings.top_p
        ));
    }
    if settings.max_tokens < 1 || settings.max_tokens > 8192 {
        return Err(format!(
            "max_tokens 必须在 1 ~ 8192 之间，当前值: {}",
            settings.max_tokens
        ));
    }
    Ok(())
}

/// 保存配置
///
/// POST /api/v1/auth/config
///
/// # 注意
/// 根据 Story 1.5 AC #1 和 Task 3.3 要求，请求体必须同时包含：
/// - Dify credential（base_url, api_key）
/// - Generic LLM credential（provider, base_url, api_key）
/// - teacher model settings（temperature, top_p, max_tokens）
///
/// # 鉴权
/// 此接口需要登录，使用 CurrentUser.user_id 替代 DEFAULT_USER_ID
#[utoipa::path(
    post,
    path = "/api/v1/auth/config",
    request_body = SaveConfigRequest,
    responses(
        (status = 200, description = "保存成功", body = ApiSuccess<SaveConfigResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "auth"
)]
pub(crate) async fn save_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
    Json(req): Json<SaveConfigRequest>,
) -> ApiResponse<SaveConfigResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;
    info!(correlation_id = %correlation_id, user_id = %user_id, "保存配置");

    // 强制校验：必须同时包含 Dify 和 Generic LLM 凭证 (Story 1.5 Task 3.3)
    if req.dify.is_none() {
        warn!("缺少 Dify 凭证配置");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "请求体必须包含 Dify 凭证配置 (dify)",
        );
    }
    if req.generic_llm.is_none() {
        warn!("缺少通用大模型凭证配置");
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "请求体必须包含通用大模型凭证配置 (generic_llm)",
        );
    }

    // 验证老师模型参数
    if let Err(e) = validate_teacher_settings(&req.teacher_settings) {
        warn!(error = %e, "老师模型参数验证失败");
        return ApiResponse::err(StatusCode::BAD_REQUEST, error_codes::VALIDATION_ERROR, e);
    }

    // 开发环境允许 HTTP
    #[cfg(debug_assertions)]
    let allow_http = true;
    #[cfg(not(debug_assertions))]
    let allow_http = false;

    // 保存 Dify 凭证（已在上面校验过存在性）
    if let Some(dify) = &req.dify {
        // 验证 URL
        if let Err(e) = validate_base_url(&dify.base_url, allow_http) {
            warn!(error = %e, "Dify URL 验证失败");
            return ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                format!("Dify Base URL 无效: {}", e),
            );
        }
        // 验证 API Key
        if let Err(e) = validate_api_key(&dify.api_key) {
            warn!(error = %e, "Dify API Key 验证失败");
            return ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                format!("Dify API Key 无效: {}", e),
            );
        }

        // 加密 API Key
        let encrypted = match state.api_key_manager.encrypt(&dify.api_key) {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "API Key 加密失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::ENCRYPTION_ERROR,
                    "API Key 加密失败",
                );
            }
        };

        // 保存到数据库
        if let Err(e) = CredentialRepo::upsert(
            &state.db,
            UpsertCredentialInput {
                user_id: user_id.clone(),
                credential_type: CredentialType::Dify,
                provider: None,
                base_url: dify.base_url.clone(),
                encrypted_api_key: encrypted.ciphertext,
                nonce: encrypted.nonce,
                salt: encrypted.salt,
            },
        )
        .await
        {
            warn!(error = %e, "保存 Dify 凭证失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "保存 Dify 凭证失败",
            );
        }
        info!("Dify 凭证保存成功");
    }

    // 保存通用大模型凭证
    if let Some(generic_llm) = &req.generic_llm {
        // 验证 provider
        if generic_llm.provider != "siliconflow" && generic_llm.provider != "modelscope" {
            return ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                format!(
                    "无效的 provider: {}，支持 siliconflow 或 modelscope",
                    generic_llm.provider
                ),
            );
        }
        // 验证 URL
        if let Err(e) = validate_base_url(&generic_llm.base_url, allow_http) {
            warn!(error = %e, "通用大模型 URL 验证失败");
            return ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                format!("通用大模型 Base URL 无效: {}", e),
            );
        }
        // 验证 API Key
        if let Err(e) = validate_api_key(&generic_llm.api_key) {
            warn!(error = %e, "通用大模型 API Key 验证失败");
            return ApiResponse::err(
                StatusCode::BAD_REQUEST,
                error_codes::VALIDATION_ERROR,
                format!("通用大模型 API Key 无效: {}", e),
            );
        }

        // 加密 API Key
        let encrypted = match state.api_key_manager.encrypt(&generic_llm.api_key) {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "API Key 加密失败");
                return ApiResponse::err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::ENCRYPTION_ERROR,
                    "API Key 加密失败",
                );
            }
        };

        // 保存到数据库
        if let Err(e) = CredentialRepo::upsert(
            &state.db,
            UpsertCredentialInput {
                user_id: user_id.clone(),
                credential_type: CredentialType::GenericLlm,
                provider: Some(generic_llm.provider.clone()),
                base_url: generic_llm.base_url.clone(),
                encrypted_api_key: encrypted.ciphertext,
                nonce: encrypted.nonce,
                salt: encrypted.salt,
            },
        )
        .await
        {
            warn!(error = %e, "保存通用大模型凭证失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "保存通用大模型凭证失败",
            );
        }
        info!("通用大模型凭证保存成功");
    }

    // 保存老师模型参数
    if let Err(e) = TeacherSettingsRepo::upsert(
        &state.db,
        UpsertTeacherSettingsInput {
            user_id: user_id.clone(),
            temperature: req.teacher_settings.temperature,
            top_p: req.teacher_settings.top_p,
            max_tokens: req.teacher_settings.max_tokens,
        },
    )
    .await
    {
        warn!(error = %e, "保存老师模型参数失败");
        return ApiResponse::err(
            StatusCode::INTERNAL_SERVER_ERROR,
            error_codes::DATABASE_ERROR,
            "保存老师模型参数失败",
        );
    }
    info!("老师模型参数保存成功");

    ApiResponse::ok(SaveConfigResponse {
        message: "配置保存成功".to_string(),
    })
}

/// 获取配置
///
/// GET /api/v1/auth/config
///
/// # 鉴权
/// 此接口需要登录，使用 CurrentUser.user_id 替代 DEFAULT_USER_ID
#[utoipa::path(
    get,
    path = "/api/v1/auth/config",
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<ConfigResponse>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "auth"
)]
pub(crate) async fn get_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    current_user: CurrentUser,
) -> ApiResponse<ConfigResponse> {
    let correlation_id = extract_correlation_id(&headers);
    let user_id = &current_user.user_id;
    info!(correlation_id = %correlation_id, user_id = %user_id, "获取配置");

    // 查询 Dify 凭证
    let dify_credential =
        CredentialRepo::find_by_user_and_type(&state.db, user_id, CredentialType::Dify)
            .await
            .ok();

    // 查询通用大模型凭证
    let generic_llm_credential =
        CredentialRepo::find_by_user_and_type(&state.db, user_id, CredentialType::GenericLlm)
            .await
            .ok();

    // 查询老师模型参数（不存在时返回默认值）
    let teacher_settings = TeacherSettingsRepo::get_or_default(&state.db, user_id)
        .await
        .unwrap_or_else(|_| crate::infra::db::repositories::TeacherSettingsRecord {
            id: String::new(),
            user_id: user_id.clone(),
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2048,
            created_at: 0,
            updated_at: 0,
        });

    // 解密并脱敏 Dify API Key
    let (has_dify_key, dify_base_url, masked_dify_key) = match dify_credential {
        Some(cred) => {
            let masked = decrypt_and_mask(&state, &cred.encrypted_api_key, &cred.nonce, &cred.salt);
            (true, Some(cred.base_url), masked)
        }
        None => (false, None, None),
    };

    // 解密并脱敏通用大模型 API Key
    let (has_generic_llm_key, generic_llm_base_url, generic_llm_provider, masked_generic_llm_key) =
        match generic_llm_credential {
            Some(cred) => {
                let masked =
                    decrypt_and_mask(&state, &cred.encrypted_api_key, &cred.nonce, &cred.salt);
                (true, Some(cred.base_url), cred.provider, masked)
            }
            None => (false, None, None, None),
        };

    ApiResponse::ok(ConfigResponse {
        has_dify_key,
        has_generic_llm_key,
        dify_base_url,
        generic_llm_base_url,
        generic_llm_provider,
        masked_dify_key,
        masked_generic_llm_key,
        teacher_settings: TeacherSettingsResponse {
            temperature: teacher_settings.temperature,
            top_p: teacher_settings.top_p,
            max_tokens: teacher_settings.max_tokens,
        },
    })
}

/// 解密 API Key 并脱敏
///
/// # TODO(Security): 优化为只解密部分字节用于脱敏
/// 当前实现会先解密完整 API Key 再脱敏，完整明文短暂存在内存中。
/// 建议后续优化：
/// - 方案 A: 在加密时同时存储脱敏后的 masked_key（无需解密）
/// - 方案 B: 只解密最后 4 字节用于脱敏显示
fn decrypt_and_mask(
    state: &AppState,
    ciphertext: &[u8],
    nonce: &[u8],
    salt: &[u8],
) -> Option<String> {
    let encrypted = EncryptedApiKey {
        ciphertext: ciphertext.to_vec(),
        nonce: nonce.to_vec(),
        salt: salt.to_vec(),
    };

    match state.api_key_manager.decrypt(&encrypted) {
        Ok(api_key) => Some(sanitize_api_key(&api_key)),
        Err(e) => {
            warn!(error = %e, "解密 API Key 失败");
            None
        }
    }
}

/// 创建公开认证路由（无需鉴权）
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/test-connection/dify", post(test_dify_connection))
        .route(
            "/test-connection/generic-llm",
            post(test_generic_llm_connection),
        )
}

/// 创建受保护的认证路由（需要鉴权，由 auth_middleware 保护）
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/config", post(save_config))
        .route("/config", get(get_config))
}

/// 创建认证路由（向后兼容，包含所有路由）
/// ⚠️ 已废弃：请使用 public_router() + protected_router() 分别挂载
#[deprecated(note = "Use public_router() and protected_router() instead")]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/test-connection/dify", post(test_dify_connection))
        .route(
            "/test-connection/generic-llm",
            post(test_generic_llm_connection),
        )
        .route("/config", post(save_config))
        .route("/config", get(get_config))
}
