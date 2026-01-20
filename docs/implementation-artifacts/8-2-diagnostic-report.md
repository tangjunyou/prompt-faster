# Story 8.2: 诊断报告

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 8-2-diagnostic-report

## Epic 8 概述

> **Epic 8: 结果输出与元优化** - 用户成果：用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**Epic 8 Story 列表**：
- 8.1 结果查看与导出（FR60, FR61，NFR18）- ✅ done
- **8.2 诊断报告（本 Story，FR63）** - MVP
- 8.3 元优化基础（FR56, FR57, FR58）- MVP
- 8.4 高级用户直接编辑老师模型 Prompt（FR59）- MVP
- 8.5 Prompt 版本对比（FR62）- Growth
- 8.6 创意任务多样性检测（FR34）- Growth

## Key Decisions (MVP)

- **诊断报告范围**：展示失败原因摘要、关键转折点、改进建议、失败用例列表。
- **自然语言解释**：基于失败模式匹配生成，不调用 LLM（保持离线可用 NFR26）。
- **差异高亮**：使用 Monaco DiffEditor 展示输入/输出/期望对比。
- **数据来源**：从 `iterations.evaluation_results`（JSON）与 `iterations.artifacts` 聚合；`failure_archive` 来自 OptimizationContext.extensions（`layer4.failure_archive`）如可用，无需新建数据表。
- **权限校验**：仅任务所有者可查看诊断报告。
- **分页策略**：失败用例列表默认最多 50 条，防止 payload 过大。
- **AR2 遵循**：所有操作需记录 correlationId，支持全链路追踪。

## Story

As a Prompt 优化用户,
I want 查看诊断报告了解"为什么之前不行",
so that 我可以理解优化过程中的问题并学习如何写更好的 Prompt。

## Acceptance Criteria

1. **Given** 优化任务完成或中途失败
   **When** 用户点击"诊断报告"
   **Then** 显示优化过程的诊断分析
   **And** 包含：失败原因摘要、关键转折点、改进建议

2. **Given** 诊断报告显示
   **When** 用户查看失败原因
   **Then** 以自然语言解释"为什么之前的 Prompt 不行"
   **And** 提供具体的失败用例示例

3. **Given** 用户想要深入分析
   **When** 点击某个失败用例
   **Then** 展开显示详细的输入/输出/期望对比
   **And** 高亮差异部分

## Tasks / Subtasks

- 文件落点以 **File Structure Requirements** 为准；本节只描述职责，避免重复写路径。

- [x] 后端：诊断报告数据模型定义（AC: 1,2）
  - [x] 创建 `diagnostic.rs`（路径见 File Structure Requirements）
    - `DiagnosticReport` 结构体（task_id, task_name, status, summary, turning_points, improvement_suggestions, failed_cases）
    - `DiagnosticSummary` 结构体（total_iterations, failed_iterations, common_failure_reasons, natural_language_explanation）
    - `TurningPoint` 结构体（round, event_type, description, pass_rate_before, pass_rate_after）
    - `FailedCaseSummary` 结构体（case_id, input_preview, failure_reason, iteration_round, test_case_id）
    - `FailedCaseDetail` 结构体（case_id, test_case_id, input, expected_output, actual_output, failure_reason, diff_segments）
    - `DiffSegment` 结构体（segment_type, content, start_index, end_index）
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增类型

- [x] 后端：诊断报告生成服务（AC: 1,2）
  - [x] 创建 `diagnostic_service/mod.rs`（路径见 File Structure Requirements）
    - `generate_diagnostic_report(task_id, user_id) -> DiagnosticReport`
    - `analyze_failure_patterns(iterations) -> Vec<FailureReasonEntry>`
    - `detect_turning_points(iterations) -> Vec<TurningPoint>`
    - `generate_improvement_suggestions(failure_patterns) -> Vec<String>`
    - `generate_natural_language_explanation(failure_patterns) -> String`
  - [x] 新增/扩展 IterationRepo 方法以同时返回 `evaluation_results`（不要仅依赖 `list_with_artifacts_by_task_id`）
  - [x] `failure_archive` 如需使用，读取 `OptimizationContext.extensions[EXT_FAILURE_ARCHIVE]`（不存在则跳过）

