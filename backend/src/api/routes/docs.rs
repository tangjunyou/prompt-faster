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
        ),
        (
            name = "test_set_templates",
            description = "测试集模板 API（保存为模板、从模板创建）"
        ),
        (
            name = "dify_variables",
            description = "Dify 变量解析与绑定配置（隶属于 test_sets）"
        ),
        (
            name = "generic_config",
            description = "通用 API 自定义变量配置（隶属于 test_sets）"
        ),
        (
            name = "optimization_tasks",
            description = "优化任务配置 API（隶属于 workspace）"
        ),
        (
            name = "history",
            description = "历史聚合 API（迭代历史 + 回滚候选）"
        ),
        (
            name = "recovery",
            description = "断点恢复与连接状态 API"
        )
    ),
    paths(
        crate::api::routes::health::health_check,
        crate::api::routes::auth::test_dify_connection,
        crate::api::routes::auth::test_generic_llm_connection,
        crate::api::routes::auth::save_config,
        crate::api::routes::auth::get_config,
        crate::api::routes::auth::list_generic_llm_models,
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
        crate::api::routes::test_sets::refresh_dify_variables,
        crate::api::routes::test_sets::save_dify_config,
        crate::api::routes::test_sets::save_generic_config,
        crate::api::routes::test_set_templates::list_test_set_templates,
        crate::api::routes::test_set_templates::get_test_set_template,
        crate::api::routes::test_set_templates::save_as_template,
        crate::api::routes::optimization_tasks::create_optimization_task,
        crate::api::routes::optimization_tasks::list_optimization_tasks,
        crate::api::routes::optimization_tasks::get_optimization_task,
        crate::api::routes::optimization_tasks::update_optimization_task_config,
        crate::api::routes::history::get_history,
        crate::api::routes::history::list_history_events,
        crate::api::routes::history::get_history_timeline,
        crate::api::routes::history::export_history,
        crate::api::routes::checkpoints::list_checkpoints,
        crate::api::routes::checkpoints::get_checkpoint,
        crate::api::routes::recovery::list_unfinished_tasks,
        crate::api::routes::recovery::recover_task,
        crate::api::routes::recovery::abort_recovery,
        crate::api::routes::recovery::get_recovery_metrics,
        crate::api::routes::recovery::get_connectivity,
        crate::api::routes::recovery::rollback_task,
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
            crate::api::routes::auth::GenericLlmModelsResponse,
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
            // Dify Variables / Config
            crate::infra::external::dify_client::DifyVariablesResponse,
            crate::infra::external::dify_client::DifyInputVariable,
            crate::infra::external::dify_client::DifyValueType,
            crate::api::routes::dify::DifyConfig,
            crate::api::routes::dify::SaveDifyConfigRequest,
            crate::api::routes::dify::SaveDifyConfigResponse,
            crate::api::routes::dify::DifyBinding,
            crate::api::routes::dify::DifyBindingSource,
            // Generic API custom variables config
            crate::api::routes::generic::GenericConfig,
            crate::api::routes::generic::GenericInputVariable,
            crate::api::routes::generic::GenericValueType,
            crate::api::routes::generic::SaveGenericConfigRequest,
            crate::api::routes::generic::SaveGenericConfigResponse,
            crate::api::routes::generic::DeleteGenericConfigResponse,
            // Test Set Templates
            crate::api::routes::test_set_templates::SaveAsTemplateRequest,
            crate::api::routes::test_set_templates::TestSetTemplateListItemResponse,
            crate::api::routes::test_set_templates::TestSetTemplateResponse,
            crate::domain::models::TestCase,
            crate::domain::models::TaskReference,
            crate::domain::models::Constraint,
            crate::domain::models::QualityDimension,
            crate::domain::models::DataSplit,
            // Optimization Tasks
            crate::api::routes::optimization_tasks::CreateOptimizationTaskRequest,
            crate::api::routes::optimization_tasks::UpdateOptimizationTaskConfigRequest,
            crate::api::routes::optimization_tasks::OptimizationTaskResponse,
            crate::api::routes::optimization_tasks::OptimizationTaskListItemResponse,
            crate::domain::models::ExecutionTargetType,
            crate::domain::models::OptimizationTaskConfig,
            crate::domain::models::DataSplitPercentConfig,
            crate::domain::models::OptimizationTaskMode,
            crate::domain::models::OptimizationTaskStatus,
            // Checkpoints
            crate::domain::models::CheckpointListResponse,
            crate::domain::models::CheckpointResponse,
            crate::domain::models::CheckpointSummary,
            crate::domain::models::PassRateSummary,
            // History
            crate::domain::models::TaskHistoryResponse,
            crate::domain::types::IterationHistorySummary,
            crate::domain::models::HistoryEvent,
            crate::domain::models::HistoryEventResponse,
            crate::domain::models::TimelineEntry,
            crate::domain::models::TimelineEntryType,
            crate::domain::models::TimelineResponse,
            crate::domain::models::HistoryExportData,
            crate::domain::models::TaskExportMeta,
            crate::domain::models::IterationExportEntry,
            crate::domain::models::BranchInfo,
            crate::domain::models::EventType,
            crate::domain::models::Actor,
            // Recovery
            crate::domain::models::UnfinishedTask,
            crate::domain::models::UnfinishedTasksResponse,
            crate::domain::models::RecoveryRequest,
            crate::domain::models::RecoveryResponse,
            crate::domain::models::RecoveryMetrics,
            crate::domain::models::RollbackRequest,
            crate::domain::models::RollbackResponse,
            crate::domain::models::ConnectivityResponse,
            crate::domain::models::ConnectivityStatus,
            crate::api::routes::recovery::AbortRecoveryResponse,
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
