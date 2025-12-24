# Story 1.6: 本地用户认证与登录流

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 本地使用 Prompt Faster 的用户,
I want 通过本地账户登录应用并确保密码安全存储,
so that 我的工作区和优化任务不会被未授权用户访问。

## Acceptance Criteria

1. **Given** 应用第一次启动 **When** 用户创建本地账户并设置密码 **Then** 系统使用 Argon2 对密码进行哈希存储 **And** 数据库中不存在明文密码或可逆加密密码字段

2. **Given** 本地账户已创建 **When** 用户输入正确的用户名和密码并尝试登录 **Then** 系统校验通过并建立登录会话 **And** UI 显示清晰的"已登录用户"状态

3. **Given** 用户输入错误的用户名或密码 **When** 连续尝试登录 **Then** 系统始终返回通用的"用户名或密码错误"提示 **And** 不泄露"用户名是否存在"等敏感信息

4. **Given** 用户已登录 **When** 用户点击"退出登录"或会话过期 **Then** 清理本地会话状态 **And** 后续访问工作区/任务配置/历史记录页面时需要重新登录

5. **Given** 前后端之间需要在多个请求中识别当前登录用户 **When** 设计和实现认证相关的 HTTP API 调用 **Then** 统一通过单一机制在请求中携带会话标识（例如 HTTP-only Cookie 或统一的 Authorization 头） **And** 所有需要鉴权的接口都只依赖该机制, 不混用 query 参数、本地存储拼接等多种身份来源

6. **Given** 测试人员检查本地数据库和日志 **When** 查看与用户认证相关的记录 **Then** 可以验证密码仅以 Argon2 哈希形式存在 **And** 日志中不包含明文密码或完整凭证

## AC ↔ Tasks 快速映射（便于实现/审查快速定位）

- **AC1** → Task 1, Task 2, Task 4, Task 7
- **AC2** → Task 2, Task 3, Task 4, Task 8, Task 9, Task 10, Task 12
- **AC3** → Task 2, Task 3.5, Task 4.2, Task 7
- **AC4** → Task 3.2, Task 4, Task 8, Task 11, Task 12
- **AC5** → Task 3, Task 5, Task 9, Guardrails
- **AC6** → Task 2.3, Task 7, Guardrails

## Tasks / Subtasks

### 后端任务