- [x] 后端：诊断报告 API 实现（AC: 1,2,3）
  - [x] 创建 `diagnostic.rs`（路径见 File Structure Requirements）
    - `GET /api/v1/tasks/{task_id}/diagnostic` 获取诊断报告
    - `GET /api/v1/tasks/{task_id}/diagnostic/cases/{case_id}` 获取失败用例详情
  - [x] 复用 `OptimizationTaskRepo::find_by_id_for_user` 权限校验
  - [x] 添加 OpenAPI 文档描述
  - [x] 在 `mod.rs` 注册新路由
  - [x] 在 `backend/src/main.rs` 挂载路由到 `/api/v1/tasks/{task_id}/diagnostic`
  - [x] 在 `backend/src/api/routes/docs.rs` 注册 OpenAPI path/schema

- [x] 前端：diagnostic-report feature 目录结构（AC: 1-3）
  - [x] 创建 `diagnostic-report/` 目录（路径见 File Structure Requirements）
  - [x] 创建 `index.ts` 导出模块
  - [x] 创建 `components/`、`hooks/`、`services/` 子目录

- [x] 前端：诊断报告主组件（AC: 1,2）
  - [x] 创建 `DiagnosticReport.tsx`（路径见 File Structure Requirements）
    - 显示失败原因摘要卡片
    - 显示自然语言解释区域
    - 集成 TurningPointTimeline 和 FailedCaseList

- [x] 前端：关键转折点时间线（AC: 1）
  - [x] 创建 `TurningPointTimeline.tsx`（路径见 File Structure Requirements）
    - 时间线形式展示转折点
    - 每个节点显示 round、event_type、pass_rate 变化
    - 使用颜色区分 improvement/regression/breakthrough

- [x] 前端：失败用例列表（AC: 2）
  - [x] 创建 `FailedCaseList.tsx`（路径见 File Structure Requirements）
    - 列表展示失败用例摘要
    - 支持点击展开详情
    - 显示 input_preview 和 failure_reason

- [x] 前端：用例对比对话框（AC: 3）
  - [x] 创建 `CaseComparisonDialog.tsx`（路径见 File Structure Requirements）
    - 使用 Monaco DiffEditor 展示差异
    - 高亮输入/输出/期望的不同部分
    - 支持关闭和导航

- [x] 前端：服务层封装（AC: 1-3）
  - [x] 创建 `diagnosticService.ts`（路径见 File Structure Requirements）
    - `getDiagnosticReport(taskId: string): Promise<DiagnosticReport>`
    - `getFailedCaseDetail(taskId: string, caseId: string): Promise<FailedCaseDetail>`
  - [x] 创建 `hooks/useDiagnostic.ts` TanStack Query hook
  - [x] 创建 `hooks/useFailedCaseDetail.ts` query hook

- [x] 前端：集成与入口（AC: 1-3）
  - [x] 在 `frontend/src/pages/RunView/` 添加诊断报告入口
  - [x] 与 Story 8.1 的结果查看入口并排或 Tab 切换
  - [x] 任务完成/失败后显示"查看诊断报告"按钮

