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

/// 防止 config_json 膨胀（未来可根据产品需要调整）
pub const OPTIMIZATION_TASK_CONFIG_MAX_JSON_BYTES: usize = 32 * 1024; // 32KB

/// 避免把大段 Prompt 原文写入 DB / 错误信息
pub const OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES: usize = 20_000;

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
    pub data_split: DataSplitPercentConfig,
}

impl Default for OptimizationTaskConfig {
    fn default() -> Self {
        Self {
            schema_version: OPTIMIZATION_TASK_CONFIG_SCHEMA_VERSION,
            initial_prompt: None,
            max_iterations: 10,
            pass_threshold_percent: 95,
            data_split: DataSplitPercentConfig::default(),
        }
    }
}

impl OptimizationTaskConfig {
    pub fn normalized_from_config_json(raw: Option<&str>) -> Self {
        OptimizationTaskConfigStorage::from_config_json(raw)
            .into_public()
            .normalized()
    }

    pub fn normalized(mut self) -> Self {
        self.initial_prompt = normalize_initial_prompt(self.initial_prompt);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
struct OptimizationTaskConfigStorage {
    pub schema_version: u32,
    pub initial_prompt: Option<String>,
    pub max_iterations: u32,
    pub pass_threshold_percent: u8,
    pub data_split: DataSplitPercentConfig,
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
            data_split: base.data_split,
            extra: BTreeMap::new(),
        }
    }
}

impl OptimizationTaskConfigStorage {
    fn from_config_json(raw: Option<&str>) -> Self {
        let Some(raw) = raw else {
            return Self::default();
        };
        let raw = raw.trim();
        if raw.is_empty() {
            return Self::default();
        }

        serde_json::from_str::<Self>(raw).unwrap_or_else(|_| Self::default())
    }

    fn into_public(self) -> OptimizationTaskConfig {
        OptimizationTaskConfig {
            schema_version: self.schema_version,
            initial_prompt: self.initial_prompt,
            max_iterations: self.max_iterations,
            pass_threshold_percent: self.pass_threshold_percent,
            data_split: self.data_split,
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
            data_split: config.data_split,
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
    let existing = OptimizationTaskConfigStorage::from_config_json(existing_raw);
    let storage = OptimizationTaskConfigStorage::from_public_and_existing_extra(config, existing);
    storage
        .to_config_json_bytes()
        .map_err(|_| "序列化任务配置失败".to_string())
}
