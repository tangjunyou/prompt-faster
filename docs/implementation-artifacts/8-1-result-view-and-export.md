# Story 8.1: 结果查看与导出

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 8-1-result-view-and-export

## Epic 8 概述

> **Epic 8: 结果输出与元优化** - 用户成果：用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**Epic 8 Story 列表**：
- **8.1 结果查看与导出（本 Story，FR60, FR61，NFR18）** - MVP
- 8.2 诊断报告（FR63）- MVP
- 8.3 元优化基础（FR56, FR57, FR58）- MVP
- 8.4 高级用户直接编辑老师模型 Prompt（FR59）- MVP
- 8.5 Prompt 版本对比（FR62）- Growth
- 8.6 创意任务多样性检测（FR34）- Growth

## Key Decisions (MVP)

- **结果展示范围**：展示最终优化结果 Prompt，包含任务元信息、最佳 Prompt、通过率、迭代轮次摘要。
- **导出格式**：支持 Markdown / JSON / XML 三种格式导出。
- **复制功能**：一键复制 Prompt 内容到剪贴板，显示"已复制"反馈。
- **界面语言**：所有界面文案使用中文（NFR18）。
- **数据来源**：复用现有 `optimization_tasks` 和 `iterations` 数据，无需新建数据表。
- **权限校验**：仅任务所有者可查看和导出结果。
- **AR2 遵循**：所有操作需记录 correlationId，支持全链路追踪。

## Story

As a Prompt 优化用户,
I want 查看最终优化结果 Prompt 并导出,
so that 我可以使用优化后的 Prompt 并分享给他人。

## Acceptance Criteria

1. **Given** 优化任务完成
   **When** 用户查看结果页面
   **Then** 显示最终优化结果 Prompt
   **And** 界面文案使用中文（NFR18）

2. **Given** 用户想要复制 Prompt
   **When** 点击"复制"按钮
   **Then** Prompt 内容复制到剪贴板
   **And** 显示"已复制"提示

3. **Given** 用户想要导出 Prompt
   **When** 点击"导出"按钮
   **Then** 显示格式选择：Markdown / JSON / XML
   **And** 用户选择后下载对应格式的文件

## Tasks / Subtasks

- 文件落点以 **File Structure Requirements** 为准；本节只描述职责，避免重复写路径。

- [x] 后端：结果数据模型定义（AC: 1）
  - [x] 创建 `evaluation_result.rs`（路径见 File Structure Requirements）
    - `TaskResultView` 结构体（task_id, task_name, status, best_prompt, pass_rate, total_iterations, completed_at, created_at）
    - `ResultExportFormat` 枚举（markdown, json, xml）
    - `ExportResultResponse` 结构体（格式化的导出内容）
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增类型

- [x] 后端：Results API 实现（AC: 1,2,3）
  - [x] 创建 `results.rs`（路径见 File Structure Requirements）
    - `GET /api/v1/tasks/{task_id}/result` 获取优化结果
    - `GET /api/v1/tasks/{task_id}/result/export?format={format}` 导出结果（返回 ExportResultResponse JSON）
  - [x] 复用 `OptimizationTaskRepo::find_by_id_for_user` 与 `IterationRepo::list_with_artifacts_by_task_id`
  - [x] 添加权限校验（仅任务所有者可访问）
  - [x] 添加 OpenAPI 文档描述
  - [x] 在 `mod.rs` 注册新路由
  - [x] 在 `backend/src/main.rs` 挂载路由到 `/api/v1/tasks/{task_id}/result`
  - [x] 在 `backend/src/api/routes/docs.rs` 注册 OpenAPI path/schema

- [x] 后端：导出格式化器（AC: 3）
  - [x] 创建 `result_formatter/` 模块（路径见 File Structure Requirements）
    - `format_as_markdown(result: &TaskResultView) -> String`
    - `format_as_json(result: &TaskResultView) -> String`
    - `format_as_xml(result: &TaskResultView) -> String`
  - [x] 确保导出内容包含：任务名称、最佳 Prompt、通过率、迭代摘要、导出时间戳

- [x] 前端：result-viewer feature 目录结构（AC: 1-3）
  - [x] 创建 `result-viewer/` 目录（路径见 File Structure Requirements）
  - [x] 创建 `index.ts` 导出模块
  - [x] 创建 `components/`、`hooks/`、`services/` 子目录

