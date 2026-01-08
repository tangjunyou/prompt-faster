use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleEngineError {
    #[error("ctx.extensions 缺少 layer1_test_results")]
    MissingLayer1TestResults,

    #[error("ctx.extensions.layer1_test_results 格式错误：{0}")]
    InvalidLayer1TestResults(String),

    #[error("存在缺失的测试结果：{missing_test_case_ids:?}")]
    MissingTestResults { missing_test_case_ids: Vec<String> },
}
