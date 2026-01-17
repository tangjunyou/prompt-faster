# Epic 6 用户介入可追踪性验证规范（A2 门槛交付物）

- 项目：Prompt Faster
- Epic：6（用户介入与控制）
- 版本：1.0
- 日期：2026-01-16
- 作者：Dana（QA Engineer）+ Amelia（Dev Agent）
- 状态：**评审通过**

> 本文档为 Epic 6 开工门槛 A2 的交付物，定义用户介入操作的可追踪证据链要求。

---

## 1. 可追踪性目标

确保所有用户介入操作（暂停/继续/停止/编辑）满足以下要求：

| 目标 | 说明 |
|------|------|
| **可审计** | 所有操作记录在日志中，支持事后审计 |
| **可回放** | 操作序列可用于状态回放（Epic 7 Checkpoint 前置） |
| **可关联** | 通过 `correlationId` 串联请求-处理-事件全链路 |
| **可诊断** | 日志包含足够信息用于问题排查 |

---

## 2. 日志格式规范

### 2.1 必填字段

所有用户介入操作日志**必须**包含以下字段：

| 字段 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `correlation_id` | string | 全链路追踪 ID | `"corr-abc-123"` |
| `user_id` | string | 操作用户 ID | `"user-456"` |
| `task_id` | string | 任务 ID | `"task-789"` |
| `action` | string | 操作类型 | `"pause"` / `"resume"` / `"stop"` |
| `prev_state` | string | 操作前 RunControlState | `"running"` |
| `new_state` | string | 操作后 RunControlState | `"paused"` |
| `iteration_state` | string | 当前 IterationState | `"extracting_rules"` |
| `timestamp` | string | ISO 8601 时间戳 | `"2026-01-16T22:00:00Z"` |

### 2.2 可选字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `iteration_index` | number | 当前迭代轮次 |
| `stage_group` | string | 当前阶段分组（来自 iteration_stage.rs） |
| `error_code` | string | 失败时的错误码 |
| `error_message` | string | 失败时的错误信息 |

### 2.3 Rust 日志实现示例

```rust
// 位置：backend/src/api/ws/events.rs
use tracing::{info, warn, instrument};

#[instrument(skip(task_repo), fields(correlation_id = %correlation_id))]
async fn handle_pause_command(
    cmd: PauseCommand,
    user_id: &str,
    correlation_id: &str,
    task_repo: &TaskRepository,
) -> Result<(), WsError> {
    let task = task_repo.get_by_id(&cmd.task_id).await?;
    let prev_state = task.run_control_state;
    
    // ... 执行暂停逻辑 ...
    
    let new_state = RunControlState::Paused;
    
    // 必填字段日志
    info!(
        correlation_id = %correlation_id,
        user_id = %user_id,
        task_id = %cmd.task_id,
        action = "pause",
        prev_state = ?prev_state,
        new_state = ?new_state,
        iteration_state = ?task.context.state,
        "用户介入操作：暂停任务"
    );
    
    Ok(())
}
```

### 2.4 日志级别规范

| 场景 | 级别 | 说明 |
|------|------|------|
| 操作成功 | `INFO` | 正常记录 |
| 状态转换失败 | `WARN` | 非法转换尝试 |
| 权限校验失败 | `WARN` | 无权操作 |
| 持久化失败 | `ERROR` | 需要告警 |
| WS 推送失败 | `WARN` | 记录但不阻塞 |

---

## 3. WS 事件追踪规范

### 3.1 事件必须携带 correlationId

```typescript
// 所有 WS 事件必须包含 correlationId
interface WsMessage<T> {
  type: string;
  payload: T;
  timestamp: string;
  correlationId: string;  // 必填！
}
```

### 3.2 correlationId 生命周期

```
前端发起命令 (correlationId = "corr-xxx")
    ↓
后端 WS handler 接收并透传
    ↓
后端处理逻辑（日志携带 correlationId）
    ↓
后端推送事件（携带相同 correlationId）
    ↓
前端接收事件（按 correlationId 匹配）
```

---

## 4. 测试用例清单

### 4.1 后端单元测试

| 测试用例 ID | 描述 | 验证点 |
|-------------|------|--------|
| `T-A2-BE-01` | 暂停操作日志包含必填字段 | 验证 tracing span 包含所有必填字段 |
| `T-A2-BE-02` | 继续操作日志包含必填字段 | 同上 |
| `T-A2-BE-03` | 停止操作日志包含必填字段 | 同上 |
| `T-A2-BE-04` | 失败操作记录 WARN 日志 | 验证非法转换尝试记录 WARN |
| `T-A2-BE-05` | correlationId 在 WS 事件中透传 | 验证响应事件的 correlationId 与请求一致 |

