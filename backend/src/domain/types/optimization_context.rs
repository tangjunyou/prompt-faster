use crate::domain::models::{Checkpoint, IterationState, RuleSystem, TestCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use ts_rs::TS;

/// 运行控制状态（与 IterationState 正交，控制整体运行/暂停/停止）
///
/// 设计说明：
/// - `RunControlState` 是顶层运行控制状态，与细粒度 `IterationState` 正交
/// - 暂停时 `IterationState` 保持不变，仅 `RunControlState` 变为 `Paused`
/// - 状态转换由编排层统一管理，禁止非法跳转（如 Paused → Stopped 需先 Resume）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum RunControlState {
    /// 空闲（任务未开始或已完成）
    #[default]
    Idle,
    /// 运行中
    Running,
    /// 已暂停（用户主动暂停，可恢复）
    Paused,
    /// 已停止（用户主动终止或发生不可恢复错误，不可恢复）
    Stopped,
}

impl RunControlState {
    /// 检查是否允许转换到目标状态
    pub fn can_transition_to(&self, target: RunControlState) -> bool {
        use RunControlState::*;
        match (*self, target) {
            // 相同状态，允许（幂等）
            (s, t) if s == t => true,
            // Idle → Running：启动任务
            (Idle, Running) => true,
            // Running → Paused：暂停
            (Running, Paused) => true,
            // Running → Stopped：终止
            (Running, Stopped) => true,
            // Running → Idle：正常完成
            (Running, Idle) => true,
            // Paused → Running：继续
            (Paused, Running) => true,
            // Paused → Stopped：暂停后终止
            (Paused, Stopped) => true,
            // Stopped → Idle：重置（用于开始新任务）
            (Stopped, Idle) => true,
            // 其他转换禁止
            _ => false,
        }
    }

    /// 尝试转换到目标状态，失败返回错误
    pub fn try_transition_to(
        &mut self,
        target: RunControlState,
    ) -> Result<(), RunControlStateTransitionError> {
        if self.can_transition_to(target) {
            *self = target;
            Ok(())
        } else {
            Err(RunControlStateTransitionError::InvalidTransition {
                from: *self,
                to: target,
            })
        }
    }

    /// 是否可以执行暂停操作
    pub fn can_pause(&self) -> bool {
        *self == RunControlState::Running
    }

    /// 是否可以执行继续操作
    pub fn can_resume(&self) -> bool {
        *self == RunControlState::Paused
    }

    /// 是否处于活跃状态（Running 或 Paused）
    pub fn is_active(&self) -> bool {
        matches!(self, RunControlState::Running | RunControlState::Paused)
    }
}

/// 运行控制状态转换错误
#[derive(Debug, Clone, Error)]
pub enum RunControlStateTransitionError {
    #[error("非法状态转换：{from:?} → {to:?}")]
    InvalidTransition {
        from: RunControlState,
        to: RunControlState,
    },
}

/// 优化上下文（贯穿整个迭代流程的共享状态）
///
/// 设计原则：
/// - 各模块通过只读引用 `&OptimizationContext` 访问
/// - 只有编排层（Orchestrator）能更新 Context
/// - extensions 字段支持未来扩展而不改结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationContext {
    pub task_id: String,
    pub execution_target_config: ExecutionTargetConfig,
    pub current_prompt: String,
    pub rule_system: RuleSystem,
    pub iteration: u32,
    pub state: IterationState,
    /// 运行控制状态（与 IterationState 正交）
    #[serde(default)]
    pub run_control_state: RunControlState,
    pub test_cases: Vec<TestCase>,
    pub config: OptimizationConfig,
    pub checkpoints: Vec<Checkpoint>,
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 执行目标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionTargetConfig {
    Dify {
        api_url: String,
        workflow_id: String,
        prompt_variable: String,
        /// 运行时注入的 Dify API Key（不允许序列化/持久化）。
        ///
        /// 说明：凭证应从 CredentialRepo 解密后注入，仅在本次执行生命周期内存在。
        #[serde(skip_serializing, skip_deserializing)]
        api_key: Option<String>,
    },
    DirectModel {
        /// OpenAI 兼容 API Base URL（例如 https://api.siliconflow.cn）。
        base_url: String,
        model_name: String,
        user_prompt_template: String,
        /// 运行时注入的 API Key（不允许序列化/持久化）。
        #[serde(skip_serializing, skip_deserializing)]
        api_key: Option<String>,
    },
}

