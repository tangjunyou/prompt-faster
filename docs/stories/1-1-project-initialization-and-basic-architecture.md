# Story 1.1: 项目初始化与基础架构

Status: done

## Story

As a 开发者,
I want 初始化 Prompt Faster 项目的前后端基础架构,
So that 后续功能开发有统一的代码组织和配置基础。

## Acceptance Criteria

1. **Given** 空的项目目录 **When** 执行项目初始化 **Then** 创建以下结构：
   - `backend/`: Rust Cargo 项目 (edition 2024)
   - `frontend/`: Vite + React + TypeScript 项目
   - 配置 SQLite 连接与 WAL 模式

2. **Given** 后端项目已创建 **When** 执行 `cargo run` **Then** 启动 HTTP 服务器并监听端口

3. **Given** 前端项目已创建 **When** 执行 `npm run dev` **Then** 启动开发服务器

4. **Given** 前后端均启动 **When** 前端调用后端 API **Then** 成功通信并返回响应

5. **Given** 后端 API 返回响应 **When** 检查响应结构 **Then** 使用 `ApiResponse<T>` 统一结构，`data` 与 `error` 字段互斥 (AR1)

6. **Given** 数据库表包含时间字段 **When** 检查字段定义 **Then** 使用 Unix 毫秒时间戳，字段命名 `*_at` 后缀 (AR3)

7. **Given** 需要数据库迁移 **When** 执行迁移命令 **Then** 使用 SQLx migrations 管理

## Tasks / Subtasks

