//! 核心算法相关领域模型
//! 依据技术算法规格文档（2025-12-14）落地的 DTO 定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use utoipa::ToSchema;

/// 测试用例结构
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum DataSplit {
    Unassigned,
    Train,
    Validation,
    Holdout,
}

/// 任务参考类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export_to = "models/")]
pub enum TaskReference {
    Exact {
        expected: String,
    },
    Constrained {
        #[serde(skip_serializing_if = "Option::is_none")]
        core_request: Option<String>,
        constraints: Vec<Constraint>,
        quality_dimensions: Vec<QualityDimension>,
    },
    Hybrid {
        exact_parts: HashMap<String, String>,
        constraints: Vec<Constraint>,
    },
}

/// 约束条件
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export_to = "models/")]
pub struct Constraint {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// 质量维度
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
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

/// 失败档案条目（用于去重/避坑，不是全量日志）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[ts(export_to = "models/")]
pub struct FailureArchiveEntry {
    /// prompt 的可诊断截断片段（≤200 chars；不得存全文）
    pub prompt_excerpt: String,
    /// prompt 原始长度（字符数）
    pub prompt_len: u32,
    /// 失败用例 ID
    pub test_case_id: String,
    /// 失败原因摘要（不得包含 prompt/testcase input 全文）
    pub failure_reason: String,
    /// 确定性指纹（用于去重/避坑；不包含可读原文片段）
    pub failure_fingerprint: String,
}

impl FailureArchiveEntry {
    pub fn new(
        prompt: &str,
        test_case_id: impl Into<String>,
        failure_reason: impl Into<String>,
    ) -> Self {
        let prompt_excerpt = prompt_excerpt_200(prompt);
        let prompt_len = u32::try_from(prompt.chars().count()).unwrap_or(u32::MAX);
        let failure_fingerprint = failure_fingerprint_v1(prompt);
        Self {
            prompt_excerpt,
            prompt_len,
            test_case_id: test_case_id.into(),
            failure_reason: failure_reason.into(),
            failure_fingerprint,
        }
    }
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

/// 候选去重/避坑指纹（v1）
///
/// - 输入：规范化后的 prompt 全文（仅用于 hash，不输出任何片段）
/// - 输出：`v1:fnv1a64:{hex}`
/// - 目的：去重/避坑（非安全散列，允许极低概率碰撞）
pub fn failure_fingerprint_v1(prompt: &str) -> String {
    let normalized = normalize_prompt_for_fingerprint(prompt);
    let hash = fnv1a64(normalized.as_bytes());
    format!("v1:fnv1a64:{hash:016x}")
}

fn normalize_prompt_for_fingerprint(prompt: &str) -> String {
    let mut out = String::new();
    let mut last_was_ws = false;
    for ch in prompt.trim().chars() {
        if ch.is_whitespace() {
            if !last_was_ws {
                out.push(' ');
                last_was_ws = true;
            }
        } else {
            out.push(ch);
            last_was_ws = false;
        }
    }
    out
}

fn prompt_excerpt_200(prompt: &str) -> String {
    let normalized = normalize_prompt_for_fingerprint(prompt);
    let redacted = redact_likely_secrets(&normalized);
    redacted.chars().take(200).collect()
}

fn redact_likely_secrets(s: &str) -> String {
    // Conservative redaction: avoid leaking possible credentials/keys in archives/logs.
    // This is NOT a perfect secret detector; it intentionally trades recall for safety.
    let mut out = String::new();
    let mut token = String::new();
    let mut prev_was_bearer = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !token.is_empty() {
                if prev_was_bearer {
                    out.push_str("***");
                    prev_was_bearer = false;
                } else {
                    let redacted = redact_token(&token);
                    prev_was_bearer = token.eq_ignore_ascii_case("bearer");
                    out.push_str(&redacted);
                }
                token.clear();
            }
            out.push(ch);
        } else {
            token.push(ch);
        }
    }

    if !token.is_empty() {
        if prev_was_bearer {
            out.push_str("***");
        } else {
            out.push_str(&redact_token(&token));
        }
    }

    out
}

fn redact_token(token: &str) -> String {
    // 1) Common "key=value" style secrets inside a single token
    if let Some(redacted) = redact_kv_like(token) {
        return redacted;
    }

    // 2) OpenAI-style keys (sk-...) or similar
    if let Some(redacted) = redact_sk_like(token) {
        return redacted;
    }

    // 3) Generic long high-entropy tokens
    if looks_like_secret_token(token) {
        return redact_keep_ends(token, 3, 3);
    }

    token.to_string()
}

