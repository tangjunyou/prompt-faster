//! 领域模型定义

pub mod algorithm;
pub mod optimization_task;
pub mod optimization_task_config;
pub mod reflection;
pub mod test_set;
pub mod user;
pub mod workspace;

pub use algorithm::{
    Checkpoint, ConflictResolutionRecord, Constraint, DataSplit, DimensionScore, EvaluationResult,
    ExecutionResult, FailurePoint, Iteration, IterationState, LineageType, OutputLength,
    QualityDimension, Rule, RuleConflict, RuleConflictType, RuleIR, RuleMergeRecord, RuleSystem,
    RuleTags, Severity, TaskReference, TestCase, TokenUsage,
};
pub use optimization_task::{
    ExecutionTargetType, OptimizationTaskEntity, OptimizationTaskMode, OptimizationTaskStatus,
};
pub use optimization_task_config::{
    AdvancedDataSplitConfig, AdvancedDataSplitStrategy, ConstraintCheckEvaluatorConfig,
    DataSplitPercentConfig, EvaluatorConfig, EvaluatorType, ExactMatchEvaluatorConfig,
    OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION, OptimizationTaskConfig, OutputConfig, OutputStrategy,
    SamplingStrategy, SemanticSimilarityEvaluatorConfig, TeacherLlmConfig,
    TeacherModelEvaluatorConfig,
};
pub use reflection::{
    ArbitrationMethod, ArbitrationResult, CandidateSource, ConflictType, FailureType,
    OptimizationResult, PromptCandidate, RecommendedAction, ReflectionResult, Suggestion,
    SuggestionConflict, SuggestionType, TerminationReason, UnifiedReflection, UnifiedSuggestion,
};
pub use test_set::TestSet;
pub use user::User;
pub use workspace::Workspace;