- [x] 前端：结果展示组件（AC: 1）
  - [x] 创建 `ResultView.tsx`（路径见 File Structure Requirements）
    - 显示任务名称、状态、完成时间
    - 显示最佳 Prompt（语法高亮，可滚动）
    - 显示通过率和迭代轮次摘要
    - 界面文案使用中文

- [x] 前端：复制功能（AC: 2）
  - [x] 在 `ResultView.tsx` 添加"复制"按钮
  - [x] 使用 `navigator.clipboard.writeText()` 复制内容
  - [x] 复制成功后显示"已复制"提示（可复用 StreamingText 的反馈样式）
  - [x] 复制失败显示错误提示

- [x] 前端：导出功能组件（AC: 3）
  - [x] 创建 `ExportDialog.tsx`（路径见 File Structure Requirements）
    - 显示格式选择：Markdown / JSON / XML
    - 每种格式显示简要说明
    - 确认后触发下载
  - [x] 文件名格式：`{task_name}_prompt_{timestamp}.{ext}`

- [x] 前端：服务层封装（AC: 1-3）
  - [x] 创建 `resultService.ts`（路径见 File Structure Requirements）
    - `getResult(taskId: string): Promise<TaskResultView>`
    - `exportResult(taskId: string, format: ResultExportFormat): Promise<{ blob: Blob; filename: string }>`
    - `exportResult` 基于 `ExportResultResponse.content/filename` 生成下载内容
  - [x] 创建 `hooks/useResult.ts` TanStack Query hook
  - [x] 创建 `hooks/useExportResult.ts` mutation hook

- [x] 前端：集成与入口（AC: 1-3）
  - [x] 在 `frontend/src/pages/RunView/` 添加结果查看入口（主导出入口）
  - [x] 任务完成后自动显示或提供"查看结果"按钮
  - [x] 集成到现有 UI 流程

- [x] 测试与回归（AC: 1-3）
  - [x] 按 **Testing Requirements** 表执行（覆盖边界/权限/导出/回归/类型生成）

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（如 RunControlState / IterationState）- N/A，本 Story 为只读操作

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][CRITICAL] 补齐后端导出测试：Markdown/JSON/XML 字段完整性、XML 可解析、导出权限/参数错误等，覆盖 Testing Requirements（含导出与权限） [backend/src/core/result_formatter/mod.rs:196]
- [x] [AI-Review][HIGH] 修正 total_iterations 计算为实际迭代总数（非最大 round），并补测试 [backend/src/api/routes/results.rs:216]
- [x] [AI-Review][MEDIUM] 处理导出 format 非法值的错误路径，确保返回 ApiResponse 且与 OpenAPI 保持一致（format 必填/校验） [backend/src/api/routes/results.rs:47]
- [x] [AI-Review][MEDIUM] 补充 Dev Agent Record File List：validation-report-2026-01-20-*.md（5 个） [docs/implementation-artifacts/8-1-result-view-and-export.md:460]

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Epic 1-7 已完成）**：
  - 优化任务数据模型已实现（`backend/src/domain/models/optimization_task.rs`）
  - 迭代历史 DTO 与状态模型已实现（`backend/src/domain/types/iteration_history.rs` / `backend/src/domain/models/algorithm.rs`）
  - 任务 API 已实现（`backend/src/api/routes/optimization_tasks.rs`）
  - 历史 API 已实现（`backend/src/api/routes/history.rs`）
  - TanStack Query 模式已建立
  - 复制反馈样式可参考 `StreamingText` 的已复制提示

- **业务价值（为什么做）**：用户完成优化后需要方便地查看和获取最终 Prompt，支持多种格式导出便于在不同场景使用和分享。

- **依赖关系**：
  - 依赖现有 `optimization_tasks` 表和 `iterations` 表
  - 复用现有认证和权限校验机制
  - 复用现有复制反馈样式（`StreamingText`）

- **范围边界（必须遵守）**：
  - 本 Story 实现：结果查看、复制、三种格式导出
  - 不包含：诊断报告（8.2）、版本对比（8.5）、历史版本管理
  - 不新增数据库表，从现有数据聚合结果

### 与其他 Story 的关系

