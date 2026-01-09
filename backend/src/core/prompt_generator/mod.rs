mod default_impl;
mod error;

pub use default_impl::DefaultPromptGenerator;
pub use error::GeneratorError;

/// `ctx.extensions` 中用于注入“优化目标”的 key。
///
/// - 类型：string
/// - 写入方：编排层（Orchestrator）
/// - 何时必需：`ctx.current_prompt` 为空/仅空白（初始 Prompt 生成阶段）
pub const EXT_OPTIMIZATION_GOAL: &str = "optimization_goal";

/// `ctx.extensions` 中用于注入“候选索引”的 key。
///
/// - 类型：number（u32）
/// - 写入方：编排层（Orchestrator）
/// - 何时必需：每次调用 `PromptGenerator.generate()`（用于确定性模板变体选择）
pub const EXT_CANDIDATE_INDEX: &str = "candidate_index";

/// Layer 2 内置的确定性模板变体数量。
///
/// 约定：`candidate_index` 必须满足 `0..TEMPLATE_VARIANT_COUNT`，否则返回可诊断错误，避免静默重复候选。
pub const TEMPLATE_VARIANT_COUNT: u32 = 10;
