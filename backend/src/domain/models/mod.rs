//! 领域模型定义

pub mod algorithm;
pub mod checkpoint;
pub mod diagnostic;
pub mod evaluation_result;
pub mod history;
pub mod history_event;
pub mod iteration_stage;
pub mod optimization_task;
pub mod optimization_task_config;
pub mod recovery;
pub mod reflection;
pub mod test_set;
pub mod user;
pub mod workspace;

pub use algorithm::{
    ConflictResolutionRecord, Constraint, DataSplit, DimensionScore, EvaluationResult,
    ExecutionResult, FailureArchiveEntry, FailurePoint, Iteration, IterationState, LineageType,
    OutputLength, QualityDimension, Rule, RuleConflict, RuleConflictType, RuleIR, RuleMergeRecord,
    RuleSystem, RuleTags, Severity, TaskReference, TestCase, TokenUsage, failure_fingerprint_v1,
};
pub use checkpoint::{
    Checkpoint, CheckpointCreateRequest, CheckpointEntity, CheckpointFull, CheckpointListResponse,
    CheckpointResponse,
};
pub use diagnostic::{
    DiagnosticReport, DiagnosticSummary, DiffSegment, DiffSegmentType, FailedCaseDetail,
    FailedCaseSummary, FailureReasonEntry, TurningPoint, TurningPointType,
};
pub use evaluation_result::{
    ExportResultResponse, IterationSummaryEntry, ResultExportFormat, TaskResultView,
};
pub use history::{
    BranchInfo, HistoryEventResponse, HistoryExportData, IterationExportEntry, TaskExportMeta,
    TaskHistoryResponse, TimelineEntry, TimelineEntryType, TimelineResponse,
};
pub use history_event::{Actor, EventType, HistoryEvent, HistoryEventFilter};
pub use iteration_stage::{
    IterationStageDescriptor, all_stages as all_iteration_stages, stage_for_state,
};
pub use optimization_task::{
    ExecutionTargetType, OptimizationTaskEntity, OptimizationTaskMode, OptimizationTaskStatus,
};
pub use optimization_task_config::{
    AdvancedDataSplitConfig, AdvancedDataSplitStrategy, ConstraintCheckEvaluatorConfig,
    DataSplitPercentConfig, EvaluatorConfig, EvaluatorType, ExactMatchEvaluatorConfig,
    ExecutionMode, OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION, OptimizationTaskConfig, OutputConfig,
    OutputStrategy, SamplingStrategy, SemanticSimilarityEvaluatorConfig, TeacherLlmConfig,
    TeacherModelEvaluatorConfig,
};
pub use recovery::{
    CheckpointSummary, CheckpointWithSummary, ConnectivityResponse, ConnectivityStatus,
    PassRateSummary, RecoveryMetrics, RecoveryRequest, RecoveryResponse, RollbackRequest,
    RollbackResponse, UnfinishedTask, UnfinishedTasksResponse,
};
pub use reflection::{
    ArbitrationMethod, ArbitrationResult, CandidateSource, ConflictType, FailureType,
    OptimizationResult, PromptCandidate, RecommendedAction, ReflectionResult, Suggestion,
    SuggestionConflict, SuggestionType, TerminationReason, UnifiedReflection, UnifiedSuggestion,
};
pub use test_set::TestSet;
pub use user::User;
pub use workspace::Workspace;