- [x] **Task 1: 后端项目初始化** (AC: #1, #2)
  - [x] 1.1 在 `backend/` 执行 `cargo init --bin --name prompt_faster`
  - [x] 1.2 配置 `Cargo.toml` 依赖（见 Dev Notes）
  - [x] 1.3 设置 Rust edition = "2024"
  - [x] 1.4 创建目录结构：`api/`, `core/`, `domain/`, `infra/`, `shared/`
  - [x] 1.5 实现 `main.rs` 基础服务器启动

- [x] **Task 2: 前端项目初始化** (AC: #1, #3)
  - [x] 2.1 在 `frontend/` 执行 `npm create vite@latest . -- --template react-ts`
  - [x] 2.2 配置 `package.json` 依赖（见 Dev Notes）
  - [x] 2.3 配置 Tailwind CSS 3.x
  - [x] 2.4 创建目录结构：`pages/`, `features/`, `components/`, `lib/`, `types/`, `hooks/`
  - [x] 2.5 配置 React Router 7.x 基础路由（使用 `react-router` 包）

- [x] **Task 3: 数据库配置** (AC: #7)
  - [x] 3.1 创建 `backend/migrations/` 目录
  - [x] 3.2 创建 `001_initial_schema.sql`：仅包含 `users` 和 `workspaces` 核心表（其他表留给后续 Story）
  - [x] 3.3 配置 SQLite 连接：WAL 模式 + FULL synchronous + 30s busy_timeout
  - [x] 3.4 实现 `infra/db/pool.rs` 连接池配置

- [x] **Task 4: ApiResponse<T> 实现** (AC: #5)
  - [x] 4.1 创建 `api/response.rs`
  - [x] 4.2 实现 `ApiSuccess<T>` 和 `ApiError` 结构
  - [x] 4.3 确保 `data` 与 `error` 互斥
  - [x] 4.4 实现 Axum 的 `IntoResponse` trait

- [x] **Task 5: 健康检查端点** (AC: #2, #4, #5)
  - [x] 5.1 创建 `api/routes/health.rs`
  - [x] 5.2 实现 `GET /api/v1/health` 端点
  - [x] 5.3 返回服务器状态和版本信息
  - [x] 5.4 **必须使用 `ApiResponse<HealthResponse>` 返回**（不允许直接返回 `Json<T>`）

- [x] **Task 6: 时间字段规范** (AC: #6)
  - [x] 6.1 创建 `shared/time.rs` 时间戳工具函数
  - [x] 6.2 实现 `now_millis()` 函数获取 Unix 毫秒时间戳
  - [x] 6.3 修改 `health.rs` 复用 `shared::time::now_millis()`
  - [x] 6.4 确保所有时间字段使用 `*_at` 后缀命名

- [x] **Task 7: 错误处理框架** (AC: #5)
  - [x] 7.1 创建 `shared/error.rs`
  - [x] 7.2 使用 `thiserror` 定义业务错误类型
  - [x] 7.3 使用 `anyhow` 包装应用层错误
  - [x] 7.4 统一错误响应格式：`{ code, message, details? }`

- [x] **Task 8: 配置管理** (AC: #1)
  - [x] 8.1 创建 `backend/.env.example`
  - [x] 8.2 创建 `shared/config.rs` 作为配置唯一入口
  - [x] 8.3 创建 `frontend/.env.example`

- [x] **Task 9: Tracing 日志配置** (AC: #2)
  - [x] 9.1 创建 `shared/tracing_setup.rs`
  - [x] 9.2 初始化 tracing-subscriber
  - [x] 9.3 配置日志级别和格式

- [x] **Task 10: 前后端通信验证** (AC: #4, #5)
  - [x] 10.1 在现有 `lib/api.ts` 基础上完善（不新建 `services/apiClient.ts`）
  - [x] 10.2 移除 `lib/api.ts` 中的 fallback `|| data`，严格执行 `ApiResponse<T>` 契约
  - [x] 10.3 验证 CORS 配置：浏览器从 `localhost:5173` 调用 `localhost:3000` 无 CORS 错误
  - [x] 10.4 测试健康检查端点调用，确认返回 `{ data: HealthResponse }` 结构

## Dev Notes

### ⚠️ Guardrails（必须遵循的事实来源）

> **以下文件为唯一事实来源，不得引入冲突实现：**
> - 后端依赖版本：以 `backend/Cargo.toml` 为准
> - 前端依赖版本：以 `frontend/package.json` 为准
> - 环境变量/端口：以 `.env.example` 和 `docker-compose.yml` 为准
> - **禁止**：引入第二套路由库、第二套 API client、第二套响应结构
> - 如果文件已存在：只允许修改以对齐规范，不要新建同功能重复文件

### 技术栈版本要求

**后端 Cargo.toml 核心依赖（以仓库实际为准）：**

```toml
[package]
name = "prompt_faster"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
axum = { version = "0.8", features = ["ws", "macros"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "time", "migrate"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }  # 注意：版本 8
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
dotenvy = "0.15"
time = { version = "0.3", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
```

**前端 package.json 核心依赖（以仓库实际为准）：**

```json
{
  "dependencies": {
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "react-router": "^7.0.0",  // 注意：是 react-router 不是 react-router-dom
    "@tanstack/react-query": "^5.0.0"
  },
  "devDependencies": {
    "typescript": "~5.9.3",
    "tailwindcss": "^3.4.15",  // 注意：版本 3.x
    "@types/react": "^19.2.5",
    "@types/react-dom": "^19.2.3",
    "vite": "^7.2.4"
  }
}
```

### 架构约束

**目录结构（后端）：**
```
backend/src/
├── main.rs                    # 入口点
├── lib.rs                     # 库导出
├── api/                       # HTTP/WS 路由层
│   ├── mod.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── health.rs          # /api/v1/health
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── correlation_id.rs
│   └── response.rs            # ApiResponse<T>
├── core/                      # 核心业务逻辑（预留）
│   └── mod.rs
├── domain/                    # 领域模型（预留）
│   └── mod.rs
├── infra/                     # 基础设施
│   ├── mod.rs
│   └── db/
│       ├── mod.rs
│       └── pool.rs            # SQLx 连接池
└── shared/                    # 共享工具
    ├── mod.rs
    ├── error.rs               # thiserror 定义
    ├── config.rs              # 配置唯一入口
    ├── time.rs                # 时间戳工具
    └── tracing_setup.rs       # tracing 初始化
```

**目录结构（前端）：**
```
frontend/src/
├── main.tsx
├── App.tsx
├── index.css                  # Tailwind 入口
├── pages/                     # 路由入口（预留）
│   └── index.ts
├── features/                  # 业务模块（预留）
│   └── index.ts
├── components/                # 可复用组件
│   └── ui/
├── lib/                       # 工具与 API 层
│   ├── api.ts                 # API 客户端 + ApiResponse<T>
│   └── utils.ts               # 通用工具（如 cn()）
├── hooks/                     # 全局 Hooks
│   └── index.ts
├── types/                     # TypeScript 类型
│   └── index.ts
└── test/                      # 测试工具
    └── setup.ts
```

### ApiResponse<T> 规范 (AR1)

**Rust 实现（与 `backend/src/api/response.rs` 一致）：**

```rust
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

/// API 成功响应
#[derive(Serialize)]
pub struct ApiSuccess<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

/// 分页元信息
#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

/// API 错误响应
#[derive(Serialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}

/// 错误详情
#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,           // 格式：DOMAIN_ACTION_REASON
    pub message: String,        // 用户可见消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>, // 仅开发环境
}

/// 统一响应类型 - data 与 error 互斥
pub enum ApiResponse<T: Serialize> {
    Success(ApiSuccess<T>),
    Error(StatusCode, ApiError),
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiResponse::Success(success) => (StatusCode::OK, Json(success)).into_response(),
            ApiResponse::Error(status, error) => (status, Json(error)).into_response(),
        }
    }
}
```

**TypeScript 实现：**

```typescript
interface ApiSuccess<T> {
  data: T;
  meta?: { page?: number; pageSize?: number; total?: number };
}

interface ApiError {
  error: {
    code: string;      // 格式：DOMAIN_ACTION_REASON
    message: string;   // 用户可见消息
    details?: Record<string, unknown>; // 仅开发环境
  };
}

type ApiResponse<T> = ApiSuccess<T> | ApiError;
```

### SQLite 配置

```rust
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::str::FromStr;
use std::time::Duration;

let opts = SqliteConnectOptions::from_str(&database_url)?
    .journal_mode(SqliteJournalMode::Wal)      // WAL 模式
    .synchronous(SqliteSynchronous::Full)       // 断电安全
    .busy_timeout(Duration::from_secs(30))      // 写入等待超时
    .create_if_missing(true);
```

### 时间字段规范 (AR3)

- **数据库存储：** `INTEGER` (Unix 毫秒时间戳)
- **字段命名：** `*_at` 后缀 (如 `created_at`, `updated_at`)
- **API 传输：** ISO 8601 字符串 (`2025-12-20T18:45:00Z`)

```rust
pub fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
```

### 命名规范

| 语言 | 风格 | 示例 |
|------|------|------|
| Rust 结构体 | PascalCase | `ApiResponse` |
| Rust 函数/变量 | snake_case | `get_health` |
| TypeScript 组件 | PascalCase | `HealthCheck` |
| TypeScript 函数/变量 | camelCase | `getHealth` |
| 跨语言边界 | `#[serde(rename_all = "camelCase")]` | — |

### 环境变量

**backend/.env.example:**
```
DATABASE_URL=sqlite://data.db
RUST_LOG=info,prompt_faster=debug
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
```

**frontend/.env.example:**
```
VITE_API_URL=http://localhost:3000/api/v1
```

### Project Structure Notes

- 与 architecture.md 完全对齐
- 目录结构预留后续 Epic 扩展空间
- 遵循 Pages / Features / Components 三层组件边界

### References

- [Source: docs/architecture.md#Core-Architectural-Decisions] - 技术栈选型
- [Source: docs/architecture.md#Implementation-Patterns] - 命名规范
- [Source: docs/architecture.md#Project-Structure] - 目录结构
- [Source: docs/epics.md#Story-1.1] - 验收标准
- [Source: docs/prd.md#7.5] - SQLite 配置

## Dev Agent Record

### Agent Model Used

Claude Sonnet 4 (Cascade)

### Debug Log References

- 后端编译: `cargo check` ✅
- 后端测试: `cargo test` ✅ (2/2 passed)
- 前端编译: `npm run build` ✅
- 健康检查: `curl http://localhost:3000/api/v1/health` ✅ 返回 `{ data: { status: "ok", version: "0.1.0", timestampMs: ... } }`
- 前后端通信: 浏览器验证 ✅ CORS 无错误

### Completion Notes List

1. **Task 3.3 修复**: `pool.rs` 添加了 `busy_timeout(30s)` 和 `create_if_missing(true)` 配置
2. **Task 5.4 修复**: `health.rs` 改用 `ApiResponse<HealthResponse>` 替代 `Json<T>`
3. **Task 6 实现**: 创建 `shared/time.rs` 并在 `health.rs` 中复用 `now_millis()`
4. **Task 8.3 实现**: 创建 `frontend/.env.example`
5. **Task 10.2 实现**: 重构 `api.ts` 类型定义，添加 `isApiError`/`isApiSuccess` 类型守卫
6. **额外**: 创建 `HealthCheck.tsx` 组件验证前后端通信

### File List

**修改的文件:**
- `backend/src/infra/db/pool.rs` - 添加 busy_timeout 配置
- `backend/src/api/routes/health.rs` - 使用 ApiResponse<T>
- `backend/src/shared/mod.rs` - 添加 time 模块导出
- `backend/src/main.rs` - 收紧 CORS 配置（仅允许 localhost:5173）
- `backend/src/shared/error.rs` - 统一错误响应结构为 { error: {...} }
- `backend/src/api/middleware/correlation_id.rs` - 修复 tracing span 记录
- `frontend/src/lib/api.ts` - 重构 ApiResponse 类型 + 添加错误处理
- `frontend/src/App.tsx` - 配置 React Router 7.x
- `frontend/src/main.tsx` - 添加 BrowserRouter
- `frontend/src/types/api.ts` - 修复 HealthResponse 字段命名
- `frontend/.env.example` - 修复 VITE_API_URL 路径
- `frontend/tests/e2e/health.spec.ts` - 修复 ApiResponse 断言
- `.gitignore` - 允许 Cargo.lock + 添加 WAL/SHM 忽略规则

**新建的文件:**
- `backend/src/shared/time.rs` - 时间戳工具函数
- `frontend/src/components/HealthCheck.tsx` - 健康检查组件
- `frontend/src/pages/HomePage.tsx` - 首页组件
- `frontend/src/pages/index.ts` - 页面导出
- `frontend/src/features/index.ts` - 业务模块导出占位
- `frontend/src/hooks/index.ts` - Hooks 导出占位

### Change Log

| 日期 | 变更类型 | 描述 |
|------|----------|------|
| 2025-12-22 | Code Review Fix | [H1] 修复 VITE_API_URL 路径不一致 - 添加 /api/v1 前缀 |
| 2025-12-22 | Code Review Fix | [H2] 配置 React Router 7.x 基础路由 |
| 2025-12-22 | Code Review Fix | [H3] 统一 Story Status 字段为 done |
| 2025-12-22 | Code Review Fix | [H4] 收紧 CORS 配置 - 限制允许的 Origin/Methods/Headers |
| 2025-12-22 | Code Review Fix | [H5] 统一错误响应结构为 { error: {...} } 格式 |
| 2025-12-22 | Code Review Fix | [H6] 修复 e2e 测试断言 - 正确访问 data.data.status |
| 2025-12-22 | Code Review Fix | [H7] 修复 types/api.ts HealthResponse 字段命名 |
| 2025-12-22 | Code Review Fix | [M2] 补充 apiRequest 错误处理 - try/catch + Content-Type 检查 |
| 2025-12-22 | Code Review Fix | [M3] 修复 correlationId tracing - 使用 info_span! + Instrument |
| 2025-12-22 | Code Review Fix | [M4+M5] 修复 .gitignore - 允许 Cargo.lock + 忽略 WAL/SHM |
| 2025-12-22 | Code Review Fix | [L1] 补充前端目录 index.ts 导出文件 |
