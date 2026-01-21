//! 统一错误码管理
//!
//! 命名规范：DOMAIN_ACTION_REASON (大写下划线分隔)
//! 混合策略：
//!   - 通用错误码：如 VALIDATION_ERROR, DATABASE_ERROR, INTERNAL_ERROR
//!   - 业务场景详细码：如 AUTH_INVALID_CREDENTIALS, WORKSPACE_NOT_FOUND
//!
//! 使用示例：
//! ```rust
//! use axum::http::StatusCode;
//! use prompt_faster::api::response::ApiResponse;
//! use prompt_faster::shared::error_codes;
//!
//! let _ = ApiResponse::<()>::err(
//!     StatusCode::UNAUTHORIZED,
//!     error_codes::AUTH_INVALID_CREDENTIALS,
//!     "无效的 API Key",
//! );
//! ```

// ============================================================================
// 通用错误码 (GENERAL)
// ============================================================================

/// 通用验证错误
pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";

/// 未授权（需要登录）
pub const UNAUTHORIZED: &str = "UNAUTHORIZED";

/// 禁止访问（已认证但无权限）
pub const FORBIDDEN: &str = "FORBIDDEN";

/// 资源不存在
pub const NOT_FOUND: &str = "NOT_FOUND";

/// 数据库操作失败
pub const DATABASE_ERROR: &str = "DATABASE_ERROR";

/// 服务器内部错误
pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";

/// 上游服务错误（通用）
pub const UPSTREAM_ERROR: &str = "UPSTREAM_ERROR";

/// 触发速率限制
pub const RATE_LIMITED: &str = "RATE_LIMITED";

/// 网络离线
pub const CONNECTIVITY_OFFLINE: &str = "CONNECTIVITY_OFFLINE";

// ============================================================================
// 认证错误码 (AUTH)
// ============================================================================

/// 认证相关验证错误
pub const AUTH_VALIDATION_ERROR: &str = "AUTH_VALIDATION_ERROR";

/// 无效的凭证（API Key 或密码）
pub const AUTH_INVALID_CREDENTIALS: &str = "AUTH_INVALID_CREDENTIALS";

/// 认证被禁止
pub const AUTH_FORBIDDEN: &str = "AUTH_FORBIDDEN";

/// 认证连接超时
pub const AUTH_CONNECTION_TIMEOUT: &str = "AUTH_CONNECTION_TIMEOUT";

/// 上游服务错误
pub const AUTH_UPSTREAM_ERROR: &str = "AUTH_UPSTREAM_ERROR";

/// 认证内部错误
pub const AUTH_INTERNAL_ERROR: &str = "AUTH_INTERNAL_ERROR";

/// 认证失败（用户名或密码错误）
pub const AUTH_FAILED: &str = "AUTH_FAILED";

// ============================================================================
// 资源错误码 (RESOURCE)
// ============================================================================

/// 工作区不存在
pub const WORKSPACE_NOT_FOUND: &str = "WORKSPACE_NOT_FOUND";

/// 测试集不存在
pub const TEST_SET_NOT_FOUND: &str = "TEST_SET_NOT_FOUND";

/// 优化任务不存在
pub const OPTIMIZATION_TASK_NOT_FOUND: &str = "OPTIMIZATION_TASK_NOT_FOUND";

/// 资源不存在（通用）
pub const RESOURCE_NOT_FOUND: &str = "RESOURCE_NOT_FOUND";

/// 资源禁止访问
pub const RESOURCE_FORBIDDEN: &str = "RESOURCE_FORBIDDEN";

// ============================================================================
// 用户错误码 (USER)
// ============================================================================

/// 用户名冲突
pub const USERNAME_CONFLICT: &str = "USERNAME_CONFLICT";

// ============================================================================
// 加密错误码 (ENCRYPTION)
// ============================================================================

/// 加密操作失败
pub const ENCRYPTION_ERROR: &str = "ENCRYPTION_ERROR";
