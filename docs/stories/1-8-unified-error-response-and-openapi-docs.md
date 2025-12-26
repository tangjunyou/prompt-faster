# Story 1.8: 统一错误响应结构与 OpenAPI 文档

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a 使用 Prompt Faster 的前端开发者/调试者,
I want 所有 HTTP API 有统一的错误响应结构并提供可浏览的 OpenAPI 文档,
So that 我可以稳定解析错误并快速理解接口。

## Acceptance Criteria

1. **Given** 服务端已实现 HTTP API
   **When** 使用 HTTP 客户端调用任意业务接口（成功或失败）
   **Then** 外层响应使用 `ApiResponse<T>` 结构, `data` 与 `error` 字段互斥 (AR1)
   **And** 当发生错误时, `error` 字段的内容符合 `{ code, message, details? }` 结构
   **And** `code` 字段遵循统一编码规范

2. **Given** Rust 服务端代码已经存在
   **When** 检查业务逻辑层与 API 层的错误处理
   **Then** 业务错误使用 `thiserror` 定义
   **And** 应用入口/HTTP 层使用 `anyhow` 或等价机制将内部错误映射为统一响应结构 (AR1 对应实现)

3. **Given** HTTP 服务已启动
   **When** 检查路由配置
   **Then** 所有对外公开的 REST API 均挂载在 `/api/v1/...` 路径下
   **And** 不存在无版本前缀的对外 API

4. **Given** 应用在本地开发模式启动
   **When** 访问 `http://localhost:PORT/swagger`
   **Then** 可以看到通过 utoipa 生成的 OpenAPI 文档
   **And** 至少包含核心业务 API 的路径及请求/响应 schema

5. **Given** 测试人员构造多个典型错误场景（参数缺失、权限不足、资源不存在、服务内部错误等）
   **When** 观察返回的 JSON 错误体
   **Then** 均符合统一结构
   **And** `message` 字段可读、明确, 便于前端展示与用户理解

## Tasks / Subtasks

