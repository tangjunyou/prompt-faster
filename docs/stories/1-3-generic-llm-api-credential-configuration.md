# Story 1.3: 通用大模型 API 凭证配置

Status: done

## Story

As a Prompt 优化用户,
I want 配置通用大模型 API 凭证（硅基流动、魔搭社区）, 
so that 系统可以使用这些平台的模型作为老师模型。

## Acceptance Criteria

1. **Given** 用户在 API 配置页面 **When** 用户选择 Provider 类型（硅基流动 / 魔搭社区） **Then** 显示对应的配置表单（API Key、Base URL）

2. **Given** 用户正在填写通用大模型凭证表单 **When** 用户输入 Base URL 和 API Key 并点击“保存” **Then** 进行表单校验（URL 格式、Key 非空） **And** 校验通过后将凭证暂存于前端状态（不落库）

3. **Given** 用户输入无效的 Base URL 或空的 API Key **When** 用户点击“保存” **Then** 显示友好的错误提示（字段级提示 + 可选顶部提示）

4. **Given** 用户已选择某个 Provider 并填写了部分或全部字段 **When** 用户切换 Provider 类型 **Then** 表单自动切换 **And** 清空之前输入（baseUrl/apiKey） **And** 状态徽章回到“未配置/待填写”

## Tasks / Subtasks

