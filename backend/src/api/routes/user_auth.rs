//! 用户认证路由
//! 提供注册、登录、登出、获取当前用户等 API

use axum::extract::ConnectInfo;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::api::middleware::CurrentUser;
use crate::api::middleware::LoginAttemptStore;
use crate::api::middleware::correlation_id::CORRELATION_ID_HEADER;
use crate::api::middleware::session::UnlockContext;
use crate::api::response::{ApiError, ApiResponse, ApiSuccess};
use crate::api::state::AppState;
use crate::infra::db::repositories::{MigrationRepo, UserRepo, UserRepoError};
use crate::shared::error_codes;
use crate::shared::password::{PasswordError, PasswordService};

/// 注册请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// 登录请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 认证成功响应
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub session_token: String,
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

/// 登出响应
#[derive(Debug, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}

/// 系统状态响应（用于判断是否需要注册）
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemStatusResponse {
    pub has_users: bool,
    pub requires_registration: bool,
}

/// 从请求头提取 correlation_id
fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// 从请求头提取 Bearer token
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .filter(|h| h.starts_with("Bearer "))
        .map(|h| h[7..].to_string())
}

/// 获取系统状态
///
/// GET /api/v1/auth/status
///
/// 返回系统是否已有用户，用于前端判断是否需要显示注册页面
#[utoipa::path(
    get,
    path = "/api/v1/auth/status",
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<SystemStatusResponse>),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "user"
)]
pub(crate) async fn get_system_status(
    State(state): State<AppState>,
) -> ApiResponse<SystemStatusResponse> {
    match UserRepo::has_any_user(&state.db).await {
        Ok(has_users) => ApiResponse::ok(SystemStatusResponse {
            has_users,
            requires_registration: !has_users,
        }),
        Err(e) => {
            warn!(error = %e, "检查用户状态失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "系统状态检查失败",
            )
        }
    }
}

