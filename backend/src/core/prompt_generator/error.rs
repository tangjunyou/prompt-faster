use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    #[error("ctx.extensions 缺少必需字段：{keys:?}")]
    MissingContext { keys: Vec<String> },

    #[error("检测到 polarity=all_passed；编排层应短路而不调用 Layer 2（rule_ids={rule_ids:?}）")]
    AllPassed { rule_ids: Vec<String> },

    #[error("规律不合法：{reason}")]
    InvalidRules { reason: String },

    #[error("ctx.extensions 字段类型不合法：{reason}")]
    InvalidContext { reason: String },

    #[error(
        "检测到重复候选：candidate_index={candidate_index} fingerprint={fingerprint}（已命中失败档案或重复内容）"
    )]
    DuplicateCandidate {
        candidate_index: u32,
        fingerprint: String,
    },
}
