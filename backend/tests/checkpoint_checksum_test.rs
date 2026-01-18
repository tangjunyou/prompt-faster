use std::collections::HashMap;

use prompt_faster::core::iteration_engine::checkpoint::{compute_checksum, verify_checksum};
use prompt_faster::domain::models::{
    CheckpointCreateRequest, CheckpointEntity, IterationState, LineageType, RuleSystem,
};
use prompt_faster::domain::types::{IterationArtifacts, RunControlState, UserGuidance};

fn build_rule_system() -> RuleSystem {
    RuleSystem {
        rules: Vec::new(),
        conflict_resolution_log: Vec::new(),
        merge_log: Vec::new(),
        coverage_map: HashMap::new(),
        version: 1,
    }
}

fn build_request() -> CheckpointCreateRequest {
    CheckpointCreateRequest {
        task_id: "task-1".to_string(),
        iteration: 1,
        state: IterationState::Completed,
        run_control_state: RunControlState::Running,
        prompt: "prompt".to_string(),
        rule_system: build_rule_system(),
        artifacts: Some(IterationArtifacts::empty()),
        user_guidance: Some(UserGuidance::new("guidance")),
        branch_id: "branch-1".to_string(),
        parent_id: None,
        lineage_type: LineageType::Automatic,
        branch_description: None,
    }
}

#[test]
fn checksum_computes_and_verifies() {
    let req = build_request();
    let checksum = compute_checksum(&req);

    let entity = CheckpointEntity {
        id: "checkpoint-1".to_string(),
        task_id: req.task_id.clone(),
        iteration: req.iteration,
        state: req.state,
        run_control_state: req.run_control_state,
        prompt: req.prompt.clone(),
        rule_system: req.rule_system.clone(),
        artifacts: req.artifacts.clone(),
        user_guidance: req.user_guidance.clone(),
        branch_id: req.branch_id.clone(),
        parent_id: req.parent_id.clone(),
        lineage_type: req.lineage_type.clone(),
        branch_description: req.branch_description.clone(),
        checksum: checksum.clone(),
        created_at: 0,
    };

    assert!(verify_checksum(&entity));

    let mut altered = entity.clone();
    altered.checksum = "wrong".to_string();
    assert!(!verify_checksum(&altered));

    let mut changed_req = req.clone();
    changed_req.iteration = 2;
    let checksum2 = compute_checksum(&changed_req);
    assert_ne!(checksum, checksum2);
}
