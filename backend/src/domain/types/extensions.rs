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

/// Layer 4：失败档案（用于候选去重/避坑）。
///
/// 形状：`Vec<domain::models::FailureArchiveEntry>`
pub const EXT_FAILURE_ARCHIVE: &str = "layer4.failure_archive";

/// Layer 4：连续无提升计数（由编排层维护并注入；Layer 4 只读使用）。
///
/// 形状：number（u32）
pub const EXT_CONSECUTIVE_NO_IMPROVEMENT: &str = "layer4.consecutive_no_improvement";

/// 失败档案条目上限（FIFO 丢弃最旧；避免无界增长）。
pub const FAILURE_ARCHIVE_MAX_ENTRIES: usize = 200;

/// Layer 1/3：逐用例评估映射（既有约定；不要引入同义 key）。
pub const EXT_EVALUATIONS_BY_TEST_CASE_ID: &str = "layer1_test_results.evaluations_by_test_case_id";

/// Optimizer 输出：是否建议采用 best candidate 更新 current_prompt（由编排层执行写回）。
pub const EXTRA_ADOPT_BEST_CANDIDATE: &str = "adopt_best_candidate";

/// 迭代状态切换前的状态快照（用于日志/审计）。
pub const EXT_PREV_ITERATION_STATE: &str = "iteration.prev_state";

/// 当前 Checkpoint 分支 ID（用于回滚后继续迭代）。
pub const EXT_BRANCH_ID: &str = "checkpoint.branch_id";

/// 用户引导信息（由编排层在 resume 时注入，Layer 1-4 消费）。
///
/// 形状：`domain::types::UserGuidance`
/// 生命周期：
///   - 注入时机：resume 后、Layer 1 开始前
///   - 消费时机：Layer 1-4 老师模型调用时读取
///   - 清理时机：当轮迭代结束后从 extensions 中移除
pub const EXT_USER_GUIDANCE: &str = "user_guidance";

/// Layer 4：候选统计（由编排层根据 Layer 3 的统计口径注入）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CandidateStats {
    pub pass_rate: f64,
    pub mean_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ext_user_guidance_key_stable() {
        // 契约回归测试：确保 key 值不被意外修改
        assert_eq!(EXT_USER_GUIDANCE, "user_guidance");
    }

    #[test]
    fn test_extension_keys_unique() {
        // 确保所有 extension key 唯一，避免冲突
        let keys = vec![
            EXT_BEST_CANDIDATE_INDEX,
            EXT_BEST_CANDIDATE_PROMPT,
            EXT_BEST_CANDIDATE_STATS,
            EXT_CURRENT_PROMPT_STATS,
            EXT_CONSECUTIVE_NO_IMPROVEMENT,
            EXT_RECENT_PRIMARY_SCORES,
            EXT_FAILURE_ARCHIVE,
            EXT_CANDIDATE_RANKING,
            EXT_EVALUATIONS_BY_TEST_CASE_ID,
            EXTRA_ADOPT_BEST_CANDIDATE,
            EXT_PREV_ITERATION_STATE,
            EXT_USER_GUIDANCE,
            EXT_BRANCH_ID,
        ];
        let unique: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(keys.len(), unique.len(), "Extension keys must be unique");
    }
}
