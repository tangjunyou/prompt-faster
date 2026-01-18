//! 迭代控制类型定义
//!
//! 定义增加轮数和终止任务相关的请求/响应 DTO。

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// 增加轮数请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct AddRoundsRequest {
    /// 要增加的轮数（1-100）
    pub additional_rounds: i32,
}

impl AddRoundsRequest {
    /// 最小允许增加轮数
    pub const MIN_ADDITIONAL_ROUNDS: i32 = 1;
    /// 最大允许增加轮数
    pub const MAX_ADDITIONAL_ROUNDS: i32 = 100;

    /// 验证请求参数
    pub fn validate(&self) -> Result<(), String> {
        if self.additional_rounds < Self::MIN_ADDITIONAL_ROUNDS {
            return Err(format!("增加轮数不能小于 {}", Self::MIN_ADDITIONAL_ROUNDS));
        }
        if self.additional_rounds > Self::MAX_ADDITIONAL_ROUNDS {
            return Err(format!("增加轮数不能超过 {}", Self::MAX_ADDITIONAL_ROUNDS));
        }
        Ok(())
    }
}

/// 增加轮数响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct AddRoundsResponse {
    /// 更新前的最大轮数
    pub previous_max_iterations: i32,
    /// 更新后的最大轮数
    pub new_max_iterations: i32,
    /// 当前已执行轮数
    pub current_round: i32,
}

/// 候选 Prompt 摘要（用于终止时选择）
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CandidatePromptSummary {
    /// 迭代 ID
    pub iteration_id: String,
    /// 轮次编号
    pub round: i32,
    /// 通过率（0.0 - 1.0）
    pub pass_rate: f64,
    /// 通过的测试用例数
    pub passed_cases: i32,
    /// 测试用例总数
    pub total_cases: i32,
    /// 候选 Prompt 内容
    pub prompt: String,
    /// Prompt 预览（前 200 字符，后端截断生成）
    pub prompt_preview: String,
    /// 迭代完成时间（ISO 8601）
    pub completed_at: String,
}

impl CandidatePromptSummary {
    /// Prompt 预览最大字符数
    pub const PREVIEW_MAX_CHARS: usize = 200;

    /// 从完整 Prompt 生成预览
    pub fn generate_preview(prompt: &str) -> String {
        let chars: Vec<char> = prompt.chars().collect();
        if chars.len() <= Self::PREVIEW_MAX_CHARS {
            prompt.to_string()
        } else {
            format!(
                "{}...",
                chars[..Self::PREVIEW_MAX_CHARS].iter().collect::<String>()
            )
        }
    }
}

/// 候选 Prompt 列表响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CandidatePromptListResponse {
    /// 候选 Prompt 列表（按通过率降序排列）
    pub candidates: Vec<CandidatePromptSummary>,
    /// 候选总数
    pub total: i32,
}

/// 终止任务请求
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TerminateTaskRequest {
    /// 选定的迭代 ID（可选，无候选时可为空表示直接终止）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_iteration_id: Option<String>,
}

/// 终止任务响应
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TerminateTaskResponse {
    /// 任务 ID
    pub task_id: String,
    /// 终止时间（ISO 8601）
    pub terminated_at: String,
    /// 选定的最终 Prompt（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_prompt: Option<String>,
    /// 选定的迭代轮次（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_round: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rounds_request_validate_valid() {
        let req = AddRoundsRequest {
            additional_rounds: 5,
        };
        assert!(req.validate().is_ok());

        let req = AddRoundsRequest {
            additional_rounds: 1,
        };
        assert!(req.validate().is_ok());

        let req = AddRoundsRequest {
            additional_rounds: 100,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_add_rounds_request_validate_invalid() {
        let req = AddRoundsRequest {
            additional_rounds: 0,
        };
        assert!(req.validate().is_err());

        let req = AddRoundsRequest {
            additional_rounds: -1,
        };
        assert!(req.validate().is_err());

        let req = AddRoundsRequest {
            additional_rounds: 101,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_generate_preview_short() {
        let prompt = "短内容";
        let preview = CandidatePromptSummary::generate_preview(prompt);
        assert_eq!(preview, "短内容");
    }

    #[test]
    fn test_generate_preview_long() {
        let prompt = "a".repeat(300);
        let preview = CandidatePromptSummary::generate_preview(&prompt);
        assert!(preview.ends_with("..."));
        // 200 chars + "..."
        assert_eq!(preview.chars().count(), 203);
    }

    #[test]
    fn test_generate_preview_exact_limit() {
        let prompt = "a".repeat(200);
        let preview = CandidatePromptSummary::generate_preview(&prompt);
        assert!(!preview.ends_with("..."));
        assert_eq!(preview.chars().count(), 200);
    }
}
