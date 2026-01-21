# Story 8.4: 高级用户直接编辑老师模型 Prompt

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

Story Key: 8-4-advanced-user-edit-teacher-model-prompt

## Epic 8 概述

> **Epic 8: 结果输出与元优化** - 用户成果：用户可以查看、导出优化结果，查看诊断报告，并使用元优化功能优化老师模型 Prompt。

**Epic 8 Story 列表**：
- 8.1 结果查看与导出（FR60, FR61，NFR18）- ✅ done
- 8.2 诊断报告（FR63）- ✅ done
- 8.3 元优化基础（FR56, FR57, FR58）- ✅ done
- **8.4 高级用户直接编辑老师模型 Prompt（本 Story，FR59）** - MVP
- 8.5 Prompt 版本对比（FR62）- Growth
- 8.6 创意任务多样性检测（FR34）- Growth

## Key Decisions (MVP)

- **编辑策略**：编辑保存 = 创建新版本（不直接修改现有版本，保留完整历史）。
- **编辑器选型**：Monaco Editor（@monaco-editor/react），与项目代码展示风格一致。
- **预览执行**：限制最多 3 条测试用例，超时 30 秒，避免长时间阻塞。
- **预览数据源**：测试用例来自历史任务关联的 `test_set_ids`（复用优化任务详情接口与测试集接口获取）。
- **回滚机制**：复用 8.3 的 `activatePromptVersion` API 切换到任意历史版本。
- **格式验证**：前端基础校验 + 后端 `/prompts/validate` 校验（MVP），后续可扩展语法检查。
- **权限校验**：仅用户自己的 Prompt 可编辑。
- **AR2 遵循**：所有操作记录 correlationId，支持全链路追踪。

## Story

As a 高级用户,
I want 直接编辑老师模型 Prompt,
so that 我可以根据经验快速调整老师模型行为。

## Acceptance Criteria

1. **Given** 用户进入老师模型管理页面
   **When** 点击"编辑"按钮
   **Then** 显示老师模型 Prompt 编辑器
   **And** 提供语法高亮和格式化

2. **Given** 用户编辑完成
   **When** 点击"保存"
   **Then** 通过 `/api/v1/meta-optimization/prompts/validate` 验证 Prompt 格式有效性
   **And** 保存为新版本

3. **Given** 用户担心改坏
   **When** 编辑过程中
   **Then** 显示"回滚到上一版本"选项
   **And** 提供"预览效果"功能（在少量测试用例上试运行）

## Tasks / Subtasks

- 文件落点以 **File Structure Requirements** 为准；本节只描述职责，避免重复写路径。

- [x] 后端：预览/验证 DTO 定义（AC: 2, 3）
  - [x] 扩展 `teacher_prompt.rs` 添加预览与验证相关 DTO
    - `PromptPreviewRequest`: content, task_ids（必填）, test_case_ids（可选）
    - `PromptPreviewResult`: test_case_id, input, reference, actual_output, passed, execution_time_ms, error_message（可选）
    - `PromptPreviewResponse`: results, total_passed, total_failed, total_execution_time_ms
    - `PromptValidationRequest`: content
    - `PromptValidationResult`: is_valid, errors, warnings
  - [x] 在 `backend/src/bin/gen-types.rs` 注册新增类型

- [x] 后端：预览执行服务逻辑（AC: 3）
  - [x] 扩展 `meta_optimization_service/mod.rs`
    - `preview_prompt(user_id, request) -> PromptPreviewResponse`
    - 测试用例来源：基于历史任务的 `test_set_ids`
      - `task_ids` → `OptimizationTaskRepo::find_by_id_scoped` 获取 `workspace_id` + `test_set_ids`
      - `TestSetRepo::list_cases_by_ids` 拉取测试用例
      - `task_ids` 为空：返回参数错误（提示先选择历史任务）
      - `test_case_ids` 为空：按任务的 `test_set_ids` 顺序取前 3 条
      - `test_case_ids` 非空：校验归属且最多 3 条
    - 复用已有工厂：`create_teacher_model(TeacherModelType::Example)` + `create_evaluator_for_task_config`
    - 将 `EvaluatorConfig` 写入 `OptimizationContext.extensions`（`EXT_TASK_EVALUATOR_CONFIG`）
    - 超时控制 30 秒（`tokio::time::timeout`）