| 功能 | Story 8.1（本 Story） | Story 8.2 | Story 8.3 |
| --- | --- | --- | --- |
| 结果查看 | ✅ 新增 | 复用 | 复用 |
| 复制导出 | ✅ 新增 | - | - |
| 诊断报告 | - | 新增 | - |
| 元优化 | - | - | 新增 |

### Database Schema (无新增)

> 本 Story 不新增数据库表，从现有表聚合数据：

```sql
-- 数据来源：
-- optimization_tasks: id, workspace_id, name, status, final_prompt, terminated_at, selected_iteration_id, created_at, updated_at
-- iterations: id, task_id, round, started_at, completed_at, status, pass_rate, artifacts, created_at
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/evaluation_result.rs（新增）
/// 注意：仅用于 API 响应 DTO，避免与 reflection.rs 中的 OptimizationResult 冲突

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum ResultExportFormat {
    Markdown,
    Json,
    Xml,
}

/// 结果查看 DTO
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TaskResultView {
    pub task_id: String,
    pub task_name: String,
    pub status: String,               // "completed" | "terminated" | "running" | "paused" | "draft"
    pub best_prompt: Option<String>,  // 最佳 Prompt 内容
    pub pass_rate: Option<f64>,       // 最终通过率（0.0-1.0）
    pub total_iterations: u32,        // 总迭代轮次
    pub completed_at: Option<String>, // ISO 8601，完成时间
    pub created_at: String,           // ISO 8601，任务创建时间
    pub iteration_summary: Vec<IterationSummaryEntry>, // 迭代摘要
}

/// 迭代摘要条目
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct IterationSummaryEntry {
    pub round: u32,
    pub pass_rate: Option<f64>,
    pub status: String,
}

/// 导出结果响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct ExportResultResponse {
    pub content: String,   // 已格式化内容（Markdown/JSON/XML）
    pub format: ResultExportFormat,
    pub filename: String,
}
```

### Result Selection Rules & Status Handling（必须遵守）

**结果可查看条件：**
- `Completed` / `Terminated`：允许查看完整结果
- `Running` / `Paused`：可返回当前摘要（如已存在 completed 迭代），否则返回友好提示
- `Draft`：返回 404 或“任务未开始”

**best_prompt 优先级：**
1. `optimization_tasks.selected_iteration_id` 对应迭代的 `artifacts.candidate_prompts` 中 `is_best=true` 的 content
2. `optimization_tasks.final_prompt`
3. 从 **completed** 迭代中按 `pass_rate desc, round desc` 选最优候选 Prompt
4. 无可用候选则返回 `None` + 前端提示

**completed_at 来源：**
- `Completed`：取最后一条 completed 迭代的 `completed_at`
- `Terminated`：取 `optimization_tasks.terminated_at`
- 其他状态：`None`

**iteration_summary 规则：**
- 仅包含 completed 迭代
- 默认按 `round desc`，最多 100 条（防止超大任务 payload）

### Suggested API Endpoints

```
# 获取优化结果
GET /api/v1/tasks/{task_id}/result
Response: ApiResponse<TaskResultView>
权限校验：仅任务所有者可访问

# 导出优化结果
GET /api/v1/tasks/{task_id}/result/export?format={markdown|json|xml}
Response: ApiResponse<ExportResultResponse>（JSON，前端生成文件下载）
权限校验：仅任务所有者可操作
```

### Frontend Query Notes（TanStack Query）

- `queryKey` 建议：`['taskResult', taskId]`
- `enabled`：需登录且 taskId 存在
- `staleTime`：
  - Running/Paused：短缓存（如 5-10s）
  - Completed/Terminated：可长缓存（如 5min）或手动刷新
- 错误处理：复用 `isApiError` 分支，抛出 `response.error.message`

### Export Format Examples

> 导出 API 返回 `ExportResultResponse.content`，其内容格式要求如下：

- **Markdown**：标题 + 任务信息列表 + `best_prompt` fenced code block + iteration summary 表 + 导出时间戳
- **JSON**：`taskName` / `bestPrompt` / `passRate` / `totalIterations` / `completedAt` / `iterationSummary`
- **XML**：根节点 `optimizationResult`；`bestPrompt` 使用 `<![CDATA[...]]>` 包裹

### Dev Agent Guardrails（避免常见踩坑）

