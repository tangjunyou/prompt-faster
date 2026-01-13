//! 优化任务配置（OptimizationTaskConfig）
//!
//! - 存储：optimization_tasks.config_json (TEXT)
//! - 对外：返回规范化后的强类型 config（永远非空）

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;
use utoipa::ToSchema;

pub const OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION: u32 = 1;

pub const OPTIMIZATION_TASK_CONFIG_MAX_ITERATIONS_MIN: u32 = 1;
pub const OPTIMIZATION_TASK_CONFIG_MAX_ITERATIONS_MAX: u32 = 100;

pub const OPTIMIZATION_TASK_CONFIG_PASS_THRESHOLD_MIN: u8 = 1;
pub const OPTIMIZATION_TASK_CONFIG_PASS_THRESHOLD_MAX: u8 = 100;

pub const OPTIMIZATION_TASK_CONFIG_CANDIDATE_PROMPT_COUNT_MIN: u32 = 1;
pub const OPTIMIZATION_TASK_CONFIG_CANDIDATE_PROMPT_COUNT_MAX: u32 = 10;

pub const OPTIMIZATION_TASK_CONFIG_DIVERSITY_INJECTION_THRESHOLD_MIN: u32 = 1;
pub const OPTIMIZATION_TASK_CONFIG_DIVERSITY_INJECTION_THRESHOLD_MAX: u32 = 10;

pub const OPTIMIZATION_TASK_CONFIG_CONFLICT_ALERT_THRESHOLD_MIN: u32 = 1;
pub const OPTIMIZATION_TASK_CONFIG_CONFLICT_ALERT_THRESHOLD_MAX: u32 = 10;

pub const OPTIMIZATION_TASK_CONFIG_K_FOLD_FOLDS_MIN: u8 = 2;
pub const OPTIMIZATION_TASK_CONFIG_K_FOLD_FOLDS_MAX: u8 = 10;

pub const OPTIMIZATION_TASK_CONFIG_SEMANTIC_SIMILARITY_THRESHOLD_MIN: u8 = 1;
pub const OPTIMIZATION_TASK_CONFIG_SEMANTIC_SIMILARITY_THRESHOLD_MAX: u8 = 100;

pub const OPTIMIZATION_TASK_CONFIG_LLM_JUDGE_SAMPLES_MIN: u32 = 1;
pub const OPTIMIZATION_TASK_CONFIG_LLM_JUDGE_SAMPLES_MAX: u32 = 5;

pub const OPTIMIZATION_TASK_CONFIG_TEACHER_LLM_MODEL_ID_MAX_LEN: usize = 128;

pub const OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_MIN: u32 = 1;
pub const OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_MAX: u32 = 64;
pub const OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_DEFAULT: u32 = 4;

/// 防止 config_json 膨胀（未来可根据产品需要调整）
pub const OPTIMIZATION_TASK_CONFIG_MAX_JSON_BYTES: usize = 32 * 1024; // 32KB

