use std::collections::HashMap;

use prompt_faster::core::optimization_engine::create_optimization_engine;
use prompt_faster::domain::models::{
    Checkpoint, EvaluatorConfig, EvaluatorType, ExecutionTargetType, LineageType,
    OptimizationTaskConfig, TaskReference, TestCase,
};
use prompt_faster::domain::types::{
    ExecutionTargetConfig, OptimizationConfig, OptimizationContext,
};

fn make_ctx(prompt: &str, test_cases: Vec<TestCase>) -> OptimizationContext {
    let rule_system = prompt_faster::domain::models::RuleSystem {
        rules: vec![],
        conflict_resolution_log: vec![],
        merge_log: vec![],
        coverage_map: HashMap::new(),
        version: 1,
    };

    OptimizationContext {
        task_id: "t-1".to_string(),
        execution_target_config: ExecutionTargetConfig::default(),
        current_prompt: prompt.to_string(),
        rule_system,
        iteration: 0,
        state: prompt_faster::domain::models::IterationState::Idle,
        run_control_state: Default::default(),
        test_cases,
        config: OptimizationConfig::default(),
        checkpoints: vec![],
        extensions: HashMap::new(),
    }
}

#[tokio::test]
async fn optimization_engine_can_run_and_is_switchable_by_feature() {
    let prompt = "p";
    let prompt_len = prompt.chars().count();

    let mut input = HashMap::new();
    input.insert(
        "secret".to_string(),
        serde_json::Value::String("TOPSECRET_DO_NOT_ECHO".to_string()),
    );

    let tc1 = TestCase {
        id: "tc-1".to_string(),
        input: input.clone(),
        reference: TaskReference::Exact {
            expected: format!(
                "example_execution_target: test_case_id=tc-1 prompt_len={prompt_len} input_keys_count=1"
            ),
        },
        split: None,
        metadata: None,
    };
    let tc2 = TestCase {
        id: "tc-2".to_string(),
        input,
        reference: TaskReference::Exact {
            expected: format!(
                "example_execution_target: test_case_id=tc-2 prompt_len={prompt_len} input_keys_count=1"
            ),
        },
        split: None,
        metadata: None,
    };

    let task_cfg = OptimizationTaskConfig {
        max_concurrency: 1,
        evaluator_config: EvaluatorConfig {
            evaluator_type: EvaluatorType::Example,
            ..Default::default()
        },
        ..Default::default()
    };

    let engine = create_optimization_engine(ExecutionTargetType::Example, task_cfg);
    let expected_name = if cfg!(feature = "alt-optimization-engine") {
        "alternate_optimization_engine"
    } else {
        "default_optimization_engine"
    };
    assert_eq!(engine.name(), expected_name);

    let mut ctx = make_ctx(prompt, vec![tc1.clone(), tc2.clone()]);
    ctx.config.iteration.max_iterations = 2;
    ctx.config.iteration.pass_threshold = 1.0;

    let out = engine.run(&mut ctx).await.unwrap();
    assert!(out.should_terminate);
    assert_eq!(out.primary.content, prompt);
    assert!(out.primary.score >= 0.0 && out.primary.score <= 1.0);

    // 脱敏硬约束：不得把 TestCase input 全文泄露到结果/扩展中（即便 input 含明显敏感值）。
    let out_json = serde_json::to_string(&out).unwrap();
    assert!(!out_json.contains("TOPSECRET_DO_NOT_ECHO"));
    let ext_json = serde_json::to_string(&ctx.extensions).unwrap();
    assert!(!ext_json.contains("TOPSECRET_DO_NOT_ECHO"));

    if cfg!(feature = "alt-optimization-engine") {
        assert_eq!(
            out.extra.get("engine_variant").and_then(|v| v.as_str()),
            Some("alternate")
        );
    }
}

#[tokio::test]
async fn optimization_engine_default_evaluator_path_does_not_require_manual_extension_wiring() {
    // 覆盖 DefaultEvaluator 路径：若 OptimizationEngine 未注入 EXT_TASK_EVALUATOR_CONFIG，
    // 该测试会在 evaluate 阶段失败（属于“隐式炸弹”）。
    let prompt = "p";
    let prompt_len = prompt.chars().count();

    let tc = TestCase {
        id: "tc-1".to_string(),
        input: HashMap::new(),
        reference: TaskReference::Exact {
            expected: format!(
                "example_execution_target: test_case_id=tc-1 prompt_len={prompt_len} input_keys_count=0"
            ),
        },
        split: None,
        metadata: None,
    };

    let task_cfg = OptimizationTaskConfig {
        max_concurrency: 1,
        evaluator_config: EvaluatorConfig {
            evaluator_type: EvaluatorType::ExactMatch,
            ..Default::default()
        },
        ..Default::default()
    };

    let engine = create_optimization_engine(ExecutionTargetType::Example, task_cfg);
    let mut ctx = make_ctx(prompt, vec![tc]);
    ctx.config.iteration.max_iterations = 1;
    ctx.config.iteration.pass_threshold = 1.0;

    let out = engine.run(&mut ctx).await.unwrap();
    assert!(out.should_terminate);
}

#[tokio::test]
async fn optimization_engine_can_resume_from_checkpoint() {
    let prompt = "resume_prompt";
    let prompt_len = prompt.chars().count();

    let tc = TestCase {
        id: "tc-1".to_string(),
        input: HashMap::new(),
        reference: TaskReference::Exact {
            expected: format!(
                "example_execution_target: test_case_id=tc-1 prompt_len={prompt_len} input_keys_count=0"
            ),
        },
        split: None,
        metadata: None,
    };

    let task_cfg = OptimizationTaskConfig {
        max_concurrency: 1,
        evaluator_config: EvaluatorConfig {
            evaluator_type: EvaluatorType::Example,
            ..Default::default()
        },
        ..Default::default()
    };

    let engine = create_optimization_engine(ExecutionTargetType::Example, task_cfg);
    let mut ctx = make_ctx("different_prompt_before_resume", vec![tc]);
    ctx.config.iteration.max_iterations = 1;
    ctx.config.iteration.pass_threshold = 1.0;

    let checkpoint = Checkpoint {
        id: "cp-1".to_string(),
        task_id: "cp-task-1".to_string(),
        iteration: 0,
        state: prompt_faster::domain::models::IterationState::Idle,
        prompt: prompt.to_string(),
        rule_system: ctx.rule_system.clone(),
        created_at: 0,
        branch_id: "b-1".to_string(),
        parent_id: None,
        lineage_type: LineageType::Restored,
        branch_description: None,
    };

    let out = engine.resume(checkpoint, &mut ctx).await.unwrap();
    assert!(out.should_terminate);
    assert_eq!(ctx.task_id, "cp-task-1");
    assert_eq!(ctx.current_prompt, prompt);
    assert_eq!(ctx.iteration, 1);
    assert_eq!(
        ctx.state,
        prompt_faster::domain::models::IterationState::Completed
    );
}
