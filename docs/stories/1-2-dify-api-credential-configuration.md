# Story 1.2: Dify API 凭证配置

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 配置 Dify 工作流的 API 凭证（地址 + API Key）,
So that 系统可以调用我的 Dify 工作流进行优化测试。

## Acceptance Criteria

1. **Given** 用户首次进入系统 **When** 用户填写 Dify API 地址和 API Key **Then** 表单验证格式正确性（URL 格式、Key 非空）**And** 凭证暂存于前端状态（尚未持久化）**And** 界面提示"已填写，待测试连接"

2. **Given** 用户输入无效的 API 地址格式 **When** 提交表单 **Then** 显示友好的错误提示

3. **Given** 用户已填写过凭证 **When** 用户清空 Base URL 和 API Key 并点击"保存" **Then** 凭证从 Store 中被清空 **And** 界面状态恢复为"未配置"

## Tasks / Subtasks

- [x] **Task 0: 对齐项目依赖与基础设施（阻塞项必须先解决）** (AC: #1, #2, #3)
  - [x] 0.1 在 `frontend/package.json` 中添加依赖：`zustand`
  - [x] 0.2 配置前端路径别名 `@/*`（用于 shadcn/ui 组件生成与统一导入风格）：
    - [x] 更新 `frontend/tsconfig.app.json`：设置 `compilerOptions.baseUrl` 与 `compilerOptions.paths`（例如 `"@/*": ["src/*"]`）
    - [x] 更新 `frontend/vite.config.ts`：设置 `resolve.alias`（例如 `@ -> <repo>/frontend/src`）
  - [x] 0.2 初始化 shadcn/ui（生成 `components.json` 等必要文件）
    - [x] 初始化时确保 `components.json` 的 `aliases` 与 `@/*` 配置一致（例如 `aliases.ui: "@/components/ui"`、`aliases.utils: "@/lib/utils"`）
  - [x] 0.3 生成并提交本 Story 需要的 shadcn/ui 组件到 `frontend/src/components/ui/`：Input, Button, Card, Badge, Label

- [x] **Task 1: 创建凭证类型定义** (AC: #1)
  - [x] 1.1 创建 `frontend/src/types/credentials.ts`
  - [x] 1.2 定义 `DifyCredential` 接口（baseUrl, apiKey, status）
  - [x] 1.3 定义 `CredentialStatus` 类型（本 Story 仅允许：'empty' | 'filled'）
  - [x] 1.4 创建 `frontend/src/types/index.ts`（如不存在）并导出 `api.ts`、`credentials.ts`
    - [x] 说明：`types/index.ts` 仅做聚合导出，不改变既有 `types/api.ts` 的内容与对外契约

- [x] **Task 2: 创建凭证状态 Store** (AC: #1, #3)
  - [x] 2.1 创建 `frontend/src/stores/useCredentialStore.ts`
  - [x] 2.2 使用 Zustand 实现 `useCredentialStore`
  - [x] 2.3 实现方法：`setDifyCredential`, `clearDifyCredential`
  - [x] 2.4 状态不持久化（本 Story 仅内存状态，持久化在 Story 1.5）

- [x] **Task 3: 创建 API 配置页面** (AC: #1)
  - [x] 3.1 创建 `frontend/src/pages/ApiConfigPage.tsx`
  - [x] 3.2 页面布局：标题 + 凭证配置区域
  - [x] 3.3 导出到 `frontend/src/pages/index.ts`

- [x] **Task 4: 对齐架构的 api-config Feature 对外接口** (AC: #1, #2)
  - [x] 4.1 创建 `frontend/src/features/api-config/index.ts` 导出文件
  - [x] 4.2 创建 `frontend/src/features/api-config/ApiConfigPanel.tsx`（作为该 feature 的对外入口，对齐 `docs/implementation-artifacts/architecture.md` 规划）
  - [x] 4.3 在 `ApiConfigPanel.tsx` 内部组合 `DifyCredentialForm`（可作为内部子组件）

- [x] **Task 5: 创建 Dify 凭证表单组件** (AC: #1, #2, #3)
  - [x] 5.1 创建 `frontend/src/features/api-config/DifyCredentialForm.tsx`
  - [x] 5.2 表单字段：Base URL（文本输入）、API Key（密码输入）
  - [x] 5.3 显示当前状态徽章（未配置/已填写，待测试）
  - [x] 5.4 点击"保存"时：若校验通过则写入 Store 并将状态置为 `filled`；若两字段都为空则执行清空并回到 `empty`

- [x] **Task 6: 实现表单验证与规范化** (AC: #2)
  - [x] 6.1 创建 `frontend/src/features/api-config/hooks/useDifyCredentialForm.ts`
  - [x] 6.2 URL 格式验证：必须是有效的 http/https URL
  - [x] 6.3 Base URL 规范化：保存到 Store 前将 URL 解析为 `origin`，并去除末尾 `/`
  - [x] 6.4 API Key 非空验证
  - [x] 6.5 实时验证反馈（字段失焦时验证 + 提交时验证）
  - [x] 6.6 错误信息友好展示（字段下方展示 + 可选顶部汇总）

- [x] **Task 7: 添加路由与导航** (AC: #1)
  - [x] 7.1 修改 `frontend/src/App.tsx` 添加 `/settings/api` 路由
  - [x] 7.2 在 `frontend/src/pages/HomePage.tsx` 添加导航入口（使用 React Router 的 `Link`，避免 `<a>` 全页刷新）

- [x] **Task 8: 最低测试要求（防回归）** (AC: #2)
  - [x] 8.1 为 `isValidUrl`/`validateDifyCredential`/规范化逻辑添加 Vitest 单元测试（建议文件：`frontend/src/features/api-config/hooks/useDifyCredentialForm.test.ts`）

## Dev Notes

### ⚠️ Guardrails（必须遵循的事实来源）

> **以下文件为唯一事实来源，不得引入冲突实现：**
> - 前端依赖版本：以 `frontend/package.json` 为准
> - 状态管理：使用 Zustand，命名约定 `use{Domain}Store`
> - 目录结构：遵循 `features/api-config/` 模式，并对齐 `docs/implementation-artifacts/architecture.md` 的对外入口命名（`ApiConfigPanel.tsx`）
> - **禁止**：引入第二套状态管理库、第二套表单库
> - **边界**：本 Story 仅负责前端表单和临时状态，不涉及后端 API 调用
> - **禁止**：在任何日志/错误信息中输出或拼接完整 `apiKey`（包括 `console.log`）
> - **导入约定**：统一使用 `@/*` 路径别名导入 `src` 下模块（与 shadcn/ui 生成代码保持一致），禁止混用多套 alias

### 技术实现要点

**类型定义：**

```typescript
// 文件：frontend/src/types/credentials.ts
export type CredentialStatus = 'empty' | 'filled';

export interface DifyCredential {
  baseUrl: string;
  apiKey: string;
  status: CredentialStatus;
}
```

**状态 Store 示例：**

```typescript
// 文件：frontend/src/stores/useCredentialStore.ts
import { create } from 'zustand';
import type { DifyCredential } from '@/types/credentials';

interface CredentialState {
  dify: DifyCredential;
  setDifyCredential: (credential: Partial<DifyCredential>) => void;
  clearDifyCredential: () => void;
}

const initialDifyCredential: DifyCredential = {
  baseUrl: '',
  apiKey: '',
  status: 'empty',
};

export const useCredentialStore = create<CredentialState>((set) => ({
  dify: initialDifyCredential,
  setDifyCredential: (credential) =>
    set((state) => {
      const merged = { ...state.dify, ...credential };
      return {
        dify: {
          ...merged,
          status: merged.baseUrl && merged.apiKey ? 'filled' : 'empty',
        },
      };
    }),
  clearDifyCredential: () => set({ dify: initialDifyCredential }),
}));
```

**URL 验证函数：**

```typescript
// 文件：frontend/src/features/api-config/hooks/useDifyCredentialForm.ts
export const isValidUrl = (url: string): boolean => {
  try {
    const parsed = new URL(url);
    return ['http:', 'https:'].includes(parsed.protocol);
  } catch {
    return false;
  }
};

export const normalizeBaseUrl = (baseUrl: string): string => {
  const trimmed = baseUrl.trim().replace(/\/+$/, '');
  const parsed = new URL(trimmed);
  return parsed.origin;
};

export const validateDifyCredential = (baseUrl: string, apiKey: string) => {
  const errors: { baseUrl?: string; apiKey?: string } = {};
  
  if (!baseUrl.trim()) {
    errors.baseUrl = 'API 地址不能为空';
  } else if (!isValidUrl(baseUrl)) {
    errors.baseUrl = '请输入有效的 HTTP/HTTPS 地址';
  }
  
  if (!apiKey.trim()) {
    errors.apiKey = 'API Key 不能为空';
  }
  
  return {
    isValid: Object.keys(errors).length === 0,
    errors,
  };
};
```

### 目录结构

```
frontend/src/
├── features/
│   └── api-config/           # 新建
│       ├── index.ts
│       ├── ApiConfigPanel.tsx
│       ├── DifyCredentialForm.tsx
│       └── hooks/
│           └── useDifyCredentialForm.ts
├── pages/
│   ├── index.ts              # 修改：添加导出
│   ├── HomePage.tsx          # 修改：添加导航链接
│   └── ApiConfigPage.tsx     # 新建
├── stores/
│   └── useCredentialStore.ts # 新建
└── types/
    ├── index.ts              # 新建：添加导出
    └── credentials.ts        # 新建
```

### UI 状态展示规范

| 状态 | 徽章颜色 | 文案 |
|------|----------|------|
| `empty` | 灰色 | 未配置 |
| `filled` | 黄色 | 已填写，待测试 |

> 注：`testing`、`valid`、`invalid` 状态将在 Story 1.4 中引入（本 Story 不定义、不实现，避免状态机漂移）

### 与后续 Story 的关系

| Story | 依赖关系 |
|-------|----------|
| **1.3 通用大模型凭证** | 复用本 Story 的表单模式和 Store 结构 |
| **1.4 连接测试** | 使用本 Story 的凭证状态，调用后端 API |
| **1.5 凭证持久化** | 将 Store 状态持久化到后端数据库 |

### Project Structure Notes

- 与 architecture.md `features/api-config/` 目录规划对齐
- 使用 Zustand 管理全局凭证状态
- 表单验证使用原生 URL API，无需额外依赖
- Base URL 保存规范：保存到 Store 的 `baseUrl` 仅保留 `origin`（不包含路径），并去除末尾 `/`
- 调试日志脱敏建议：如确需输出调试信息，仅输出 `apiKey` 的掩码版本（例如仅保留前后 4 位）

### References

- [Source: docs/implementation-artifacts/architecture.md#Frontend-Architecture] - 前端架构设计
- [Source: docs/implementation-artifacts/architecture.md#State-Management] - 状态管理策略
- [Source: docs/implementation-artifacts/epics.md#Story-1.2] - 验收标准原文
- [Source: docs/implementation-artifacts/prd.md#7.3] - 前端架构要求
- [Source: docs/stories/1-1-project-initialization-and-basic-architecture.md] - 前置 Story 实现参考

## Dev Agent Record

### Agent Model Used

Cascade

### Debug Log References

- vitest.config.ts 存在类型警告（vitest 内置 vite 与项目 vite 版本不匹配），不影响运行时行为，测试和构建均正常通过

### Completion Notes List

- 2025-12-22: Story 1-2 实现完成
  - Task 0: 添加 zustand 依赖、配置 @/* 路径别名、初始化 shadcn/ui、生成 UI 组件 (Input, Button, Card, Badge, Label)
  - Task 1: 创建凭证类型定义 (DifyCredential, CredentialStatus)
  - Task 2: 创建 Zustand Store (useCredentialStore)
  - Task 3: 创建 API 配置页面 (ApiConfigPage)
  - Task 4: 创建 api-config Feature 对外接口 (ApiConfigPanel)
  - Task 5: 创建 Dify 凭证表单组件 (DifyCredentialForm)
  - Task 6: 实现表单验证与规范化 (useDifyCredentialForm hook)
  - Task 7: 添加路由 /settings/api 与导航链接
  - Task 8: 添加 12 个单元测试覆盖 isValidUrl、normalizeBaseUrl、validateDifyCredential
  - 所有 AC 均满足：AC#1 表单验证与状态提示、AC#2 友好错误提示、AC#3 清空凭证功能

### Change Log

- 2025-12-22: Story 1-2 实现完成，所有任务和测试通过
- 2025-12-22: 代码审查修复（共 11 个问题）
  - **C1**: 修复 ESLint 错误（为 shadcn/ui 组件添加忽略规则）
  - **H1**: 添加 Hook 级测试（5 个新测试覆盖 AC#1-3 场景）
  - **H2+M2**: 重构 Store/Hook 实现单一事实来源，修复清空逻辑
  - **M1**: 修复 Badge 颜色（添加 warning variant 实现黄色，符合 Story 规范）
  - **M3**: 收紧 Store API（`updateDifyCredentialFromForm`/`setDifyFormField` 替代 `Partial` 合并）
  - **M4**: 修复 features/index.ts 导出
  - **M5**: 添加 normalizeBaseUrl 防御性编程（try-catch）
  - **L1**: 添加成功/失败反馈消息
  - **L2**: 收窄 api-config 导出（只暴露 ApiConfigPanel）
  - **L3**: 修复导航链接样式（text-primary 替代硬编码 text-blue-500）

### File List

**新增文件：**
- frontend/src/types/credentials.ts
- frontend/src/types/index.ts
- frontend/src/stores/useCredentialStore.ts
- frontend/src/pages/ApiConfigPage.tsx
- frontend/src/features/api-config/index.ts
- frontend/src/features/api-config/ApiConfigPanel.tsx
- frontend/src/features/api-config/DifyCredentialForm.tsx
- frontend/src/features/api-config/hooks/useDifyCredentialForm.ts
- frontend/src/features/api-config/hooks/useDifyCredentialForm.test.ts
- frontend/src/components/ui/input.tsx
- frontend/src/components/ui/button.tsx
- frontend/src/components/ui/card.tsx
- frontend/src/components/ui/badge.tsx
- frontend/src/components/ui/label.tsx
- frontend/components.json

**修改文件：**
- frontend/package.json (添加 zustand)
- frontend/tsconfig.json (添加路径别名)
- frontend/tsconfig.app.json (添加路径别名)
- frontend/vite.config.ts (添加 resolve.alias)
- frontend/vitest.config.ts (添加 resolve.alias)
- frontend/tailwind.config.js (shadcn/ui 更新)
- frontend/src/index.css (shadcn/ui CSS 变量)
- frontend/src/lib/utils.ts (shadcn/ui 更新)
- frontend/src/App.tsx (添加 /settings/api 路由)
- frontend/src/pages/index.ts (导出 ApiConfigPage)
- frontend/src/pages/HomePage.tsx (添加导航链接，修复样式)
- frontend/eslint.config.js (添加 shadcn/ui 组件忽略规则)
- frontend/src/features/index.ts (添加 api-config 导出)
