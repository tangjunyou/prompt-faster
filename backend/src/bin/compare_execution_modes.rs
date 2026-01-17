use std::collections::HashMap;
use std::sync::Arc;

use prompt_faster::core::evaluator::{DefaultEvaluator, EXT_TASK_EVALUATOR_CONFIG};
use prompt_faster::core::iteration_engine::executor::{parallel_execute, serial_execute};
use prompt_faster::core::traits::{Evaluator, ExecutionTarget};
use prompt_faster::domain::models::{
    EvaluatorConfig as TaskEvaluatorConfig, EvaluatorType, ExecutionResult, IterationState,
    RuleSystem, TaskReference, TestCase,
};
use prompt_faster::domain::types::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext,
};

#[derive(Debug)]
struct MockExecutionTarget {
    sleep_ms: u64,
}

#[async_trait::async_trait]
impl ExecutionTarget for MockExecutionTarget {
    async fn execute(
        &self,
        _execution_target_config: &ExecutionTargetConfig,
        _prompt: &str,
        input: &HashMap<String, serde_json::Value>,
        test_case_id: &str,
    ) -> Result<ExecutionResult, prompt_faster::core::execution_target::ExecutionError> {
        let start = tokio::time::Instant::now();
        tokio::time::sleep(tokio::time::Duration::from_millis(self.sleep_ms)).await;

        let output = input
            .get("expected")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(ExecutionResult {
            test_case_id: test_case_id.to_string(),
            output,
            latency_ms: start.elapsed().as_millis() as u64,
            token_usage: None,
            raw_response: None,
        })
    }

    fn name(&self) -> &str {
        "mock"
    }
}

fn build_test_cases(batch_size: usize) -> Vec<TestCase> {
    (0..batch_size)
        .map(|i| {
            let expected = if i % 5 == 0 { "BAD" } else { "OK" };
            let mut input = HashMap::new();
            input.insert(
                "expected".to_string(),
                serde_json::Value::String(expected.to_string()),
            );
            TestCase {
                id: format!("tc-{i}"),
                input,
                reference: TaskReference::Exact {
                    expected: expected.to_string(),
                },
                split: None,
                metadata: None,
            }
        })
        .collect()
}

fn build_ctx(test_cases: Vec<TestCase>) -> OptimizationContext {
    let mut ctx = OptimizationContext {
        task_id: "mock-task".to_string(),
        execution_target_config: ExecutionTargetConfig::default(),
        current_prompt: "mock prompt".to_string(),
        rule_system: RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: HashMap::new(),
            version: 1,
        },
        iteration: 1,
        state: IterationState::RunningTests,
        run_control_state: Default::default(),
        test_cases,
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions: HashMap::new(),
    };

    let task_evaluator_config = TaskEvaluatorConfig {
        evaluator_type: EvaluatorType::ExactMatch,
        ..TaskEvaluatorConfig::default()
    };
    ctx.extensions.insert(
        EXT_TASK_EVALUATOR_CONFIG.to_string(),
        serde_json::to_value(task_evaluator_config).expect("serialize task_evaluator_config"),
    );

    ctx
}

fn summarize(evals: &[prompt_faster::domain::models::EvaluationResult]) -> (f64, f64) {
    let total = evals.len() as f64;
    let passed = evals.iter().filter(|e| e.passed).count() as f64;
    let mean_score = evals.iter().map(|e| e.score).sum::<f64>() / total;
    (passed / total, mean_score)
}

async fn run_compare(batch_size: usize, max_concurrency: u32, sleep_ms: u64) -> anyhow::Result<()> {
    let test_cases = build_test_cases(batch_size);
    let ctx = build_ctx(test_cases.clone());
    let evaluator = DefaultEvaluator::new(None);

    let execution_target: Arc<dyn ExecutionTarget> = Arc::new(MockExecutionTarget { sleep_ms });
    let prompt = "p";

    let serial_results = serial_execute(
        execution_target.as_ref(),
        &ctx.execution_target_config,
        prompt,
        &test_cases,
    )
    .await?;
    let serial_pairs = test_cases
        .iter()
        .cloned()
        .zip(serial_results.iter().map(|r| r.output.clone()))
        .collect::<Vec<_>>();
    let serial_evals = evaluator.evaluate_batch(&ctx, &serial_pairs).await?;
    let (serial_passed_rate, serial_mean_score) = summarize(&serial_evals);

    let parallel_results = parallel_execute(
        Arc::clone(&execution_target),
        &ctx.execution_target_config,
        prompt,
        &test_cases,
        max_concurrency,
    )
    .await?;
    let parallel_pairs = test_cases
        .iter()
        .cloned()
        .zip(parallel_results.iter().map(|r| r.output.clone()))
        .collect::<Vec<_>>();
    let parallel_evals = evaluator.evaluate_batch(&ctx, &parallel_pairs).await?;
    let (parallel_passed_rate, parallel_mean_score) = summarize(&parallel_evals);

    println!("# compare_execution_modes");
    println!();
    println!("- batch_size: {batch_size}");
    println!("- sleep_ms_per_call: {sleep_ms}");
    println!("- max_concurrency: {max_concurrency}");
    println!();
    println!("| mode | passed_rate | mean_score |");
    println!("|---|---:|---:|");
    println!(
        "| serial | {:.2}% | {:.4} |",
        serial_passed_rate * 100.0,
        serial_mean_score
    );
    println!(
        "| parallel | {:.2}% | {:.4} |",
        parallel_passed_rate * 100.0,
        parallel_mean_score
    );
    println!();
    println!(
        "- abs_delta_passed_rate: {:.2} percentage points",
        (serial_passed_rate - parallel_passed_rate).abs() * 100.0
    );
    println!(
        "- abs_delta_mean_score: {:.4}",
        (serial_mean_score - parallel_mean_score).abs()
    );
    println!();
    println!("Expected: deltas < 5% (MVP threshold).");

    Ok(())
}