- **复制兼容性**：`navigator.clipboard.writeText()` 需 HTTPS 或 localhost，失败需降级提示
- **文件名安全化**：任务名必须 sanitize，避免非法字符导致下载失败
- **导出转义**：XML 用 CDATA，Markdown 遇 ``` 改用 ````，JSON 必须用 `serde_json`
- **空/未完成结果**：无可用候选 prompt 时需显示友好提示

### Technical Requirements（必须满足）

- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- 导出 API 返回 `ApiResponse<ExportResultResponse>`（JSON），前端用 `content` 生成文件下载
- 导出文件名需 sanitize，避免非法字符
- iteration_summary 默认最多 100 条，按 round desc 排序
- status 仅允许 `draft/running/paused/completed/terminated`（与 OptimizationTaskStatus 对齐）
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`

### Backward Compatibility / Non-Regressions（必须遵守）

- `/api/v1/tasks/{task_id}/result` 为新增端点，不得修改现有 `/history` 或 `/tasks` 响应字段/语义
- 不更改 `optimization_tasks` / `iterations` schema；旧数据缺字段时需容错（`final_prompt`/`selected_iteration_id`/`candidate_prompts` 为空则回退）
- 新增 DTO 字段保持可选；错误码沿用现有 `error_codes`（避免破坏前端分支）

### Deployment / Environment Notes（补充）

- 路由挂载应置于需要鉴权的 task scope（与 `/api/v1/tasks/{task_id}` 同级受保护路由一致）
- 前端 API 基础路径保持 `/api/v1`（与现有 `API_BASE_URL` 约定一致）
- 导出为 JSON 响应，不依赖浏览器直接下载 Content-Disposition

### Security / Performance Notes（补充）

- 权限校验必须通过 `OptimizationTaskRepo::find_by_id_for_user` 等同现有逻辑
- 导出内容仅包含必要字段，避免拼接完整迭代详情（使用 summary + best_prompt）
- iteration_summary 有上限（100）以控制 payload，避免大任务导致响应过大

### UX Alignment Notes（补充）

- 结果查看与导出入口在 Run View 为主（完成后可见），历史导出入口在 Workspace View（保持与 UX 设计一致）
- 本 Story 仅提供“结果查看/导出/复制”；“为什么更好”的诊断说明留给 Story 8.2

### Integration Boundaries（必须遵守）

- 前端入口固定在 `frontend/src/pages/RunView/RunView.tsx`：任务完成后显示“查看结果/导出”入口（可用按钮或弹窗，不新增页面路由）
- Workspace 侧保持历史入口不变（8.1 不新增 Workspace 入口，避免与 History 入口重复）
- 不引入外部服务；仅使用现有 `/api/v1/tasks/{task_id}/result` 与 `.../export` 端点

### Previous Story Learnings (Story 7.4 复盘/模式/测试)

- **后端路由模式**：`history.rs` 采用 `CurrentUser` + `OptimizationTaskRepo::find_by_id_for_user` 权限校验，并统一提取 correlationId + A2 日志字段
- **查询与分页**：`history.rs` 与 `history_event_repo.rs` 均限制 `limit`/`offset`（limit 1-100，offset ≤ 10000），并按时间倒序分页，避免全量加载
- **Repo 拼接方式**：使用 `QueryBuilder` 组合过滤条件（避免手写拼接/注入风险），统一 `ORDER BY created_at DESC`
- **前端结构模式**：feature 目录采用 `components/` + `hooks/` + `services/` + `index.ts`，测试同目录（`HistoryPanel.test.tsx`/`TimelineView.test.tsx`）
- **测试实践**：使用 MSW + `QueryClientProvider`，通过 `useAuthStore` 注入登录态，Monaco 使用 mock；覆盖空态/列表/导出/权限错误流程
- **Review 结论承接**：7.4 强调“单一入口 + 分页/上限”，导出与时间线都禁止全量加载；8.1 同样必须遵循这一边界

### Latest Technical Notes（基于当前项目版本）

