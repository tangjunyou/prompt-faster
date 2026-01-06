//! 领域模型定义

pub mod algorithm;
pub mod optimization_task;
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
pub use test_set::TestSet;
pub use user::User;
pub use workspace::Workspace;