- [x] 测试与回归（AC: 1-3）
  - [x] 按 **Testing Requirements** 表执行
  - [x] 新增/覆盖测试文件
    - `backend/src/core/diagnostic_service/mod.rs`（单元测试）
    - `frontend/src/features/diagnostic-report/components/DiagnosticReport.test.tsx`
    - `frontend/src/features/diagnostic-report/components/TurningPointTimeline.test.tsx`

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/task_id/action/prev_state/new_state/iteration_state/timestamp）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（如 RunControlState / IterationState）- N/A，本 Story 为只读操作

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][HIGH] 补齐部署/环境说明与版本最佳实践（详见 Latest Technical Notes & Deployment Notes）
- [x] [AI-Review][MEDIUM] 明确 UX 对齐（Run View 使用 Tab 区分“结果查看/诊断报告”，并在任务未完成时提示不可用）
- [x] [AI-Review][HIGH] 诊断报告已接入 `layer4.failure_archive` 数据源，保证失败原因/用例统计完整（backend/src/core/diagnostic_service/mod.rs）
- [x] [AI-Review][MEDIUM] `build_diff_segments` 已修正 start/end 索引，插入/删除语义一致（backend/src/core/diagnostic_service/mod.rs）
- [x] [AI-Review][MEDIUM] 诊断报告 API 已限制仅完成/失败任务可访问（backend/src/api/routes/diagnostic.rs, backend/src/core/diagnostic_service/mod.rs）
- [x] [AI-Review][LOW] failed case input_preview 截断严格不超过 100 字符（backend/src/core/diagnostic_service/mod.rs）
- [x] [AI-Review][MEDIUM] Story File List 已补齐实际变更文件（含 8.1 & validation reports）

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Epic 1-7 + Story 8.1 已完成）**：
  - 优化任务数据模型已实现（`backend/src/domain/models/optimization_task.rs`）
  - 迭代历史 DTO 与状态模型已实现（`backend/src/domain/types/iteration_history.rs` / `backend/src/domain/models/algorithm.rs`）
  - 结果查看 API 已实现（`backend/src/api/routes/results.rs`）
  - TanStack Query 模式已建立
  - Monaco DiffEditor 已在项目中可用

- **业务价值（为什么做）**：用户完成优化后需要理解"为什么之前的 Prompt 不行"，诊断报告提供失败分析和改进建议，帮助用户学习如何写更好的 Prompt。

- **依赖关系**：
  - 依赖现有 `optimization_tasks` 表和 `iterations` 表
  - 依赖 `iterations.evaluation_results`（JSON: EvaluationResultSummary[]）
  - 复用现有认证和权限校验机制
  - 复用 Story 8.1 的模块结构模式

- **范围边界（必须遵守）**：
  - 本 Story 实现：诊断报告查看、失败用例对比
  - 不包含：结果导出（8.1）、版本对比（8.5）、元优化（8.3）
  - 不新增数据库表，从现有数据聚合结果
  - 自然语言解释基于模式匹配，不调用 LLM

### 与其他 Story 的关系

| 功能 | Story 8.1 | Story 8.2（本 Story） | Story 8.3 |
| --- | --- | --- | --- |
| 结果查看 | ✅ 已实现 | 复用入口 | 复用 |
| 复制导出 | ✅ 已实现 | - | - |
| 诊断报告 | - | ✅ 新增 | - |
| 元优化 | - | - | 新增 |

### Database Schema (无新增)

> 本 Story 不新增数据库表，从现有表聚合数据：

```sql
-- 数据来源：
-- optimization_tasks: id, workspace_id, name, status, final_prompt, terminated_at, created_at
-- iterations: id, task_id, round, started_at, completed_at, status, pass_rate, artifacts, evaluation_results, created_at
-- iterations.artifacts (JSON): patterns, candidate_prompts, user_guidance, updated_at
-- iterations.evaluation_results (JSON): EvaluationResultSummary[]
-- failure_archive: OptimizationContext.extensions["layer4.failure_archive"]（非表字段）
-- test_cases: id, test_set_id, input, expected_output (用于对比)
```

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/diagnostic.rs（新增）

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 诊断报告
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiagnosticReport {
    pub task_id: String,
    pub task_name: String,
    pub status: String,               // "completed" | "terminated" | "running" | "paused"
    pub summary: DiagnosticSummary,
    pub turning_points: Vec<TurningPoint>,
    pub improvement_suggestions: Vec<String>,
    pub failed_cases: Vec<FailedCaseSummary>,
}