- [x] 后端：Prompt 验证服务逻辑（AC: 2）
  - [x] 在 `meta_optimization_service/mod.rs` 增加 `validate_prompt(content) -> PromptValidationResult`
    - 非空校验
    - 长度限制（最大 100KB）
    - 预留 warnings 结构（MVP 可为空）

- [x] 后端：预览/验证 API（AC: 2, 3）
  - [x] 扩展 `meta_optimization.rs`
    - `POST /api/v1/meta-optimization/prompts/preview` 预览执行
    - `POST /api/v1/meta-optimization/prompts/validate` 格式验证
  - [x] 权限校验：需登录（`CurrentUser` 提取 user_id，不从 request body 读取）
  - [x] correlationId：从 headers 提取并写入 tracing 日志
  - [x] 添加 OpenAPI 文档描述
  - [x] 在 `docs.rs` 注册新增 path/schema

- [x] 前端：Monaco Editor 集成（AC: 1）
  - [x] 确认 `@monaco-editor/react` 已在 `frontend/package.json`（无需重复安装）
  - [x] 创建 `PromptEditor.tsx` 编辑器组件（动态 import，参考 `ArtifactEditor.tsx`）
    - 语法高亮（markdown 或 plaintext 模式）
    - 主题与现有一致（默认 `vs-light`；如后续有主题系统再统一切换）
    - 代码格式化支持
    - 只读/编辑模式切换

- [x] 前端：编辑模式实现（AC: 1, 2）
  - [x] 修改 `PromptVersionDetail.tsx` 添加编辑模式
    - "编辑"按钮入口
    - 编辑器与只读视图切换
    - 保存/取消操作
  - [x] 保存流程：先调用 `validatePrompt` 校验，通过后调用 `createPromptVersion` 创建新版本
  - [x] 格式验证：非空校验 + 长度限制（最大 100KB）+ 必填变更说明

- [x] 前端：预览面板组件（AC: 3）
  - [x] 创建 `PromptPreviewPanel.tsx`
    - 测试用例选择器（最多 3 条）
      - 使用历史任务 → `GET /api/v1/workspaces/{workspace_id}/optimization-tasks/{task_id}` 获取 `test_set_ids`
      - 使用测试集接口加载用例数据供选择
    - 预览执行触发按钮
    - 结果展示：通过/失败状态、输入输出对比
    - 执行状态指示器（loading/success/error）
    - 预览输入使用编辑器当前内容（未保存版本）

- [x] 前端：预览服务层封装（AC: 2, 3）
  - [x] 扩展 `metaOptimizationService.ts`
    - `previewPrompt(request): Promise<PromptPreviewResponse>`
    - `validatePrompt(request): Promise<PromptValidationResult>`
  - [x] 创建 `hooks/usePromptPreview.ts` TanStack Query mutation hook

- [x] 前端：回滚入口实现（AC: 3）
  - [x] 在编辑界面添加"回滚到上一版本"按钮
    - 获取当前活跃版本的前一个版本（按 version desc 排序取 index=1；不存在则禁用）
    - 调用 `activatePromptVersion` 切换版本
    - 刷新编辑器内容

- [x] 测试与回归（AC: 1-3）
  - [x] 按 **Testing Requirements** 表执行
  - [x] 新增/覆盖测试文件
    - `backend/tests/meta_optimization_test.rs`（扩展预览测试）
    - `frontend/src/features/meta-optimization/components/PromptEditor.test.tsx`
    - `frontend/src/features/meta-optimization/components/PromptPreviewPanel.test.tsx`
    - `frontend/src/features/meta-optimization/components/PromptVersionDetail.test.tsx`（保存前验证）

### Hard Gate Checklist

> 必填：跨 Story 硬门禁清单（若不适用请标注 N/A 并说明原因）。

- [x] correlationId 全链路透传（HTTP/WS/日志）
- [x] A2 日志字段齐全（correlation_id/user_id/version_id/action/prev_state/new_state/timestamp；task_id/iteration_state 标注 N/A）
- [x] 新增/变更类型已运行 gen-types 并提交生成产物
- [x] 状态一致性与幂等性已校验（如版本保存幂等）

### Review Follow-ups (AI)

> 轻量但强制：把 review 里发现的可执行项落到这里，避免"只记在聊天里/只散落在文档里"。

