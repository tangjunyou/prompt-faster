use crate::domain::models::{Checkpoint, IterationState, RuleSystem, TestCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
