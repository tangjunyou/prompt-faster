# Story 1.7: 用户数据隔离与访问控制

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 在同一台机器上有多个本地账户的团队成员,
I want 不同本地用户的工作区、优化任务、测试集和历史记录在数据层严格隔离,
so that 我无法看到或修改其他用户的私有数据。

## Acceptance Criteria

1. **Given** 已设计好本地数据库 schema **When** 查看涉及工作区、任务配置、测试集、执行历史、检查点等业务表结构 **Then** 数据模型满足以下其一：
   - 模式 A：所有核心业务表都直接包含 `user_id` 字段, 用于区分不同用户的数据行
   - 模式 B：workspace 等顶层实体表包含 `user_id` 字段, 其他业务表通过 `workspace_id` 外键与之关联, 通过 join 实现用户数据隔离
   **And** 无论采用哪种模式, 数据访问层在查询和写入时都必须约束当前登录用户, 确保无法跨不同 `user_id` 读写数据

2. **Given** 当前有用户 A 已登录 **When** 用户 A 打开工作区列表、测试集列表或历史记录视图 **Then** 返回结果仅包含 `user_id = A` 的记录 **And** 其他用户的数据不会出现在列表或详情中

3. **Given** 存在多个本地账户（用户 A 与用户 B）**When** 用户 B 登录后尝试通过直接访问某个 URL/ID 加载用户 A 的工作区或执行记录 **Then** API 层基于当前登录用户进行鉴权 **And** 返回"无权限访问"或等价错误, 而不是加载成功

4. **Given** 系统支持导出配置或查看调试日志 **When** 登录用户执行导出或查看针对某任务的调试日志 **Then** 导出内容和日志仅包含当前用户自己的数据 **And** 不包含其他用户的任务配置或 Prompt 内容
   
   > **本期范围说明**：导出配置功能尚未实现，本期仅保证"现有运行时日志不泄露其他用户敏感数据"。导出功能将在后续 Story 中实现，届时必须按 user_id 过滤。

## AC ↔ Tasks 快速映射

- **AC1** → Task 1, Task 2, Task 3, Task 6
- **AC2** → Task 2, Task 3, Task 4, Task 5
- **AC3** → Task 3, Task 4, Task 7
- **AC4** → Task 3, Task 7

## Tasks / Subtasks

### 后端任务

