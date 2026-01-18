use std::collections::HashMap;

use prompt_faster::core::optimization_engine::save_checkpoint_after_layer;
use prompt_faster::domain::models::{IterationState, RuleSystem};
use prompt_faster::domain::types::{ExecutionTargetConfig, OptimizationConfig, OptimizationContext, RunControlState};

fn build_context(task_id: &str) -> OptimizationContext {
    OptimizationContext {
        task_id: task_id.to_string(),
        execution_target_config: ExecutionTargetConfig::default(),
        current_prompt: "prompt".to_string(),
        rule_system: RuleSystem {
            rules: vec![],
            conflict_resolution_log: vec![],
            merge_log: vec![],
            coverage_map: HashMap::new(),
            version: 1,
        },
        iteration: 1,
        state: IterationState::Completed,
        run_control_state: RunControlState::Running,
        test_cases: vec![],
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions: HashMap::new(),
    }
}

#[tokio::test]
async fn checkpoint_save_degrades_without_db() {
    let ctx = build_context("task-no-db");
    save_checkpoint_after_layer(&ctx).await;
}