/// 避免把大段 Prompt 原文写入 DB / 错误信息
pub const OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES: usize = 20_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum OutputStrategy {
    #[default]
    Single,
    Adaptive,
    Multi,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ExecutionMode {
    #[default]
    Serial,
    Parallel,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct OutputConfig {
    pub strategy: OutputStrategy,
    pub conflict_alert_threshold: u32,
    pub auto_recommend: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            strategy: OutputStrategy::Single,
            conflict_alert_threshold: 3,
            auto_recommend: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum EvaluatorType {
    #[default]
    Auto,
    ExactMatch,
    SemanticSimilarity,
    ConstraintCheck,
    TeacherModel,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct ExactMatchEvaluatorConfig {
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct SemanticSimilarityEvaluatorConfig {
    pub threshold_percent: u8,
}

impl Default for SemanticSimilarityEvaluatorConfig {
    fn default() -> Self {
        Self {
            threshold_percent: 85,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct ConstraintCheckEvaluatorConfig {
    pub strict: bool,
}

impl Default for ConstraintCheckEvaluatorConfig {
    fn default() -> Self {
        Self { strict: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct TeacherModelEvaluatorConfig {
    pub llm_judge_samples: u32,
}

impl Default for TeacherModelEvaluatorConfig {
    fn default() -> Self {
        Self {
            llm_judge_samples: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct EvaluatorConfig {
    pub evaluator_type: EvaluatorType,
    pub exact_match: ExactMatchEvaluatorConfig,
    pub semantic_similarity: SemanticSimilarityEvaluatorConfig,
    pub constraint_check: ConstraintCheckEvaluatorConfig,
    pub teacher_model: TeacherModelEvaluatorConfig,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            evaluator_type: EvaluatorType::Auto,
            exact_match: ExactMatchEvaluatorConfig::default(),
            semantic_similarity: SemanticSimilarityEvaluatorConfig::default(),
            constraint_check: ConstraintCheckEvaluatorConfig::default(),
            teacher_model: TeacherModelEvaluatorConfig::default(),
        }
    }
}

/// 老师模型（通用大模型）配置
///
/// 注意：与 EvaluatorConfig.teacher_model 不同，本配置用于“为任务覆盖老师模型 model_id”。
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct TeacherLlmConfig {
    /// 覆盖老师模型的 model_id；None 表示系统默认（不覆盖）
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum AdvancedDataSplitStrategy {
    #[default]
    Percent,
    KFold,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum SamplingStrategy {
    #[default]
    Random,
    Stratified,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(default, rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct AdvancedDataSplitConfig {
    pub strategy: AdvancedDataSplitStrategy,
    pub k_fold_folds: u8,
    pub sampling_strategy: SamplingStrategy,
}

impl Default for AdvancedDataSplitConfig {
    fn default() -> Self {
        Self {
            strategy: AdvancedDataSplitStrategy::Percent,
            k_fold_folds: 5,
            sampling_strategy: SamplingStrategy::Random,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct DataSplitPercentConfig {
    pub train_percent: u8,
    pub validation_percent: u8,
    /// 本 Story 固定为 0（仅预留字段，后续 Story 再开放）
    pub holdout_percent: u8,
}

impl Default for DataSplitPercentConfig {
    fn default() -> Self {
        Self {
            train_percent: 80,
            validation_percent: 20,
            holdout_percent: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub struct OptimizationTaskConfig {
    pub schema_version: u32,
    pub initial_prompt: Option<String>,
    pub max_iterations: u32,
    pub pass_threshold_percent: u8,
    pub candidate_prompt_count: u32,
    pub diversity_injection_threshold: u32,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    pub max_concurrency: u32,
    pub data_split: DataSplitPercentConfig,
    pub output_config: OutputConfig,
    pub evaluator_config: EvaluatorConfig,
    #[serde(default)]
    pub teacher_llm: TeacherLlmConfig,
    pub advanced_data_split: AdvancedDataSplitConfig,
}

impl Default for OptimizationTaskConfig {
    fn default() -> Self {
        Self {
            schema_version: OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION,
            initial_prompt: None,
            max_iterations: 10,
            pass_threshold_percent: 95,
            candidate_prompt_count: 5,
            diversity_injection_threshold: 3,
            execution_mode: ExecutionMode::Serial,
            max_concurrency: OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_DEFAULT,
            data_split: DataSplitPercentConfig::default(),
            output_config: OutputConfig::default(),
            evaluator_config: EvaluatorConfig::default(),
            teacher_llm: TeacherLlmConfig::default(),
            advanced_data_split: AdvancedDataSplitConfig::default(),
        }
    }
}

impl OptimizationTaskConfig {
    pub fn normalized_from_config_json(raw: Option<&str>) -> Self {
        OptimizationTaskConfigStorage::from_config_json_lossy(raw)
            .into_public()
            .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.initial_prompt = normalize_initial_prompt(self.initial_prompt);
        self.teacher_llm.model_id = normalize_teacher_llm_model_id(self.teacher_llm.model_id);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.max_iterations < OPTIMIZATION_TASK_CONFIG_MAX_ITERATIONS_MIN
            || self.max_iterations > OPTIMIZATION_TASK_CONFIG_MAX_ITERATIONS_MAX
        {
            return Err(format!(
                "最大迭代轮数仅允许 {}-{}",
                OPTIMIZATION_TASK_CONFIG_MAX_ITERATIONS_MIN,
                OPTIMIZATION_TASK_CONFIG_MAX_ITERATIONS_MAX
            ));
        }

        if self.pass_threshold_percent < OPTIMIZATION_TASK_CONFIG_PASS_THRESHOLD_MIN
            || self.pass_threshold_percent > OPTIMIZATION_TASK_CONFIG_PASS_THRESHOLD_MAX
        {
            return Err("通过率阈值仅允许 1-100%".to_string());
        }

        if self.candidate_prompt_count < OPTIMIZATION_TASK_CONFIG_CANDIDATE_PROMPT_COUNT_MIN
            || self.candidate_prompt_count > OPTIMIZATION_TASK_CONFIG_CANDIDATE_PROMPT_COUNT_MAX
        {
            return Err(format!(
                "候选 Prompt 生成数量仅允许 {}-{}",
                OPTIMIZATION_TASK_CONFIG_CANDIDATE_PROMPT_COUNT_MIN,
                OPTIMIZATION_TASK_CONFIG_CANDIDATE_PROMPT_COUNT_MAX
            ));
        }

        if self.diversity_injection_threshold
            < OPTIMIZATION_TASK_CONFIG_DIVERSITY_INJECTION_THRESHOLD_MIN
            || self.diversity_injection_threshold
                > OPTIMIZATION_TASK_CONFIG_DIVERSITY_INJECTION_THRESHOLD_MAX
        {
            return Err(format!(
                "多样性注入阈值仅允许 {}-{}",
                OPTIMIZATION_TASK_CONFIG_DIVERSITY_INJECTION_THRESHOLD_MIN,
                OPTIMIZATION_TASK_CONFIG_DIVERSITY_INJECTION_THRESHOLD_MAX
            ));
        }

        if !(OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_MIN
            ..=OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_MAX)
            .contains(&self.max_concurrency)
        {
            return Err(format!(
                "并发数仅允许 {}-{}",
                OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_MIN,
                OPTIMIZATION_TASK_CONFIG_MAX_CONCURRENCY_MAX
            ));
        }

        if let Some(p) = &self.initial_prompt {
            if p.len() > OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES {
                return Err(format!(
                    "初始 Prompt 过长（最多 {} bytes）",
                    OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES
                ));
            }
        }

        // 本 Story：只暴露 Train/Validation；Holdout 固定为 0%
        if self.data_split.holdout_percent != 0 {
            return Err("本版本暂不支持配置 holdout_percent（需为 0%）".to_string());
        }

        let sum = self.data_split.train_percent as u32 + self.data_split.validation_percent as u32;
        if sum != 100 {
            return Err("数据划分比例要求 Train% + Validation% = 100%".to_string());
        }

        if self.output_config.conflict_alert_threshold
            < OPTIMIZATION_TASK_CONFIG_CONFLICT_ALERT_THRESHOLD_MIN
            || self.output_config.conflict_alert_threshold
                > OPTIMIZATION_TASK_CONFIG_CONFLICT_ALERT_THRESHOLD_MAX
        {
            return Err(format!(
                "冲突告警阈值仅允许 {}-{}",
                OPTIMIZATION_TASK_CONFIG_CONFLICT_ALERT_THRESHOLD_MIN,
                OPTIMIZATION_TASK_CONFIG_CONFLICT_ALERT_THRESHOLD_MAX
            ));
        }

        match self.advanced_data_split.strategy {
            AdvancedDataSplitStrategy::Percent => {}
            AdvancedDataSplitStrategy::KFold => {
                if self.advanced_data_split.k_fold_folds < OPTIMIZATION_TASK_CONFIG_K_FOLD_FOLDS_MIN
                    || self.advanced_data_split.k_fold_folds
                        > OPTIMIZATION_TASK_CONFIG_K_FOLD_FOLDS_MAX
                {
                    return Err(format!(
                        "交叉验证折数仅允许 {}-{}",
                        OPTIMIZATION_TASK_CONFIG_K_FOLD_FOLDS_MIN,
                        OPTIMIZATION_TASK_CONFIG_K_FOLD_FOLDS_MAX
                    ));
                }
            }
        }

        match self.evaluator_config.evaluator_type {
            EvaluatorType::Auto | EvaluatorType::ExactMatch | EvaluatorType::ConstraintCheck => {}
            EvaluatorType::SemanticSimilarity => {
                let v = self.evaluator_config.semantic_similarity.threshold_percent;
                if !(OPTIMIZATION_TASK_CONFIG_SEMANTIC_SIMILARITY_THRESHOLD_MIN
                    ..=OPTIMIZATION_TASK_CONFIG_SEMANTIC_SIMILARITY_THRESHOLD_MAX)
                    .contains(&v)
                {
                    return Err(format!(
                        "语义相似度阈值仅允许 {}-{}",
                        OPTIMIZATION_TASK_CONFIG_SEMANTIC_SIMILARITY_THRESHOLD_MIN,
                        OPTIMIZATION_TASK_CONFIG_SEMANTIC_SIMILARITY_THRESHOLD_MAX
                    ));
                }
            }
            EvaluatorType::TeacherModel => {
                let v = self.evaluator_config.teacher_model.llm_judge_samples;
                if !(OPTIMIZATION_TASK_CONFIG_LLM_JUDGE_SAMPLES_MIN
                    ..=OPTIMIZATION_TASK_CONFIG_LLM_JUDGE_SAMPLES_MAX)
                    .contains(&v)
                {
                    return Err(format!(
                        "老师模型采样数仅允许 {}-{}",
                        OPTIMIZATION_TASK_CONFIG_LLM_JUDGE_SAMPLES_MIN,
                        OPTIMIZATION_TASK_CONFIG_LLM_JUDGE_SAMPLES_MAX
                    ));
                }
            }
        }

        if let Some(model_id) = &self.teacher_llm.model_id {
            if model_id.chars().any(|c| c.is_control()) {
                return Err("老师模型 model_id 不允许包含控制字符".to_string());
            }
            if model_id.chars().count() > OPTIMIZATION_TASK_CONFIG_TEACHER_LLM_MODEL_ID_MAX_LEN {
                return Err(format!(
                    "老师模型 model_id 过长（最多 {} 字符）",
                    OPTIMIZATION_TASK_CONFIG_TEACHER_LLM_MODEL_ID_MAX_LEN
                ));
            }
            if model_id.trim().is_empty() {
                return Err("老师模型 model_id 不能为空".to_string());
            }
        }

        Ok(())
    }
}

fn normalize_initial_prompt(raw: Option<String>) -> Option<String> {
    raw.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_teacher_llm_model_id(raw: Option<String>) -> Option<String> {
    raw.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn default_candidate_prompt_count() -> u32 {
    OptimizationTaskConfig::default().candidate_prompt_count
}

fn default_diversity_injection_threshold() -> u32 {
    OptimizationTaskConfig::default().diversity_injection_threshold
}

fn default_max_concurrency() -> u32 {
    OptimizationTaskConfig::default().max_concurrency
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
struct OptimizationTaskConfigStorage {
    pub schema_version: u32,
    pub initial_prompt: Option<String>,
    pub max_iterations: u32,
    pub pass_threshold_percent: u8,
    #[serde(default = "default_candidate_prompt_count")]
    pub candidate_prompt_count: u32,
    #[serde(default = "default_diversity_injection_threshold")]
    pub diversity_injection_threshold: u32,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,
    pub data_split: DataSplitPercentConfig,
    pub output_config: OutputConfig,
    pub evaluator_config: EvaluatorConfig,
    #[serde(default)]
    pub teacher_llm: TeacherLlmConfig,
    pub advanced_data_split: AdvancedDataSplitConfig,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Default for OptimizationTaskConfigStorage {
    fn default() -> Self {
        let base = OptimizationTaskConfig::default();
        Self {
            schema_version: base.schema_version,
            initial_prompt: base.initial_prompt,
            max_iterations: base.max_iterations,
            pass_threshold_percent: base.pass_threshold_percent,
            candidate_prompt_count: base.candidate_prompt_count,
            diversity_injection_threshold: base.diversity_injection_threshold,
            execution_mode: base.execution_mode,
            max_concurrency: base.max_concurrency,
            data_split: base.data_split,
            output_config: base.output_config,
            evaluator_config: base.evaluator_config,
            teacher_llm: base.teacher_llm,
            advanced_data_split: base.advanced_data_split,
            extra: BTreeMap::new(),
        }
    }
}

impl OptimizationTaskConfigStorage {
    fn from_config_json_lossy(raw: Option<&str>) -> Self {
        let Some(raw) = raw else {
            return Self::default();
        };
        let raw = raw.trim();
        if raw.is_empty() {
            return Self::default();
        }

        serde_json::from_str::<Self>(raw).unwrap_or_else(|_| Self::default())
    }

    fn try_from_config_json(raw: Option<&str>) -> Result<Self, String> {
        let Some(raw) = raw else {
            return Ok(Self::default());
        };
        let raw = raw.trim();
        if raw.is_empty() {
            return Ok(Self::default());
        }

        serde_json::from_str::<Self>(raw)
            .map_err(|_| "任务配置解析失败（请尝试重置为默认配置后再更新）".to_string())
    }

    fn into_public(self) -> OptimizationTaskConfig {
        OptimizationTaskConfig {
            schema_version: self.schema_version,
            initial_prompt: self.initial_prompt,
            max_iterations: self.max_iterations,
            pass_threshold_percent: self.pass_threshold_percent,
            candidate_prompt_count: self.candidate_prompt_count,
            diversity_injection_threshold: self.diversity_injection_threshold,
            execution_mode: self.execution_mode,
            max_concurrency: self.max_concurrency,
            data_split: self.data_split,
            output_config: self.output_config,
            evaluator_config: self.evaluator_config,
            teacher_llm: self.teacher_llm,
            advanced_data_split: self.advanced_data_split,
        }
    }

    fn from_public_and_existing_extra(
        config: OptimizationTaskConfig,
        existing: OptimizationTaskConfigStorage,
    ) -> Self {
        Self {
            schema_version: config.schema_version,
            initial_prompt: config.initial_prompt,
            max_iterations: config.max_iterations,
            pass_threshold_percent: config.pass_threshold_percent,
            candidate_prompt_count: config.candidate_prompt_count,
            diversity_injection_threshold: config.diversity_injection_threshold,
            execution_mode: config.execution_mode,
            max_concurrency: config.max_concurrency,
            data_split: config.data_split,
            output_config: config.output_config,
            evaluator_config: config.evaluator_config,
            teacher_llm: config.teacher_llm,
            advanced_data_split: config.advanced_data_split,
            extra: existing.extra,
        }
    }

    fn to_config_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

pub fn serialize_config_with_existing_extra(
    config: OptimizationTaskConfig,
    existing_raw: Option<&str>,
) -> Result<Vec<u8>, String> {
    let existing = OptimizationTaskConfigStorage::try_from_config_json(existing_raw)?;
    let storage = OptimizationTaskConfigStorage::from_public_and_existing_extra(config, existing);
    storage
        .to_config_json_bytes()
        .map_err(|_| "序列化任务配置失败".to_string())
}