/// 用户注册
///
/// POST /api/v1/auth/register
///
/// 创建新用户账户并返回会话 token
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "注册成功", body = ApiSuccess<AuthResponse>),
        (status = 400, description = "参数错误", body = ApiError),
        (status = 409, description = "用户名冲突", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "user"
)]
pub(crate) async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> ApiResponse<AuthResponse> {
    let correlation_id = extract_correlation_id(&headers);
    info!(correlation_id = %correlation_id, username = %req.username, "用户注册");

    // 验证用户名
    if req.username.trim().is_empty() {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "用户名不能为空",
        );
    }
    if req.username.len() < 3 || req.username.len() > 32 {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "用户名长度必须在 3-32 个字符之间",
        );
    }

    // 验证密码
    if req.password.len() < 6 {
        return ApiResponse::err(
            StatusCode::BAD_REQUEST,
            error_codes::VALIDATION_ERROR,
            "密码长度至少 6 个字符",
        );
    }

    // 对密码进行 Argon2 哈希
    let password_hash = match PasswordService::hash_password(&req.password) {
        Ok(hash) => hash,
        Err(PasswordError::HashError) => {
            warn!("密码哈希失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "注册失败，请稍后重试",
            );
        }
        Err(_) => {
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::INTERNAL_ERROR,
                "注册失败，请稍后重试",
            );
        }
    };

    let was_empty = match UserRepo::has_any_user(&state.db).await {
        Ok(has_any) => !has_any,
        Err(e) => {
            warn!(error = %e, "检查用户状态失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "注册失败，请稍后重试",
            );
        }
    };

    // 创建用户
    let user = match UserRepo::create_user(&state.db, &req.username, &password_hash).await {
        Ok(user) => user,
        Err(UserRepoError::UsernameConflict) => {
            return ApiResponse::err(
                StatusCode::CONFLICT,
                error_codes::USERNAME_CONFLICT,
                "用户名已存在",
            );
        }
        Err(e) => {
            warn!(error = %e, "创建用户失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "注册失败，请稍后重试",
            );
        }
    };

    if was_empty {
        match UserRepo::get_first_user(&state.db).await {
            Ok(Some(first_user)) if first_user.id == user.id => {
                match MigrationRepo::migrate_legacy_default_user_data(&state.db, &user.id).await {
                    Ok(result) => {
                        info!(
                            migrated_api_credentials = result.migrated_api_credentials,
                            migrated_teacher_settings = result.migrated_teacher_settings,
                            "历史数据迁移完成"
                        );
                    }
                    Err(e) => {
                        warn!(error = %e, "历史数据迁移失败");
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                warn!(error = %e, "获取首个用户失败，跳过历史数据迁移");
            }
        }
    }

    // 创建会话（包含解锁上下文）
    let unlock_context = UnlockContext::new(req.password);
    let session_token = state
        .session_store
        .create_session(user.id.clone(), Some(unlock_context))
        .await;

    info!(user_id = %user.id, "用户注册成功");

    ApiResponse::ok(AuthResponse {
        session_token,
        user: UserInfo {
            id: user.id,
            username: user.username,
        },
    })
}

/// 用户登录
///
/// POST /api/v1/auth/login
///
/// 验证用户凭证并返回会话 token
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功", body = ApiSuccess<AuthResponse>),
        (status = 401, description = "认证失败", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "user"
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> ApiResponse<AuthResponse> {
    let correlation_id = extract_correlation_id(&headers);
    info!(correlation_id = %correlation_id, username = %req.username, "用户登录");

    let ip = addr.ip().to_string();
    let key = LoginAttemptStore::make_key(&ip, &req.username);

    if state.login_attempt_store.is_blocked(&key).await {
        warn!(correlation_id = %correlation_id, username = %req.username, ip = %ip, "登录尝试被冷却");
        return ApiResponse::err(
            StatusCode::UNAUTHORIZED,
            error_codes::AUTH_FAILED,
            "用户名或密码错误",
        );
    }

    // 查找用户（不区分用户不存在/密码错误，AC #3）
    let user = match UserRepo::find_by_username(&state.db, &req.username).await {
        Ok(user) => user,
        Err(UserRepoError::NotFound) => {
            state.login_attempt_store.record_failure(key.clone()).await;
            // 不暴露用户是否存在
            warn!("登录失败：用户名或密码错误");
            return ApiResponse::err(
                StatusCode::UNAUTHORIZED,
                error_codes::AUTH_FAILED,
                "用户名或密码错误",
            );
        }
        Err(e) => {
            warn!(error = %e, "查询用户失败");
            return ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "登录失败，请稍后重试",
            );
        }
    };

    // 验证密码
    if PasswordService::verify_password(&req.password, &user.password_hash).is_err() {
        state.login_attempt_store.record_failure(key.clone()).await;
        // 不暴露具体失败原因
        warn!("登录失败：用户名或密码错误");
        return ApiResponse::err(
            StatusCode::UNAUTHORIZED,
            error_codes::AUTH_FAILED,
            "用户名或密码错误",
        );
    }

    state.login_attempt_store.reset(&key).await;

    // 创建会话（包含解锁上下文）
    let unlock_context = UnlockContext::new(req.password);
    let session_token = state
        .session_store
        .create_session(user.id.clone(), Some(unlock_context))
        .await;

    info!(user_id = %user.id, "用户登录成功");

    ApiResponse::ok(AuthResponse {
        session_token,
        user: UserInfo {
            id: user.id,
            username: user.username,
        },
    })
}

/// 用户登出
///
/// POST /api/v1/auth/logout
///
/// 注销当前会话
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "登出成功", body = ApiSuccess<LogoutResponse>),
        (status = 401, description = "未授权", body = ApiError)
    ),
    tag = "user"
)]
pub(crate) async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResponse<LogoutResponse> {
    let correlation_id = extract_correlation_id(&headers);
    info!(correlation_id = %correlation_id, "用户登出");

    // 提取 token
    let token = match extract_bearer_token(&headers) {
        Some(t) => t,
        None => {
            return ApiResponse::err(
                StatusCode::UNAUTHORIZED,
                error_codes::UNAUTHORIZED,
                "未提供有效的会话令牌",
            );
        }
    };

    // 移除会话
    let removed = state.session_store.remove_session(&token).await;

    if removed {
        info!("用户登出成功");
    } else {
        info!("会话已不存在或已过期");
    }

    ApiResponse::ok(LogoutResponse {
        message: "登出成功".to_string(),
    })
}

/// 获取当前用户信息
///
/// GET /api/v1/auth/me
///
/// 需要鉴权，返回当前登录用户的信息
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "获取成功", body = ApiSuccess<UserInfo>),
        (status = 401, description = "未授权", body = ApiError),
        (status = 500, description = "服务器错误", body = ApiError)
    ),
    tag = "user"
)]
pub(crate) async fn get_me(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> ApiResponse<UserInfo> {
    // 从数据库获取完整用户信息
    match UserRepo::find_by_id(&state.db, &current_user.user_id).await {
        Ok(user) => ApiResponse::ok(UserInfo {
            id: user.id,
            username: user.username,
        }),
        Err(e) => {
            warn!(error = %e, user_id = %current_user.user_id, "获取用户信息失败");
            ApiResponse::err(
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::DATABASE_ERROR,
                "获取用户信息失败",
            )
        }
    }
}

/// 创建用户认证路由
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_system_status))
        .route("/register", post(register))
        .route("/login", post(login))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/logout", post(logout))
        .route("/me", get(get_me))
}

pub fn router() -> Router<AppState> {
    public_router().merge(protected_router())
}
