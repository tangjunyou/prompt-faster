# Story {{epic_num}}.{{story_num}}: {{story_title}}

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a {{role}},
I want {{action}},
so that {{benefit}}.

## Acceptance Criteria

1. [Add acceptance criteria from epics/PRD]

## Tasks / Subtasks

- [ ] Task 1 (AC: #)
  - [ ] Subtask 1.1
- [ ] Task 2 (AC: #)
  - [ ] Subtask 2.1

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [ ] correlationId 全链路透传（HTTP/WS/日志）
- [ ] A2 日志字段齐全（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）
- [ ] 新增/变更类型已运行 gen-types 并提交生成产物
- [ ] 状态一致性与幂等性已校验（如 RunControlState / IterationState）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免“只记在聊天里/只散落在文档里”。

- [ ] [AI-Review] (placeholder) 将本 Story 的 review 结论沉淀到 `## Review Notes`（含风险/遗留）

## Dev Notes

- Relevant architecture patterns and constraints
- Source tree components to touch
- Testing standards summary

### Project Structure Notes

- Alignment with unified project structure (paths, modules, naming)
- Detected conflicts or variances (with rationale)

### References

- Cite all technical details with source paths and sections, e.g. [Source: docs/<file>.md#Section]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [ ] (placeholder) 发现的问题/建议（按严重度：CRITICAL/HIGH/MEDIUM/LOW）

### Decisions

- [ ] (placeholder) 本次做了哪些关键取舍？为什么？

### Risks / Tech Debt

- [ ] (placeholder) 风险与遗留（如果暂时不修，写清楚“不修的理由”和“触发条件”）

### Follow-ups

- [ ] (placeholder) 后续行动项（并同步到 `### Review Follow-ups (AI)`）
