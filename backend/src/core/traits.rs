//! 7 Trait 定义
//! 核心算法接口，支持 Mock 替换

use async_trait::async_trait;

/// 规律引擎 Trait
#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// 从测试用例中提取规律
    async fn extract_rules(&self, test_cases: &[TestCase]) -> anyhow::Result<Vec<Rule>>;
}

/// Prompt 生成器 Trait
#[async_trait]
pub trait PromptGenerator: Send + Sync {
    /// 基于规律生成候选 Prompt
    async fn generate(&self, rules: &[Rule], context: &OptimizationContext) -> anyhow::Result<Vec<String>>;
}

/// 评估器 Trait
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// 评估 Prompt 效果
    async fn evaluate(&self, prompt: &str, test_cases: &[TestCase]) -> anyhow::Result<EvaluationResult>;
}

/// 反馈聚合器 Trait
#[async_trait]
pub trait FeedbackAggregator: Send + Sync {
    /// 聚合评估反馈
    async fn aggregate(&self, results: &[EvaluationResult]) -> anyhow::Result<AggregatedFeedback>;
}

/// 优化器 Trait
#[async_trait]
pub trait Optimizer: Send + Sync {
    /// 执行优化步骤
    async fn optimize(&self, feedback: &AggregatedFeedback, context: &OptimizationContext) -> anyhow::Result<String>;
}

/// 老师模型 Trait
#[async_trait]
pub trait TeacherModel: Send + Sync {
    /// 生成 LLM 响应
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;
    
    /// 流式生成（返回 channel）
    async fn generate_stream(&self, prompt: &str) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>>;
}

/// 执行目标 Trait
#[async_trait]
pub trait ExecutionTarget: Send + Sync {
    /// 执行 Prompt 并获取结果
    async fn execute(&self, prompt: &str, input: &serde_json::Value) -> anyhow::Result<String>;
}

// 占位类型定义（将在 domain 模块中完善）
pub struct TestCase;
pub struct Rule;
pub struct OptimizationContext;
pub struct EvaluationResult;
pub struct AggregatedFeedback;