impl Default for ExecutionTargetConfig {
    fn default() -> Self {
        Self::DirectModel {
            base_url: "http://localhost".to_string(),
            model_name: "unknown".to_string(),
            user_prompt_template: "{input}".to_string(),
            api_key: None,
        }
    }
}

/// 优化配置（用户可调整的算法参数）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizationConfig {
    pub output: OutputConfig,
    pub minibatch: MinibatchConfig,
    pub oscillation: OscillationConfig,
    pub rule: RuleConfig,
    pub iteration: IterationConfig,
    pub data_split: DataSplitConfig,
    pub evaluator: EvaluatorConfig,
    pub budget: BudgetConfig,
    pub racing: RacingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_strategy")]
    pub strategy: OutputStrategy,
    #[serde(default = "default_conflict_alert_threshold")]
    pub conflict_alert_threshold: u32,
    #[serde(default = "default_true")]
    pub auto_recommend: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            strategy: default_output_strategy(),
            conflict_alert_threshold: default_conflict_alert_threshold(),
            auto_recommend: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputStrategy {
    #[default]
    Single,
    Adaptive,
    Multi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinibatchConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_minibatch_size")]
    pub size: u32,
    #[serde(default = "default_full_eval_interval")]
    pub full_eval_interval: u32,
    #[serde(default = "default_minibatch_recommend_threshold")]
    pub recommend_threshold: u32,
}