/// 诊断摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiagnosticSummary {
    pub total_iterations: u32,
    pub failed_iterations: u32, // status=failed/terminated 的迭代数（与 pass_rate < 1.0 不等价）
    pub success_iterations: u32,
    pub common_failure_reasons: Vec<FailureReasonEntry>,
    pub natural_language_explanation: String, // 自然语言解释"为什么之前不行"
}

/// 失败原因条目
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct FailureReasonEntry {
    pub reason: String,
    pub count: u32,
    pub percentage: f64,
}

/// 关键转折点
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct TurningPoint {
    pub round: u32,
    pub event_type: TurningPointType,  // "improvement" | "regression" | "breakthrough"
    pub description: String,
    pub pass_rate_before: Option<f64>,
    pub pass_rate_after: Option<f64>,
    pub timestamp: String,             // ISO 8601
}

/// 转折点类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum TurningPointType {
    Improvement,  // 通过率提升 >= 10%
    Regression,   // 通过率下降 >= 10%
    Breakthrough, // 达到 100% 通过或首次达到 pass_rate >= 0.5
}

/// 失败用例摘要（列表项）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct FailedCaseSummary {
    /// case_id 生成规则："{iteration_id}:{test_case_id}"
    pub case_id: String,
    pub input_preview: String,        // 截断的输入预览（最多 100 字符）
    pub failure_reason: String,
    pub iteration_round: u32,
    pub test_case_id: Option<String>, // 关联的测试用例 ID
}

/// 失败用例详情（展开对比用）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct FailedCaseDetail {
    pub case_id: String,
    pub test_case_id: Option<String>,
    pub input: String,
    pub expected_output: Option<String>,
    /// 实际输出（若未持久化则为 None，前端显示占位文案）
    pub actual_output: Option<String>,
    pub failure_reason: String,
    pub iteration_round: u32,
    pub prompt_used: Option<String>,  // 当时使用的 Prompt
    pub diff_segments: Vec<DiffSegment>,
}

/// 差异片段（用于高亮）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct DiffSegment {
    pub segment_type: DiffSegmentType, // "added" | "removed" | "unchanged"
    pub content: String,
    pub start_index: u32,
    pub end_index: u32,
}

/// 差异类型
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "models/")]
pub enum DiffSegmentType {
    Added,
    Removed,
    Unchanged,
}
```

### Diagnostic Generation Logic (必须遵守)

**关键转折点检测规则：**
- `Improvement`：当前轮次 pass_rate - 前一轮次 pass_rate >= 0.1（10%）
- `Regression`：当前轮次 pass_rate - 前一轮次 pass_rate <= -0.1（-10%）
- `Breakthrough`：pass_rate 达到 1.0（100%）或首次达到 pass_rate >= 0.5
- 边界规则：第 1 轮没有前置数据则跳过 Improvement/Regression；pass_rate 缺失/为 None 则不参与转折点判断
- 去重规则：连续多轮满足同一类型时只记录首次出现；若 Breakthrough 与 Improvement 同轮满足，优先记 Breakthrough

**失败原因分类规则：**
- 从 `iterations.evaluation_results` 中的 `failure_reason` 提取
- 归一化后聚合相同失败原因（trim/小写/去标点），计算出现次数和百分比
- 按出现次数降序排列，最多显示 10 种

**自然语言解释生成规则：**
- 基于失败模式匹配生成，不调用 LLM
- 模板示例：
  - "主要失败原因是 {top_reason}，占比 {percentage}%。建议关注 {suggestion}。"
  - "在第 {round} 轮出现关键转折，通过率从 {before}% 提升到 {after}%。"

**改进建议生成规则：**
- 基于失败模式映射到预定义建议库
- 每种失败模式对应 1-3 条建议
- 最多返回 5 条建议

**失败模式最小建议库（示例，至少覆盖以下 5 类）：**
- 格式不匹配（format）→ 建议：补充输出格式示例；在 Prompt 中明确格式约束
- 长度不符（length）→ 建议：强调字数/长度要求；提供截断/扩写规则
- 关键字段缺失（missing_field）→ 建议：列出必需字段清单；给出字段顺序
- 语义偏差（semantic_drift）→ 建议：增加任务目标强调；提供正反例
- 输出结构错误（structure）→ 建议：定义结构模板；要求严格遵循结构

### Suggested API Endpoints

```
# 获取诊断报告
GET /api/v1/tasks/{task_id}/diagnostic
Response: ApiResponse<DiagnosticReport>
权限校验：仅任务所有者可访问
Query Params:
  - failed_cases_limit: u32 (default: 50, max: 100)

