//! 迭代中间产物类型定义
//!
//! 定义用于用户编辑的中间产物类型，支持规律假设和候选 Prompt 的查看与编辑。
//! 注意：这些类型是面向编辑视图的轻量结构，与 `RuleSystem` 有映射关系。

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use crate::domain::models::optimization_task_config::OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES;

/// 产物来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ArtifactSource {
    /// 系统生成
    System,
    /// 用户编辑
    UserEdited,
}

impl Default for ArtifactSource {
    fn default() -> Self {
        Self::System
    }
}

/// 规律假设
///
/// 面向编辑视图的轻量结构。
/// 映射关系：`PatternHypothesis.pattern` 对应 `Rule.description`
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PatternHypothesis {
    /// 唯一标识（对应 Rule.id）
    pub id: String,
    /// 规律描述（对应 Rule.description）
    pub pattern: String,
    /// 来源（system/user_edited）
    #[serde(default)]
    pub source: ArtifactSource,
    /// 置信度（0-1，对应 Rule.verification_score）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// 候选 Prompt
///
/// 面向编辑视图的轻量结构。
/// 来源：当前迭代生成的候选 Prompt 列表或当前最佳 Prompt。
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct CandidatePrompt {
    /// 唯一标识
    pub id: String,
    /// Prompt 内容
    pub content: String,
    /// 来源（system/user_edited）
    #[serde(default)]
    pub source: ArtifactSource,
    /// 评估分数（如有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// 是否为当前最佳候选
    #[serde(default)]
    pub is_best: bool,
}

/// 迭代中间产物集合
///
/// 包含当前迭代的规律假设列表和候选 Prompt 列表。
/// 用户编辑后的产物将在恢复/继续前映射回 `OptimizationContext`。
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationArtifacts {
    /// 规律假设列表
    #[serde(default)]
    pub patterns: Vec<PatternHypothesis>,
    /// 候选 Prompt 列表
    #[serde(default)]
    pub candidate_prompts: Vec<CandidatePrompt>,
    /// 最后更新时间戳（ISO 8601 格式）
    #[serde(default)]
    pub updated_at: String,
}

impl IterationArtifacts {
    /// 创建空的产物集合
    pub fn empty() -> Self {
        Self::default()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty() && self.candidate_prompts.is_empty()
    }

    /// 获取指定 ID 的规律假设
    pub fn get_pattern(&self, id: &str) -> Option<&PatternHypothesis> {
        self.patterns.iter().find(|p| p.id == id)
    }

    /// 获取指定 ID 的候选 Prompt
    pub fn get_candidate_prompt(&self, id: &str) -> Option<&CandidatePrompt> {
        self.candidate_prompts.iter().find(|p| p.id == id)
    }

    /// 获取所有规律假设的 ID 集合
    pub fn pattern_ids(&self) -> Vec<&str> {
        self.patterns.iter().map(|p| p.id.as_str()).collect()
    }

    /// 获取所有候选 Prompt 的 ID 集合
    pub fn candidate_prompt_ids(&self) -> Vec<&str> {
        self.candidate_prompts.iter().map(|p| p.id.as_str()).collect()
    }

    /// 验证更新操作的合法性（仅允许修改/删除已有 id，禁止新增）
    ///
    /// 返回 `Ok(())` 表示验证通过，`Err(reason)` 表示验证失败
    pub fn validate_update(&self, updated: &IterationArtifacts) -> Result<(), String> {
        let existing_pattern_ids: std::collections::HashSet<&str> =
            self.patterns.iter().map(|p| p.id.as_str()).collect();
        let existing_prompt_ids: std::collections::HashSet<&str> =
            self.candidate_prompts.iter().map(|p| p.id.as_str()).collect();

        // 检查是否有新增的规律假设 ID
        for pattern in &updated.patterns {
            if !existing_pattern_ids.contains(pattern.id.as_str()) {
                return Err(format!(
                    "禁止新增规律假设：id '{}' 不存在于原有产物中",
                    pattern.id
                ));
            }
        }

        // 检查是否有新增的候选 Prompt ID
        for prompt in &updated.candidate_prompts {
            if !existing_prompt_ids.contains(prompt.id.as_str()) {
                return Err(format!(
                    "禁止新增候选 Prompt：id '{}' 不存在于原有产物中",
                    prompt.id
                ));
            }
        }

        Ok(())
    }

    /// 验证产物内容长度（基于初始 Prompt 长度限制）
    pub fn validate_content_length(&self) -> Result<(), String> {
        let max_bytes = OPTIMIZATION_TASK_CONFIG_MAX_INITIAL_PROMPT_BYTES;

        for pattern in &self.patterns {
            if pattern.pattern.as_bytes().len() > max_bytes {
                return Err(format!(
                    "规律假设内容过长（超过 {max_bytes} 字节）"
                ));
            }
        }

        for prompt in &self.candidate_prompts {
            if prompt.content.as_bytes().len() > max_bytes {
                return Err(format!(
                    "候选 Prompt 内容过长（超过 {max_bytes} 字节）"
                ));
            }
        }

        Ok(())
    }

    /// 应用用户编辑，返回新的产物集合
    ///
    /// - 仅更新已存在的条目
    /// - 将编辑后的条目来源标记为 `UserEdited`
    /// - 删除的条目不会出现在结果中
    pub fn apply_update(&self, updated: &IterationArtifacts) -> Self {
        let updated_pattern_map: std::collections::HashMap<&str, &PatternHypothesis> =
            updated.patterns.iter().map(|p| (p.id.as_str(), p)).collect();
        let updated_prompt_map: std::collections::HashMap<&str, &CandidatePrompt> =
            updated.candidate_prompts.iter().map(|p| (p.id.as_str(), p)).collect();

        // 应用规律假设更新
        let new_patterns: Vec<PatternHypothesis> = self
            .patterns
            .iter()
            .filter_map(|original| {
                updated_pattern_map.get(original.id.as_str()).map(|updated| {
                    let mut result = (*updated).clone();
                    // 如果内容有变化，标记为用户编辑
                    if result.pattern != original.pattern {
                        result.source = ArtifactSource::UserEdited;
                    }
                    result
                })
            })
            .collect();

        // 应用候选 Prompt 更新
        let new_prompts: Vec<CandidatePrompt> = self
            .candidate_prompts
            .iter()
            .filter_map(|original| {
                updated_prompt_map.get(original.id.as_str()).map(|updated| {
                    let mut result = (*updated).clone();
                    // 如果内容有变化，标记为用户编辑
                    if result.content != original.content {
                        result.source = ArtifactSource::UserEdited;
                    }
                    result
                })
            })
            .collect();

        Self {
            patterns: new_patterns,
            candidate_prompts: new_prompts,
            updated_at: updated.updated_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_source_default() {
        assert_eq!(ArtifactSource::default(), ArtifactSource::System);
    }

    #[test]
    fn test_iteration_artifacts_empty() {
        let artifacts = IterationArtifacts::empty();
        assert!(artifacts.is_empty());
        assert!(artifacts.patterns.is_empty());
        assert!(artifacts.candidate_prompts.is_empty());
    }

    #[test]
    fn test_iteration_artifacts_get_pattern() {
        let artifacts = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "test pattern".to_string(),
                source: ArtifactSource::System,
                confidence: Some(0.9),
            }],
            candidate_prompts: vec![],
            updated_at: "2026-01-17T12:00:00Z".to_string(),
        };

        assert!(artifacts.get_pattern("p1").is_some());
        assert!(artifacts.get_pattern("p2").is_none());
    }

    #[test]
    fn test_validate_update_no_new_ids() {
        let original = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "original".to_string(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![CandidatePrompt {
                id: "c1".to_string(),
                content: "original prompt".to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: false,
            }],
            updated_at: "".to_string(),
        };

        // 合法更新：修改已有条目
        let valid_update = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "modified".to_string(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![],
            updated_at: "".to_string(),
        };
        assert!(original.validate_update(&valid_update).is_ok());

        // 非法更新：新增 ID
        let invalid_update = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p2".to_string(),
                pattern: "new pattern".to_string(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![],
            updated_at: "".to_string(),
        };
        assert!(original.validate_update(&invalid_update).is_err());
    }

    #[test]
    fn test_apply_update_marks_user_edited() {
        let original = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "original".to_string(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![CandidatePrompt {
                id: "c1".to_string(),
                content: "original prompt".to_string(),
                source: ArtifactSource::System,
                score: None,
                is_best: false,
            }],
            updated_at: "".to_string(),
        };

        let updated = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "modified pattern".to_string(),
                source: ArtifactSource::System, // 用户提交时可能仍为 System
                confidence: None,
            }],
            candidate_prompts: vec![CandidatePrompt {
                id: "c1".to_string(),
                content: "original prompt".to_string(), // 未修改
                source: ArtifactSource::System,
                score: None,
                is_best: false,
            }],
            updated_at: "2026-01-17T12:00:00Z".to_string(),
        };

        let result = original.apply_update(&updated);

        // 修改过的应该标记为 UserEdited
        assert_eq!(result.patterns[0].source, ArtifactSource::UserEdited);
        // 未修改的保持原样
        assert_eq!(result.candidate_prompts[0].source, ArtifactSource::System);
    }

    #[test]
    fn test_apply_update_removes_deleted() {
        let original = IterationArtifacts {
            patterns: vec![
                PatternHypothesis {
                    id: "p1".to_string(),
                    pattern: "pattern 1".to_string(),
                    source: ArtifactSource::System,
                    confidence: None,
                },
                PatternHypothesis {
                    id: "p2".to_string(),
                    pattern: "pattern 2".to_string(),
                    source: ArtifactSource::System,
                    confidence: None,
                },
            ],
            candidate_prompts: vec![],
            updated_at: "".to_string(),
        };

        // 用户删除了 p2
        let updated = IterationArtifacts {
            patterns: vec![PatternHypothesis {
                id: "p1".to_string(),
                pattern: "pattern 1".to_string(),
                source: ArtifactSource::System,
                confidence: None,
            }],
            candidate_prompts: vec![],
            updated_at: "".to_string(),
        };

        let result = original.apply_update(&updated);
        assert_eq!(result.patterns.len(), 1);
        assert_eq!(result.patterns[0].id, "p1");
    }
}