impl Default for MinibatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            size: default_minibatch_size(),
            full_eval_interval: default_full_eval_interval(),
            recommend_threshold: default_minibatch_recommend_threshold(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillationConfig {
    #[serde(default = "default_oscillation_threshold")]
    pub threshold: u32,
    #[serde(default)]
    pub action: OscillationAction,
}

impl Default for OscillationConfig {
    fn default() -> Self {
        Self {
            threshold: default_oscillation_threshold(),
            action: OscillationAction::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OscillationAction {
    #[default]
    DiversityInject,
    HumanIntervention,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    #[serde(default = "default_max_abstraction_level")]
    pub max_abstraction_level: u32,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f64,
    #[serde(default)]
    pub enable_clustering: bool,
    #[serde(default = "default_clustering_threshold")]
    pub clustering_threshold: u32,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            max_abstraction_level: default_max_abstraction_level(),
            similarity_threshold: default_similarity_threshold(),
            enable_clustering: false,
            clustering_threshold: default_clustering_threshold(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationConfig {
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_pass_threshold")]
    pub pass_threshold: f64,
    #[serde(default = "default_diversity_inject_after")]
    pub diversity_inject_after: u32,
}

impl Default for IterationConfig {
    fn default() -> Self {
        Self {
            max_iterations: default_max_iterations(),
            pass_threshold: default_pass_threshold(),
            diversity_inject_after: default_diversity_inject_after(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSplitConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_train_ratio")]
    pub train_ratio: f64,
    #[serde(default = "default_validation_ratio")]
    pub validation_ratio: f64,
    #[serde(default)]
    pub strategy: SplitStrategy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default = "default_overfitting_threshold")]
    pub overfitting_threshold: f64,
}

impl Default for DataSplitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            train_ratio: default_train_ratio(),
            validation_ratio: default_validation_ratio(),
            strategy: SplitStrategy::default(),
            seed: None,
            overfitting_threshold: default_overfitting_threshold(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SplitStrategy {
    #[default]
    Random,
    Stratified,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    #[serde(default = "default_true")]
    pub ensemble_enabled: bool,
    #[serde(default = "default_confidence_high_threshold")]
    pub confidence_high_threshold: f64,
    #[serde(default = "default_confidence_low_threshold")]
    pub confidence_low_threshold: f64,
    #[serde(default = "default_llm_judge_samples")]
    pub llm_judge_samples: u32,
    #[serde(default = "default_hard_checks_weight")]
    pub hard_checks_weight: f64,
    #[serde(default = "default_agreement_weight")]
    pub agreement_weight: f64,
    #[serde(default = "default_variance_penalty")]
    pub variance_penalty: f64,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            ensemble_enabled: default_true(),
            confidence_high_threshold: default_confidence_high_threshold(),
            confidence_low_threshold: default_confidence_low_threshold(),
            llm_judge_samples: default_llm_judge_samples(),
            hard_checks_weight: default_hard_checks_weight(),
            agreement_weight: default_agreement_weight(),
            variance_penalty: default_variance_penalty(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_llm_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_secs: Option<u64>,
    #[serde(default = "default_budget_warn_threshold")]
    pub warn_threshold: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_llm_calls: None,
            max_tokens: None,
            max_duration_secs: None,
            warn_threshold: default_budget_warn_threshold(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RacingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_racing_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_elimination_rounds")]
    pub elimination_rounds: u32,
    #[serde(default = "default_survival_threshold")]
    pub survival_threshold: f64,
    #[serde(default = "default_early_stop_confidence")]
    pub early_stop_confidence: f64,
}

impl Default for RacingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            pool_size: default_racing_pool_size(),
            elimination_rounds: default_elimination_rounds(),
            survival_threshold: default_survival_threshold(),
            early_stop_confidence: default_early_stop_confidence(),
        }
    }
}

fn default_output_strategy() -> OutputStrategy {
    OutputStrategy::Single
}
fn default_conflict_alert_threshold() -> u32 {
    3
}
fn default_true() -> bool {
    true
}
fn default_minibatch_size() -> u32 {
    10
}
fn default_full_eval_interval() -> u32 {
    5
}
fn default_minibatch_recommend_threshold() -> u32 {
    20
}
fn default_oscillation_threshold() -> u32 {
    3
}
fn default_max_abstraction_level() -> u32 {
    3
}
fn default_similarity_threshold() -> f64 {
    0.8
}
fn default_clustering_threshold() -> u32 {
    50
}
fn default_max_iterations() -> u32 {
    20
}
fn default_pass_threshold() -> f64 {
    0.95
}
fn default_diversity_inject_after() -> u32 {
    3
}
fn default_train_ratio() -> f64 {
    0.70
}
fn default_validation_ratio() -> f64 {
    0.15
}
fn default_overfitting_threshold() -> f64 {
    0.10
}
fn default_confidence_high_threshold() -> f64 {
    0.8
}
fn default_confidence_low_threshold() -> f64 {
    0.5
}
fn default_llm_judge_samples() -> u32 {
    1
}
fn default_hard_checks_weight() -> f64 {
    0.4
}
fn default_agreement_weight() -> f64 {
    0.4
}
fn default_variance_penalty() -> f64 {
    0.2
}
fn default_budget_warn_threshold() -> f64 {
    0.8
}
fn default_racing_pool_size() -> u32 {
    3
}
fn default_elimination_rounds() -> u32 {
    3
}
fn default_survival_threshold() -> f64 {
    0.85
}
fn default_early_stop_confidence() -> f64 {
    0.95
}

#[cfg(test)]
mod run_control_state_tests {
    use super::*;

    #[test]
    fn default_state_is_idle() {
        assert_eq!(RunControlState::default(), RunControlState::Idle);
    }

    #[test]
    fn valid_transitions_are_allowed() {
        use RunControlState::*;

        // Idle → Running
        assert!(Idle.can_transition_to(Running));
        // Running → Paused
        assert!(Running.can_transition_to(Paused));
        // Running → Stopped
        assert!(Running.can_transition_to(Stopped));
        // Running → Idle (正常完成)
        assert!(Running.can_transition_to(Idle));
        // Paused → Running
        assert!(Paused.can_transition_to(Running));
        // Paused → Stopped
        assert!(Paused.can_transition_to(Stopped));
        // Stopped → Idle
        assert!(Stopped.can_transition_to(Idle));
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        use RunControlState::*;

        // Idle 不能直接 → Paused
        assert!(!Idle.can_transition_to(Paused));
        // Idle 不能直接 → Stopped
        assert!(!Idle.can_transition_to(Stopped));
        // Paused 不能直接 → Idle（需先 Resume 再完成）
        assert!(!Paused.can_transition_to(Idle));
        // Stopped 不能直接 → Running
        assert!(!Stopped.can_transition_to(Running));
        // Stopped 不能直接 → Paused
        assert!(!Stopped.can_transition_to(Paused));
    }

    #[test]
    fn idempotent_transitions_are_allowed() {
        use RunControlState::*;

        // 相同状态转换应该成功（幂等）
        assert!(Idle.can_transition_to(Idle));
        assert!(Running.can_transition_to(Running));
        assert!(Paused.can_transition_to(Paused));
        assert!(Stopped.can_transition_to(Stopped));
    }

    #[test]
    fn try_transition_to_mutates_state_on_success() {
        let mut state = RunControlState::Idle;

        // Idle → Running
        assert!(state.try_transition_to(RunControlState::Running).is_ok());
        assert_eq!(state, RunControlState::Running);

        // Running → Paused
        assert!(state.try_transition_to(RunControlState::Paused).is_ok());
        assert_eq!(state, RunControlState::Paused);

        // Paused → Running
        assert!(state.try_transition_to(RunControlState::Running).is_ok());
        assert_eq!(state, RunControlState::Running);
    }

    #[test]
    fn try_transition_to_returns_error_on_invalid() {
        let mut state = RunControlState::Idle;

        let err = state
            .try_transition_to(RunControlState::Paused)
            .unwrap_err();
        assert!(matches!(
            err,
            RunControlStateTransitionError::InvalidTransition { .. }
        ));
        // 状态应保持不变
        assert_eq!(state, RunControlState::Idle);
    }

    #[test]
    fn can_pause_only_when_running() {
        assert!(!RunControlState::Idle.can_pause());
        assert!(RunControlState::Running.can_pause());
        assert!(!RunControlState::Paused.can_pause());
        assert!(!RunControlState::Stopped.can_pause());
    }

    #[test]
    fn can_resume_only_when_paused() {
        assert!(!RunControlState::Idle.can_resume());
        assert!(!RunControlState::Running.can_resume());
        assert!(RunControlState::Paused.can_resume());
        assert!(!RunControlState::Stopped.can_resume());
    }

    #[test]
    fn is_active_for_running_and_paused() {
        assert!(!RunControlState::Idle.is_active());
        assert!(RunControlState::Running.is_active());
        assert!(RunControlState::Paused.is_active());
        assert!(!RunControlState::Stopped.is_active());
    }

    #[test]
    fn serializes_to_snake_case() {
        let json = serde_json::to_string(&RunControlState::Running).unwrap();
        assert_eq!(json, "\"running\"");

        let json = serde_json::to_string(&RunControlState::Paused).unwrap();
        assert_eq!(json, "\"paused\"");
    }

    #[test]
    fn deserializes_from_snake_case() {
        let state: RunControlState = serde_json::from_str("\"running\"").unwrap();
        assert_eq!(state, RunControlState::Running);

        let state: RunControlState = serde_json::from_str("\"paused\"").unwrap();
        assert_eq!(state, RunControlState::Paused);
    }
}
