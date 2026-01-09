//! 领域类型定义

pub mod extensions;
pub mod optimization_context;

pub use extensions::{
    CandidateStats, EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_CANDIDATE_RANKING, EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID,
    EXT_RECENT_PRIMARY_SCORES, EXTRA_ADOPT_BEST_CANDIDATE, METRIC_EPS,
};
pub use optimization_context::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, OscillationAction,
    OscillationConfig, OutputConfig, OutputStrategy, RacingConfig, RuleConfig, SplitStrategy,
};