# 获取失败用例详情
GET /api/v1/tasks/{task_id}/diagnostic/cases/{case_id}
Response: ApiResponse<FailedCaseDetail>
权限校验：仅任务所有者可操作
```

### Frontend Component Notes

**DiagnosticReport.tsx 结构：**
```tsx
// 主容器
<DiagnosticReportContainer>
  {/* 摘要卡片 */}
  <SummaryCard summary={report.summary} />
  
  {/* 自然语言解释 */}
  <ExplanationSection explanation={report.summary.naturalLanguageExplanation} />
  
  {/* 关键转折点时间线 */}
  <TurningPointTimeline turningPoints={report.turningPoints} />
  
  {/* 改进建议 */}
  <ImprovementSuggestions suggestions={report.improvementSuggestions} />
  
  {/* 失败用例列表 */}
  <FailedCaseList 
    cases={report.failedCases} 
    onCaseClick={handleCaseClick} 
  />
</DiagnosticReportContainer>

{/* 用例对比对话框 */}
<CaseComparisonDialog 
  open={dialogOpen}
  caseDetail={selectedCase}
  onClose={handleClose}
/>
```

**CaseComparisonDialog.tsx 使用 Monaco DiffEditor：**
```tsx
import { DiffEditor } from '@monaco-editor/react';

<DiffEditor
  original={caseDetail.expectedOutput}
  modified={caseDetail.actualOutput}
  language="text"
  options={{ readOnly: true, renderSideBySide: true }}
/>
```

> 若 `actualOutput` 为空，显示占位文案并避免初始化 DiffEditor（或使用空字符串 + 只读提示）。

**UX 对齐（Run View 右侧面板）**：
- 诊断报告入口位于 RunView 右侧面板，与结果查看并列
- “失败原因”对应摘要与自然语言解释
- “版本对比”对应用例 Diff 展示

### Dev Agent Guardrails（避免常见踩坑）

- **差异计算**：使用 `similar` 或 `diff` crate 计算文本差异，避免手写算法
- **差异映射**：`similar::ChangeTag::Insert/Delete/Equal` → `DiffSegmentType::Added/Removed/Unchanged`，索引单位使用“字符索引”
- **分页保护**：failed_cases 必须有上限（默认 50，最大 100）
- **空态处理**：无失败用例时显示"恭喜！所有用例都通过了"
- **预览截断**：input_preview 最多 100 字符，超出用 "..." 截断
- **Monaco 兼容**：确保 Monaco DiffEditor 在 React 19 下正常工作
- **日志安全**：诊断日志与 failure_reason 不得包含输入/输出/Prompt 全文，仅记录必要摘要

### Technical Requirements（必须满足）

- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- API 响应使用 `ApiResponse<T>` 统一结构
- failed_cases 默认最多 50 条，按 iteration_round desc 排序
- diff_segments 计算使用 `similar` crate 或等效库
- `evaluation_results` 为空/NULL 时返回空失败列表与空原因统计（不得抛错）
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`
- 自然语言解释基于模式匹配，不调用外部 LLM API

### Backward Compatibility / Non-Regressions（必须遵守）

- `/api/v1/tasks/{task_id}/diagnostic` 为新增端点，不得修改现有 `/result` 或 `/history` 响应字段/语义
- 不更改 `optimization_tasks` / `iterations` schema
- 新增 DTO 字段保持可选；错误码沿用现有 `error_codes`

### Previous Story Learnings (Story 8.1 复盘/模式/测试)