- [x] [AI-Review][CRITICAL] 对齐预览 DTO 字段与 `TestCase/TaskReference`，移除 `test_case_name/expected_output` 冲突字段；更新 gen-types 产物
- [x] [AI-Review][CRITICAL] 预览数据源统一为“历史任务 → test_set_ids → test_sets.cases_json”，并在 API/服务/前端链路中落地
- [x] [AI-Review][HIGH] 新增 `/api/v1/meta-optimization/prompts/validate` 与 `validate_prompt` 服务，前端保存前先校验再保存
- [x] [AI-Review][HIGH] 明确“上一版本”定义为 version desc 排序 index=1，不存在则禁用回滚
- [x] [AI-Review][MEDIUM] Monaco 使用 lazy import + `vs-light` 主题，复用现有 `ArtifactEditor.tsx` 模式
- [x] [AI-Review][HIGH] 预览执行在 Dify/Generic 场景未注入 api_key，导致预览必失败；需在预览构建 ExecutionTargetConfig 时注入解密后的 API Key 或统一注入管道 [backend/src/core/meta_optimization_service/mod.rs:651]
- [x] [AI-Review][MEDIUM] 回滚目标版本固定为“版本列表第 2 个”，未按当前选中版本计算“上一版本”，编辑旧版本时回滚不正确 [frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx:76]
- [x] [AI-Review][MEDIUM] Prompt 预览耗时字段使用 i64 生成 bigint 类型，JSON 返回为 number，前后端类型不一致需统一为 number/string [backend/src/domain/models/teacher_prompt.rs:104]
- [x] [AI-Review][MEDIUM] 变更文件 `docs/implementation-artifacts/validation-report-2026-01-20-232605.md` 未记录在 Story File List，需补齐或移除 [docs/implementation-artifacts/validation-report-2026-01-20-232605.md:1]
- [x] [AI-Review][LOW] 跨工作区任务选择时仍可触发预览，需在 UI 禁用或明确提示以避免必然失败 [frontend/src/features/meta-optimization/components/PromptPreviewPanel.tsx:53]

## Dev Notes

### Developer Context (Read This First)

- **现状基线（Story 8.3 已完成）**：
  - `teacher_prompts` 表和版本管理已就绪
  - `meta_optimization_service` 服务层已实现（版本 CRUD + 统计）
  - 前端 `meta-optimization/` 模块已建立
  - `PromptVersionList.tsx` 版本列表组件已存在
  - `PromptVersionDetail.tsx` 版本详情组件已存在（只读模式）
  - `createPromptVersion` / `activatePromptVersion` API 已可用
  - 优化任务详情 API 已可返回 `test_set_ids`
  - 测试集 CRUD 与 test_cases 数据读取能力已存在

- **业务价值（为什么做）**：高级用户需要根据经验直接调整老师模型 Prompt，而不是等待系统自动优化。编辑功能配合预览执行，让用户可以安全地实验新 Prompt。

- **依赖关系**：
  - 依赖 Story 8.3 的版本管理基础设施
  - 依赖 `TeacherModel` trait 执行预览
  - 依赖 `Evaluator` trait 评估预览结果
  - 依赖优化任务详情与测试集接口获取测试用例
  - 复用 TanStack Query 数据获取模式

- **范围边界（必须遵守）**：
  - 本 Story 实现：编辑器 UI、保存为新版本、预览执行、回滚入口、服务端 Prompt 验证
  - 不包含：版本对比（8.5）、多样性检测（8.6）
  - 预览执行限制：最多 3 条测试用例，超时 30 秒

### 与其他 Story 的关系

| 功能 | Story 8.3 | Story 8.4（本 Story） | Story 8.5 |
| --- | --- | --- | --- |
| Prompt 版本管理 | ✅ 已实现 | 复用 | 复用 |
| 版本成功率统计 | ✅ 已实现 | 复用 | 复用 |
| 高级编辑 | - | ✅ 新增 | - |
| 预览执行 | - | ✅ 新增 | - |
| 版本对比 | - | - | 新增 |

### Suggested Data Structures