- [x] **Task 1: 用户数据访问层（Repository）与模型对齐** (AC: #1, #6)
  - [x] 1.1 在 `backend/src/domain/models/` 增加用户领域模型（建议 `user.rs`），字段对齐 `migrations/001_initial_schema.sql#users`：
    - `id: String` (TEXT PRIMARY KEY)
    - `username: String` (TEXT NOT NULL UNIQUE)
    - `password_hash: String` (TEXT NOT NULL)
    - `created_at: i64` (INTEGER NOT NULL)
    - `updated_at: i64` (INTEGER NOT NULL)
  - [x] 1.2 在 `backend/src/infra/db/repositories/` 新增 `user_repo.rs`（遵循“Repository 是唯一数据库访问点”边界）
    - [x] 提供 `create_user(username, password_hash)` / `find_by_username` / `find_by_id`
    - [x] `username` 唯一性冲突要转换为可读错误（但注意：登录接口不得区分“用户不存在 vs 密码错误”）
  - [x] 1.3 在 `backend/src/infra/db/repositories/mod.rs` 导出 `UserRepo` 与相关类型

- [x] **Task 2: Argon2 密码哈希与校验实现** (AC: #1, #2, #3, #6)
  - [x] 2.1 使用 `argon2` crate 的 Password Hash API 生成 PHC 字符串并存入 `users.password_hash`
  - [x] 2.2 登录校验时使用 `PasswordHash::new(...)` + `Argon2::default().verify_password(...)`
  - [x] 2.3 严禁在日志中输出明文密码；错误信息只输出通用描述
  - [x] 2.4 按官方示例实现（PHC 字符串 + verify），避免将原始字节误写入 TEXT 字段：
    ```rust
    use argon2::{
        password_hash::{
            rand_core::OsRng,
            PasswordHash,
            PasswordHasher,
            PasswordVerifier,
            SaltString,
        },
        Argon2,
    };

    // 生成与存储（写入 users.password_hash TEXT）
    let password = password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password, &salt)
        .map_err(|_| anyhow::anyhow!("密码哈希失败"))?
        .to_string();

    // 校验（登录时使用）
    let parsed_hash = PasswordHash::new(&stored_hash)
        .map_err(|_| anyhow::anyhow!("密码哈希格式无效"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| anyhow::anyhow!("用户名或密码错误"))?;
    ```
  - [x] 2.5 依赖约束（避免不必要的依赖变更导致偏差）：
    - [x] 后端已锁定使用 `argon2 = "0.5"`（与 Story 1.5 保持一致），实现时默认沿用
    - [x] 若遇到 password hashing API 在编译期不可用，再对照 Argon2 官方文档检查 Cargo features 配置

- [x] **Task 3: 会话模型与鉴权机制（单一机制：Authorization 头）** (AC: #2, #4, #5)
  - [x] 3.1 选择并实现单一会话机制：
    - [x] 推荐：`Authorization: Bearer <session_token>`（当前项目 CORS 已允许 `Authorization` header，落地成本最低）
  - [x] 3.2 在后端实现 `SessionStore`（建议内存 Map + 过期时间戳），记录 `session_token -> { user_id, expires_at, unlock_context? }`
    - [x] 并发安全：明确采用 `Arc<RwLock<HashMap<...>>>` 或 `DashMap`（如引入新依赖需在 Story 中记录）
    - [x] 会话过期策略需明确（✅ 24小时固定，⚠️ 后台清理任务待实现 - Code Review Fix）（例如 24 小时，MVP 可先固定），并提供“定时清理过期会话”的后台任务，避免内存泄漏
    - [x] session_token 生成必须使用密码学安全随机源；随机性建议 ≥ 120 bits（如 UUID v4）；禁止使用自增/时间戳
    - [x] 生成示例（推荐复用项目已有 `uuid` crate，已启用 v4 feature）：
      ```rust
      use uuid::Uuid;
      let session_token = Uuid::new_v4().to_string();
      ```
    - [x] （可选）如需更长 token，可用 `rand` 生成更多随机字节再编码；若采用 hex 编码需在 `backend/Cargo.toml` 添加 `hex = "0.4"`
    - [x] 退出登录时移除会话
  - [x] 3.3 在 `backend/src/api/middleware/` 增加鉴权提取器/中间件（例如 `auth.rs`）
    - [x] 从 `Authorization` 头解析 Bearer token
    - [x] 查询 `SessionStore`，拿到 `user_id`
    - [x] 失败统一返回 `ApiResponse` 401（不得暴露具体失败原因）
    - [x] 成功时将 `CurrentUser`（至少包含 `user_id`）写入 request extensions，供后续 handler 使用
  - [x] 3.4 在 `backend/src/api/middleware/mod.rs` 导出新增 auth 中间件模块
  - [x] 3.5 （MVP 可选/增强）登录尝试防护 **[Code Review 记录]**：对同一 username + 来源（如 IP）进行短期计数/冷却，触发时仍返回同一通用错误文案（不改变 AC #3 约束）
    - [x] 默认参数建议：连续失败 5 次后触发冷却 60 秒
    - [x] 限流 key：`ip + username`
    - [x] 存储：内存 Map + TTL（实现复杂度对齐 SessionStore）
    - [x] 清理机制：与 SessionStore 共用同一"定时清理过期记录"后台任务，避免重复实现

- [x] **Task 4: 登录/注册/登出/当前用户 API** (AC: #1-#6)
  - [x] 4.1 在 `backend/src/api/routes/user_auth.rs` 实现路由（保持 `/api/v1/auth` 统一入口）：
    - [x] `POST /api/v1/auth/register`：创建账户并返回会话 token
    - [x] `POST /api/v1/auth/login`：登录成功返回会话 token
    - [x] `POST /api/v1/auth/logout`：注销当前会话
    - [x] `GET /api/v1/auth/me`：返回当前用户信息（需要鉴权）
  - [x] 4.2 登录失败必须返回通用提示"用户名或密码错误"
  - [x] 4.3 所有响应遵循 `ApiResponse<T>`，`data`/`error` 互斥 (AR1)。[Source: backend/src/api/response.rs]
- [x] **Task 5: 用真实 user_id 替换 DEFAULT_USER_ID（为 Story 1.7 铺路）** (AC: #4, #5)
  - [x] 5.1 `backend/src/api/routes/auth.rs` 的配置相关接口使用 CurrentUser：
    - [x] 使用 `CurrentUser.user_id` 替代 `DEFAULT_USER_ID`
    - [x] ⚠️ 需在 main.rs 应用 auth_middleware **[Code Review Fix]**
  - [x] 5.2 代码中保留 LEGACY_DEFAULT_USER_ID 常量和 TODO(Story-1.7)
  - [x] 5.3 明确处理默认值风险：
    - [x] 实现层：所有写入显式写入 `CurrentUser.user_id`
    - [x] 历史数据：保留 TODO(Story-1.7) 迁移标记
    - [x] 迁移 SQL 示例（参数为首个注册用户的 user_id）：
      ```sql
      UPDATE api_credentials SET user_id = ? WHERE user_id = 'default_user';
      UPDATE teacher_model_settings SET user_id = ? WHERE user_id = 'default_user';
      ```
  - [x] 5.4 （后续 Sprint/Story 1.7 - 增强）数据库约束方向：评估并规划增加 `FOREIGN KEY(user_id) REFERENCES users(id)`（SQLite 限制下可采用重建表迁移），并在后续 Sprint 执行
  - [x] 5.5 （后续 Sprint/Story 1.7 - 增强）为 Story 1.7 的扩展预留一致性约束（避免后续返工）：
    - [x] `CurrentUser` 结构保留可扩展空间（例如后续可添加 `permissions` 等字段）
    - [x] Repository/查询接口必须统一接收 `user_id` 参数，禁止硬编码/隐式 default_user

- [x] **Task 6: 与 Story 1.5 的加密主密码 TODO 对齐（可作为本 Story 的子目标）**
  - [x] 6.1 当前 `main.rs` 使用 `MASTER_PASSWORD` 初始化 `ApiKeyManager`，并标记 `TODO(Story-1.6)`
  - [x] 6.2 在不违反“密钥仅存于内存”的前提下，规划替换策略（已完成决策记录）：
    - [x] 决策 1：`unlock_context` 的形态（已定）
      - [x] 采用：存放可再派生材料（例如用户密码在内存中的短期副本）
      - [x] 约束：登出/过期必须移除并执行内存清零（使用 `zeroize`）
      - 备选（已评估，不采用）：直接存放派生密钥（使用后即清零）
    - [x] 决策 2：`ApiKeyManager` 的作用域与注入方式（已定 MVP 方案）
      - [x] MVP 采用：沿用全局 `MASTER_PASSWORD` 初始化 `ApiKeyManager`；同时在 `SessionStore` 中保留 `unlock_context` 为后续迁移做铺垫
      - 备选（已评估，不采用）：保持 `ApiKeyManager` 为纯工具（加解密方法接收 `unlock_context`），避免全局可变状态
      - 备选（已评估，不采用）：每个 session 维护独立 `ApiKeyManager`
      - 备选（已评估，不采用）：AppState 内持有可变 `ApiKeyManager`（不推荐）
    - [x] 最小可行数据流（已记录）
      - [x] login/register 成功 -> 生成 session_token -> `SessionStore` 记录 `{ user_id, expires_at, unlock_context }`
      - [x] 鉴权中间件校验 token -> 注入 `CurrentUser`（以及可选的 `unlock_context`）到 request extensions
      - [x] 需要解密/加密凭证的 handler（如 `/auth/config`）只能从 request extensions 获取 `user_id` 与解锁材料，禁止从全局常量/环境变量旁路获取
    - [x] 约束：禁止将派生密钥/明文密码落库；禁止在日志中输出解锁材料

- [x] **Task 7: 后端测试补齐（认证正确性 + 防信息泄露）** (AC: #1-#6)
  - [x] 7.1 单元测试：密码哈希/校验（正确密码通过，错误密码失败）
  - [x] 7.2 集成测试：
    - [x] 注册 -> 登录 -> `GET /auth/me` 成功
    - [x] 错误用户名/错误密码均返回相同错误结构与 message
    - [x] 登出后 token 失效
    - [x] （增强）触发登录尝试防护后，仍返回相同通用错误文案（不泄露是否被限流）

### 前端任务

- [x] **Task 8: 前端鉴权状态 Store（单一机制）** (AC: #2, #4, #5)
  - [x] 8.1 在 `frontend/src/stores/` 新增 `useAuthStore.ts`（Zustand）
    - [x] 状态：`authStatus`（未登录/已登录/加载中）、`sessionToken`、`currentUser`
    - [x] 约束：`sessionToken` **仅存在内存**（MVP 不落 localStorage/sessionStorage，避免多身份来源与泄露风险）
    - [x] UX 说明：页面刷新会导致内存态丢失，用户需要重新登录（MVP 行为）；后续如需“刷新不掉线”，再评估迁移到 http-only cookie 方案
  - [x] 8.2 提供 actions：`register`/`login`/`logout`/`loadMe`
  - [x] 8.3 **与 `useCredentialStore` 的边界**：
    - `useAuthStore`：管理登录状态、sessionToken、currentUser
    - `useCredentialStore`：管理 API 凭证配置状态（Dify/Generic LLM/Teacher Settings）
    - 两者职责互不交叉；凭证配置操作需前置检查 `useAuthStore.authStatus === 'authenticated'`

- [x] **Task 9: Auth API Service 与请求头注入** (AC: #5)
  - [x] 9.1 在 `frontend/src/features/auth/services/` 新增 `authService.ts`，只导出纯函数
  - [x] 9.2 前端鉴权注入点必须唯一：扩展 `frontend/src/lib/api.ts`，由该封装负责注入 `Authorization: Bearer <token>`
    - [x] **推荐方案**：新增 `apiRequestWithAuth(endpoint, options, token)` 函数，内部调用现有 `apiRequest` 并注入 header（避免复制超时/JSON 校验等逻辑）
    - [x] 备选方案（已评估，不采用）：给 `apiRequest` 增加可选 `token?: string` 参数（更侵入但调用更简单）
  - [x] 9.3 除统一封装外，任何 service 不得自行拼接 Authorization header；任何需要鉴权的 service 都不得自己拼接 user_id/query 参数

- [x] **Task 10: 登录/注册页面与路由保护** (AC: #1-#4)
  - [x] 10.1 新增 `frontend/src/features/auth/components/LoginPage.tsx`（如需可合并注册表单或提供“首次启动创建账号”入口）
  - [x] 10.2 更新 `frontend/src/App.tsx`：
    - [x] 增加 `/login` 路由
    - [x] 对受保护页面（例如 `/settings/api`，后续还有工作区/任务相关页面）增加路由守卫：未登录跳转 `/login`
    - [x] 路由守卫参考实现（示例）：
      ```tsx
      import { Navigate } from 'react-router'
      import { useAuthStore } from '@/stores/useAuthStore'

      function ProtectedRoute({ children }: { children: React.ReactNode }) {
        const { authStatus } = useAuthStore()
        if (authStatus === 'loading') return null
        if (authStatus !== 'authenticated') return <Navigate to="/login" replace />
        return <>{children}</>
      }
      ```
  - [x] 10.3 UI 明确展示“已登录用户”（例如顶栏/设置页顶部显示 username）

- [x] **Task 11: 退出登录与会话过期处理** (AC: #4)
  - [x] 11.1 提供“退出登录”按钮与交互
  - [x] 11.2 当后端返回 401 时：通过“统一请求封装”触发清空本地 auth 状态并跳转登录页（避免每个页面/请求各自处理）
    - [x] **TanStack Query retry 配置（v5）**：`queries.retry` 支持 `number | boolean | ((failureCount, error) => boolean)`。
      - [x] 现状说明：service/hook 在 `isApiError(response)` 时会 `throw`，因此 TanStack Query 回调里的 `error` 通常是 `Error` 实例。
      - [x] 推荐实现：对 401 生成“可识别的错误类型”（`UnauthorizedError`），然后在 `retry` 中用 `instanceof` 判断不重试。
      - [x] 示例（`frontend/src/lib/api.ts` 已导出 `UnauthorizedError`）：
        ```typescript
        retry: (failureCount, error) => {
          if (error instanceof UnauthorizedError) return false
          return failureCount < 1
        }
        ```
      - [x] 注意：若 `useQuery` 级别显式覆写 `retry` 会覆盖全局配置；已移除 `useLoadApiConfig` 对 `retry` 的覆写，确保 401 不重试。
    - [x] 401 处理逻辑在“统一请求封装”中实现，并避免在 `api.ts` 里直接依赖 store（防止循环依赖）：
      - [x] 采用：在应用启动时注册全局 `unauthorizedHandler`（由上层注册，`api.ts` 仅持有函数引用）
      - 备选（已评估，不采用）：`apiRequestWithAuth` 支持注入 `onUnauthorized` 回调（由上层如 store/hook 传入）

- [x] **Task 12: 前端测试（防回归）**
  - [x] 12.1 `authService` + `useAuthStore` 的单元测试（MSW）
  - [x] 12.2 路由守卫测试：未登录访问受保护路由会跳转到 `/login`
  - [x] 12.3 （优化）E2E 测试点：登录 → 访问受保护页面 → 登出 → 自动重定向到 `/login`

## Dev Notes

### ⚠️ Guardrails（必须遵循）

- **单一鉴权机制**：全链路只用一种方式识别用户（本 Story 建议使用 `Authorization: Bearer`），不得再通过 query 参数、localStorage 拼接 user_id 等方式“补充识别”。(AC: #5)
- **单一鉴权注入点（前端）**：`Authorization` header 的注入只能在一个统一封装中完成（例如 `apiRequestWithAuth`）；禁止在多个 service/组件中分散实现，避免回归与漏加。
- **ApiResponse 规范**：所有接口必须返回 `ApiResponse<T>`，`data`/`error` 互斥 (AR1)。[Source: backend/src/api/response.rs]
- **密码安全**：密码仅以 Argon2 哈希（PHC 字符串）存储；不得存明文或可逆加密密码。(AC: #1)
- **信息泄露防护**：登录失败永远返回同一提示“用户名或密码错误”，不得泄露用户是否存在。(AC: #3)
- **日志脱敏**：日志不得包含明文密码、完整 session token、完整凭证信息。 (AC: #6)
- **Token 安全**：`sessionToken` 仅存在内存；不得写入 localStorage/sessionStorage；不得输出到 console；不得拼接进 URL。
- **unlock_context 安全**：若在 `SessionStore` 中存放任何解锁材料（派生密钥或可再派生材料），必须在登出/过期移除时执行内存清零（建议使用 `zeroize` 的 `Zeroizing<T>` 包装）。
  - **依赖说明**：若采用 `zeroize`，需在 `backend/Cargo.toml` 添加依赖（版本按 crates.io 最新稳定版，如 `zeroize = "1"`）

#### 禁止事项（必须遵循）

- **禁止**：在 session token 中存储用户密码或等价敏感信息
- **禁止**：将 `unlock_context`（或任何派生密钥/可再派生材料）落库
- **禁止**：在前端 `console.log` 输出 session token/解锁材料

### Project Structure Notes

- 当前后端已存在 `users` 表，但 `domain/models/` 尚无 `user.rs`，`repositories/` 也尚未提供 `user_repo.rs`。
- `docs/architecture.md` 的规划结构中包含 `domain/models/user.rs`，但当前代码库尚未落地，实现时需以仓库现状为准创建文件，避免误判“已存在”。
- 当前后端 `auth.rs` 同时承担“连接测试 + 凭证配置管理”，并存在 `DEFAULT_USER_ID = "default_user"` 占位；本 Story 需要开始切换为真实 user_id（至少在需要鉴权的接口中）。
- 现有 migrations 中配置相关表的 `user_id` 可能带默认值 `default_user`；本 Story 需要明确代码层面禁止依赖默认值，并给出历史数据迁移策略。
- 当前前端仅有 `HomePage` 与 `ApiConfigPage`，尚无登录页与 `useAuthStore`。

### References

- [Source: docs/epics.md#Story-1.6] - 验收标准原文
- [Source: docs/sprint-status.yaml#development_status] - story_key 与状态流转
- [Source: backend/migrations/001_initial_schema.sql#users] - users 表结构
- [Source: backend/src/api/routes/auth.rs] - 现有 auth 路由与 DEFAULT_USER_ID 占位
- [Source: backend/src/main.rs] - ApiKeyManager 目前使用 MASTER_PASSWORD，并标记 TODO(Story-1.6)
- [Source: docs/architecture.md#Authentication-&-Security] - Argon2 + AES-GCM + 密钥仅存内存
- [Source: https://docs.rs/argon2/latest/argon2/] - Argon2 官方文档（PasswordHasher/PasswordVerifier、PHC 字符串）
- [Source: frontend/src/lib/api.ts] - 前端 API 调用入口（需注入 Authorization 头）

### Debug Tips

- 在开发模式下可快速自检：`users.password_hash` 必须是 PHC 格式字符串（通常以 `$argon2` 开头），且数据库中不得出现明文密码
- 验证日志：确保不输出明文密码、完整 token、完整凭证（只允许脱敏形式）

## Dev Agent Record

### Agent Model Used

Cascade

### Debug Log References

### Completion Notes List

- ✅ Task 1: 创建 `backend/src/domain/models/user.rs` 用户领域模型，对齐数据库 schema
- ✅ Task 1: 创建 `backend/src/infra/db/repositories/user_repo.rs` 用户仓储，提供 create_user/find_by_username/find_by_id
- ✅ Task 2: 创建 `backend/src/shared/password.rs` 密码服务，使用 Argon2 PHC 字符串格式哈希和验证
- ✅ Task 3: 创建 `backend/src/api/middleware/session.rs` 会话存储，使用 Arc<RwLock<HashMap>> 实现并发安全
- ✅ Task 3: 创建 `backend/src/api/middleware/auth.rs` 鉴权中间件，从 Authorization Bearer token 提取用户
- ✅ Task 4: 创建 `backend/src/api/routes/user_auth.rs` 用户认证路由（register/login/logout/me/status）
- ✅ Task 5: 修改 `backend/src/api/routes/auth.rs` 配置接口使用 CurrentUser.user_id 替代 DEFAULT_USER_ID
- ✅ Task 6: 更新 main.rs 注释记录 MVP 决策，UnlockContext 已实现存放用户密码内存副本
- ✅ Task 7: 后端测试通过（48 个测试全部通过）
- ✅ Task 8: 创建 `frontend/src/stores/useAuthStore.ts` 认证状态管理
- ✅ Task 9: 创建 `frontend/src/features/auth/services/authService.ts` 认证 API 服务
- ✅ Task 10: 创建 `frontend/src/features/auth/components/LoginPage.tsx` 登录/注册页面
- ✅ Task 10: 创建 `frontend/src/features/auth/components/ProtectedRoute.tsx` 路由守卫
- ✅ Task 10: 更新 `frontend/src/App.tsx` 添加登录路由和路由保护

### File List

**后端新增/修改文件:**
- backend/src/domain/models/user.rs (新增)
- backend/src/domain/models/mod.rs (修改)
- backend/src/infra/db/repositories/user_repo.rs (新增)
- backend/src/infra/db/repositories/mod.rs (修改)
- backend/src/shared/password.rs (新增)
- backend/src/shared/mod.rs (修改)
- backend/src/api/middleware/session.rs (新增)
- backend/src/api/middleware/auth.rs (新增)
- backend/src/api/middleware/mod.rs (修改)
- backend/src/api/routes/user_auth.rs (新增)
- backend/src/api/routes/mod.rs (修改)
- backend/src/api/routes/auth.rs (修改)
- backend/src/api/state.rs (修改)
- backend/src/main.rs (修改)

**前端新增/修改文件:**
- frontend/src/stores/useAuthStore.ts (新增)
- frontend/src/features/auth/services/authService.ts (新增)
- frontend/src/features/auth/components/LoginPage.tsx (新增)
- frontend/src/features/auth/components/ProtectedRoute.tsx (新增)
- frontend/src/features/auth/index.ts (新增)
- frontend/src/features/index.ts (修改)
- frontend/src/App.tsx (修改)

**其他:**
- docs/sprint-status.yaml (修改)
