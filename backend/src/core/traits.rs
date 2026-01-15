//! 7 Trait 定义
//! 核心算法接口，支持 Mock 替换

use crate::core::evaluator::EvaluatorError;
use crate::core::execution_target::ExecutionError;
use crate::core::feedback_aggregator::AggregatorError;
use crate::core::optimizer::OptimizerError;
use crate::core::prompt_generator::GeneratorError;
use crate::core::rule_engine::RuleEngineError;
use crate::domain::models::{
    ArbitrationResult, EvaluationResult, ExecutionResult, OptimizationResult, ReflectionResult,
    Rule, RuleConflict, SuggestionConflict, TerminationReason, TestCase, UnifiedReflection,
};
use crate::domain::types::{ExecutionTargetConfig, OptimizationContext};
use async_trait::async_trait;
use std::collections::HashMap;

/// 规律引擎 Trait
#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// 从测试用例中提取规律
    async fn extract_rules(
        &self,
        ctx: &OptimizationContext,
        test_cases: &[TestCase],
    ) -> Result<Vec<Rule>, RuleEngineError>;

    async fn detect_conflicts(
        &self,
        ctx: &OptimizationContext,
        rules: &[Rule],
    ) -> Result<Vec<RuleConflict>, RuleEngineError>;

    async fn resolve_conflict(
        &self,
        ctx: &OptimizationContext,
        conflict: &RuleConflict,
    ) -> Result<Rule, RuleEngineError>;

    async fn merge_similar_rules(
        &self,
        ctx: &OptimizationContext,
        rules: &[Rule],
    ) -> Result<Vec<Rule>, RuleEngineError>;

    fn name(&self) -> &str;
}

/// Prompt 生成器 Trait
#[async_trait]
pub trait PromptGenerator: Send + Sync {
    /// 基于规律体系生成 Prompt
    async fn generate(&self, ctx: &OptimizationContext) -> Result<String, GeneratorError>;

    /// 生成器名称
    fn name(&self) -> &str;
}

/// 评估器 Trait
#[async_trait]
pub trait Evaluator: Send + Sync {
    /// 评估单个测试用例的输出
    async fn evaluate(
        &self,
        ctx: &OptimizationContext,
        test_case: &TestCase,
        output: &str,
    ) -> Result<EvaluationResult, EvaluatorError>;

    /// 批量评估（results 不应在 Evaluator 内被过滤/重排）
    async fn evaluate_batch(
        &self,
        ctx: &OptimizationContext,
        results: &[(TestCase, String)],
    ) -> Result<Vec<EvaluationResult>, EvaluatorError>;

    /// 评估器名称
    fn name(&self) -> &str;
}

/// 反馈聚合器 Trait
#[async_trait]
pub trait FeedbackAggregator: Send + Sync {
    /// 聚合反思结果
    async fn aggregate(
        &self,
        ctx: &OptimizationContext,
        reflections: &[ReflectionResult],
    ) -> Result<UnifiedReflection, AggregatorError>;

    /// 仲裁冲突的建议
    async fn arbitrate(
        &self,
        ctx: &OptimizationContext,
        conflicts: &[SuggestionConflict],
    ) -> Result<ArbitrationResult, AggregatorError>;

    fn name(&self) -> &str;
}

/// 优化器 Trait
#[async_trait]
pub trait Optimizer: Send + Sync {
    /// 基于统一反馈执行一步优化
    async fn optimize_step(
        &self,
        ctx: &OptimizationContext,
        unified_reflection: &UnifiedReflection,
    ) -> Result<OptimizationResult, OptimizerError>;

    /// 判断是否应该终止迭代
    fn should_terminate(
        &self,
        ctx: &OptimizationContext,
        history: &[OptimizationResult],
    ) -> Option<TerminationReason>;

    fn name(&self) -> &str;
}

/// 老师模型 Trait
#[async_trait]
pub trait TeacherModel: Send + Sync {
    /// 生成 LLM 响应
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;

    /// 流式生成（返回 channel）
    async fn generate_stream(
        &self,
        prompt: &str,
    ) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>>;
}

/// 执行目标 Trait
#[async_trait]
pub trait ExecutionTarget: Send + Sync {
    /// 执行 Prompt 并返回输出
    async fn execute(
        &self,
        execution_target_config: &ExecutionTargetConfig,
        prompt: &str,
        input: &HashMap<String, serde_json::Value>,
        test_case_id: &str,
    ) -> Result<ExecutionResult, ExecutionError>;

    /// 批量执行（默认串行，同序返回；并行由编排层负责调度）
    async fn execute_batch(
        &self,
        execution_target_config: &ExecutionTargetConfig,
        prompt: &str,
        inputs: &[HashMap<String, serde_json::Value>],
        test_case_ids: &[String],
    ) -> Result<Vec<ExecutionResult>, ExecutionError> {
        if inputs.len() != test_case_ids.len() {
            return Err(ExecutionError::InvalidRequest {
                test_case_id: "unknown".to_string(),
                message: format!(
                    "inputs/test_case_ids length mismatch: {} vs {}",
                    inputs.len(),
                    test_case_ids.len()
                ),
            });
        }

        let mut results = Vec::with_capacity(inputs.len());
        for (input, test_case_id) in inputs.iter().zip(test_case_ids.iter()) {
            results.push(
                self.execute(execution_target_config, prompt, input, test_case_id)
                    .await?,
            );
        }
        Ok(results)
    }

    fn name(&self) -> &str;
}