### 4.2 前端组件测试

| 测试用例 ID | 描述 | 验证点 |
|-------------|------|--------|
| `T-A2-FE-01` | PauseResumeControl 发送命令携带 correlationId | 验证 WS 命令包含 correlationId |
| `T-A2-FE-02` | 事件处理按 correlationId 过滤 | 验证不处理其他任务的事件 |

### 4.3 集成测试

| 测试用例 ID | 描述 | 验证点 |
|-------------|------|--------|
| `T-A2-INT-01` | 暂停→继续全链路 correlationId 一致性 | 从发送到接收，correlationId 保持一致 |
| `T-A2-INT-02` | 多任务并行操作隔离 | 两个任务的 pause/resume 不互相影响 |
| `T-A2-INT-03` | 日志可用于状态重建 | 解析日志可还原操作序列 |

### 4.4 测试实现示例

```rust
// backend/src/api/ws/events.rs
#[cfg(test)]
mod traceability_tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn pause_operation_logs_required_fields() {
        // Arrange
        let correlation_id = "test-corr-001";
        let user_id = "user-001";
        let task_id = "task-001";
        
        // Act
        // ... 调用 handle_pause_command ...
        
        // Assert: 验证日志包含必填字段
        assert!(logs_contain("correlation_id"));
        assert!(logs_contain("user_id"));
        assert!(logs_contain("task_id"));
        assert!(logs_contain("action"));
        assert!(logs_contain("prev_state"));
        assert!(logs_contain("new_state"));
        assert!(logs_contain("iteration_state"));
    }

    #[tokio::test]
    async fn ws_event_preserves_correlation_id() {
        // Arrange
        let request_correlation_id = "test-corr-002";
        
        // Act
        let response_event = /* 调用并获取响应事件 */;
        
        // Assert
        assert_eq!(response_event.correlation_id, request_correlation_id);
    }
}
```

```typescript
// frontend/src/features/user-intervention/PauseResumeControl.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/react';
import { PauseResumeControl } from './PauseResumeControl';

describe('PauseResumeControl 可追踪性', () => {
  it('T-A2-FE-01: 发送命令携带 correlationId', async () => {
    const mockSendCommand = vi.fn();
    
    render(<PauseResumeControl onSendCommand={mockSendCommand} />);
    
    fireEvent.click(screen.getByRole('button', { name: /暂停/i }));
    
    expect(mockSendCommand).toHaveBeenCalledWith(
      expect.objectContaining({
        correlationId: expect.any(String),
      })
    );
  });
});
```

---

## 5. 验收清单

### 5.1 代码实现检查点

| 检查项 | 负责人 | 验收标准 |
|--------|--------|----------|
| 后端日志包含必填字段 | Dev Agent | 所有 T-A2-BE-* 测试通过 |
| WS 事件携带 correlationId | Dev Agent | T-A2-BE-05 测试通过 |
| 前端命令携带 correlationId | Dev Agent | T-A2-FE-01 测试通过 |
| 集成测试通过 | QA | 所有 T-A2-INT-* 测试通过 |

### 5.2 Story 6.1 DoD 检查点

在 Story 6.1 完成时，需验证以下可追踪性要求：

- [ ] 所有 pause/resume/stop 操作记录 INFO 日志
- [ ] 日志包含 correlation_id, user_id, task_id, action, prev_state, new_state, iteration_state
- [ ] WS 事件携带与请求相同的 correlationId
- [ ] 非法操作记录 WARN 日志
- [ ] 测试用例 T-A2-BE-01 ~ T-A2-INT-03 全部通过

---

## 6. 与 A1 设计的关联

本文档的日志格式规范与 `@/docs/implementation-artifacts/epic-6-run-control-state-design.md` 第 7 节保持一致：

- 日志字段定义相同
- correlationId 规则相同
- 错误处理优先级相同

---

## 7. 评审确认

### 7.1 评审清单

- [x] 日志必填字段定义完整
- [x] 日志级别规范明确
- [x] correlationId 生命周期定义
- [x] 测试用例覆盖后端/前端/集成
- [x] 验收清单可执行
- [x] 与 A1 设计保持一致

### 7.2 评审结论

**通过**：本规范满足 A2 门槛的成功标准，可作为 Story 6.1 可追踪性验证的权威依据。

---

## 附录：相关文档

- A1 状态设计：`docs/implementation-artifacts/epic-6-run-control-state-design.md`
- 架构 correlationId 规范：`docs/project-planning-artifacts/architecture.md#Communication Patterns`
- 契约权威：`docs/developer-guides/contracts.md`
- Story 6.1：`docs/implementation-artifacts/6-1-pause-and-resume-iteration.md`