```rust
/// 位置：backend/src/domain/models/teacher_prompt.rs（扩展）
use std::collections::HashMap;
use crate::domain::models::TaskReference;

/// 预览执行请求
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptPreviewRequest {
    /// 待预览的 Prompt 内容
    pub content: String,
    /// 必填：历史任务 ID（用于解析 test_set_ids）
    #[serde(default)]
    pub task_ids: Vec<String>,
    /// 可选：指定测试用例 ID，为空时自动选择最多 3 条
    #[serde(default)]
    pub test_case_ids: Vec<String>,
}

/// 单条测试用例的预览结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptPreviewResult {
    pub test_case_id: String,
    pub input: HashMap<String, serde_json::Value>,
    pub reference: TaskReference,
    pub actual_output: String,
    pub passed: bool,
    pub execution_time_ms: i64,
    pub error_message: Option<String>,
}

/// 预览执行响应
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptPreviewResponse {
    pub results: Vec<PromptPreviewResult>,
    pub total_passed: i32,
    pub total_failed: i32,
    pub total_execution_time_ms: i64,
}

/// Prompt 验证请求
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptValidationRequest {
    pub content: String,
}

/// Prompt 验证结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "models/")]
pub struct PromptValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

### Suggested API Endpoints

```
# 预览执行（新增）
POST /api/v1/meta-optimization/prompts/preview
Request: PromptPreviewRequest
Response: ApiResponse<PromptPreviewResponse>
权限校验：需登录
限制：task_ids 必填；test_case_ids 最多 3 条，超时 30 秒；test_case_ids 来源于所选 task 的 test_set_ids

# 验证 Prompt（本 Story 实现）
POST /api/v1/meta-optimization/prompts/validate
Request: PromptValidationRequest
Response: ApiResponse<PromptValidationResult>
权限校验：需登录
```

### Frontend Component Notes

**PromptEditor.tsx 结构：**
```tsx
import { lazy, Suspense } from 'react';
const MonacoEditor = lazy(async () => import('@monaco-editor/react'));

interface PromptEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  height?: string;
}

export function PromptEditor({ value, onChange, readOnly = false, height = '400px' }: PromptEditorProps) {
  return (
    <Suspense fallback={<div className="h-[300px] text-sm text-muted-foreground">加载编辑器中...</div>}>
      <MonacoEditor
        height={height}
        defaultLanguage="markdown"
        theme="vs-light"
        value={value}
        onChange={(val) => onChange(val || '')}
        options={{
          readOnly,
          minimap: { enabled: false },
          wordWrap: 'on',
          lineNumbers: 'on',
          fontSize: 14,
          scrollBeyondLastLine: false,
        }}
      />
    </Suspense>
  );
}
```

**PromptVersionDetail.tsx 编辑模式扩展：**
```tsx
// 状态管理
const [isEditing, setIsEditing] = useState(false);
const [editContent, setEditContent] = useState('');
const [changeNote, setChangeNote] = useState('');
const [isSaving, setIsSaving] = useState(false);

// 编辑模式 UI
{isEditing ? (
  <div className="space-y-4">
    <PromptEditor 
      value={editContent} 
      onChange={setEditContent} 
    />
    <div className="flex gap-2">
      <Button onClick={handleSave} disabled={isSaving || !changeNote.trim()}>
        {isSaving ? '保存中...' : '保存为新版本'}
      </Button>
      <Button variant="outline" onClick={() => setIsEditing(false)}>
        取消
      </Button>
      <Button variant="ghost" onClick={handlePreview}>
        预览效果
      </Button>
    </div>
    {/* 保存前弹窗输入变更说明 + 调用 validatePrompt */}
  </div>
) : (
  <div className="space-y-4">
    <PromptEditor value={prompt.content} readOnly />
    <Button onClick={() => { setEditContent(prompt.content); setIsEditing(true); }}>
      编辑
    </Button>
  </div>
)}
```

**PromptPreviewPanel.tsx 结构：**
```tsx
<PreviewPanel>
  {/* props: content（编辑器当前内容） */}
  <TestCaseSelector 
    testCases={availableTestCases}
    selected={selectedIds}
    onSelect={setSelectedIds}
    maxCount={3}
  />
  <Button onClick={handlePreview} disabled={isPreviewing}>
    {isPreviewing ? '执行中...' : '预览效果'}
  </Button>
  
  {previewResult && (
    <PreviewResults>
      <Summary passed={previewResult.totalPassed} failed={previewResult.totalFailed} />
      {previewResult.results.map(result => (
        <ResultCard 
          key={result.testCaseId}
          result={result}
        />
      ))}
    </PreviewResults>
  )}