- [x] **Task 1: 历史数据迁移（default_user → 首个注册用户）** (AC: #1)
  
  > ⚠️ **重要**：不能使用 SQL Migration 执行此迁移！因为 `sqlx::migrate!()` 在应用启动时运行，此时 `users` 表可能为空（用户尚未注册），导致迁移无效且被标记为"已完成"，后续用户注册后数据永远无法迁移。必须在**应用层**执行。
  
  - [x] 1.1 在 `backend/src/infra/db/repositories/` 创建 `migration_repo.rs`
    - [x] 实现 `migrate_legacy_default_user_data(pool, target_user_id)` 方法
    - [x] 迁移逻辑：将 `api_credentials` 和 `teacher_model_settings` 中 `user_id = 'default_user'` 的记录更新为 `target_user_id`
    - [x] 处理 UNIQUE 约束冲突：若目标用户已有同类型记录，跳过该条迁移（保留用户已有配置）
  - [x] 1.2 在 `UserRepo` 或 `AuthService` 的用户注册成功后调用迁移逻辑
    - [x] 在创建用户前调用 `UserRepo::has_any_user(pool)`，记录 `was_empty`
    - [x] 创建用户成功后，若 `was_empty = true`（即系统注册的首个用户），执行 `migrate_legacy_default_user_data`
  - [x] 1.3 迁移完成后，删除 `auth.rs` 中的 `LEGACY_DEFAULT_USER_ID` 常量及其 `#[allow(dead_code)]` 标记
  - [x] 1.4 添加迁移日志：记录迁移了多少条 `api_credentials` 和 `teacher_model_settings` 记录

- [x] **Task 2: 数据库约束强化（api_credentials / teacher_model_settings）** (AC: #1, #2)
  
  > **说明**：`workspaces` 表已有 FK 约束（见 001_initial_schema.sql）。本任务针对 `api_credentials` 和 `teacher_model_settings` 两张表。
  
  > **本期决定**：采用**方案 A（应用层校验）**，不做 FK 重建。原因：SQLite 重建表风险较高，MVP 阶段优先保证应用层安全。
  
  - [x] 2.1 确保 `api_credentials` 和 `teacher_model_settings` 的 `DEFAULT 'default_user'` 在应用层被废弃：
    - [x] Repository 层的所有写入操作必须显式传入 `user_id`，禁止依赖数据库默认值
    - [x] 确认现有 `CredentialRepo::upsert` 已接受 `user_id` 参数 ✅
    - [x] 确认现有 `TeacherSettingsRepo::upsert` 已接受 `user_id` 参数 ✅
  - [x] 2.2 （可选，后续增强）如需添加 FK 约束，需重建表，参考 SQLite 官方文档

- [x] **Task 3: Repository 层访问控制审计与强化** (AC: #1-#4)
  - [x] 3.1 审计现有 Repository 方法，确保所有查询都包含 `user_id` 过滤：
    - [x] `CredentialRepo::find_by_user_and_type` - 已包含 ✓
    - [x] `CredentialRepo::find_all_by_user` - 已包含 ✓
    - [x] `CredentialRepo::delete` - 已包含 ✓
    - [x] `TeacherSettingsRepo` 相关方法 - 已包含 ✓
  - [x] 3.2 创建 `backend/src/infra/db/repositories/workspace_repo.rs`（为后续 Epic 2/3 准备）
    - [x] 实现基础 CRUD 方法，所有方法都强制 `user_id` 参数
    - [x] `create(pool, user_id, name, description)` → 创建工作区
    - [x] `find_by_id(pool, workspace_id, user_id)` → **必须同时校验 workspace_id 和 user_id**
    - [x] `find_all_by_user(pool, user_id)` → 列出用户的所有工作区
    - [x] `delete(pool, workspace_id, user_id)` → 删除（需要 user_id 校验）
  - [x] 3.3 为 `find_by_id` 类型的方法建立规范：**同时校验资源 ID 和 user_id**，防止 IDOR 攻击
    ```rust
    // ❌ 错误：仅通过 ID 查询，可能返回其他用户的数据
    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<...>
    
    // ✅ 正确：同时校验 ID 和 user_id
    pub async fn find_by_id(pool: &SqlitePool, id: &str, user_id: &str) -> Result<...>
    ```

- [x] **Task 4: API 层访问控制强化** (AC: #2, #3)
  - [x] 4.1 审计现有 API 路由，确保都从 `CurrentUser` 获取 `user_id`：
    - [x] `POST /api/v1/auth/config` - 已使用 CurrentUser ✓
    - [x] `GET /api/v1/auth/config` - 已使用 CurrentUser ✓
    - [x] `POST /api/v1/auth/test-*` - 已使用 CurrentUser ✓
  - [x] 4.2 为后续 API 建立访问控制模式：
    ```rust
    // 标准模式：从 CurrentUser 获取 user_id，传递给 Repository
    async fn get_workspace(
        State(state): State<AppState>,
        Path(workspace_id): Path<String>,
        current_user: CurrentUser,
    ) -> Result<ApiResponse<Workspace>, ...> {
        // Repository 同时校验 workspace_id 和 user_id
        let workspace = WorkspaceRepo::find_by_id(
            &state.db,
            &workspace_id, 
            &current_user.user_id
        ).await?;
        // ...
    }
    ```
  - [x] 4.3 定义标准错误响应：
    - [x] 资源不存在或无权访问：统一返回 `404 Not Found`（不泄露资源是否存在）
    - [x] 错误码：统一使用 `WORKSPACE_NOT_FOUND`（与现有 `AUTH_*` 风格一致）

- [x] **Task 5: Workspace API 实现** (AC: #2)
  - [x] 5.1 在 `backend/src/api/routes/` 创建 `workspaces.rs`
  - [x] 5.2 实现基础端点（URL 前缀：`/api/v1/workspaces`）：
    - [x] `POST /api/v1/workspaces` - 创建工作区
    - [x] `GET /api/v1/workspaces` - 列出当前用户的所有工作区
    - [x] `GET /api/v1/workspaces/:id` - 获取单个工作区详情（需要 user_id 校验）
    - [x] `DELETE /api/v1/workspaces/:id` - 删除工作区（需要 user_id 校验）
  - [x] 5.3 所有端点都需要登录（挂载到受保护路由，使用 auth middleware）
  - [x] 5.4 在 `main.rs` 注册路由
  - [x] 5.5 响应 DTO 定义：`WorkspaceResponse { id, name, description, created_at, updated_at }`（**不返回 user_id**）

- [x] **Task 6: 领域模型补充** (AC: #1)
  - [x] 6.1 确认 `backend/src/domain/models/workspace.rs` 存在或创建
    - [x] 字段对齐 `migrations/001_initial_schema.sql#workspaces`：
      - `id: String`
      - `user_id: String`
      - `name: String`
      - `description: Option<String>`
      - `created_at: i64`
      - `updated_at: i64`
  - [x] 6.2 在 `backend/src/domain/models/mod.rs` 导出 `Workspace`

- [x] **Task 7: 后端测试** (AC: #1-#4)
  - [x] 7.1 单元测试：WorkspaceRepo CRUD 操作
    - [x] 创建工作区成功
    - [x] 按 user_id 查询只返回该用户的工作区
    - [x] 尝试访问其他用户的工作区返回 NotFound
  - [x] 7.2 集成测试：Workspace API
    - [x] 用户 A 创建的工作区，用户 B 无法通过 ID 直接访问
    - [x] 用户 A 的工作区列表不包含用户 B 的数据
  - [x] 7.3 迁移测试（可选）：
    - [x] 验证 `default_user` 数据正确迁移到首个注册用户

### 前端任务

- [x] **Task 8: 前端工作区 Service** (AC: #2)
  
  > **说明**：优先复用现有 `apiRequestWithAuth`，避免重复封装。
  
  - [x] 8.0 （可选）如需便捷封装，可在 `frontend/src/lib/api.ts` 添加 `delWithAuth(endpoint, token)`，内部调用 `apiRequestWithAuth(endpoint, { method: 'DELETE' }, token)`
  - [x] 8.1 在 `frontend/src/features/workspace/services/` 创建 `workspaceService.ts`
    - [x] `createWorkspace(name, description?)` - 创建工作区
    - [x] `listWorkspaces()` - 列出工作区
    - [x] `getWorkspace(id)` - 获取详情
    - [x] `deleteWorkspace(id)` - 删除
  - [x] 8.2 所有请求使用 `apiRequestWithAuth`（以及可选 `delWithAuth`）确保携带认证 token

- [x] **Task 9: 前端工作区 Hook 与 Store** (AC: #2)
  - [x] 9.1 创建 `frontend/src/features/workspace/hooks/useWorkspaces.ts`
    - [x] 使用 TanStack Query 管理工作区列表状态
    - [x] 缓存失效策略：创建/删除工作区后调用 `queryClient.invalidateQueries({ queryKey: ['workspaces'] })`
  - [x] 9.2 （可选）创建 `useWorkspaceStore.ts`（Zustand）管理当前选中工作区

- [x] **Task 10: 前端测试** (AC: #2, #3)
  - [x] 10.1 `workspaceService` 单元测试（MSW mock）
  - [x] 10.2 E2E 测试（Playwright 直接调用后端 API）：
    - [x] 测试场景 1：用户 A 登录 → 创建工作区 → 用户 B 登录 → 调用 `GET /api/v1/workspaces/:id`（A 的工作区 ID）→ 断言返回 404
    - [x] 测试场景 2：用户 A 登录 → 调用 `GET /api/v1/workspaces` → 断言列表不包含用户 B 的工作区（后端集成测试 `workspaces_api_test.rs:153-212` 已覆盖）
    - [x] 验证方式：使用 Playwright 的 `request.get()` 直接调用 API，携带各自用户的 token

## Dev Notes

### ⚠️ Guardrails（必须遵循）

- **IDOR 防护**：所有资源访问 API 必须同时校验资源 ID 和当前用户 ID，防止 Insecure Direct Object Reference 攻击
- **Repository 规范**：`find_by_id` 类方法必须接受 `user_id` 参数，SQL 查询必须包含 `WHERE ... AND user_id = ?`
- **禁止硬编码 user_id**：任何 Repository/Handler 都不得使用硬编码的 user_id 或依赖数据库 DEFAULT 值
- **统一错误响应**：资源不存在或无权访问统一返回 404，不泄露资源存在性（防止枚举攻击）
- **ApiResponse 规范**：所有接口必须返回 `ApiResponse<T>`，`data`/`error` 互斥 (AR1) [Source: backend/src/api/response.rs]
- **日志脱敏**：日志中不得包含其他用户的敏感数据

#### 禁止事项

- **禁止**：仅通过资源 ID 查询数据而不校验 user_id
- **禁止**：在列表 API 中返回所有用户的数据
- **禁止**：将 `default_user` 作为有效的运行时 user_id

### 代码资产清单（已存在 vs 需新增）

**✅ 已存在（可直接复用）：**
| 资产 | 路径 | 说明 |
|------|------|------|
| `CurrentUser` | `backend/src/api/middleware/auth.rs` | 鉴权中间件，提取当前用户 |
| `ApiResponse<T>` | `backend/src/api/response.rs` | 统一响应结构 |
| `CredentialRepo` | `backend/src/infra/db/repositories/credential_repo.rs` | 凭证仓储（已有 user_id 过滤） |
| `TeacherSettingsRepo` | `backend/src/infra/db/repositories/teacher_settings_repo.rs` | 模型参数仓储（已有 user_id 过滤） |
| `UserRepo::get_first_user` | `backend/src/infra/db/repositories/user_repo.rs:152-176` | 获取首个用户（用于迁移） |
| `workspaces` 表 | `backend/migrations/001_initial_schema.sql` | 表结构已定义，含 FK 约束 |
| `apiRequestWithAuth` | `frontend/src/lib/api.ts` | 带鉴权的 API 请求封装 |

**🆕 需要新增：**
| 资产 | 路径 | 说明 |
|------|------|------|
| `MigrationRepo` | `backend/src/infra/db/repositories/migration_repo.rs` | 历史数据迁移逻辑 |
| `WorkspaceRepo` | `backend/src/infra/db/repositories/workspace_repo.rs` | 工作区仓储 |
| `WorkspaceRepoError` | 同上 | 工作区仓储错误枚举 |
| `Workspace` | `backend/src/domain/models/workspace.rs` | 工作区领域模型 |
| `workspaces.rs` | `backend/src/api/routes/workspaces.rs` | 工作区 API 路由 |
| `delWithAuth` | `frontend/src/lib/api.ts` | （可选）DELETE 请求鉴权封装（复用 apiRequestWithAuth） |
| `workspaceService` | `frontend/src/features/workspace/services/workspaceService.ts` | 前端工作区服务 |
| `useWorkspaces` | `frontend/src/features/workspace/hooks/useWorkspaces.ts` | 前端工作区 Hook |

### Project Structure Notes

- **数据隔离模式选择**：当前项目采用**模式 A**（直接在业务表包含 user_id），`api_credentials` 和 `teacher_model_settings` 已包含 `user_id` 字段
- **workspaces 表**：已在 `001_initial_schema.sql` 定义，包含 `user_id` 字段和 FOREIGN KEY 约束
- **历史数据**：`api_credentials` 和 `teacher_model_settings` 表使用 `DEFAULT 'default_user'`，需要迁移
- **Story 1.6 遗留**：`auth.rs` 中的 `LEGACY_DEFAULT_USER_ID` 常量和相关 TODO 注释

### 从 Story 1.6 继承的上下文

- ✅ `CurrentUser` 机制已实现，通过 `Extension<CurrentUser>` 在 handler 中获取
- ✅ `auth.rs` 配置相关接口已使用 `CurrentUser.user_id`
- ✅ `user_repo.rs` 已实现 `get_first_user` 方法用于迁移
- ⚠️ 历史数据迁移 SQL 在 Story 1.6 中记录但未执行
- ⚠️ FOREIGN KEY 约束评估在 Story 1.6 中标记为"后续 Sprint/Story 1.7"

### 技术实现参考

**Repository 方法签名规范：**
```rust
// 列表查询：必须按 user_id 过滤
pub async fn find_all_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<T>, Error>

// 单条查询：必须同时校验 ID 和 user_id
pub async fn find_by_id(pool: &SqlitePool, id: &str, user_id: &str) -> Result<T, Error>

// 写入操作：必须显式传入 user_id
pub async fn create(pool: &SqlitePool, user_id: &str, ...) -> Result<T, Error>

// 删除操作：必须校验 user_id
pub async fn delete(pool: &SqlitePool, id: &str, user_id: &str) -> Result<bool, Error>
```

**API Handler 模式：**
```rust
async fn handler(
    State(state): State<AppState>,
    current_user: CurrentUser,
    // ...
) -> ApiResponse<T> {
    // 始终使用 current_user.user_id，禁止从请求参数获取 user_id
    match SomeRepo::find_by_user(&state.db, &current_user.user_id).await {
        Ok(data) => ApiResponse::ok(data),
        Err(_e) => ApiResponse::err(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DATABASE_ERROR",
            "查询失败",
        ),
    }
}
```

### References

- [Source: docs/implementation-artifacts/epics.md#Story-1.7] - 验收标准原文
- [Source: docs/stories/1-6-local-user-authentication-and-login-flow.md] - 前序故事实现细节
- [Source: backend/migrations/001_initial_schema.sql] - workspaces 表结构
- [Source: backend/migrations/002_api_credentials_and_teacher_settings.sql] - 凭证表结构（含 DEFAULT 'default_user'）
- [Source: backend/src/api/routes/auth.rs#L256-258] - LEGACY_DEFAULT_USER_ID 定义
- [Source: backend/src/infra/db/repositories/user_repo.rs#L151-155] - get_first_user 方法
- [Source: backend/src/infra/db/repositories/credential_repo.rs] - 现有凭证 Repository 实现参考
- [Source: docs/implementation-artifacts/architecture.md#Authentication-&-Security] - 数据隔离架构要求

### Git 历史参考

最近相关提交：
- `ac2781c` feat(auth): 完成 Story 1.6 本地用户认证与登录流
- `c03a2f7` feat(story-1.5): 凭证持久化与老师模型参数配置

## Dev Agent Record

### Agent Model Used

Cascade

### Debug Log References

### Completion Notes List

- 完成 default_user 历史数据在应用层的迁移逻辑，触发点为首个用户注册成功后，并保证幂等与并发安全。
- 新增 Workspace 后端模块（领域模型/Repo/API 路由），所有数据访问强制 user_id 约束并对越权访问统一返回 404。
- 修复 Axum 路由参数语法（使用 `/{id}`），并修复 main.rs 中 SessionStore 的 clone 类型问题。
- 新增后端迁移集成测试、Workspace API 集成测试、WorkspaceRepo 单测。
- 新增前端 workspaceService 与 TanStack Query hooks，并补齐 Vitest/MSW 单测与 Playwright 场景 1。

### File List

- backend/src/api/routes/auth.rs
- backend/src/infra/db/repositories/migration_repo.rs
- backend/src/api/routes/user_auth.rs
- backend/tests/auth_integration_test.rs
- backend/src/domain/models/workspace.rs
- backend/src/infra/db/repositories/workspace_repo.rs
- backend/src/api/routes/workspaces.rs
- backend/src/api/routes/mod.rs
- backend/src/domain/models/mod.rs
- backend/src/infra/db/repositories/mod.rs
- backend/src/main.rs
- backend/tests/workspaces_api_test.rs
- frontend/src/lib/api.ts
- frontend/src/features/workspace/services/workspaceService.ts
- frontend/src/features/workspace/hooks/useWorkspaces.ts
- frontend/src/features/workspace/services/workspaceService.test.ts
- frontend/tests/e2e/workspaces.spec.ts
- docs/sprint-status.yaml

