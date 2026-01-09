mod default_impl;
mod error;

pub use default_impl::DefaultOptimizer;
pub use error::OptimizerError;

pub use crate::domain::types::{
    EXT_BEST_CANDIDATE_INDEX, EXT_BEST_CANDIDATE_PROMPT, EXT_BEST_CANDIDATE_STATS,
    EXT_CANDIDATE_RANKING, EXT_CURRENT_PROMPT_STATS, EXT_EVALUATIONS_BY_TEST_CASE_ID,
    EXT_RECENT_PRIMARY_SCORES, EXTRA_ADOPT_BEST_CANDIDATE,
};