</PreviewPanel>
```

**UX 对齐**：
- 编辑按钮位于版本详情页顶部
- 保存时弹出确认对话框，要求输入版本变更说明
- 预览执行显示进度指示器
- 回滚按钮在编辑界面显眼位置
- 验证失败时显示具体错误信息

### Dev Agent Guardrails（避免常见踩坑）

- **编辑器延迟加载**：Monaco Editor 体积较大，使用动态 import 延迟加载
- **依赖确认**：`@monaco-editor/react` 已在 `frontend/package.json`，无需重复安装
- **预览超时处理**：必须处理 30 秒超时，显示友好提示
- **保存确认**：保存前弹出对话框，避免误操作
- **版本变更说明**：保存时必须填写变更说明（非空校验）
- **回滚确认**：回滚前确认对话框，告知用户当前编辑内容将丢失
- **并发控制**：预览执行时禁用保存按钮，避免状态混乱
- **日志安全**：日志不得包含 Prompt 完整内容，仅记录 id/version
- **内容长度**：前端校验 Prompt 内容不超过 100KB
- **测试用例选择**：按 `test_set_ids` 顺序取用例，自动选择最多 3 条

### Technical Requirements（必须满足）

- 时间戳使用 Unix 毫秒存储，API 返回 ISO 8601
- API 响应使用 `ApiResponse<T>` 统一结构
- 所有操作记录 tracing 日志，包含 A2 必填字段
- 前端错误提示不得直接展示 `error.details`
- Monaco Editor 使用动态 import 延迟加载
- 预览执行最多 3 条测试用例，超时 30 秒
- 保存时必须填写版本变更说明
- 预览测试用例来源：历史任务 → `test_set_ids` → `test_sets.cases_json`
- 新增 `/api/v1/meta-optimization/prompts/validate` 作为服务端校验入口

### Backward Compatibility / Non-Regressions（必须遵守）

- 复用 Story 8.3 的 `teacher_prompts` 表，不新增数据库迁移
- 复用 Story 8.3 的 `createPromptVersion` / `activatePromptVersion` API
- 新增 `/api/v1/meta-optimization/prompts/preview` 与 `/api/v1/meta-optimization/prompts/validate` 端点，不修改现有 API
- 扩展现有组件（`PromptVersionDetail.tsx`），保持只读模式兼容

### Previous Story Learnings (Story 8.3 复盘/模式/测试)

- **后端路由模式**：使用 `CurrentUser` 提取器进行权限校验
- **DTO 设计模式**：使用 `#[serde(rename_all = "camelCase")]` + `#[ts(export_to = "models/")]`
- **前端模块结构**：采用 `components/` + `hooks/` + `services/` + `index.ts`
- **测试实践**：使用 MSW + `QueryClientProvider`，通过 `useAuthStore` 注入登录态
- **版本管理**：编辑 = 创建新版本，保留完整历史
- **性能优化**：统计查询使用聚合 SQL 消除 N+1

### Latest Technical Notes（基于当前项目版本）

**Breaking Changes / Best Practices**
- Monaco Editor：使用 `@monaco-editor/react` 包，支持 React 19
- TanStack Query v5：mutation 使用 `useMutation` hook
- Axum 0.8：路由路径参数语法 `/{param}`

**Performance / Deprecation Notes**
- Monaco Editor 延迟加载，避免首屏加载过重
- 预览执行使用独立超时控制，不影响主流程

### Architecture Compliance（必须遵守）

- **模块位置**：遵循架构定义
  - `backend/src/domain/models/teacher_prompt.rs`：扩展预览/验证 DTO
  - `backend/src/core/meta_optimization_service/mod.rs`：扩展预览/验证服务
  - `backend/src/api/routes/meta_optimization.rs`：扩展预览/验证 API
  - `frontend/src/features/meta-optimization/components/PromptEditor.tsx`：编辑器（新增）
  - `frontend/src/features/meta-optimization/components/PromptPreviewPanel.tsx`：预览面板（新增）
  - `frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx`：扩展编辑模式
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
- **@monaco-editor/react**：代码编辑器（已存在依赖）

### Deployment / Environment Notes（部署/环境）

- 本 Story 不新增数据库迁移
- 前端依赖无需新增：`@monaco-editor/react` 已存在
- 部署验证：建议执行 `cargo test`、`pnpm vitest run`、`pnpm vite build`