- [x] **Task 0: 明确 UI 选择器实现方式（避免引入新依赖）** (AC: #1, #4)
  - [x] 0.1 **首选方案：使用现有 Button 组件实现分段选择**（见 Dev Notes 代码示例），不引入新依赖
  - [x] 0.2 如确需引入新的 shadcn/ui 组件（例如 Select），必须：生成到 `frontend/src/components/ui/` 并确保 `@/*` alias 与 `components.json` 一致

- [x] **Task 1: 扩展通用大模型凭证类型定义** (AC: #1)
  - [x] 1.1 在 `frontend/src/types/credentials.ts` 中新增通用大模型相关类型（不要新建重复的 credentials 文件）
  - [x] 1.2 定义 Provider 枚举/联合类型（例如 `type GenericLlmProvider = 'siliconflow' | 'modelscope'`）
  - [x] 1.3 定义 `GenericLlmCredential`（provider、baseUrl、apiKey、status）
  - [x] 1.4 确保 `frontend/src/types/index.ts` 仍可聚合导出（不改变既有对外契约）

- [x] **Task 2: 扩展凭证 Store（Zustand）** (AC: #2, #4)
  - [x] 2.1 在 `frontend/src/stores/useCredentialStore.ts` 中新增通用大模型凭证状态（与 dify 并列）
  - [x] 2.2 仅暴露领域动作（不要允许外部直接设置 status）：
    - [x] `setGenericLlmProvider(provider)`（切换 provider 时清空 baseUrl/apiKey）
    - [x] `setGenericLlmFormField(field, value)`（受控输入）
    - [x] `updateGenericLlmCredentialFromForm(baseUrl, apiKey)`（保存时写入 + 推断 status）
    - [x] `clearGenericLlmCredential()`
  - [x] 2.3 确保对已存在的 Dify 功能无回归（`dify` 分支逻辑不改变）

- [x] **Task 3: 实现通用大模型凭证表单 Hook** (AC: #2, #3, #4)
  - [x] 3.1 新建 `frontend/src/features/api-config/hooks/useGenericLlmCredentialForm.ts`
  - [x] 3.2 **先抽取公共 URL 工具模块**：将 `isValidUrl`/`normalizeBaseUrl` 从 `useDifyCredentialForm.ts` 迁移到 `frontend/src/features/api-config/hooks/url-utils.ts`，Dify 与 Generic 共用
  - [x] 3.3 保存策略：校验通过后规范化 baseUrl（提取 origin、去除末尾 `/`）再写入 Store
  - [x] 3.4 错误提示：字段失焦验证 + 提交时验证，保持与 Dify 表单一致的交互体验

- [x] **Task 4: 新增通用大模型凭证表单组件** (AC: #1, #2, #3, #4)
  - [x] 4.1 新建 `frontend/src/features/api-config/GenericLlmCredentialForm.tsx`（作为 api-config feature 的内部实现细节，不导出）
  - [x] 4.2 Provider 选择区：硅基流动 / 魔搭社区
  - [x] 4.3 表单字段：Base URL（文本输入）、API Key（密码输入）
  - [x] 4.4 状态徽章：未配置 / 已填写（待测试）
  - [x] 4.5 Provider 切换时：调用 Store 的 `setGenericLlmProvider` 完成清空与状态重置

- [x] **Task 5: 将通用大模型表单接入现有 API 配置页面** (AC: #1)
  - [x] 5.1 修改 `frontend/src/features/api-config/ApiConfigPanel.tsx`：在现有 `DifyCredentialForm` 下方加入通用大模型凭证区域
  - [x] 5.2 不修改路由结构：保持 `/settings/api` 为唯一入口（避免新增重复页面）

- [x] **Task 6: 最低测试要求（防回归）** (AC: #2, #3, #4)
  - [x] 6.1 为通用大模型表单校验与规范化逻辑添加 Vitest 单元测试（文件：`frontend/src/features/api-config/hooks/useGenericLlmCredentialForm.test.ts`）
  - [x] 6.2 为 Store 的 provider 切换清空行为添加测试（可用 `useCredentialStore.getState()` 断言）
  - [x] 6.3 复跑既有 `useDifyCredentialForm.test.ts`，确保无回归
  - [x] 6.4 **必测场景清单**（每个场景至少 1 个测试用例）：
    - [x] 切换 provider 清空：`setGenericLlmProvider` 后 `baseUrl`/`apiKey` 应为空
    - [x] blur 校验：失焦时触发字段级错误提示
    - [x] submit 校验：提交时验证所有字段，阻止无效输入
    - [x] baseUrl normalize：提取 origin + 去尾斜杠
    - [x] 双字段都空时清空凭证字段：调用 `clearGenericLlmFields`（保留 provider 选择）

## Dev Notes

### ⚠️ Guardrails（必须遵循）

- 不新增第二套状态管理：仍使用 Zustand（`useCredentialStore`）。
- 不新增第二套 API 配置页面：仍使用 `/settings/api` → `ApiConfigPage` → `ApiConfigPanel`。
- 本 Story **不实现后端接口、不做持久化、不做连接测试**（分别在 Story 1.4/1.5）。
- 本 Story **仅前端临时状态，不需要 ts-rs 类型生成**（虽然 architecture.md 提到 ts-rs，但本 Story 不涉及后端类型共享）。
- 禁止在任何日志/错误信息中输出或拼接完整 `apiKey`（包括 `console.log`、错误堆栈拼接）。
- 导入路径：统一使用 `@/*`（与 shadcn/ui 生成代码保持一致）。

### 与现有实现的对齐点（避免造轮子）

- 复用 `features/api-config/` 既有结构与交互规范：
  - Dify 表单：`DifyCredentialForm.tsx` + `hooks/useDifyCredentialForm.ts`
  - Store：`stores/useCredentialStore.ts`（单一事实来源，表单直接读写 Store）
- URL 校验/规范化：**必须**抽取到 `frontend/src/features/api-config/hooks/url-utils.ts`，Dify 与 Generic 共用。

### 技术实现要点

**类型定义（扩展 `frontend/src/types/credentials.ts`）：**

```typescript
// 新增 Provider 类型
export type GenericLlmProvider = 'siliconflow' | 'modelscope';

// 新增 GenericLlmCredential 接口
export interface GenericLlmCredential {
  provider: GenericLlmProvider | null;  // null 表示未选择
  baseUrl: string;
  apiKey: string;
  status: CredentialStatus;  // 由 Store 方法推断，不允许外部直接设置
}
```

**Store 扩展示例（`frontend/src/stores/useCredentialStore.ts`）：**

```typescript
const initialGenericLlmCredential: GenericLlmCredential = {
  provider: null,
  baseUrl: '',
  apiKey: '',
  status: 'empty',
};

// setGenericLlmProvider 实现要点：切换时必须清空 baseUrl/apiKey
setGenericLlmProvider: (provider) =>
  set((state) => ({
    genericLlm: {
      ...initialGenericLlmCredential,
      provider,
      status: 'empty',
    },
  })),

// updateGenericLlmCredentialFromForm 实现要点：保存时推断 status
updateGenericLlmCredentialFromForm: (baseUrl, apiKey) =>
  set((state) => ({
    genericLlm: {
      ...state.genericLlm,
      baseUrl,
      apiKey,
      status: baseUrl && apiKey ? 'filled' : 'empty',
    },
  })),
```

**Provider 选择器首选方案（复用现有 Button 组件）：**

```tsx
<div className="flex gap-2">
  <Button 
    variant={provider === 'siliconflow' ? 'default' : 'outline'}
    onClick={() => setGenericLlmProvider('siliconflow')}
  >
    硅基流动
  </Button>
  <Button 
    variant={provider === 'modelscope' ? 'default' : 'outline'}
    onClick={() => setGenericLlmProvider('modelscope')}
  >
    魔搭社区
  </Button>
</div>
```

### UI 状态展示规范

| 状态 | 徽章颜色 | 文案 |
|------|----------|------|
| `empty` (provider 未选择) | 灰色 | 未配置 |
| `empty` (provider 已选择) | 灰色 | 待填写 |
| `filled` | 黄色 (warning) | 已填写，待测试 |

> 注：`testing`、`valid`、`invalid` 状态将在 Story 1.4 中引入（本 Story 不定义、不实现）

### 预期影响的文件（实现落点）

- 前端（主要）：
  - `frontend/src/types/credentials.ts`（扩展：新增 GenericLlmProvider、GenericLlmCredential）
  - `frontend/src/stores/useCredentialStore.ts`（扩展：新增 genericLlm 状态和 4 个领域动作）
  - `frontend/src/features/api-config/ApiConfigPanel.tsx`（修改：集成 GenericLlmCredentialForm）
  - `frontend/src/features/api-config/GenericLlmCredentialForm.tsx`（新建）
  - `frontend/src/features/api-config/hooks/useGenericLlmCredentialForm.ts`（新建）
  - `frontend/src/features/api-config/hooks/useGenericLlmCredentialForm.test.ts`（新建）
  - `frontend/src/features/api-config/hooks/url-utils.ts`（**必须新建**：从 useDifyCredentialForm.ts 抽取公共函数）
  - `frontend/src/features/api-config/hooks/useDifyCredentialForm.ts`（修改：改为从 url-utils.ts 导入公共函数）

### Testing 标准摘要

- 单元测试：Vitest
- 关键断言（与 Task 6.4 必测场景清单对应）：
  - Provider 切换必须清空 baseUrl/apiKey
  - blur 校验必须触发字段级错误提示
  - submit 校验必须阻止无效输入
  - baseUrl 规范化结果稳定（origin + 去尾斜杠）
  - 双字段都空时必须清空凭证

### 与后续 Story 的关系

| Story | 依赖关系 |
|-------|----------|
| **1.4 连接测试** | 使用本 Story 的凭证状态，扩展 status 为 `testing`/`valid`/`invalid` |
| **1.5 凭证持久化** | 将 Store 状态持久化到后端数据库 |

### References

- [Source: docs/sprint-status.yaml#development_status] - 目标 Story Key
- [Source: docs/implementation-artifacts/epics.md#Story-1.3] - 验收标准原文
- [Source: docs/stories/1-2-dify-api-credential-configuration.md] - 既有实现模式与 Guardrails
- [Source: docs/implementation-artifacts/architecture.md#Frontend-Architecture] - 前端目录与边界规范

## Dev Agent Record

### Agent Model Used

Cascade

### Debug Log References

无

### Completion Notes List

- 使用现有 Button 组件实现 Provider 分段选择器（default/outline variant 切换）
- 抽取 url-utils.ts 公共模块，Dify 和 Generic LLM 表单共用 isValidUrl/normalizeBaseUrl
- 扩展 useCredentialStore 添加 genericLlm 状态和 4 个领域动作
- 实现 useGenericLlmCredentialForm Hook，遵循单一事实来源原则
- 实现 GenericLlmCredentialForm 组件，支持 Provider 选择、表单校验、状态徽章
- 所有 82 个测试通过（url-utils: 10, form-utils: 9, useFeedback: 8, Dify Hook: 10, Generic Hook: 19, Dify 组件: 11, Generic 组件: 15），无回归

### Change Log

- 2025-12-23: Story 1.3 实现完成 - 通用大模型 API 凭证配置前端功能
- 2025-12-23: Code Review #1 完成 - 修复 7 个问题（3 MEDIUM + 4 LOW），新增 15 个组件测试
- 2025-12-23: Code Review #2 完成 - 修复 7 个问题（4 MEDIUM + 3 LOW），消除代码重复，新增 DifyCredentialForm 组件测试
- 2025-12-23: Code Review #3 完成 - 修复 7 个问题（3 MEDIUM + 4 LOW），新增 useFeedback/form-utils 测试，maxLength/loading/data-testid/JSDoc
- 2025-12-23: Code Review #4 完成 - 修复 5 个问题（3 MEDIUM + 2 LOW），抽取共享常量/FeedbackAlert组件，修复 onSubmit 异步问题

### File List

- frontend/src/types/credentials.ts (修改: 新增 GenericLlmProvider, GenericLlmCredential)
- frontend/src/stores/useCredentialStore.ts (修改: 新增 genericLlm 状态和领域动作)
- frontend/src/features/api-config/constants.ts (新建: 共享常量 API_KEY_MAX_LENGTH, BASE_URL_MAX_LENGTH)
- frontend/src/features/api-config/FeedbackAlert.tsx (新建: 共享反馈消息组件)
- frontend/src/features/api-config/hooks/url-utils.ts (新建: 公共 URL 工具函数)
- frontend/src/features/api-config/hooks/url-utils.test.ts (新建: 10 个测试用例)
- frontend/src/features/api-config/hooks/form-utils.ts (新建: 共享校验函数和类型)
- frontend/src/features/api-config/hooks/form-utils.test.ts (新建: 9 个测试用例)
- frontend/src/features/api-config/hooks/useFeedback.ts (新建: 反馈消息 Hook)
- frontend/src/features/api-config/hooks/useFeedback.test.ts (新建: 8 个测试用例)
- frontend/src/features/api-config/hooks/useDifyCredentialForm.ts (修改: 使用共享模块)
- frontend/src/features/api-config/hooks/useDifyCredentialForm.test.ts (修改: 移除冗余测试)
- frontend/src/features/api-config/hooks/useGenericLlmCredentialForm.ts (新建: 使用共享模块)
- frontend/src/features/api-config/hooks/useGenericLlmCredentialForm.test.ts (新建: 19 个测试用例)
- frontend/src/features/api-config/GenericLlmCredentialForm.tsx (新建: 使用共享常量/FeedbackAlert/150ms延迟)
- frontend/src/features/api-config/GenericLlmCredentialForm.test.tsx (新建: 15 个组件测试)
- frontend/src/features/api-config/DifyCredentialForm.tsx (修改: 使用共享常量/FeedbackAlert/150ms延迟)
- frontend/src/features/api-config/DifyCredentialForm.test.tsx (新建: 11 个组件测试)
- frontend/src/features/api-config/ApiConfigPanel.tsx (修改: 集成 GenericLlmCredentialForm)
- frontend/src/features/api-config/index.ts (修改: 保持只导出 ApiConfigPanel)
- frontend/package.json (修改: 新增 @testing-library/user-event 依赖)
