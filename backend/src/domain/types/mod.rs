//! 领域类型定义

pub mod optimization_context;

pub use optimization_context::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, OscillationAction,
    OscillationConfig, OutputConfig, OutputStrategy, RacingConfig, RuleConfig, SplitStrategy,
};