### File Structure Requirements（落点约束）

**后端**：
- 预览/验证 DTO：`backend/src/domain/models/teacher_prompt.rs`（扩展）
- 预览/验证服务：`backend/src/core/meta_optimization_service/mod.rs`（扩展）
- 预览/验证 API：`backend/src/api/routes/meta_optimization.rs`（扩展）
- OpenAPI：`backend/src/api/routes/docs.rs`（扩展）
- 类型生成：`backend/src/bin/gen-types.rs`（扩展）

**前端**：
- 编辑器组件：`frontend/src/features/meta-optimization/components/PromptEditor.tsx`（新增）
- 预览面板：`frontend/src/features/meta-optimization/components/PromptPreviewPanel.tsx`（新增）
- 版本详情：`frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx`（扩展）
- 服务层：`frontend/src/features/meta-optimization/services/metaOptimizationService.ts`（扩展）
- 预览 Hook：`frontend/src/features/meta-optimization/hooks/usePromptPreview.ts`（新增）
- 生成类型：`frontend/src/types/generated/models/`（自动生成）

**测试**：
- 后端测试：`backend/tests/meta_optimization_test.rs`（扩展）
- 编辑器测试：`frontend/src/features/meta-optimization/components/PromptEditor.test.tsx`（新增）
- 预览面板测试：`frontend/src/features/meta-optimization/components/PromptPreviewPanel.test.tsx`（新增）

### Testing Requirements（必须补齐）

| 测试类型 | 覆盖范围 | 关键用例 |
| --- | --- | --- |
| 后端单测 | 预览执行 | 正确执行测试用例并返回结果 |
| 后端单测 | 预览超时 | 超过 30 秒正确返回超时错误 |
| 后端单测 | 预览限制 | 超过 3 条测试用例正确拒绝 |
| 后端单测 | 权限校验 | 未登录返回 401 |
| 后端单测 | Prompt 验证 | 非空/长度校验返回错误 |
| 前端测试 | 编辑器渲染 | 正确渲染 Monaco Editor |
| 前端测试 | 编辑模式切换 | 只读/编辑模式正确切换 |
| 前端测试 | 保存流程 | 先调用 validatePrompt，再调用 createPromptVersion |
| 前端测试 | 预览执行 | 显示执行结果 |
| 前端测试 | 回滚功能 | 调用 activatePromptVersion API |
| 回归 | 全量回归 | `cargo test` + `vitest` + `vite build` 必须通过 |

### Project Structure Notes

- 参考 `frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx` 现有实现
- 参考 `backend/src/api/routes/meta_optimization.rs` 路由模式
- 复用 `backend/src/core/meta_optimization_service/mod.rs` 服务层
- Monaco Editor 参考项目中其他代码展示场景

### References

- Epic/Story 定义：`docs/project-planning-artifacts/epics.md`（Epic 8 / Story 8.4）
- PRD 元优化：`docs/project-planning-artifacts/prd.md#能力区域 9: 元优化`
- 架构（元优化）：`docs/project-planning-artifacts/architecture.md#9. 元优化`
- Story 8.3（前序）：`docs/implementation-artifacts/8-3-meta-optimization-basics.md`
- 元优化服务：`backend/src/core/meta_optimization_service/mod.rs`
- 版本详情组件：`frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx`
- 优化任务详情 API：`backend/src/api/routes/optimization_tasks.rs`
- 测试集接口：`frontend/src/features/test-set-manager/services/testSetService.ts`

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex)

### Debug Log References

- `cargo run --bin gen-types`
- `cargo test`
- `cargo test --test meta_optimization_test`
- `pnpm vitest run`
- `pnpm vite build`

### Completion Notes List

- 新增 Prompt 预览/验证 DTO、服务逻辑与 API，并补齐 OpenAPI/类型生成产物
- 前端完成 Monaco 编辑器、编辑保存校验、预览面板与回滚入口
- 覆盖预览/验证/超时与前端编辑/预览流程测试

### File List