- **后端路由模式**：`results.rs` 采用 `CurrentUser` + `OptimizationTaskRepo::find_by_id_for_user` 权限校验
- **DTO 设计模式**：使用 `#[serde(rename_all = "camelCase")]` + `#[ts(export_to = "models/")]`
- **前端模块结构**：`result-viewer/` 采用 `components/` + `hooks/` + `services/` + `index.ts`
- **测试实践**：使用 MSW + `QueryClientProvider`，通过 `useAuthStore` 注入登录态
- **Review 结论承接**：8.1 强调"单一入口 + 分页/上限"，8.2 同样必须遵循

### Latest Technical Notes（基于当前项目版本）

**Breaking Changes / Best Practices**
- React 19：渲染错误不再自动重新抛出（改用 root 的 error 回调）
- TanStack Query v5：仅支持对象签名；`cacheTime` 改名 `gcTime`
- Axum 0.8：路由路径参数语法改为 `/{param}`
- Monaco Editor React：确保使用与 React 19 兼容的版本
- `similar` crate：Rust 文本差异计算库，推荐用于 diff_segments 生成（后端生成 diff_segments，前端仅渲染）

**Performance / Deprecation Notes**
- 避免全量加载：失败用例详情应按 case_id 精准查询，避免加载全部 test_cases
- 大文本 diff：对比内容过长时优先分页或延迟加载详情

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/diagnostic.rs`：诊断报告 DTO（新增）
  - `backend/src/core/diagnostic_service/mod.rs`：诊断报告生成服务（新增）
  - `backend/src/api/routes/diagnostic.rs`：诊断报告 API 路由（新增）
  - `frontend/src/features/diagnostic-report/`：诊断报告功能模块（新增）
- **路由挂载**：`diagnostic.rs` 挂在 `/api/v1/tasks/{task_id}/diagnostic`
- **响应结构**：遵循 `ApiResponse<T>` 结构，`data` 与 `error` 互斥
- **错误处理**：后端 `thiserror` + `anyhow`
- **命名约定**：TypeScript camelCase，Rust snake_case，跨端 `serde(rename_all = "camelCase")`
- **类型生成**：新增类型后运行 `cd backend && cargo run --bin gen-types`

### Library / Framework Requirements (Version Snapshot)

- Axum：项目依赖 `axum@0.8.x`
- SQLx：项目依赖 `sqlx@0.8.x`
- similar：Rust 文本差异计算（推荐添加）
- tokio：异步运行时
- chrono：时间戳处理
- React：`react@19.x`
- TanStack Query：服务端状态管理
- shadcn/ui：UI 组件库
- @monaco-editor/react：Monaco 差异编辑器

### Deployment / Environment Notes（部署/环境）

- 本 Story 不新增环境变量或配置项，部署流程沿用现有后端/前端构建方式
- 不新增数据库表或迁移；仅使用现有 `iterations`/`optimization_tasks` 数据
- 诊断报告为只读 API，需确保现有认证/鉴权中间件在部署环境正常启用
- **Monaco Editor**：前端使用 `@monaco-editor/react`（当前为 `^4.7.0`，与 React 19 兼容）
- **Backend Dependencies**：新增 `similar = "2"` 用于 diff_segments 生成
- **部署验证**：建议执行 `cargo test`、`pnpm vitest run`、`pnpm vite build`

### File Structure Requirements（落点约束）

**后端**：
- 诊断报告 DTO：`backend/src/domain/models/diagnostic.rs`（新增）
- 诊断服务：`backend/src/core/diagnostic_service/mod.rs`（新增）
- 诊断 API：`backend/src/api/routes/diagnostic.rs`（新增）
- 路由注册：`backend/src/api/routes/mod.rs`（更新）
- 路由挂载：`backend/src/main.rs`（更新，挂载到 `/api/v1/tasks/{task_id}/diagnostic`）
- OpenAPI 注册：`backend/src/api/routes/docs.rs`（更新，新增 diagnostic 路由）
- 类型生成：`backend/src/bin/gen-types.rs`（更新）
- 依赖：`backend/Cargo.toml`（添加 `similar` crate）

**前端**：
- 诊断报告组件：`frontend/src/features/diagnostic-report/components/DiagnosticReport.tsx`（新增）
- 转折点时间线：`frontend/src/features/diagnostic-report/components/TurningPointTimeline.tsx`（新增）
- 失败用例列表：`frontend/src/features/diagnostic-report/components/FailedCaseList.tsx`（新增）
- 用例对比对话框：`frontend/src/features/diagnostic-report/components/CaseComparisonDialog.tsx`（新增）
- 服务层：`frontend/src/features/diagnostic-report/services/diagnosticService.ts`（新增）
- Diagnostic Hook：`frontend/src/features/diagnostic-report/hooks/useDiagnostic.ts`（新增）
- Case Detail Hook：`frontend/src/features/diagnostic-report/hooks/useFailedCaseDetail.ts`（新增）
- 模块入口：`frontend/src/features/diagnostic-report/index.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 诊断报告获取 | 任务存在返回报告；任务不存在返回 404 |
| 后端单测 | 未完成任务 | 返回 409（任务尚未完成） |
| 后端单测 | 无失败用例 | 返回空 failed_cases + 友好提示 |
| 后端单测 | 转折点检测 | 正确检测 improvement/regression/breakthrough |
| 后端单测 | 失败原因聚合 | 正确统计和排序 |
| 后端单测 | evaluation_results 为空 | failure_archive 有值则用于统计与 failed_cases 回退，否则返回空 failed_cases + 空原因统计 |
| 后端单测 | 分页/上限 | 返回最多 50 条 failed_cases |
| 后端单测 | 权限校验 | 非任务所有者返回 403 |
| 后端单测 | 用例详情获取 | 正确返回差异片段 |
| 前端测试 | 诊断报告展示 | 正确渲染摘要、时间线、用例列表 |
| 前端测试 | 转折点时间线 | 正确显示颜色和类型 |
| 前端测试 | 失败用例列表 | 点击展开正确调用详情 API |
| 前端测试 | 对比对话框 | Monaco DiffEditor 正确渲染差异 |
| 前端测试 | 空态处理 | 无失败用例显示成功提示 |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 参考 `backend/src/api/routes/results.rs` 的路由实现模式
- 参考 `frontend/src/features/result-viewer/` 的模块结构
- 复用 `backend/src/infra/db/repositories/optimization_task_repo.rs` 获取任务数据
- 复用 `backend/src/infra/db/repositories/iteration_repo.rs` 获取迭代数据
- 复用 `EvaluationResultSummary`（`backend/src/domain/types/iteration_history.rs`），避免重复定义失败原因结构
- 遵循 Repository 模式访问数据库

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 8 / Story 8.2）
- PRD 结果输出：`docs/project-planning-artifacts/prd.md#能力区域 10: 结果输出与分析`
- 架构（结果输出）：`docs/project-planning-artifacts/architecture.md#10. 结果输出与分析`
- 结果 API 实现：`backend/src/api/routes/results.rs`
- 历史 API 实现：`backend/src/api/routes/history.rs`
- 迭代 DTO：`backend/src/domain/types/iteration_history.rs`
- Story 8.1（前序）：`docs/implementation-artifacts/8-1-result-view-and-export.md`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