- [ ] 任务 1: 配置 API 版本前缀 `/api/v1` (AC: #3)
  - [ ] 1.1 修改 `main.rs` 中的路由配置, 添加 `/api/v1` 前缀
  - [ ] 1.2 确保所有业务路由都挂载在 `/api/v1` 下
  - [ ] 1.3 保留 `/health` 端点不带版本前缀（用于负载均衡健康检查）
  - [ ] 1.4 验证无版本前缀的对外 API 不存在

- [ ] 任务 2: 设置 utoipa OpenAPI 配置 (AC: #4)
  - [ ] 2.1 在 `main.rs` 中配置 `utoipa::OpenApi` 和 `utoipa_swagger_ui::SwaggerUi`
  - [ ] 2.2 配置 Swagger UI 路由挂载在 `/swagger` 路径
  - [ ] 2.3 添加基本信息：title, version, description
  - [ ] 2.4 启用开发环境下的 Swagger UI 访问

- [ ] 任务 3: 为现有 API 添加 utoipa 注解 (AC: #4)
  - [ ] 3.1 为 `auth.rs` 中的所有 handler 添加 `#[utoipa::path]` 注解
  - [ ] 3.2 为 `user_auth.rs` 中的所有 handler 添加 `#[utoipa::path]` 注解
  - [ ] 3.3 为 `workspaces.rs` 中的所有 handler 添加 `#[utoipa::path]` 注解
  - [ ] 3.4 为请求/响应结构体添加 `#[derive(utoipa::ToSchema)]` 注解
  - [ ] 3.5 在 OpenAPI 配置中注册所有路由

- [ ] 任务 4: 定义统一的错误编码规范 (AC: #1, #5)
  - [ ] 4.1 在 `shared/error.rs` 中定义错误码枚举或常量
  - [ ] 4.2 建立错误码命名约定（格式：`DOMAIN_ACTION_REASON`）
  - [ ] 4.3 更新所有错误返回使用统一编码
  - [ ] 4.4 在文档中记录错误码规范

- [ ] 任务 5: 验证统一错误响应结构 (AC: #1, #2, #5)
  - [ ] 5.1 确认 `api/response.rs` 中的 `ApiResponse<T>` 正确实现 data/error 互斥
  - [ ] 5.2 验证 `thiserror` 定义的 `AppError` 正确映射到 `ApiResponse`
  - [ ] 5.3 测试各种错误场景返回正确的 JSON 结构
  - [ ] 5.4 验证错误 message 字段可读且对用户友好

## Dev Notes

### 相关架构模式和约束 (AR1)

**ApiResponse<T> 结构要求：**
- `data` 与 `error` 字段互斥，不可同时存在非空值
- 成功响应：`{ data: T, meta?: {...} }`
- 错误响应：`{ error: { code: string, message: string, details?: object } }`

**现有实现位置：**
- `backend/src/api/response.rs` - 已实现 `ApiResponse<T>` 结构
- `backend/src/shared/error.rs` - 已实现 `thiserror` 定义的 `AppError`

### 技术实现要点

**1. API 版本前缀配置：**

在 `main.rs` 中修改路由配置：
```rust
// 健康检查端点不带版本前缀
let app = Router::new()
    .route("/health", get(health::handler))
    // 所有业务 API 挂载在 /api/v1 下
    .nest("/api/v1", api_routes());
```

**2. utoipa 配置示例：**

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        auth::save_credentials,
        auth::test_connection,
        user_auth::login,
        user_auth::logout,
        // ... 其他路由
    ),
    components(schemas(
        // 请求/响应结构体
    )),
    tags(
        (name = "auth", description = "API 凭证管理"),
        (name = "user", description = "用户认证"),
        // ... 其他标签
    ),
    info(
        title = "Prompt Faster API",
        version = "0.1.0",
        description = "AI Prompt 自动迭代优化系统 API 文档"
    )
)]
struct ApiDoc;
```

**3. Handler 注解示例：**

```rust
/// 保存 API 凭证
#[utoipa::path(
    post,
    path = "/api/v1/auth/credentials",
    request_body = SaveCredentialsRequest,
    responses(
        (status = 200, description = "保存成功", body = ApiResponse<Credentials>),
        (status = 400, description = "验证错误", body = ApiErrorResponse),
        (status = 500, description = "服务器错误", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn save_credentials(/* ... */) -> Result<Json<Credentials>, AppError> {
    // ...
}
```

**4. 错误编码规范：**

| 域名 | 动作 | 原因 | 示例 |
|------|------|------|------|
| AUTH | SAVE | INVALID_URL | AUTH_SAVE_INVALID_URL |
| AUTH | TEST | CONNECTION_FAILED | AUTH_TEST_CONNECTION_FAILED |
| USER | LOGIN | INVALID_CREDENTIALS | USER_LOGIN_INVALID_CREDENTIALS |
| WORKSPACE | CREATE | ALREADY_EXISTS | WORKSPACE_CREATE_ALREADY_EXISTS |

### Source Tree Components to Touch

- `backend/src/main.rs` - 路由前缀配置、Swagger UI 配置
- `backend/src/api/routes/auth.rs` - 添加 utoipa 注解
- `backend/src/api/routes/user_auth.rs` - 添加 utoipa 注解
- `backend/src/api/routes/workspaces.rs` - 添加 utoipa 注解
- `backend/src/api/routes/health.rs` - 确认无版本前缀
- `backend/src/shared/error.rs` - 定义错误码规范

### Testing Standards Summary

1. **单元测试：** 验证错误码格式符合 `DOMAIN_ACTION_REASON` 规范
2. **集成测试：** 测试各种错误场景返回正确的 JSON 结构
3. **手动测试：** 启动服务访问 `/swagger` 验证 OpenAPI 文档可浏览
4. **验证测试：** 确认所有业务 API 路径都包含 `/api/v1` 前缀

### Project Structure Notes

**Alignment with unified project structure:**
- 路由模块位于 `api/routes/`
- API 响应结构位于 `api/response.rs`
- 错误处理位于 `shared/error.rs`

**No conflicts detected** - 现有结构符合架构规范。

### References

- [Source: docs/architecture.md#API-Response-Format](docs/architecture.md) - API 响应格式规范
- [Source: docs/architecture.md#API-Documentation](docs/architecture.md) - utoipa + Swagger UI 配置
- [Source: docs/architecture.md#API-Naming-Conventions](docs/architecture.md) - API 版本策略
- [Source: docs/epics.md#Story-1.8](docs/epics.md) - Story 详细需求

---

## Previous Story Intelligence

### Story 1-7 完成经验

**关键实现：**
- Story 1-7 实现了用户数据隔离，所有数据访问都需要通过 `user_id` 过滤
- 认证中间件 (`middleware/auth.rs`) 已实现，用于验证用户身份
- Session 中间件 (`middleware/session.rs`) 处理会话管理

**对当前 Story 的启示：**
1. 添加 utoipa 注解时需要注意认证相关的路由可能有额外的安全要求
2. 错误处理应考虑用户未认证（401）和未授权（403）的场景区分

### Git 提交模式分析

**最近 5 次提交：**
1. `style(backend): 运行 cargo fmt 修复代码格式问题`
2. `feat(auth): 实现用户数据隔离和访问控制 (Story 1-7)`
3. `bmad 新增其他 IDE 支持`
4. `fix(frontend): 补齐 QueryClientProvider 避免 API 配置页白屏`
5. `fix(ci): 修复 cargo-audit 配置与 E2E 导航`

**代码模式：**
- 使用约定式提交：`feat:` / `fix:` / `style:`
- Story 提交格式：`feat(scope): 实现功能描述 (Story X-Y)`
- Story 编号格式：`1-7`、`1-8` 等

**当前 Story 建议的提交格式：**
- `feat(api): 实现统一错误响应结构与 OpenAPI 文档 (Story 1-8)`

---

## Dev Agent Record

### Agent Model Used

glm-4.7 (claude-opus-4-5-20251101)

### Debug Log References

---

### Completion Notes List

---

### File List
