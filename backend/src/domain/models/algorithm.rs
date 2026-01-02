//! 核心算法相关领域模型
//! 依据技术算法规格文档（2025-12-14）落地的 DTO 定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// 测试用例结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct TestCase {
    /// 唯一标识
    pub id: String,
    /// 输入变量
    pub input: HashMap<String, serde_json::Value>,
    /// 期望输出/约束
    pub reference: TaskReference,
    /// 数据划分归属
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<DataSplit>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 数据划分类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum DataSplit {
    Unassigned,
    Train,
    Validation,
    Holdout,
}

/// 任务参考类型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum TaskReference {
    Exact { expected: String },
    Constrained {
        constraints: Vec<Constraint>,
        quality_dimensions: Vec<QualityDimension>,
    },
    Hybrid {
        exact_parts: HashMap<String, String>,
        constraints: Vec<Constraint>,
    },
}

/// 约束条件
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct Constraint {
    pub name: String,
    pub description: String,
    pub weight: Option<f64>,
}

/// 质量维度
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct QualityDimension {
    pub name: String,
    pub description: String,
    pub weight: f64,
}

/// 执行结果结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct ExecutionResult {
    pub test_case_id: String,
    pub output: String,
    #[ts(type = "number")]
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<serde_json::Value>,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 评估结果结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct EvaluationResult {
    pub passed: bool,
    pub score: f64,
    pub dimensions: HashMap<String, DimensionScore>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failure_points: Vec<FailurePoint>,
    pub evaluator_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 单维度评分
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct DimensionScore {
    pub score: f64,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// 失败点（供 Reflection Agent 分析）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct FailurePoint {
    pub dimension: String,
    pub description: String,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

/// 失败严重程度
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum Severity {
    Critical,
    Major,
    Minor,
}

/// 规律结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct Rule {
    pub id: String,
    pub description: String,
    pub tags: RuleTags,
    pub source_test_cases: Vec<String>,
    pub abstraction_level: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_rules: Vec<String>,
    pub verified: bool,
    pub verification_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir: Option<RuleIR>,
}

/// 规律标签
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct RuleTags {
    #[serde(default)]
    pub output_format: Vec<String>,
    #[serde(default)]
    pub output_structure: Vec<String>,
    pub output_length: OutputLength,
    #[serde(default)]
    pub semantic_focus: Vec<String>,
    #[serde(default)]
    pub key_concepts: Vec<String>,
    #[serde(default)]
    pub must_include: Vec<String>,
    #[serde(default)]
    pub must_exclude: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 输出长度枚举
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "models/")]
pub enum OutputLength {
    Short,
    Medium,
    Long,
    Flexible,
}

/// 规律中间表示
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct RuleIR {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    pub exceptions: Vec<String>,
}

/// 规律体系
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct RuleSystem {
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub conflict_resolution_log: Vec<ConflictResolutionRecord>,
    #[serde(default)]
    pub merge_log: Vec<RuleMergeRecord>,
    #[serde(default)]
    pub coverage_map: HashMap<String, Vec<String>>,
    pub version: u32,
}

/// 冲突解决记录
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct ConflictResolutionRecord {
    pub id: String,
    pub conflicting_rule_ids: Vec<String>,
    pub resolved_rule_id: String,
    pub resolution: String,
    #[ts(type = "number")]
    pub timestamp_ms: i64,
}

/// 规律合并记录
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct RuleMergeRecord {
    pub id: String,
    pub source_rule_ids: Vec<String>,
    pub merged_rule_id: String,
    pub reason: String,
    #[ts(type = "number")]
    pub timestamp_ms: i64,
}

/// 规律冲突
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct RuleConflict {
    pub rule1: Rule,
    pub rule2: Rule,
    pub conflict_type: RuleConflictType,
    pub description: String,
    #[serde(default)]
    pub related_test_cases: Vec<TestCase>,
}

/// 规律冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum RuleConflictType {
    DirectContradiction,
    ScopeConflict,
    PriorityAmbiguity,
}

/// 迭代状态（细粒度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum IterationState {
    Idle,
    Initializing,
    ExtractingRules,
    DetectingConflicts,
    ResolvingConflicts,
    MergingSimilarRules,
    ValidatingRules,
    GeneratingPrompt,
    RunningTests,
    Evaluating,
    ClusteringFailures,
    Reflecting,
    UpdatingRules,
    Optimizing,
    SmartRetesting,
    SafetyChecking,
    WaitingUser,
    HumanIntervention,
    Completed,
    MaxIterationsReached,
    UserStopped,
    Failed,
}

/// 分支类型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum LineageType {
    Automatic,
    ManualPromptEdit,
    ManualRuleEdit,
    DialogueGuided,
    Restored,
}

/// Checkpoint 结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct Checkpoint {
    pub id: String,
    pub task_id: String,
    pub iteration: u32,
    pub state: IterationState,
    pub prompt: String,
    pub rule_system: RuleSystem,
    #[ts(type = "number")]
    pub created_at: i64,
    pub branch_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub lineage_type: LineageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_description: Option<String>,
}

/// 优化任务（占位最小结构，后续 Story 将补齐字段）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct OptimizationTask {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub goal: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 状态占位：running/completed/failed 等
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub updated_at: i64,
}

/// 迭代记录（占位最小结构，后续 Story 将补齐字段）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct Iteration {
    pub id: String,
    pub task_id: String,
    pub index: u32,
    pub state: IterationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation: Option<EvaluationResult>,
    #[ts(type = "number")]
    pub started_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number")]
    pub finished_at: Option<i64>,
}
