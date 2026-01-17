//! 历史迭代类型定义
//!
//! 用于历史迭代查看功能的数据传输对象（DTO）。
//! 这些类型是面向 API 响应的结构，与数据库表结构有映射关系。

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use ts_rs::TS;
use utoipa::ToSchema;

use super::IterationArtifacts;

/// 迭代状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum IterationStatus {
    /// 进行中
    #[default]
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 被用户终止
    Terminated,
}

impl IterationStatus {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Terminated => "terminated",
        }
    }
}

impl FromStr for IterationStatus {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "terminated" => Self::Terminated,
            _ => Self::Running,
        })
    }
}

/// 评估结果摘要（历史查看专用）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct EvaluationResultSummary {
    /// 测试用例 ID
    pub test_case_id: String,
    /// 是否通过
    pub passed: bool,
    /// 分数（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// 失败原因（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

/// 历史迭代列表项
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationHistorySummary {
    /// 迭代 ID
    pub id: String,
    /// 轮次编号（从 1 开始）
    pub round: i32,
    /// 迭代开始时间（ISO 8601）
    pub started_at: String,
    /// 迭代结束时间（ISO 8601，进行中为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// 通过率（0.0 - 1.0）
    pub pass_rate: f64,
    /// 测试用例总数
    pub total_cases: i32,
    /// 通过的测试用例数
    pub passed_cases: i32,
    /// 迭代状态
    pub status: IterationStatus,
}

/// 历史迭代详情响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationHistoryDetail {
    /// 迭代 ID
    pub id: String,
    /// 轮次编号（从 1 开始）
    pub round: i32,
    /// 迭代开始时间（ISO 8601）
    pub started_at: String,
    /// 迭代结束时间（ISO 8601，进行中为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// 通过率（0.0 - 1.0）
    pub pass_rate: f64,
    /// 测试用例总数
    pub total_cases: i32,
    /// 通过的测试用例数
    pub passed_cases: i32,
    /// 迭代状态
    pub status: IterationStatus,
    /// 完整产物
    pub artifacts: IterationArtifacts,
    /// 评估结果
    pub evaluation_results: Vec<EvaluationResultSummary>,
    /// 反思总结（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflection_summary: Option<String>,
}

/// 将 Unix 毫秒时间戳转换为 ISO 8601 格式字符串
pub fn unix_ms_to_iso8601(unix_ms: i64) -> String {
    use time::OffsetDateTime;
    let secs = unix_ms / 1000;
    let nanos = (unix_ms % 1000) * 1_000_000;
    OffsetDateTime::from_unix_timestamp(secs)
        .ok()
        .and_then(|dt| dt.checked_add(time::Duration::nanoseconds(nanos)))
        .and_then(|dt| {
            dt.format(&time::format_description::well_known::Rfc3339)
                .ok()
        })
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_status_from_str() {
        assert_eq!(
            "running".parse::<IterationStatus>().unwrap(),
            IterationStatus::Running
        );
        assert_eq!(
            "completed".parse::<IterationStatus>().unwrap(),
            IterationStatus::Completed
        );
        assert_eq!(
            "failed".parse::<IterationStatus>().unwrap(),
            IterationStatus::Failed
        );
        assert_eq!(
            "terminated".parse::<IterationStatus>().unwrap(),
            IterationStatus::Terminated
        );
        assert_eq!(
            "unknown".parse::<IterationStatus>().unwrap(),
            IterationStatus::Running
        );
    }

    #[test]
    fn test_iteration_status_as_str() {
        assert_eq!(IterationStatus::Running.as_str(), "running");
        assert_eq!(IterationStatus::Completed.as_str(), "completed");
        assert_eq!(IterationStatus::Failed.as_str(), "failed");
        assert_eq!(IterationStatus::Terminated.as_str(), "terminated");
    }

    #[test]
    fn test_unix_ms_to_iso8601() {
        let result = unix_ms_to_iso8601(1705507200000);
        assert!(result.contains("2024-01-17"));
    }

    #[test]
    fn test_iteration_history_summary_serialization() {
        let summary = IterationHistorySummary {
            id: "iter-1".to_string(),
            round: 1,
            started_at: "2024-01-17T12:00:00Z".to_string(),
            completed_at: Some("2024-01-17T12:05:00Z".to_string()),
            pass_rate: 0.85,
            total_cases: 10,
            passed_cases: 8,
            status: IterationStatus::Completed,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"id\":\"iter-1\""));
        assert!(json.contains("\"round\":1"));
        assert!(json.contains("\"passRate\":0.85"));
    }

    #[test]
    fn test_evaluation_result_summary_serialization() {
        let result = EvaluationResultSummary {
            test_case_id: "tc-1".to_string(),
            passed: true,
            score: Some(0.95),
            failure_reason: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"testCaseId\":\"tc-1\""));
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"score\":0.95"));
        assert!(!json.contains("failureReason"));
    }
}
