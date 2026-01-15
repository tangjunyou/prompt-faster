# Story 1.9: 前端应用架构与类型安全 API 客户端

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 参与 Prompt Faster 的前端开发者,
I want 前端项目使用约定的路由、数据获取与类型生成方案（React Router 7、TanStack Query、ts-rs）,
So that 页面路由清晰、数据获取模式统一, 并与后端类型保持一致。

## Acceptance Criteria

**AC1: Given** 前端项目已创建
**When** 查看路由配置
**Then** 采用 React Router 7.x 的官方推荐写法, 并为主视图（Run View/Focus View/Workspace View）预留清晰的路由层级

**AC2: Given** 某个需要从后端获取数据的前端模块（如工作区列表、测试集列表）
**When** 检查数据请求逻辑
**Then** 使用 TanStack Query 管理请求、缓存与 loading/error 状态
**And** 避免在业务组件中散落裸露的 `fetch`/`axios` 调用

**AC3: Given** 后端已使用 Rust 定义核心 DTO（请求/响应结构）
**When** 运行 ts-rs 类型生成流程
**Then** 在前端代码中可以直接 import 对应的 TypeScript 类型
**And** 不需要手写重复的请求/响应类型定义

**AC4: Given** 有新前端开发者加入项目
**When** 查阅项目内文档或示例代码
**Then** 可以看到一个"标准页面"示例, 展示路由、TanStack Query 和 ts-rs 类型结合使用的推荐模式

## AC ↔ Tasks 快速映射

- **AC1** → Task 2, Task 3
- **AC2** → Task 4, Task 5, Task 6
- **AC3** → Task 1, Task 6
- **AC4** → Task 5, Task 7, Task 8

## Tasks / Subtasks

### 0) 前置校验（结构与约定）

