//! 领域类型定义

pub mod artifacts;
pub mod extensions;
pub mod iteration_control;
pub mod iteration_history;
pub mod optimization_context;

pub use artifacts::{
    ArtifactSource, CandidatePrompt, GuidanceStatus, IterationArtifacts, PatternHypothesis,
    UserGuidance,
};
pub use extensions::{
    CandidateStats, EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_BRANCH_ID, EXT_CANDIDATE_RANKING, EXT_CONSECUTIVE_NO_IMPROVEMENT, EXT_CURRENT_PROMPT_STATS,
    EXT_EVALUATIONS_BY_TEST_CASE_ID, EXT_FAILURE_ARCHIVE, EXT_PREV_ITERATION_STATE,
    EXT_RECENT_PRIMARY_SCORES, EXT_USER_GUIDANCE, EXTRA_ADOPT_BEST_CANDIDATE,
    FAILURE_ARCHIVE_MAX_ENTRIES, METRIC_EPS,
};
pub use iteration_control::{
    AddRoundsRequest, AddRoundsResponse, CandidatePromptListResponse, CandidatePromptSummary,
    TerminateTaskRequest, TerminateTaskResponse,
};
pub use iteration_history::{
    EvaluationResultSummary, IterationHistoryDetail, IterationHistorySummary, IterationStatus,
    unix_ms_to_iso8601,
};
pub use optimization_context::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext, OscillationAction,
    OscillationConfig, OutputConfig, OutputStrategy, RacingConfig, RuleConfig, RunControlState,
    RunControlStateTransitionError, SplitStrategy,
};
