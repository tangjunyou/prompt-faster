use std::path::PathBuf;

use prompt_faster::api::response::{ApiError, ApiSuccess, ErrorDetail, PaginationMeta};
use prompt_faster::api::routes::auth::{
    ConfigResponse, CredentialInput, GenericLlmCredentialInput, GenericLlmModelsResponse,
    SaveConfigRequest, SaveConfigResponse, TeacherSettingsInput, TeacherSettingsResponse,
    TestDifyConnectionRequest, TestGenericLlmConnectionRequest,
};
use prompt_faster::api::routes::dify::{
    DifyBinding, DifyBindingSource, DifyConfig, SaveDifyConfigRequest, SaveDifyConfigResponse,
};
use prompt_faster::api::routes::generic::{
    DeleteGenericConfigResponse, GenericConfig, GenericInputVariable, GenericValueType,
    SaveGenericConfigRequest, SaveGenericConfigResponse,
};
use prompt_faster::api::routes::health::HealthResponse;
use prompt_faster::api::routes::optimization_tasks::{
    CreateOptimizationTaskRequest, MetaOptimizationTaskHint, OptimizationTaskListItemResponse,
    OptimizationTaskResponse, UpdateOptimizationTaskConfigRequest,
};
use prompt_faster::api::routes::test_set_templates::{
    SaveAsTemplateRequest, TestSetTemplateListItemResponse, TestSetTemplateResponse,
};
use prompt_faster::api::routes::test_sets::{
    CreateTestSetRequest, DeleteTestSetResponse, TestSetListItemResponse, TestSetResponse,
    UpdateTestSetRequest,
};
use prompt_faster::api::routes::user_auth::{
    AuthResponse, LoginRequest, LogoutResponse, RegisterRequest, SystemStatusResponse, UserInfo,
};
use prompt_faster::api::routes::workspaces::{
    CreateWorkspaceRequest, DeleteWorkspaceResponse, WorkspaceResponse,
};
use prompt_faster::api::ws::events::{
    ArtifactGetAckPayload, ArtifactGetPayload, ArtifactUpdateAckPayload, ArtifactUpdatePayload,
    ArtifactUpdatedPayload, GuidanceAppliedPayload, GuidanceSendAckPayload, GuidanceSendPayload,
    GuidanceSentPayload, IterationPausedPayload, IterationResumedPayload, TaskControlAckPayload,
    TaskControlPayload, TaskTerminatedPayload,
};
use prompt_faster::domain::models::{
    Actor, BranchInfo, Checkpoint, CheckpointListResponse, CheckpointResponse, CheckpointSummary,
    ConflictResolutionRecord, ConnectivityResponse, ConnectivityStatus, Constraint,
    CreateTeacherPromptInput, DataSplit, DiagnosticReport, DiagnosticSummary, DiffSegment,
    DiffSegmentType, DimensionScore, EvaluationResult, EventType, ExecutionResult,
    ExecutionTargetType, ExportResultResponse, FailedCaseDetail, FailedCaseSummary, FailurePoint,
    FailureReasonEntry, HistoryEvent, HistoryEventResponse, HistoryExportData, Iteration,
    IterationExportEntry, IterationState, IterationSummaryEntry, LineageType,
    MetaOptimizationOverview, MetaOptimizationTaskSummary, OptimizationTaskEntity,
    OptimizationTaskMode, OptimizationTaskStatus, OutputLength, PassRateSummary,
    PromptPreviewRequest, PromptPreviewResponse, PromptPreviewResult, PromptValidationRequest,
    PromptValidationResult, QualityDimension, RecoveryMetrics, RecoveryRequest, RecoveryResponse,
    ResultExportFormat, RollbackRequest, RollbackResponse, Rule, RuleConflict, RuleConflictType,
    RuleIR, RuleMergeRecord, RuleSystem, RuleTags, Severity, TaskExportMeta, TaskHistoryResponse,
    TaskReference, TaskResultView, TeacherPrompt, TeacherPromptStats, TeacherPromptVersion,
    TestCase, TestSet, TimelineEntry, TimelineEntryType, TimelineResponse, TokenUsage, TurningPoint,
    TurningPointType, UnfinishedTask, UnfinishedTasksResponse, User, Workspace,
};
use prompt_faster::domain::types::{
    AddRoundsRequest, AddRoundsResponse, ArtifactSource, CandidatePrompt,
    CandidatePromptListResponse, CandidatePromptSummary, EvaluationResultSummary,
    IterationArtifacts, IterationHistoryDetail, IterationHistorySummary, IterationStatus,
    PatternHypothesis, RunControlState, TerminateTaskRequest, TerminateTaskResponse,
};
use prompt_faster::infra::external::dify_client::{
    DifyInputVariable, DifyValueType, DifyVariablesResponse, TestConnectionResult,
};
use ts_rs::TS;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/src/types/generated");
    std::fs::create_dir_all(&out_dir)?;

    // API 通用响应
    ApiSuccess::<()>::export_all_to(&out_dir)?;
    ApiError::export_all_to(&out_dir)?;
    ErrorDetail::export_all_to(&out_dir)?;
    PaginationMeta::export_all_to(&out_dir)?;

    // Health
    HealthResponse::export_all_to(&out_dir)?;

    // 连接测试
    TestDifyConnectionRequest::export_all_to(&out_dir)?;
    TestGenericLlmConnectionRequest::export_all_to(&out_dir)?;
    TestConnectionResult::export_all_to(&out_dir)?;
    DifyVariablesResponse::export_all_to(&out_dir)?;
    DifyInputVariable::export_all_to(&out_dir)?;
    DifyValueType::export_all_to(&out_dir)?;

    // 认证相关
    RegisterRequest::export_all_to(&out_dir)?;
    LoginRequest::export_all_to(&out_dir)?;
    AuthResponse::export_all_to(&out_dir)?;
    UserInfo::export_all_to(&out_dir)?;
    LogoutResponse::export_all_to(&out_dir)?;
    SystemStatusResponse::export_all_to(&out_dir)?;

    // 配置管理
    SaveConfigRequest::export_all_to(&out_dir)?;
    CredentialInput::export_all_to(&out_dir)?;
    GenericLlmCredentialInput::export_all_to(&out_dir)?;
    TeacherSettingsInput::export_all_to(&out_dir)?;
    ConfigResponse::export_all_to(&out_dir)?;
    TeacherSettingsResponse::export_all_to(&out_dir)?;
    SaveConfigResponse::export_all_to(&out_dir)?;
    GenericLlmModelsResponse::export_all_to(&out_dir)?;

    // 工作区
    CreateWorkspaceRequest::export_all_to(&out_dir)?;
    WorkspaceResponse::export_all_to(&out_dir)?;
    DeleteWorkspaceResponse::export_all_to(&out_dir)?;

    // 测试集
    CreateTestSetRequest::export_all_to(&out_dir)?;
    UpdateTestSetRequest::export_all_to(&out_dir)?;
    TestSetListItemResponse::export_all_to(&out_dir)?;
    TestSetResponse::export_all_to(&out_dir)?;
    DeleteTestSetResponse::export_all_to(&out_dir)?;

    // Dify 变量配置（测试集维度）
    DifyConfig::export_all_to(&out_dir)?;
    SaveDifyConfigRequest::export_all_to(&out_dir)?;
    SaveDifyConfigResponse::export_all_to(&out_dir)?;
    DifyBinding::export_all_to(&out_dir)?;
    DifyBindingSource::export_all_to(&out_dir)?;

    // 通用 API 自定义变量配置（测试集维度）
    GenericConfig::export_all_to(&out_dir)?;
    SaveGenericConfigRequest::export_all_to(&out_dir)?;
    SaveGenericConfigResponse::export_all_to(&out_dir)?;
    DeleteGenericConfigResponse::export_all_to(&out_dir)?;
    GenericInputVariable::export_all_to(&out_dir)?;
    GenericValueType::export_all_to(&out_dir)?;

    // 测试集模板
    SaveAsTemplateRequest::export_all_to(&out_dir)?;
    TestSetTemplateListItemResponse::export_all_to(&out_dir)?;
    TestSetTemplateResponse::export_all_to(&out_dir)?;

    // 优化任务
    CreateOptimizationTaskRequest::export_all_to(&out_dir)?;
    MetaOptimizationTaskHint::export_all_to(&out_dir)?;
    OptimizationTaskResponse::export_all_to(&out_dir)?;
    OptimizationTaskListItemResponse::export_all_to(&out_dir)?;
    UpdateOptimizationTaskConfigRequest::export_all_to(&out_dir)?;

    // 核心模型
    Workspace::export_all_to(&out_dir)?;
    User::export_all_to(&out_dir)?;
    TestCase::export_all_to(&out_dir)?;
    TestSet::export_all_to(&out_dir)?;
    OptimizationTaskEntity::export_all_to(&out_dir)?;
    Iteration::export_all_to(&out_dir)?;
    EvaluationResult::export_all_to(&out_dir)?;
    Checkpoint::export_all_to(&out_dir)?;
    CheckpointResponse::export_all_to(&out_dir)?;
    CheckpointListResponse::export_all_to(&out_dir)?;
    CheckpointSummary::export_all_to(&out_dir)?;
    PassRateSummary::export_all_to(&out_dir)?;
    TaskHistoryResponse::export_all_to(&out_dir)?;
    TaskResultView::export_all_to(&out_dir)?;
    IterationSummaryEntry::export_all_to(&out_dir)?;
    ExportResultResponse::export_all_to(&out_dir)?;
    DiagnosticReport::export_all_to(&out_dir)?;
    DiagnosticSummary::export_all_to(&out_dir)?;
    FailureReasonEntry::export_all_to(&out_dir)?;
    TurningPoint::export_all_to(&out_dir)?;
    TurningPointType::export_all_to(&out_dir)?;
    FailedCaseSummary::export_all_to(&out_dir)?;
    FailedCaseDetail::export_all_to(&out_dir)?;
    DiffSegment::export_all_to(&out_dir)?;
    DiffSegmentType::export_all_to(&out_dir)?;
    CreateTeacherPromptInput::export_all_to(&out_dir)?;
    TeacherPrompt::export_all_to(&out_dir)?;
    TeacherPromptVersion::export_all_to(&out_dir)?;
    TeacherPromptStats::export_all_to(&out_dir)?;
    MetaOptimizationOverview::export_all_to(&out_dir)?;
    MetaOptimizationTaskSummary::export_all_to(&out_dir)?;
    PromptPreviewRequest::export_all_to(&out_dir)?;
    PromptPreviewResult::export_all_to(&out_dir)?;
    PromptPreviewResponse::export_all_to(&out_dir)?;
    PromptValidationRequest::export_all_to(&out_dir)?;
    PromptValidationResult::export_all_to(&out_dir)?;
    HistoryEvent::export_all_to(&out_dir)?;
    HistoryEventResponse::export_all_to(&out_dir)?;
    TimelineEntry::export_all_to(&out_dir)?;
    TimelineEntryType::export_all_to(&out_dir)?;
    TimelineResponse::export_all_to(&out_dir)?;
    HistoryExportData::export_all_to(&out_dir)?;
    TaskExportMeta::export_all_to(&out_dir)?;
    IterationExportEntry::export_all_to(&out_dir)?;
    BranchInfo::export_all_to(&out_dir)?;
    EventType::export_all_to(&out_dir)?;
    Actor::export_all_to(&out_dir)?;
    UnfinishedTask::export_all_to(&out_dir)?;
    UnfinishedTasksResponse::export_all_to(&out_dir)?;
    RecoveryRequest::export_all_to(&out_dir)?;
    RecoveryResponse::export_all_to(&out_dir)?;
    RollbackRequest::export_all_to(&out_dir)?;
    RollbackResponse::export_all_to(&out_dir)?;
    RecoveryMetrics::export_all_to(&out_dir)?;
    ConnectivityStatus::export_all_to(&out_dir)?;
    ConnectivityResponse::export_all_to(&out_dir)?;
    // 规则与评估相关模型
    Rule::export_all_to(&out_dir)?;
    RuleTags::export_all_to(&out_dir)?;
    RuleIR::export_all_to(&out_dir)?;
    RuleSystem::export_all_to(&out_dir)?;
    RuleMergeRecord::export_all_to(&out_dir)?;
    RuleConflict::export_all_to(&out_dir)?;
    RuleConflictType::export_all_to(&out_dir)?;
    ConflictResolutionRecord::export_all_to(&out_dir)?;
    // 评估/执行相关模型
    DimensionScore::export_all_to(&out_dir)?;
    FailurePoint::export_all_to(&out_dir)?;
    Severity::export_all_to(&out_dir)?;
    ExecutionResult::export_all_to(&out_dir)?;
    TokenUsage::export_all_to(&out_dir)?;
    // 枚举/辅助模型
    DataSplit::export_all_to(&out_dir)?;
    ExecutionTargetType::export_all_to(&out_dir)?;
    OptimizationTaskMode::export_all_to(&out_dir)?;
    OptimizationTaskStatus::export_all_to(&out_dir)?;
    TaskReference::export_all_to(&out_dir)?;
    Constraint::export_all_to(&out_dir)?;
    QualityDimension::export_all_to(&out_dir)?;
    OutputLength::export_all_to(&out_dir)?;
    IterationState::export_all_to(&out_dir)?;
    LineageType::export_all_to(&out_dir)?;
    RunControlState::export_all_to(&out_dir)?;
    ResultExportFormat::export_all_to(&out_dir)?;

    // WS 事件负载
    TaskControlPayload::export_all_to(&out_dir)?;
    TaskControlAckPayload::export_all_to(&out_dir)?;
    IterationPausedPayload::export_all_to(&out_dir)?;
    IterationResumedPayload::export_all_to(&out_dir)?;
    GuidanceSendPayload::export_all_to(&out_dir)?;
    GuidanceSendAckPayload::export_all_to(&out_dir)?;
    GuidanceSentPayload::export_all_to(&out_dir)?;
    GuidanceAppliedPayload::export_all_to(&out_dir)?;
    TaskTerminatedPayload::export_all_to(&out_dir)?;

    // Artifact 相关类型
    IterationArtifacts::export_all_to(&out_dir)?;
    PatternHypothesis::export_all_to(&out_dir)?;
    CandidatePrompt::export_all_to(&out_dir)?;
    ArtifactSource::export_all_to(&out_dir)?;
    ArtifactGetPayload::export_all_to(&out_dir)?;
    ArtifactGetAckPayload::export_all_to(&out_dir)?;
    ArtifactUpdatePayload::export_all_to(&out_dir)?;
    ArtifactUpdateAckPayload::export_all_to(&out_dir)?;
    ArtifactUpdatedPayload::export_all_to(&out_dir)?;

    // 历史迭代类型
    IterationStatus::export_all_to(&out_dir)?;
    EvaluationResultSummary::export_all_to(&out_dir)?;
    IterationHistorySummary::export_all_to(&out_dir)?;
    IterationHistoryDetail::export_all_to(&out_dir)?;

    // 迭代控制类型
    AddRoundsRequest::export_all_to(&out_dir)?;
    AddRoundsResponse::export_all_to(&out_dir)?;
    CandidatePromptSummary::export_all_to(&out_dir)?;
    CandidatePromptListResponse::export_all_to(&out_dir)?;
    TerminateTaskRequest::export_all_to(&out_dir)?;
    TerminateTaskResponse::export_all_to(&out_dir)?;

    Ok(())
}
