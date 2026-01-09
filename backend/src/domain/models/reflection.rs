//! Layer 4（Reflection / Iteration）相关领域模型
//!
//! 依据 `docs/analysis/research/technical-algorithm-specification-research-2025-12-14.md`
//! 的 4.2.2/4.2.3/4.2.4 增量补丁落地。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// 候选来源
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum CandidateSource {
    /// 首次从规律生成
    InitialGeneration,
    /// 规律体系更新后重新生成
    RuleSystemUpdate,
    /// 仅表达层优化
    ExpressionRefinement,
    /// 多样性注入
    DiversityInjection,
    /// 用户手动编辑
    ManualEdit,
}

/// 终止原因
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum TerminationReason {
    /// 全部测试通过
    AllTestsPassed,
    /// 达到通过率阈值
    PassThresholdReached { threshold: f64, actual: f64 },
    /// 达到最大迭代轮数
    MaxIterationsReached { max: u32 },
    /// 检测到震荡
    OscillationDetected,
    /// 用户手动终止
    UserStopped,
    /// 需要人工介入
    HumanInterventionRequired { reason: String },
}

/// 失败类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum FailureType {
    /// 规律不完备（缺少某种模式的规律）
    RuleIncomplete,
    /// 规律错误（现有规律有问题）
    RuleIncorrect,
    /// 表达问题（规律正确但 Prompt 表达不当）
    ExpressionIssue,
    /// 边界情况（测试用例是特殊边界）
    EdgeCase,
    /// 无法判断（需要人工介入）
    Undetermined,
}

/// 建议类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum SuggestionType {
    /// 新增规律
    AddRule,
    /// 修改规律
    ModifyRule,
    /// 删除规律
    RemoveRule,
    /// 修改 Prompt 格式
    ChangeFormat,
    /// 修改 Prompt 措辞
    Rephrase,
    /// 增加示例
    AddExample,
    /// 增加约束说明
    AddConstraint,
}

/// 改进建议
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct Suggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 建议内容
    pub content: String,
    /// 置信度 0.0-1.0
    pub confidence: f64,
    /// 预期影响范围（受影响的测试用例数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_impact: Option<u32>,
}

/// 反思结果结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct ReflectionResult {
    /// 失败类型
    pub failure_type: FailureType,

    /// 详细分析
    pub analysis: String,
    /// 根因判断
    pub root_cause: String,

    /// 建议列表
    pub suggestions: Vec<Suggestion>,

    /// 关联的失败测试用例 ID
    pub failed_test_case_ids: Vec<String>,
    /// 关联的规律 ID（如果是规律问题）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related_rule_ids: Vec<String>,
    /// 关联的 EvaluationResult（用于追溯）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluation_ref: Option<String>,

    /// 预留扩展
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum ConflictType {
    /// 直接矛盾（A 说加，B 说删）
    DirectContradiction,
    /// 资源竞争（都要修改同一规律）
    ResourceCompetition,
    /// 优先级冲突（都重要但只能选一个）
    PriorityConflict,
}

/// 建议冲突
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct SuggestionConflict {
    /// 冲突的建议 A
    pub suggestion_a: Suggestion,
    /// 冲突的建议 B
    pub suggestion_b: Suggestion,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 冲突描述
    pub description: String,
}

/// 仲裁方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum ArbitrationMethod {
    /// 投票（多数决）
    Voting,
    /// LLM 仲裁
    LLMArbitration,
    /// 人工仲裁
    HumanArbitration,
    /// 全部保留（Pareto）
    KeepAll,
}

/// 统一建议（聚合后）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct UnifiedSuggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,
    /// 聚合后的建议内容
    pub content: String,
    /// 聚合置信度
    pub confidence: f64,
    /// 支持此建议的原始 ReflectionResult 数量
    pub support_count: u32,
    /// 优先级（1 最高）
    pub priority: u32,
}

/// 仲裁结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct ArbitrationResult {
    /// 选择的建议
    pub chosen_suggestions: Vec<UnifiedSuggestion>,
    /// 仲裁推理
    pub reasoning: String,
    /// 仲裁方式
    pub method: ArbitrationMethod,
}

/// 推荐行动
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub enum RecommendedAction {
    /// 更新规律体系后重新生成 Prompt
    UpdateRulesAndRegenerate,
    /// 仅优化 Prompt 表达
    RefineExpression,
    /// 需要人工介入
    RequestHumanIntervention { reason: String },
    /// 注入多样性
    InjectDiversity,
    /// 终止迭代
    Terminate { reason: TerminationReason },
}

/// 统一反思结构（聚合后）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct UnifiedReflection {
    /// 主要失败类型（投票或权重决定）
    pub primary_failure_type: FailureType,
    /// 聚合后的改进建议（已去重、合并、排序）
    pub unified_suggestions: Vec<UnifiedSuggestion>,

    /// 是否存在建议冲突
    pub has_conflicts: bool,
    /// 冲突详情（如有）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub conflicts: Vec<SuggestionConflict>,
    /// 仲裁结果（如有冲突）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arbitration_result: Option<ArbitrationResult>,

    /// 聚合的原始 ReflectionResult 数量
    pub source_count: u32,
    /// 失败类型分布
    pub failure_type_distribution: HashMap<String, u32>,

    /// 推荐的下一步行动
    pub recommended_action: RecommendedAction,

    /// 预留扩展
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Prompt 候选
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct PromptCandidate {
    /// 候选 ID
    pub id: String,
    /// Prompt 内容
    pub content: String,
    /// 综合评分 0.0-1.0
    pub score: f64,
    /// 来源（首次生成 / 规律更新 / 表达优化）
    pub source: CandidateSource,
    /// 失败指纹（用于去重）
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub failure_fingerprints: Vec<String>,
}

/// 优化结果结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "models/")]
pub struct OptimizationResult {
    /// 主候选 Prompt（MVP 只用这个）
    pub primary: PromptCandidate,

    /// 备选候选列表
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub alternatives: Vec<PromptCandidate>,

    /// 是否建议终止迭代
    pub should_terminate: bool,
    /// 终止原因（should_terminate=true 时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_reason: Option<TerminationReason>,

    /// 当前迭代轮次
    pub iteration: u32,
    /// 本轮改进摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub improvement_summary: Option<String>,

    /// 预留扩展
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
