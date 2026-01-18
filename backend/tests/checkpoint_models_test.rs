use prompt_faster::domain::models::CheckpointResponse;

#[test]
fn checkpoint_response_serializes_camel_case() {
    let response = CheckpointResponse {
        id: "checkpoint-1".to_string(),
        task_id: "task-1".to_string(),
        iteration: 1,
        state: "completed".to_string(),
        run_control_state: "running".to_string(),
        prompt_preview: "preview".to_string(),
        has_artifacts: true,
        has_user_guidance: false,
        checksum: "checksum".to_string(),
        integrity_ok: true,
        created_at: "2026-01-18T00:00:00Z".to_string(),
    };

    let value = serde_json::to_value(response).expect("序列化失败");
    assert!(value.get("taskId").is_some());
    assert!(value.get("promptPreview").is_some());
    assert!(value.get("hasUserGuidance").is_some());
    assert!(value.get("integrityOk").is_some());
}
