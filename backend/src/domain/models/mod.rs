//! 领域模型定义

pub mod user;
pub mod workspace;
pub mod algorithm;

pub use user::User;
pub use workspace::Workspace;
pub use algorithm::{
    Checkpoint, ConflictResolutionRecord, Constraint, DataSplit, DimensionScore, EvaluationResult,
    ExecutionResult, FailurePoint, Iteration, IterationState, LineageType, OptimizationTask,
    OutputLength, QualityDimension, Rule, RuleConflict, RuleConflictType, RuleIR, RuleMergeRecord,
    RuleSystem, RuleTags, Severity, TaskReference, TestCase, TokenUsage,
};