- Implementation Plan: 补齐诊断报告 DTO/类型导出，完成服务与路由逻辑，再落地前端模块与 RunView 入口，最后跑全量回归。
- Tests: `cargo run --bin gen-types` / `cargo test` / `pnpm vitest run` / `pnpm vite build`

### Completion Notes List

- ✅ 后端新增诊断报告服务与 API（失败原因聚合、转折点检测、改进建议、用例详情对比）并补齐权限校验与 OpenAPI。
- ✅ 前端新增 diagnostic-report feature（摘要、时间线、失败用例列表、对比对话框）并接入 RunView。
- ✅ 扩展迭代仓库返回 evaluation_results，新增差异片段生成与用例详情查询。
- ✅ 接入 `failure_archive` 作为兜底统计来源，限制未完成任务访问，修复 diff 索引与预览截断。
- ✅ 完成类型生成与全量后端/前端回归测试。

### File List

- backend/src/api/routes/diagnostic.rs
- backend/src/api/routes/docs.rs
- backend/src/api/routes/mod.rs
- backend/src/bin/gen-types.rs
- backend/src/core/diagnostic_service/mod.rs
- backend/src/core/iteration_engine/checkpoint.rs
- backend/src/core/iteration_engine/pause_state.rs
- backend/src/core/iteration_engine/recovery.rs
- backend/src/core/mod.rs
- backend/src/core/optimization_engine/common.rs
- backend/src/domain/models/diagnostic.rs
- backend/src/domain/models/mod.rs
- backend/src/domain/types/artifacts.rs
- backend/src/infra/db/repositories/iteration_repo.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/infra/db/repositories/optimization_task_repo.rs
- backend/src/infra/db/repositories/test_set_repo.rs
- backend/src/main.rs
- backend/Cargo.toml
- backend/Cargo.lock
- docs/implementation-artifacts/8-1-result-view-and-export.md
- docs/implementation-artifacts/8-2-diagnostic-report.md
- docs/implementation-artifacts/sprint-status.yaml
- docs/implementation-artifacts/validation-report-2026-01-20-145355.md
- docs/implementation-artifacts/validation-report-2026-01-20-151537.md
- frontend/src/features/diagnostic-report/index.ts
- frontend/src/features/diagnostic-report/services/diagnosticService.ts
- frontend/src/features/diagnostic-report/hooks/useDiagnostic.ts
- frontend/src/features/diagnostic-report/hooks/useFailedCaseDetail.ts
- frontend/src/features/diagnostic-report/components/DiagnosticReport.tsx
- frontend/src/features/diagnostic-report/components/FailedCaseList.tsx
- frontend/src/features/diagnostic-report/components/TurningPointTimeline.tsx
- frontend/src/features/diagnostic-report/components/CaseComparisonDialog.tsx
- frontend/src/features/diagnostic-report/components/DiagnosticReport.test.tsx
- frontend/src/features/diagnostic-report/components/TurningPointTimeline.test.tsx
- frontend/src/pages/RunView/RunView.tsx
- frontend/src/types/generated/models/DiagnosticReport.ts
- frontend/src/types/generated/models/DiagnosticSummary.ts
- frontend/src/types/generated/models/FailureArchiveEntry.ts
- frontend/src/types/generated/models/FailureReasonEntry.ts
- frontend/src/types/generated/models/TurningPoint.ts
- frontend/src/types/generated/models/TurningPointType.ts
- frontend/src/types/generated/models/FailedCaseSummary.ts
- frontend/src/types/generated/models/FailedCaseDetail.ts
- frontend/src/types/generated/models/DiffSegment.ts
- frontend/src/types/generated/models/DiffSegmentType.ts
- frontend/src/types/generated/models/IterationArtifacts.ts

## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [x] [HIGH] 诊断报告已读取 `layer4.failure_archive`，补齐失败统计数据源。
- [x] [MEDIUM] `build_diff_segments` 索引语义已统一，插入/删除段映射稳定。
- [x] [MEDIUM] 诊断报告 API 已限制仅完成/失败任务访问，符合入口约束。
- [x] [MEDIUM] Story File List 已补齐实际变更文件，保证可追溯性。
- [x] [LOW] input_preview 截断长度已严格控制在 100 字符以内。

### Decisions

- [x] 直接修复并同步文档，保持代码与 Story 对齐。

### Risks / Tech Debt

- [x] failure_archive 已纳入，诊断数据源完整。
- [x] diff_segments 索引语义已明确，后续高亮扩展风险降低。

### Follow-ups

- [x] 已同步并闭环 `### Review Follow-ups (AI)`。