- React 19：升级指南要求启用新的 JSX Transform，渲染错误不再自动重新抛出（改用 root 的 error 回调），函数组件的 `propTypes`/`defaultProps` 已移除（仅类组件仍支持 `defaultProps`），并建议先升级到 React 18.3 暴露弃用警告
- TanStack Query v5：仅支持对象签名；`useQuery` 的 `onSuccess/onError/onSettled` 回调已移除（改用 effect 或 mutation）；`cacheTime` 改名 `gcTime`；`keepPreviousData` 被 `placeholderData` identity 替代
- Axum 0.8：路由路径参数语法改为 `/{param}` 或 `/{*param}`；`Option<T>` extractor 需要实现 `OptionalFromRequestParts/OptionalFromRequest`；自定义 extractor 需移除 `#[async_trait]`
- SQLx 0.8：RUSTSEC-2024-0363 修复在 `>=0.8.1`；建议升级并限制请求体大小、校验输入（即便 SQLite 影响较小）
- 继续使用 `serde_json` 生成 JSON 导出内容，避免手写字符串拼接

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/evaluation_result.rs`：结果响应 DTO（新增）
  - `backend/src/api/routes/results.rs`：结果 API 路由（新增）
  - `backend/src/core/result_formatter/`：导出格式化器（新增）
  - `frontend/src/features/result-viewer/`：结果查看功能模块（新增）
- **路由挂载**：`results.rs` 仍挂在 `/api/v1/tasks/{task_id}/result`（保持与现有 task scope 一致）
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- tokio：异步运行时
- chrono：时间戳处理
- React：`react@19.x`
- TanStack Query：服务端状态管理
- shadcn/ui：UI 组件库

### File Structure Requirements（落点约束）

**后端**：
- 结果响应 DTO：`backend/src/domain/models/evaluation_result.rs`（新增）
- 结果 API：`backend/src/api/routes/results.rs`（新增）
- 格式化器：`backend/src/core/result_formatter/mod.rs`（新增）
- 路由注册：`backend/src/api/routes/mod.rs`（更新）
- 路由挂载：`backend/src/main.rs`（更新，挂载到 `/api/v1/tasks/{task_id}/result`）
- OpenAPI 注册：`backend/src/api/routes/docs.rs`（更新，新增 results 路由）
- 类型生成：`backend/src/bin/gen-types.rs`（更新）

**前端**：
- 结果查看组件：`frontend/src/features/result-viewer/components/ResultView.tsx`（新增）
- 导出对话框：`frontend/src/features/result-viewer/components/ExportDialog.tsx`（新增）
- 服务层：`frontend/src/features/result-viewer/services/resultService.ts`（新增）
- Result Hook：`frontend/src/features/result-viewer/hooks/useResult.ts`（新增）
- Export Hook：`frontend/src/features/result-viewer/hooks/useExportResult.ts`（新增）
- 模块入口：`frontend/src/features/result-viewer/index.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 结果获取 | 任务存在返回结果；任务不存在返回 404 |
| 后端单测 | 未完成任务 | 返回当前状态和已有数据 |
| 后端单测 | 导出 Markdown | 格式正确，包含所有字段 |
| 后端单测 | 导出 JSON | 格式正确，可解析 |
| 后端单测 | 导出 XML | 格式正确，可解析 |
| 后端单测 | 权限校验 | 非任务所有者返回 403 |
| 后端单测 | best_prompt 兜底 | selected_iteration_id / final_prompt / completed 迭代排序 |
| 后端单测 | 导出转义 | XML/Markdown 特殊字符不破坏格式 |
| 前端测试 | 结果展示 | 正确渲染任务信息和 Prompt |
| 前端测试 | 复制功能 | 复制成功显示“已复制”提示 |
| 前端测试 | 导出对话框 | 格式选择和下载触发 |
| 前端测试 | 无完成迭代 | 显示友好提示而非空白 |
| 前端测试 | 文件名安全化 | 任务名含非法字符仍可下载 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 参考 `backend/src/api/routes/history.rs` 的路由实现模式
- 参考 `frontend/src/features/user-intervention/history/` 的模块结构
- 复用 `backend/src/infra/db/repositories/optimization_task_repo.rs` 获取任务数据
- 复用 `backend/src/infra/db/repositories/iteration_repo.rs` 获取迭代数据
- 遵循 Repository 模式访问数据库

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 8 / Story 8.1）
- PRD 结果输出：`docs/project-planning-artifacts/prd.md#能力区域 10: 结果输出与分析`
- 架构（结果输出）：`docs/project-planning-artifacts/architecture.md#10. 结果输出与分析`
- 任务 API 实现：`backend/src/api/routes/optimization_tasks.rs`
- 历史 API 实现：`backend/src/api/routes/history.rs`
- 迭代 DTO：`backend/src/domain/types/iteration_history.rs`
- 候选 Prompt 结构：`backend/src/domain/types/artifacts.rs`
- Story 7.4（前序）：`docs/implementation-artifacts/7-4-complete-iteration-history-record.md`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- Implementation Plan: 后端新增结果 DTO + 结果/导出 API 与格式化器；前端新增 result-viewer feature + RunView 入口；补齐导出/复制与测试回归。
- Tests: `cargo run --bin gen-types`, `cargo test`, `pnpm vitest run`, `pnpm lint`, `pnpm vite build`.