fn redact_kv_like(token: &str) -> Option<String> {
    // Redact patterns like "api_key=XXX", "token:YYY", "\"password\":\"ZZZ\"".
    // Keep the key part for diagnostics.
    const KEYS: [&str; 6] = [
        "api_key",
        "apikey",
        "token",
        "secret",
        "password",
        "access_key",
    ];
    let lower = token.to_ascii_lowercase();
    for key in KEYS {
        if let Some(pos) = lower.find(key) {
            // Look for a delimiter after the key
            let after_key = pos + key.len();
            let bytes = lower.as_bytes();
            let mut i = after_key;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'"' || bytes[i] == b'\'') {
                i += 1;
            }
            if i >= bytes.len() {
                continue;
            }
            if bytes[i] == b'=' || bytes[i] == b':' {
                // Keep everything through the delimiter, then redact the rest.
                let keep = &token[..i + 1];
                return Some(format!("{keep}***"));
            }
        }
    }
    None
}

fn redact_sk_like(token: &str) -> Option<String> {
    // If token contains "sk-" followed by a long ascii run, redact it.
    // We don't attempt to parse URLs/punctuation; this is token-scoped.
    let bytes = token.as_bytes();
    let mut i = 0usize;
    while i + 3 <= bytes.len() {
        if bytes[i] == b's' && bytes[i + 1] == b'k' && bytes[i + 2] == b'-' {
            let start = i;
            let mut end = i + 3;
            while end < bytes.len() {
                let b = bytes[end];
                let ok = b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'=');
                if !ok {
                    break;
                }
                end += 1;
            }
            if end - start >= 12 {
                let prefix = &token[..start];
                let suffix = &token[end..];
                return Some(format!("{prefix}sk-***{suffix}"));
            }
        }
        i += 1;
    }
    None
}

fn looks_like_secret_token(token: &str) -> bool {
    let bytes = token.as_bytes();
    if bytes.len() < 24 {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'=' | b'/' | b'+'))
}

fn redact_keep_ends(token: &str, keep_start: usize, keep_end: usize) -> String {
    let bytes = token.as_bytes();
    if bytes.len() <= keep_start + keep_end + 3 {
        return "***".to_string();
    }
    let start = &token[..keep_start];
    let end = &token[bytes.len() - keep_end..];
    format!("{start}***{end}")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET_BASIS;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod failure_archive_tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic_and_normalizes_whitespace() {
        let a = "hello   world\n\nfoo";
        let b = "  hello world foo  ";
        assert_eq!(failure_fingerprint_v1(a), failure_fingerprint_v1(b));
        assert!(failure_fingerprint_v1(a).starts_with("v1:fnv1a64:"));
        assert!(!failure_fingerprint_v1(a).contains("hello"));
    }

    #[test]
    fn fingerprint_distinguishes_different_prompts() {
        let a = "hello world";
        let b = "hello worlds";
        assert_ne!(failure_fingerprint_v1(a), failure_fingerprint_v1(b));
    }

    #[test]
    fn failure_archive_entry_truncates_excerpt_and_sets_len() {
        let prompt = "a".repeat(500);
        let e = FailureArchiveEntry::new(&prompt, "tc1", "reason");
        assert_eq!(e.prompt_len, 500);
        assert!(e.prompt_excerpt.chars().count() <= 200);
        assert_eq!(e.test_case_id, "tc1");
        assert_eq!(e.failure_reason, "reason");
        assert_eq!(e.failure_fingerprint, failure_fingerprint_v1(&prompt));
    }

    #[test]
    fn prompt_excerpt_redacts_bearer_and_sk_like_tokens() {
        let prompt = "Bearer sk-abcdefghijklmnopqrstuvwxyz0123456789 SOME_TEXT";
        let e = FailureArchiveEntry::new(prompt, "tc1", "r");
        assert!(e.prompt_excerpt.contains("Bearer ***"));
        assert!(
            !e.prompt_excerpt
                .contains("abcdefghijklmnopqrstuvwxyz0123456789")
        );
    }

    #[test]
    fn prompt_excerpt_redacts_kv_like_tokens() {
        let prompt = r#"api_key=ABCDEF0123456789ABCDEF0123456789 password:"hunter2""#;
        let e = FailureArchiveEntry::new(prompt, "tc1", "r");
        assert!(e.prompt_excerpt.contains("api_key="));
        assert!(e.prompt_excerpt.contains("password:"));
        assert!(
            !e.prompt_excerpt
                .contains("ABCDEF0123456789ABCDEF0123456789")
        );
        assert!(!e.prompt_excerpt.contains("hunter2"));
    }

    #[test]
    fn prompt_excerpt_redacts_sk_like_tokens_when_not_bearer() {
        let prompt = "prefix sk-abcdefghijklmnopqrstuvwxyz0123456789 suffix";
        let e = FailureArchiveEntry::new(prompt, "tc1", "r");
        assert!(e.prompt_excerpt.contains("sk-***"));
        assert!(
            !e.prompt_excerpt
                .contains("abcdefghijklmnopqrstuvwxyz0123456789")
        );
    }
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
