use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::models::TaskReference;

/// 老师模型 Prompt（数据库完整记录）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPrompt {
    pub id: String,
    pub user_id: String,
    pub version: i32,
    pub content: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 老师模型 Prompt 版本摘要（列表展示用）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPromptVersion {
    pub id: String,
    pub version: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub success_rate: Option<f64>,
    pub task_count: i32,
    pub created_at: String,
}

/// 版本统计信息
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TeacherPromptStats {
    pub version_id: String,
    pub version: i32,
    pub total_tasks: i32,
    pub successful_tasks: i32,
    pub success_rate: Option<f64>,
    pub average_pass_rate: Option<f64>,
}

/// 创建新版本的输入
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CreateTeacherPromptInput {
    pub content: String,
    pub description: Option<String>,
    #[serde(default = "default_activate")]
    pub activate: bool,
}

fn default_activate() -> bool {
    true
}

/// 元优化概览
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct MetaOptimizationOverview {
    pub total_versions: i32,
    pub active_version: Option<TeacherPromptVersion>,
    pub best_version: Option<TeacherPromptVersion>,
    pub stats: Vec<TeacherPromptStats>,
}

/// 元优化历史任务摘要（用于选择入口展示）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct MetaOptimizationTaskSummary {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub status: String,
    pub pass_rate: Option<f64>,
    pub created_at: String,
}

/// Prompt 预览执行请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptPreviewRequest {
    /// 待预览的 Prompt 内容
    pub content: String,
    /// 必填：历史任务 ID（用于解析 test_set_ids）
    #[serde(default)]
    pub task_ids: Vec<String>,
    /// 可选：指定测试用例 ID，为空时自动选择最多 3 条
    #[serde(default)]
    pub test_case_ids: Vec<String>,
}

/// 单条测试用例的预览结果
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptPreviewResult {
    pub test_case_id: String,
    pub input: HashMap<String, serde_json::Value>,
    pub reference: TaskReference,
    pub actual_output: String,
    pub passed: bool,
    #[ts(type = "number")]
    pub execution_time_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// 预览执行响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptPreviewResponse {
    pub results: Vec<PromptPreviewResult>,
    pub total_passed: i32,
    pub total_failed: i32,
    #[ts(type = "number")]
    pub total_execution_time_ms: i64,
}

/// Prompt 对比执行请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptCompareRequest {
    /// 版本 A 的 ID
    pub version_id_a: String,
    /// 版本 B 的 ID
    pub version_id_b: String,
    /// 必填：历史任务 ID（用于解析 test_set_ids）
    #[serde(default)]
    pub task_ids: Vec<String>,
    /// 可选：指定测试用例 ID，为空时自动选择最多 10 条
    #[serde(default)]
    pub test_case_ids: Vec<String>,
}

/// 单个版本的对比结果
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct VersionCompareResult {
    pub version_id: String,
    pub version: i32,
    pub total_passed: i32,
    pub total_failed: i32,
    pub pass_rate: f64,
}

/// 单条测试用例的对比结果
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CaseComparisonResult {
    pub test_case_id: String,
    pub input: HashMap<String, serde_json::Value>,
    pub reference: TaskReference,
    /// 版本 A 的输出
    pub version_a_output: String,
    /// 版本 A 是否通过
    pub version_a_passed: bool,
    /// 版本 A 的错误信息
    pub version_a_error: Option<String>,
    /// 版本 B 的输出
    pub version_b_output: String,
    /// 版本 B 是否通过
    pub version_b_passed: bool,
    /// 版本 B 的错误信息
    pub version_b_error: Option<String>,
    /// A 与 B 结果是否不同
    pub is_different: bool,
    /// 差异说明（帮助用户理解为什么某版本更好/更差）
    pub difference_note: Option<String>,
}

/// 对比摘要统计
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CompareSummary {
    /// 通过率差异（B - A），正值表示 B 更好
    pub pass_rate_diff: f64,
    /// 改进的用例数（B 通过但 A 失败）
    pub improved_cases: i32,
    /// 退化的用例数（A 通过但 B 失败）
    pub regressed_cases: i32,
    /// 输出不同但同为通过的用例数
    pub output_diff_cases: i32,
    /// 无变化的用例数
    pub unchanged_cases: i32,
    /// 总执行时间（毫秒）
    #[ts(type = "number")]
    pub total_execution_time_ms: i64,
}

/// 对比执行响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptCompareResponse {
    /// 版本 A 的结果摘要
    pub version_a: VersionCompareResult,
    /// 版本 B 的结果摘要
    pub version_b: VersionCompareResult,
    /// 版本 A 的 Prompt 内容（用于 Diff 视图）
    pub version_a_content: String,
    /// 版本 B 的 Prompt 内容（用于 Diff 视图）
    pub version_b_content: String,
    /// 每条测试用例的对比结果
    pub case_comparisons: Vec<CaseComparisonResult>,
    /// 对比摘要统计
    pub summary: CompareSummary,
}

/// Prompt 验证请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptValidationRequest {
    pub content: String,
}

/// Prompt 验证结果
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
