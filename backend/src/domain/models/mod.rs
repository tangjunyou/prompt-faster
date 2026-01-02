//! 领域模型定义

pub mod algorithm;
pub mod user;
pub mod workspace;

pub use algorithm::{
    Checkpoint, ConflictResolutionRecord, Constraint, DataSplit, DimensionScore, EvaluationResult,
    ExecutionResult, FailurePoint, Iteration, IterationState, LineageType, OptimizationTask,
    OutputLength, QualityDimension, Rule, RuleConflict, RuleConflictType, RuleIR, RuleMergeRecord,
    RuleSystem, RuleTags, Severity, TaskReference, TestCase, TokenUsage,
};
pub use user::User;
pub use workspace::Workspace;