- `backend/src/domain/models/teacher_prompt.rs`
- `backend/src/domain/models/mod.rs`
- `backend/src/bin/gen-types.rs`
- `backend/src/core/meta_optimization_service/mod.rs`
- `backend/src/core/teacher_model/mod.rs`
- `backend/src/api/routes/meta_optimization.rs`
- `backend/src/api/routes/docs.rs`
- `backend/tests/meta_optimization_test.rs`
- `frontend/src/features/meta-optimization/components/PromptEditor.tsx`
- `frontend/src/features/meta-optimization/components/PromptEditor.test.tsx`
- `frontend/src/features/meta-optimization/components/PromptPreviewPanel.tsx`
- `frontend/src/features/meta-optimization/components/PromptPreviewPanel.test.tsx`
- `frontend/src/features/meta-optimization/components/PromptVersionDetail.tsx`
- `frontend/src/features/meta-optimization/components/PromptVersionDetail.test.tsx`
- `frontend/src/features/meta-optimization/hooks/usePromptPreview.ts`
- `frontend/src/features/meta-optimization/services/metaOptimizationService.ts`
- `frontend/src/pages/MetaOptimizationPage.tsx`
- `frontend/src/types/generated/models/PromptPreviewRequest.ts`
- `frontend/src/types/generated/models/PromptPreviewResponse.ts`
- `frontend/src/types/generated/models/PromptPreviewResult.ts`
- `frontend/src/types/generated/models/PromptValidationRequest.ts`
- `frontend/src/types/generated/models/PromptValidationResult.ts`
- `docs/implementation-artifacts/sprint-status.yaml`
- `docs/implementation-artifacts/8-4-advanced-user-edit-teacher-model-prompt.md`
- `docs/implementation-artifacts/validation-report-2026-01-20-232605.md`
## Review Notes

> 统一结构（便于后续检索/复用/持续改进）

### Findings

- [ ] [CRITICAL] PromptPreview DTO 与现有 `TestCase/TaskReference` 不一致（`test_case_name`/`expected_output`/string input），易导致前后端契约冲突
- [ ] [CRITICAL] 预览测试用例数据源缺失（未定义从历史任务或测试集获取链路）
- [ ] [HIGH] `/prompts/validate` 在 Story 中出现但未纳入 Tasks/AC，范围不一致
- [ ] [HIGH] “最近 3 条测试用例”不可实现（TestCase 无时间字段，cases 存于 JSON）
- [ ] [MEDIUM] Monaco 主题与加载方式与现有项目实现不一致（现有为 lazy + vs-light）
- [ ] [HIGH] 预览执行在 Dify/Generic 场景未注入 api_key，导致真实预览必失败（ExecutionTargetConfig api_key 为空）
- [ ] [MEDIUM] 回滚目标版本固定为“版本列表第 2 个”，未按当前选中版本计算“上一版本”
- [ ] [MEDIUM] Prompt 预览耗时字段使用 bigint 类型，API JSON 返回 number，前后端类型不一致
- [ ] [MEDIUM] 变更文件 `validation-report-2026-01-20-232605.md` 未记录在 Story File List
- [ ] [LOW] 跨工作区任务选择时仍可触发预览，易导致必然失败的请求

### Decisions

- [ ] [DECISION] 将 `/prompts/validate` 纳入本 Story（避免“可选但未落地”的范围漂移）
- [ ] [DECISION] 预览数据源以“历史任务 → test_set_ids → test_sets.cases_json”为唯一来源，减少新增 API
- [ ] [DECISION] 统一 DTO 字段与现有模型一致，避免新增不可用字段
- [ ] [DECISION] Monaco 主题使用 vs-light（与现有组件一致）

### Risks / Tech Debt

- [ ] [RISK] 预览流程依赖历史任务的 test_set_ids；若任务未配置测试集则预览不可用（需前端提示并阻止调用）
- [ ] [RISK] Prompt 校验目前仅非空/长度限制，语法级校验留待后续扩展

### Follow-ups

- [ ] 补充测试：validatePrompt 失败不调用 createPromptVersion；预览接口 task_ids 为空返回 400
- [ ] 前端错误提示不要直接展示 `error.details`（沿用现有错误处理规范）
- [x] 预览执行需注入解密后的 API Key（或复用统一注入机制），覆盖 Dify/Generic 真实任务
- [x] 回滚目标版本按“当前选中版本的上一版本”计算，并在无上一版本时禁用入口
- [x] 统一预览耗时字段类型（后端用 u64 + ts number 或返回 string）
- [x] Story File List 补齐或删除未记录的 validation-report 文件
- [x] 预览按钮需在 workspaceId 无效或 casesError 时禁用