### Completion Notes List

- ✅ 新增结果查看与导出后端 API（含权限校验、状态/优先级规则、OpenAPI/TS 类型生成）并实现 Markdown/JSON/XML 格式化器。
- ✅ 前端新增 result-viewer 组件体系，提供最佳 Prompt 展示、复制反馈与导出对话框，并集成 RunView 入口。
- ✅ 补齐前后端测试（结果获取/权限/优先级/格式化、导出对话框、复制/空态、文件名安全化）并通过全量回归与构建。

### File List

- backend/src/api/routes/results.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/bin/gen-types.rs
- backend/src/infra/db/repositories/iteration_repo.rs
- backend/src/core/mod.rs
- backend/src/core/result_formatter/mod.rs
- backend/src/domain/models/evaluation_result.rs
- backend/src/domain/models/mod.rs
- backend/src/main.rs
- backend/Cargo.toml
- backend/Cargo.lock
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/8-1-result-view-and-export.md
- docs/implementation-artifacts/validation-report-2026-01-20-111926.md
- docs/implementation-artifacts/validation-report-2026-01-20-113254.md
- docs/implementation-artifacts/validation-report-2026-01-20-114553.md
- docs/implementation-artifacts/validation-report-2026-01-20-115738.md
- docs/implementation-artifacts/validation-report-2026-01-20-121150.md
- frontend/src/features/result-viewer/index.ts
- frontend/src/features/result-viewer/components/ExportDialog.tsx
- frontend/src/features/result-viewer/components/ExportDialog.test.tsx
- frontend/src/features/result-viewer/components/ResultView.tsx
- frontend/src/features/result-viewer/components/ResultView.test.tsx
- frontend/src/features/result-viewer/hooks/useExportResult.ts
- frontend/src/features/result-viewer/hooks/useResult.ts
- frontend/src/features/result-viewer/services/resultService.ts
- frontend/src/features/result-viewer/services/resultService.test.ts
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/types/generated/models/ExportResultResponse.ts
- frontend/src/types/generated/models/IterationSummaryEntry.ts
- frontend/src/types/generated/models/ResultExportFormat.ts
- frontend/src/types/generated/models/TaskResultView.ts

## Change Log

- 2026-01-20：实现结果查看与导出 API（含格式化器与类型生成）与前端 result-viewer 模块，集成 RunView 入口并补齐测试/回归验证。
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [CRITICAL] 后端导出测试未覆盖 Testing Requirements（Markdown/JSON/XML 字段完整性、XML 可解析、导出参数/权限等）→ 已补齐并通过 cargo test。
- [x] [HIGH] total_iterations 使用最大 round 代替实际迭代数量 → 已改为真实 count 并补测试。
- [x] [MEDIUM] 导出 format 非法值返回非 ApiResponse → 已改为手动校验并返回 ApiResponse。
- [x] [MEDIUM] Dev Agent Record File List 遗漏 validation-report 文件 → 已补齐。

### Decisions

- 本次 review 已直接修复并补齐测试，回归通过后将状态恢复为 review。

### Risks / Tech Debt

- 暂无新增遗留。

### Follow-ups

- [x] 补齐后端导出测试覆盖导出格式字段、XML 可解析、参数/权限错误路径。
- [x] 修正 total_iterations 统计逻辑为真实迭代数量并补测试。
- [x] 统一导出 format 校验与 OpenAPI 描述，确保错误响应使用 ApiResponse。
- [x] 补充 validation-report 文件的 File List 记录。