async fn run_benchmark(batch_size: usize, sleep_ms: u64) -> anyhow::Result<()> {
    let test_cases = build_test_cases(batch_size);
    let execution_target: Arc<dyn ExecutionTarget> = Arc::new(MockExecutionTarget { sleep_ms });
    let prompt = "p";
    let execution_target_config = ExecutionTargetConfig::default();

    let concurrencies = [1_u32, 2, 4, 8];

    let mut best_serial_ms = u128::MAX;
    for _ in 0..3 {
        let start = tokio::time::Instant::now();
        let _ = serial_execute(
            execution_target.as_ref(),
            &execution_target_config,
            prompt,
            &test_cases,
        )
        .await?;
        best_serial_ms = best_serial_ms.min(start.elapsed().as_millis());
    }
    let batch_size_u128 = batch_size.max(1) as u128;

    println!("# Execution scheduler benchmark");
    println!();
    println!("- batch_size: {batch_size}");
    println!("- sleep_ms_per_call: {sleep_ms}");
    println!("- baseline_serial_wall_clock_ms: {best_serial_ms}");
    println!();
    println!("## Method");
    println!();
    println!(
        "- baseline_serial_wall_clock_ms：串行跑 3 次取最小 wall_clock_ms（作为“模型调用时间”口径）"
    );
    println!(
        "- expected_ms(N) = ceil(batch_size / N) * baseline_serial_wall_clock_ms / batch_size"
    );
    println!(
        "- scheduling_overhead_ms = wall_clock_ms - expected_ms（调度开销口径，不含模型调用时间）"
    );
    println!("- wall_clock_ms 取 3 次运行的最小值（减小抖动影响）");
    println!();
    println!("## Results");
    println!();
    println!("| max_concurrency | wall_clock_ms | expected_ms | scheduling_overhead_ms |");
    println!("|---:|---:|---:|---:|");

    for &n in &concurrencies {
        let mut best_ms = u128::MAX;
        for _ in 0..3 {
            let start = tokio::time::Instant::now();
            let _ = parallel_execute(
                Arc::clone(&execution_target),
                &execution_target_config,
                prompt,
                &test_cases,
                n,
            )
            .await?;
            best_ms = best_ms.min(start.elapsed().as_millis());
        }
        let chunks = (batch_size as u128).div_ceil(n as u128);
        let expected_ms = chunks * best_serial_ms / batch_size_u128;
        let overhead_ms = best_ms.saturating_sub(expected_ms);
        println!("| {n} | {best_ms} | {expected_ms} | {overhead_ms} |");
    }

    let expected_serial_ms = best_serial_ms;
    let overhead_serial_ms = best_serial_ms.saturating_sub(expected_serial_ms);
    println!();
    println!("### Serial (for reference)");
    println!();
    println!("| mode | wall_clock_ms | expected_ms | scheduling_overhead_ms |");
    println!("|---|---:|---:|---:|");
    println!("| serial | {best_serial_ms} | {expected_serial_ms} | {overhead_serial_ms} |");

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut batch_size = 64usize;
    let mut max_concurrency = 8u32;
    let mut sleep_ms = 20u64;
    let mut benchmark = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--benchmark" => benchmark = true,
            "--batch-size" => {
                batch_size = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--batch-size requires a value"))?
                    .parse()?
            }
            "--max-concurrency" => {
                max_concurrency = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--max-concurrency requires a value"))?
                    .parse()?
            }
            "--sleep-ms" => {
                sleep_ms = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--sleep-ms requires a value"))?
                    .parse()?
            }
            _ => return Err(anyhow::anyhow!("unknown arg: {arg}")),
        }
    }

    if benchmark {
        run_benchmark(batch_size, sleep_ms).await?;
    } else {
        run_compare(batch_size, max_concurrency, sleep_ms).await?;
    }
    Ok(())
}