- [x] **Task 0: 前端项目结构核验** (AC: #1, #2, #4)
  > **说明**：确保现有结构与架构约定一致，避免后续返工。

  - [x] 0.1 确认 `frontend/src/features/`、`frontend/src/features/*/services/`、`frontend/src/features/*/hooks/` 目录存在
  - [x] 0.2 确认 `services/*Service.ts` 仅导出纯函数（无 React hooks）
  - [x] 0.3 确认 TanStack Query hooks 仅存在于 `features/*/hooks/`

### 后端任务（类型生成支持）

- [x] **Task 1: 验证并更新 ts-rs 后端集成** (AC: #3)
  > **说明**：ts-rs 需要在后端 Rust 结构体上添加 `#[derive(TS)]` 与 `#[ts(export)]` 以生成 TypeScript 类型定义。

  - [x] 1.1 在 `backend/Cargo.toml` 中确认 ts-rs 依赖版本
    - [x] 检查 `ts-rs` 是否已添加到依赖
    - [x] 如未添加，添加 `ts-rs = "10"`（与当前仓库版本对齐）
  - [x] 1.2 为核心 DTO 添加 `#[derive(TS)]` 和 `#[ts(export)]`
    - [x] 核心 DTO 列表（见 Dev Notes"现有 DTO 清单"）
    - [x] 为 `backend/src/api/response.rs` 中的 `ApiSuccess`、`ApiError`、`ErrorDetail`、`PaginationMeta` 添加 derive
    - [x] 为 `backend/src/domain/models/` 中已存在模型添加 derive（Workspace, User）
  - [x] 为 `backend/src/domain/models/` 中尚未实现的核心模型添加 derive（TestCase/OptimizationTask/Iteration/EvaluationResult/Checkpoint）
  - [x] 1.3 采用 **`backend/src/bin/gen-types.rs`** 作为唯一生成入口
    - [x] 新增 `backend/src/bin/gen-types.rs`，在其中定义导出清单与输出路径
    - [x] 输出目录：`frontend/src/types/generated/`（建议按 `api/`、`models/` 子目录分组）
  - [x] 1.4 测试类型生成
    - [x] 运行 `cargo run --bin gen-types` 生成类型
    - [x] 验证 `frontend/src/types/generated/` 下生成的文件结构与类型内容

### 前端任务（路由与数据层）

- [x] **Task 2: 建立 React Router 7.x 路由层级结构** (AC: #1)
  > **说明**：为三视图模式（Run View/Focus View/Workspace View）建立清晰的路由层级。

  - [x] 2.1 创建 `frontend/src/pages/RunView/` 目录
    - [x] 创建 `RunView.tsx` 页面组件
    - [x] 创建 `index.ts` 导出文件
  - [x] 2.2 创建 `frontend/src/pages/FocusView/` 目录
    - [x] 创建 `FocusView.tsx` 页面组件
    - [x] 创建 `index.ts` 导出文件
  - [x] 2.3 创建 `frontend/src/pages/WorkspaceView/` 目录
    - [x] 创建 `WorkspaceView.tsx` 页面组件
    - [x] 创建 `index.ts` 导出文件
  - [x] 2.4 在 `App.tsx` 中添加路由
    - [x] 添加 `/run` 路由 → RunView（默认视图）
    - [x] 添加 `/focus` 路由 → FocusView
    - [x] 添加 `/workspace` 路由 → WorkspaceView
    - [x] 保留现有 `/login` 和 `/settings/api` 路由
  - [x] 2.5 更新 `frontend/src/pages/index.ts`
    - [x] 导出新的页面组件
  - [x] 2.6 添加路由与视图切换测试（最小覆盖）
    - [x] 单元测试：验证 `/run`、`/focus`、`/workspace` 路由渲染
    - [x] E2E（可选）：验证快捷键切换视图

- [x] **Task 3: 添加视图切换器组件** (AC: #1)
  > **说明**：提供便捷的视图切换入口（顶栏切换器 + 快捷键）。

  - [x] 3.1 创建 `ViewSwitcher` 组件
    - [x] 位置：`frontend/src/components/common/ViewSwitcher.tsx`
    - [x] 显示三个视图选项（Run/Focus/Workspace）
    - [x] 高亮当前视图
  - [x] 3.2 添加快捷键支持
    - [x] Cmd/Ctrl + 1 → Run View
    - [x] Cmd/Ctrl + 2 → Focus View
    - [x] Cmd/Ctrl + 3 → Workspace View
    - [x] 通过 `event.metaKey`/`event.ctrlKey` 识别平台快捷键
    - [x] 在输入框/文本域聚焦时不触发（避免影响输入）
    - [x] 需要时 `event.preventDefault()` 避免浏览器默认行为
  - [x] 3.3 在主布局中集成 ViewSwitcher
    - [x] 在顶栏添加视图切换器
    - [x] 添加快捷键监听（useEffect）
  - [x] 3.4 添加 ViewSwitcher 组件测试（最小覆盖）
    - [x] 单元测试：验证当前视图高亮与快捷键切换逻辑

- [x] **Task 4: 完善 TanStack Query 服务层模式** (AC: #2)
  > **说明**：确保所有 API 调用通过 TanStack Query hooks，避免裸 fetch。

  - [x] 4.1 审计现有 API 调用
    - [x] 搜索 `frontend/src/` 中所有 `fetch` 调用
    - [x] 确认是否已有直接 fetch 使用
  - [x] 4.2 统一 Service 层模式
    - [x] 确认 `services/*Service.ts` 只导出纯函数（无 React hooks）
    - [x] 确认 `features/*/hooks/` 是 TanStack Query hooks 唯一位置
  - [x] 4.3 更新现有 hooks（如需要）
    - [x] 审计 `useApiConfig.ts`、`useWorkspaces.ts` 等现有 hooks
    - [x] 确保使用 `useQuery`、`useMutation` 等标准 hooks
    - [x] 确保正确处理 loading、error、data 状态
  - [x] 4.4 文档化最佳实践
    - [x] 在 `frontend/docs/FRONTEND_GUIDE.md` 中记录 TanStack Query 推荐模式
    - [x] 包含错误处理、重试、缓存策略

- [x] **Task 5: 创建标准页面示例** (AC: #4)
  > **说明**：为新开发者展示推荐的前端开发模式。

  - [x] 5.1 使用 `WorkspaceView` 作为标准页面示例
    - [x] 参考 `frontend/src/features/workspace/hooks/useWorkspaces.ts`（包含 `useCreateWorkspace`）的现有模式
  - [x] 5.2 实现标准页面示例
    - [x] 展示 `useQuery` 与 `useMutation` 的组合使用（引用现有 hooks）
    - [x] 展示 loading 和 error 状态的 UI 处理
    - [x] 展示 ts-rs 生成类型的导入与使用
  - [x] 5.3 添加注释说明
    - [x] 解释每个部分的职责
    - [x] 说明为什么这样做
    - [x] 提供可复制的代码片段

- [x] **Task 6: 完善类型定义与错误处理** (AC: #2, #3)
  > **说明**：确保前端类型与后端保持一致，错误处理统一。

  - [x] 6.1 集成 ts-rs 生成的类型
    - [x] 在 `frontend/src/types/index.ts` 中导出生成的类型
    - [x] 按 `generated/api/*`、`generated/models/*` 组织导出
    - [x] 更新 services 和 hooks 使用生成的类型（使用 `@/types/generated/*` 路径）
  - [x] 6.2 统一错误处理模式
    - [x] 确认 `lib/api.ts` 中的 `ApiResponse<T>` 与后端一致
    - [x] 确认所有 API 错误通过 `isApiError` 类型守卫检查
    - [x] 确认 UI 层统一显示 `error.message`（不显示 details）
  - [x] 6.3 添加类型安全测试
    - [x] 创建 `frontend/src/types/api.test.ts` 验证类型一致性
    - [x] 测试 ApiResponse 类型守卫

### 文档任务

- [x] **Task 7: 更新前端架构文档** (AC: #4)
  > **说明**：在适当位置记录前端架构规范和最佳实践。

  - [x] 7.1 更新项目 README 或创建 `frontend/README.md`
    - [x] 记录前端技术栈（React 19, React Router 7, TanStack Query 5, ts-rs）
    - [x] 记录项目结构（pages/features/components/stores/services）
    - [x] 记录路由约定
    - [x] 记录类型生成工作流（`cargo run --bin gen-types`）
  - [x] 7.2 创建 `frontend/docs/FRONTEND_GUIDE.md`
    - [x] 标准页面开发流程
    - [x] TanStack Query 使用指南
    - [x] ts-rs 类型生成流程
    - [x] 常见问题解答

- [x] **Task 8: 创建开发者快速上手指南** (AC: #4)
  > **说明**：为新开发者提供快速上手路径。

  - [x] 8.1 创建 `frontend/docs/ONBOARDING.md`
    - [x] 环境设置（Node 版本、依赖安装）
    - [x] 运行项目（npm run dev）
    - [x] 代码结构导读
    - [x] 推荐的开发顺序（先看哪个文件）
  - [x] 8.2 在代码中添加更多注释
    - [x] 在 `main.tsx` 中解释 Provider 设置
    - [x] 在 `lib/query-client.ts` 中解释 TanStack Query 配置
    - [x] 在 `lib/api.ts` 中解释 API 调用模式

## Dev Notes

### ⚠️ Guardrails（必须遵循）

- **React Router 7.x 规范**：使用 `<Routes>` 和 `<Route>` 定义路由，路由参数使用 `useParams()` 获取 [Source: docs/project-planning-artifacts/architecture.md#Frontend-Architecture]
- **TanStack Query 规范**：数据获取必须使用 `useQuery`，数据变更必须使用 `useMutation`，禁止在组件中直接调用 fetch/axios [Source: docs/project-planning-artifacts/architecture.md#State-Management]
- **ts-rs 类型同步**：后端 DTO 变更后必须运行 `cargo run --bin gen-types` 重新生成类型，确保前后端一致
- **服务层职责**：`services/*Service.ts` 只导出纯函数（无 React hooks），hooks 必须放在 `features/*/hooks/` 目录
- **错误处理规范**：前端不得直接展示 `error.details` 给用户，统一使用 `error.message` [Source: docs/project-planning-artifacts/architecture.md#Error-Handling-Layers]
- **三视图 UX 约束**：Run/Focus/Workspace 三视图均需存在且可切换；顶栏切换器 + `Cmd/Ctrl + 1/2/3` 快捷键；视图切换应保持画布与面板的关键状态 [Source: docs/project-planning-artifacts/ux-design-specification.md#Chosen-Direction-三视图模式架构]

#### 禁止事项
- **禁止**：在业务组件中直接使用 `fetch` 或 `axios` 调用 API
- **禁止**：在 Service 层使用 React hooks（包括 `useQuery`、`useMutation`）
- **禁止**：手写与后端 DTO 重复的 TypeScript 类型定义（应使用 ts-rs 生成）
- **禁止**：在 UI 中展示 `error.details` 内容

### 版本策略（依赖一致性）

- React Router 使用 `^7.0.0`（允许 7.x 补丁升级）
- TanStack Query 使用 `^5.0.0`
- 依赖版本以 `frontend/package.json` 与锁文件为准，避免随意改动

### 代码资产清单（已存在 vs 需新增）

**✅ 已存在（可直接复用）：**
| 资产 | 路径 | 说明 |
|------|------|------|
| React Router ^7.0.0 | `frontend/package.json` | 已安装，已在 main.tsx 配置 BrowserRouter |
| TanStack Query ^5.0.0 | `frontend/package.json` | 已安装，已在 query-client.ts 配置 |
| API 客户端 | `frontend/src/lib/api.ts` | `apiRequest<T>`、`ApiResponse<T>`、类型守卫 |
| 路由基础 | `frontend/src/App.tsx` | 基本路由结构已存在 |
| 基础页面 | `frontend/src/pages/` | HomePage, ApiConfigPage |
| Service 示例 | `frontend/src/features/*/services/` | authService, workspaceService, credentialService |

**🆕 需要新增：**
| 资产 | 路径 | 说明 |
|------|------|------|
| 三视图页面 | `frontend/src/pages/{Run,Focus,Workspace}View/` | Run View（默认）、Focus View、Workspace View |
| 视图切换器 | `frontend/src/components/common/ViewSwitcher.tsx` | 顶栏视图切换 + 快捷键 |
| ts-rs 配置 | `backend/Cargo.toml` + `backend/src/bin/gen-types.rs` | 后端 ts-rs 集成 |
| 生成的类型 | `frontend/src/types/generated/` | ts-rs 自动生成的类型定义 |
| 标准页面示例 | `frontend/src/pages/WorkspaceView/WorkspaceView.tsx` | 作为标准示例页面 |
| 前端文档 | `frontend/docs/*.md` | 架构指南、上手指南 |

### Project Structure Notes

**当前前端结构：**
```
frontend/src/
├── main.tsx                 # 应用入口，配置 Providers
├── App.tsx                  # 路由配置
├── lib/
│   ├── api.ts              # API 客户端（已完善）
│   └── query-client.ts     # TanStack Query 配置
├── pages/                   # 页面组件
│   ├── HomePage.tsx
│   └── ApiConfigPage.tsx
├── features/                # 业务功能模块
│   ├── api-config/
│   ├── auth/
│   └── workspace/
│       └── hooks/           # TanStack Query hooks
├── stores/                  # Zustand 全局状态
└── types/                   # TypeScript 类型
    └── api.ts              # ApiResponse 类型
```

**需要调整的结构：**
1. **三视图路由**：添加 `/run`、`/focus`、`/workspace` 路由
2. **类型生成**：添加 `types/generated/` 目录存放 ts-rs 生成的类型
3. **文档**：添加 `frontend/docs/` 存放架构文档

### 从前序故事继承的上下文

- ✅ **Story 1.1**：已定义 `ApiResponse<T>` 结构（后端），前端已同步类型定义
- ✅ **Story 1.6**：已实现本地用户认证，前端有 `useAuthStore` 和 `apiRequestWithAuth`
- ✅ **Story 1.8**：已统一错误响应结构，`ApiError` 类型包含 `code`、`message`、`details?`
- ✅ React Router ^7.0.0 已安装并配置
- ✅ TanStack Query ^5.0.0 已安装并配置
- ✅ 基础 API 客户端模式已建立（`lib/api.ts`）

### 现有 DTO 清单（需添加 ts-rs derive）

**backend/src/api/response.rs:**
- `ApiSuccess<T>`
- `ApiError`
- `ErrorDetail`
- `PaginationMeta`

**backend/src/domain/models/ (核心模型):**
- ✅ `Workspace`
- ✅ `User`
- ✅ `TestCase`
- ✅ `OptimizationTask`
- ✅ `Iteration`
- ✅ `EvaluationResult`
- ✅ `Checkpoint`

**backend/src/api/routes/ 中的请求/响应 DTO:**
- `HealthResponse`
- `TestDifyConnectionRequest`
- `TestGenericLlmConnectionRequest`
- `TestConnectionResult`
- `LoginRequest`
- `RegisterRequest`
- `AuthResponse`
- `UserInfo`
- `CreateWorkspaceRequest`
- `WorkspaceResponse`
- `DeleteWorkspaceResponse`

### ts-rs 集成参考

**统一入口：`backend/src/bin/gen-types.rs`**（唯一生成方式）

- 依赖版本：`ts-rs = "10"`（以 `backend/Cargo.toml` 为准）
- 输出目录：`frontend/src/types/generated/`（建议 `generated/api/`、`generated/models/` 分组）
- 生成命令：`cargo run --bin gen-types`

**结构体标注（示例）:**
```rust
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub user_id: i64,
    pub created_at: i64,
}
```

**gen-types.rs 导出清单（示例）:**
```rust
// 在 gen-types.rs 中集中导出已标注 TS 的类型
ts_rs::export! {
    Workspace => "../frontend/src/types/generated/models/workspace.ts",
}
```

### TanStack Query 推荐模式

以现有实现为准（避免二次发明）：

- `frontend/src/features/workspace/hooks/useWorkspaces.ts`：`useQuery` + `enabled` 认证条件，含 `useCreateWorkspace`/`useDeleteWorkspace`
- `frontend/src/features/workspace/services/workspaceService.ts`：Service 层统一 `apiRequestWithAuth` + `isApiError`
- 认证状态获取：优先在 hooks 内部使用 `useAuthStore`，避免页面层重复处理 token
- 缓存刷新策略：`useCreateWorkspace`/`useDeleteWorkspace` 成功后 `invalidateQueries`

### 标准页面示例

以 `WorkspaceView` 为标准示例，在页面中展示：

- 通过既有 hooks 获取数据与执行变更
- loading/error 的 UI 处理
- ts-rs 生成类型的导入与使用

### Git 历史参考

最近相关提交：
- `9563f70` fix(frontend): 补齐 QueryClientProvider，避免 API 配置页白屏
- `ac2781c` feat(auth): 完成 Story 1.6 本地用户认证与登录流
- `1857afc` feat(auth): 实现用户数据隔离和访问控制 (Story 1.7)
- `2105dfd` feat(backend): 实现 Story 1-8 统一错误响应和 OpenAPI 文档

**可复用模式：**
- `frontend/src/features/workspace/hooks/useWorkspaces.ts` — TanStack Query 标准用法
- `frontend/src/features/workspace/services/workspaceService.ts` — Service 层纯函数模式

### References

- [Source: docs/project-planning-artifacts/epics.md#Story-1.9] - Story 验收标准原文
- [Source: docs/project-planning-artifacts/architecture.md#Frontend-Architecture] - 前端架构决策
- [Source: docs/project-planning-artifacts/ux-design-specification.md#Chosen-Direction-三视图模式架构] - 三视图 UX 约束
- [Source: docs/implementation-artifacts/1-8-unified-error-response-and-openapi-docs.md] - 统一错误响应实现
- [Source: frontend/package.json] - 前端依赖版本
- [Source: frontend/src/lib/api.ts] - API 客户端实现
- [Source: frontend/src/lib/query-client.ts] - TanStack Query 配置
- [Source: frontend/src/features/workspace/hooks/useWorkspaces.ts] - 标准 hooks 模式
- [Source: frontend/src/features/workspace/services/workspaceService.ts] - 标准 service 模式
- [Source: ts-rs 文档](https://github.com/Aleph-Alpha/ts-rs) - ts-rs 官方文档

## Dev Agent Record

### Agent Model Used

GPT-5 (Codex CLI)

### Debug Log References

关键执行记录：
- `cargo run --bin gen-types`（生成 ts-rs 类型）
- `npm test -- --run`（前端单测）
- `npm run lint`（前端 lint）
- `cargo test`（后端测试）
- `cargo fmt --check`（后端格式校验）
- ts-rs 对 `serde(skip_serializing_if = "Option::is_none")` 的解析提示仅为警告，不影响运行
- `npm test -- --run`（通过；已消除 `--localstorage-file` 警告）
- `npm run build`（前端构建通过）

### Implementation Plan

1. 补齐后端 ts-rs 标注与生成入口，输出前端类型
2. 建立三视图路由与视图切换器，并补齐单测
3. 服务层/Hook 层统一使用生成类型与错误守卫
4. 完成 WorkspaceView 标准页面示例与文档补充
5. 运行前后端测试与 lint 校验

### Completion Notes List

- ✅ 完成 ts-rs 集成与类型生成入口，输出至 `frontend/src/types/generated/`
- ✅ 实现 `/run`、`/focus`、`/workspace` 路由与 ViewSwitcher（含快捷键与测试）
- ✅ 服务层/Hook 统一使用生成类型与 `isApiError` 守卫
- ✅ 完成 WorkspaceView 标准页面示例与前端文档/注释更新
- ✅ 测试与校验：`cargo test`、`npm test -- --run`、`npm run lint`
- ✅ Code Review Fix：移除 ts-rs 生成的别名类型导出，修复 TS 编译错误
- ✅ Code Review Fix：`PaginationMeta.total` 统一为 number 类型
- ✅ Code Review Fix：HealthCheck 使用 TanStack Query Hook，避免组件内直请求
- ✅ 补齐核心领域模型 DTO（TestCase/OptimizationTask/Iteration/EvaluationResult/Checkpoint）
- ✅ E2E：覆盖视图切换快捷键（Ctrl + 1/2/3）
- ✅ 核对 React Router 7 文档：Web 端组件/Hook 推荐从 `react-router` 导入（与当前实现一致）
- ✅ 测试环境注入内存版 localStorage，消除 MSW 触发的 `--localstorage-file` 警告
- ✅ CI 修复：补齐格式化 + 修复前端类型导出冲突/缺失

### File List

**新增文件：**
- `backend/src/bin/gen-types.rs`
- `backend/src/domain/models/algorithm.rs`
- `docs/implementation-artifacts/validation-report-20260101-131857.md`
- `docs/implementation-artifacts/1-9-frontend-architecture-and-type-safe-api-client.md`
- `frontend/docs/FRONTEND_GUIDE.md`
- `frontend/docs/ONBOARDING.md`
- `frontend/src/App.routes.test.tsx`
- `frontend/src/components/common/ViewSwitcher.test.tsx`
- `frontend/src/components/common/ViewSwitcher.tsx`
- `frontend/src/features/auth/hooks/index.ts`
- `frontend/src/features/health/hooks/useHealth.ts`
- `frontend/src/features/health/services/healthService.ts`
- `frontend/src/pages/FocusView/FocusView.tsx`
- `frontend/src/pages/FocusView/index.ts`
- `frontend/src/pages/RunView/RunView.tsx`
- `frontend/src/pages/RunView/index.ts`
- `frontend/src/pages/WorkspaceView/WorkspaceView.tsx`
- `frontend/src/pages/WorkspaceView/index.ts`
- `frontend/src/types/api.test.ts`
- `frontend/tests/e2e/view-switcher.spec.ts`
- `frontend/src/types/generated/api/ApiError.ts`
- `frontend/src/types/generated/api/ApiSuccess.ts`
- `frontend/src/types/generated/api/AuthResponse.ts`
- `frontend/src/types/generated/api/ConfigResponse.ts`
- `frontend/src/types/generated/api/CreateWorkspaceRequest.ts`
- `frontend/src/types/generated/api/CredentialInput.ts`
- `frontend/src/types/generated/api/DeleteWorkspaceResponse.ts`
- `frontend/src/types/generated/api/ErrorDetail.ts`
- `frontend/src/types/generated/api/GenericLlmCredentialInput.ts`
- `frontend/src/types/generated/api/HealthResponse.ts`
- `frontend/src/types/generated/api/index.ts`
- `frontend/src/types/generated/api/LoginRequest.ts`
- `frontend/src/types/generated/api/LogoutResponse.ts`
- `frontend/src/types/generated/api/PaginationMeta.ts`
- `frontend/src/types/generated/api/RegisterRequest.ts`
- `frontend/src/types/generated/api/SaveConfigRequest.ts`
- `frontend/src/types/generated/api/SaveConfigResponse.ts`
- `frontend/src/types/generated/api/SystemStatusResponse.ts`
- `frontend/src/types/generated/api/TeacherSettingsInput.ts`
- `frontend/src/types/generated/api/TeacherSettingsResponse.ts`
- `frontend/src/types/generated/api/TestConnectionResult.ts`
- `frontend/src/types/generated/api/TestDifyConnectionRequest.ts`
- `frontend/src/types/generated/api/TestGenericLlmConnectionRequest.ts`
- `frontend/src/types/generated/api/UserInfo.ts`
- `frontend/src/types/generated/api/WorkspaceResponse.ts`
- `frontend/src/types/generated/models/index.ts`
- `frontend/src/types/generated/models/Checkpoint.ts`
- `frontend/src/types/generated/models/ConflictResolutionRecord.ts`
- `frontend/src/types/generated/models/Constraint.ts`
- `frontend/src/types/generated/models/DataSplit.ts`
- `frontend/src/types/generated/models/DimensionScore.ts`
- `frontend/src/types/generated/models/EvaluationResult.ts`
- `frontend/src/types/generated/models/ExecutionResult.ts`
- `frontend/src/types/generated/models/FailurePoint.ts`
- `frontend/src/types/generated/models/Iteration.ts`
- `frontend/src/types/generated/models/IterationState.ts`
- `frontend/src/types/generated/models/LineageType.ts`
- `frontend/src/types/generated/models/OptimizationTask.ts`
- `frontend/src/types/generated/models/OutputLength.ts`
- `frontend/src/types/generated/models/QualityDimension.ts`
- `frontend/src/types/generated/models/Rule.ts`
- `frontend/src/types/generated/models/RuleConflict.ts`
- `frontend/src/types/generated/models/RuleConflictType.ts`
- `frontend/src/types/generated/models/RuleIR.ts`
- `frontend/src/types/generated/models/RuleMergeRecord.ts`
- `frontend/src/types/generated/models/RuleSystem.ts`
- `frontend/src/types/generated/models/RuleTags.ts`
- `frontend/src/types/generated/models/Severity.ts`
- `frontend/src/types/generated/models/TaskReference.ts`
- `frontend/src/types/generated/models/TestCase.ts`
- `frontend/src/types/generated/models/TokenUsage.ts`
- `frontend/src/types/generated/models/User.ts`
- `frontend/src/types/generated/models/Workspace.ts`
- `frontend/src/types/generated/serde_json/JsonValue.ts`

**删除文件：**
- `frontend/src/types/generated/api/LoginResponse.ts`
- `frontend/src/types/generated/api/TestConnectionRequest.ts`
- `frontend/src/types/generated/api/UserResponse.ts`
- `frontend/src/types/generated/api/WorkspaceCreateRequest.ts`
- `frontend/src/types/generated/api/WorkspaceListResponse.ts`

**修改文件：**
- `backend/Cargo.lock`
- `backend/Cargo.toml`
- `backend/src/api/response.rs`
- `backend/src/api/routes/auth.rs`
- `backend/src/api/routes/health.rs`
- `backend/src/api/routes/user_auth.rs`
- `backend/src/api/routes/workspaces.rs`
- `backend/src/domain/models/mod.rs`
- `backend/src/domain/models/user.rs`
- `backend/src/domain/models/workspace.rs`
- `backend/src/infra/external/dify_client.rs`
- `docs/implementation-artifacts/sprint-status.yaml`
- `frontend/src/components/HealthCheck.tsx`
- `frontend/src/types/generated/api/PaginationMeta.ts`
- `frontend/src/types/generated/api/index.ts`
- `frontend/README.md`
- `frontend/src/App.tsx`
- `frontend/src/features/api-config/hooks/useApiConfig.ts`
- `frontend/src/features/api-config/hooks/useApiConfig.test.ts`
- `frontend/src/features/api-config/hooks/useTestConnection.test.tsx`
- `frontend/src/features/api-config/hooks/useTestConnection.ts`
- `frontend/src/features/api-config/services/configService.test.ts`
- `frontend/src/features/api-config/services/configService.ts`
- `frontend/src/features/api-config/services/credentialService.test.ts`
- `frontend/src/features/api-config/services/credentialService.ts`
- `frontend/src/features/auth/services/authService.ts`
- `frontend/src/features/workspace/hooks/useWorkspaces.ts`
- `frontend/src/features/workspace/services/workspaceService.test.ts`
- `frontend/src/features/workspace/services/workspaceService.ts`
- `frontend/src/lib/api.ts`
- `frontend/src/lib/query-client.ts`
- `frontend/src/main.tsx`
- `frontend/src/pages/index.ts`
- `frontend/src/stores/useAuthStore.ts`
- `frontend/src/types/api.ts`
- `frontend/src/types/credentials.ts`
- `frontend/src/types/index.ts`
- `frontend/src/test/setup.ts`
- `frontend/tests/e2e/auth.spec.ts`

### Change Log

- 2026-01-01：完成 Story 1.9 的前端路由、ViewSwitcher、ts-rs 类型生成与文档补充
- 2026-01-02：Code Review Fix：修复 ts-rs 导出别名、PaginationMeta 类型、HealthCheck Query 化，回退未完成任务状态
- 2026-01-02：补齐核心领域模型 DTO，新增视图切换快捷键 E2E
- 2026-01-02：修复 useApiConfig 测试的 act(...) 警告
- 2026-01-02：核对 React Router 7 官方导入方式（react-router）
- 2026-01-02：测试环境注入内存 localStorage，消除 `--localstorage-file` 警告
- 2026-01-02：CI 修复（cargo fmt + 前端类型导出冲突修正）
- 2026-01-02：修复认证 E2E（前端路由内跳转，避免刷新导致内存态登录丢失）

## Review Notes

> 说明：补齐最小结构化 Review Notes，用于后续跨 story 检索与持续改进。历史执行证据以本文件的 Dev Agent Record（验证命令/文件清单）为准。

### Findings

- [LEGACY] 本 Story 当时未沉淀独立的结构化 review 结论；当前仅补齐统一结构。

### Decisions

- 无

### Risks / Tech Debt

- 无

### Follow-ups

- 无
