use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::models::IterationState;

/// 迭代“阶段/口径”说明（由后端提供，前端只做展示映射，不做推断）。
///
/// 目标：
/// - 阶段口径集中化，避免前端/后端各写一套导致漂移
/// - 为 Epic 5（Thinking Panel / Stage Indicator）提供可复用的权威映射
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct IterationStageDescriptor {
    pub state: IterationState,
    /// 稳定的分组 key（用于 UI 归类/配色），snake_case。
    pub group: String,
    /// 面向用户的阶段名（当前为中文；如需多语言，后续扩展字段）。
    pub label: String,
    /// 推荐展示顺序（越小越靠前）。
    pub order: u32,
}

fn descriptor(
    state: IterationState,
    group: &str,
    label: &str,
    order: u32,
) -> IterationStageDescriptor {
    IterationStageDescriptor {
        state,
        group: group.to_string(),
        label: label.to_string(),
        order,
    }
}

/// 获取单个 IterationState 的阶段描述（权威映射）。
pub fn stage_for_state(state: IterationState) -> IterationStageDescriptor {
    match state {
        IterationState::Idle => descriptor(state, "idle", "空闲", 0),
        IterationState::Initializing => descriptor(state, "setup", "初始化", 10),
        IterationState::ExtractingRules => descriptor(state, "rules", "提取规律", 20),
        IterationState::DetectingConflicts => descriptor(state, "rules", "检测冲突", 30),
        IterationState::ResolvingConflicts => descriptor(state, "rules", "解决冲突", 40),
        IterationState::MergingSimilarRules => descriptor(state, "rules", "合并相似规律", 50),
        IterationState::ValidatingRules => descriptor(state, "rules", "验证规律", 60),
        IterationState::GeneratingPrompt => descriptor(state, "prompt", "生成候选 Prompt", 70),
        IterationState::RunningTests => descriptor(state, "execution", "执行测试", 80),
        IterationState::Evaluating => descriptor(state, "evaluation", "评估", 90),
        IterationState::ClusteringFailures => descriptor(state, "evaluation", "聚类失败", 100),
        IterationState::Reflecting => descriptor(state, "reflection", "反思", 110),
        IterationState::UpdatingRules => descriptor(state, "rules", "更新规律", 120),
        IterationState::Optimizing => descriptor(state, "optimization", "优化", 130),
        IterationState::SmartRetesting => descriptor(state, "execution", "智能复测", 140),
        IterationState::SafetyChecking => descriptor(state, "safety", "安全检查", 150),
        IterationState::WaitingUser => descriptor(state, "control", "等待用户", 160),
        IterationState::HumanIntervention => descriptor(state, "control", "用户介入", 170),
        IterationState::Completed => descriptor(state, "terminal", "完成", 900),
        IterationState::MaxIterationsReached => {
            descriptor(state, "terminal", "达到最大迭代次数", 910)
        }
        IterationState::UserStopped => descriptor(state, "terminal", "用户停止", 920),
        IterationState::Failed => descriptor(state, "terminal", "失败", 930),
    }
}

/// 返回所有 IterationState 的阶段描述列表（按 order 排序）。
pub fn all_stages() -> Vec<IterationStageDescriptor> {
    use IterationState::*;
    let mut out = vec![
        stage_for_state(Idle),
        stage_for_state(Initializing),
        stage_for_state(ExtractingRules),
        stage_for_state(DetectingConflicts),
        stage_for_state(ResolvingConflicts),
        stage_for_state(MergingSimilarRules),
        stage_for_state(ValidatingRules),
        stage_for_state(GeneratingPrompt),
        stage_for_state(RunningTests),
        stage_for_state(Evaluating),
        stage_for_state(ClusteringFailures),
        stage_for_state(Reflecting),
        stage_for_state(UpdatingRules),
        stage_for_state(Optimizing),
        stage_for_state(SmartRetesting),
        stage_for_state(SafetyChecking),
        stage_for_state(WaitingUser),
        stage_for_state(HumanIntervention),
        stage_for_state(Completed),
        stage_for_state(MaxIterationsReached),
        stage_for_state(UserStopped),
        stage_for_state(Failed),
    ];
    out.sort_by_key(|d| d.order);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_stages_are_unique_and_sorted() {
        let stages = all_stages();
        assert!(!stages.is_empty());

        let mut seen = std::collections::HashSet::new();
        for s in &stages {
            assert!(
                seen.insert(format!("{:?}", s.state)),
                "duplicate state: {:?}",
                s.state
            );
        }

        for w in stages.windows(2) {
            assert!(w[0].order <= w[1].order, "order not sorted");
        }
    }
}
