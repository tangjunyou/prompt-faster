//! OptimizationContext.extensions 约定键（跨 Layer/编排层协作契约）
//!
//! 设计目标：
//! - 统一维护 extensions 的 key，避免 core 模块之间互相依赖常量
//! - 作为“编排层注入/消费”的契约，不属于某个单一 Trait 模块

use serde::{Deserialize, Serialize};

/// 评分/通过率类指标的浮点比较 epsilon。
///
/// 这些指标被设计为 `[0, 1]`，使用绝对误差足够。
pub const METRIC_EPS: f64 = 1e-12;

/// Layer 4：候选排序（来自 Layer 3 `rank_candidates` 输出）。
pub const EXT_CANDIDATE_RANKING: &str = "layer4.candidate_ranking";
pub const EXT_BEST_CANDIDATE_INDEX: &str = "layer4.best_candidate_index";
pub const EXT_BEST_CANDIDATE_PROMPT: &str = "layer4.best_candidate_prompt";
pub const EXT_CURRENT_PROMPT_STATS: &str = "layer4.current_prompt_stats";
pub const EXT_BEST_CANDIDATE_STATS: &str = "layer4.best_candidate_stats";

/// Layer 4：用于在 `optimize_step` 阶段做“震荡/停滞”检测的历史分数序列。
///
/// 形状：`Vec<f64>`，表示历史轮次的 `OptimizationResult.primary.score`（不包含当前轮）。
pub const EXT_RECENT_PRIMARY_SCORES: &str = "layer4.recent_primary_scores";

/// Layer 1/3：逐用例评估映射（既有约定；不要引入同义 key）。
pub const EXT_EVALUATIONS_BY_TEST_CASE_ID: &str = "layer1_test_results.evaluations_by_test_case_id";

/// Optimizer 输出：是否建议采用 best candidate 更新 current_prompt（由编排层执行写回）。
pub const EXTRA_ADOPT_BEST_CANDIDATE: &str = "adopt_best_candidate";

/// Layer 4：候选统计（由编排层根据 Layer 3 的统计口径注入）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CandidateStats {
    pub pass_rate: f64,
    pub mean_score: f64,
}
