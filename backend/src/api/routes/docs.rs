//! OpenAPI / Swagger UI 路由
//! 提供可浏览的 API 文档界面

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI 文档定义
/// 包含所有 API 路由、标签、schema 定义
#[derive(OpenApi)]
#[openapi(
    tags(
        (
            name = "health",
            description = "健康检查端点"
        ),
        (
            name = "auth",
            description = "认证相关 API（连接测试、凭证配置）"
        ),
        (
            name = "user",
            description = "用户管理 API（注册、登录、登出）"
        ),
        (
            name = "workspaces",
            description = "工作区管理 API（创建、查询、删除）"
        ),
        (
            name = "test_sets",
            description = "测试集管理 API（CRUD，隶属于 workspace）"
        )
    ),
    paths(
        crate::api::routes::health::health_check,
        crate::api::routes::auth::test_dify_connection,
        crate::api::routes::auth::test_generic_llm_connection,
        crate::api::routes::auth::save_config,
        crate::api::routes::auth::get_config,
        crate::api::routes::user_auth::get_system_status,
        crate::api::routes::user_auth::register,
        crate::api::routes::user_auth::login,
        crate::api::routes::user_auth::logout,
        crate::api::routes::user_auth::get_me,
        crate::api::routes::workspaces::create_workspace,
        crate::api::routes::workspaces::list_workspaces,
        crate::api::routes::workspaces::get_workspace,
        crate::api::routes::workspaces::delete_workspace,
        crate::api::routes::test_sets::list_test_sets,
        crate::api::routes::test_sets::create_test_set,
        crate::api::routes::test_sets::get_test_set,
        crate::api::routes::test_sets::update_test_set,
        crate::api::routes::test_sets::delete_test_set,
    ),
    components(
        schemas(
            // Health
            crate::api::routes::health::HealthResponse,
            // Auth (connection tests)
            crate::api::routes::auth::TestDifyConnectionRequest,
            crate::api::routes::auth::TestGenericLlmConnectionRequest,
            crate::infra::external::dify_client::TestConnectionResult,
            // Auth (config)
            crate::api::routes::auth::CredentialInput,
            crate::api::routes::auth::GenericLlmCredentialInput,
            crate::api::routes::auth::TeacherSettingsInput,
            crate::api::routes::auth::ConfigResponse,
            crate::api::routes::auth::TeacherSettingsResponse,
            crate::api::routes::auth::SaveConfigResponse,
            // User Auth
            crate::api::routes::user_auth::RegisterRequest,
            crate::api::routes::user_auth::LoginRequest,
            crate::api::routes::user_auth::AuthResponse,
            crate::api::routes::user_auth::UserInfo,
            crate::api::routes::user_auth::SystemStatusResponse,
            crate::api::routes::user_auth::LogoutResponse,
            // Workspaces
            crate::api::routes::workspaces::CreateWorkspaceRequest,
            crate::api::routes::workspaces::WorkspaceResponse,
            crate::api::routes::workspaces::DeleteWorkspaceResponse,
            // Test Sets
            crate::api::routes::test_sets::CreateTestSetRequest,
            crate::api::routes::test_sets::UpdateTestSetRequest,
            crate::api::routes::test_sets::TestSetListItemResponse,
            crate::api::routes::test_sets::TestSetResponse,
            crate::api::routes::test_sets::DeleteTestSetResponse,
            crate::domain::models::TestCase,
            crate::domain::models::TaskReference,
            crate::domain::models::Constraint,
            crate::domain::models::QualityDimension,
            crate::domain::models::DataSplit,
        )
    ),
    info(
        title = "Prompt Faster API",
        version = "0.1.0",
        description = "AI Prompt 自动迭代优化系统 REST API",
        contact(
            name = "Prompt Faster Team",
        )
    )
)]
pub struct ApiDoc;

/// 创建 Swagger UI 路由
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::<S>::from(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
