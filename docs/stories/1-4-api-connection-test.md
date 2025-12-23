# Story 1.4: API 连接测试

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a Prompt 优化用户,
I want 测试已配置的 API 连接是否成功,
so that 我能确认配置正确后再开始优化任务。

## Acceptance Criteria

1. **Given** 用户已填写 Dify API 凭证 **When** 用户点击"测试连接" **Then** 系统调用 Dify API 验证连接 **And** 成功时显示绿色"连接成功"提示 **And** 失败时显示红色错误信息（含具体原因）

2. **Given** 用户已填写通用大模型 API 凭证 **When** 用户点击"测试连接" **Then** 系统发送请求验证 API Key 有效性（调用 `/v1/models` 端点） **And** 成功时显示模型列表或"连接成功" **And** 失败时显示具体错误（401/网络错误等）

3. **Given** API 调用失败 **When** 显示错误信息 **Then** 日志中 API Key 被脱敏处理 (NFR11) **And** 用户界面显示可读的错误信息，不暴露敏感凭证

4. **Given** 用户点击"测试连接" **When** 请求超时（超过 60 秒） **Then** 显示"连接超时"错误 (NFR23)

5. **Given** 凭证表单字段未完整填写 **When** 用户点击"测试连接" **Then** 按钮禁用或提示先完成填写

## Tasks / Subtasks

### 后端任务

- [x] **Task 1: 添加 reqwest 依赖并创建 HTTP 客户端封装** (AC: #1, #2, #4)
  - [x] 1.1 在 `backend/Cargo.toml` 中添加 `reqwest = { version = "0.12", features = ["json", "rustls-tls"] }` 依赖
  - [x] 1.2 创建 `backend/src/infra/external/http_client.rs`
  - [x] 1.3 配置默认超时 60 秒 (NFR23)
  - [x] 1.4 配置连接超时 10 秒

- [x] **Task 2: 实现日志脱敏工具** (AC: #3)
  - [x] 2.1 创建 `backend/src/shared/log_sanitizer.rs`
  - [x] 2.2 实现 `sanitize_api_key()` 函数：保留前 4 位 + `****` + 后 4 位
  - [x] 2.3 实现 tracing Layer 或 visitor 模式自动脱敏（可选增强）
  - [x] 2.4 在所有涉及 API Key 的日志输出处使用脱敏函数

- [x] **Task 3: 实现 Dify 连接测试客户端** (AC: #1)
  - [x] 3.1 创建 `backend/src/infra/external/dify_client.rs`
  - [x] 3.2 实现 `test_connection(base_url: &str, api_key: &str) -> Result<TestConnectionResult>`
  - [x] 3.3 调用 Dify 工作流 API（发送简单请求验证 API Key 有效性）
  - [x] 3.4 解析响应：成功返回 `{ success: true }`，失败返回错误详情

- [x] **Task 4: 填充通用大模型连接测试客户端** (AC: #2)
  - [x] 4.1 **填充现有** `backend/src/infra/external/llm_client.rs`（文件已存在，目前是空壳）
  - [x] 4.2 实现 `test_connection(base_url: &str, api_key: &str, provider: &str) -> Result<TestConnectionResult>`
  - [x] 4.3 调用 OpenAI 兼容 API `/v1/models` 端点获取模型列表
  - [x] 4.4 支持硅基流动（siliconflow）和魔搭社区（modelscope）Provider
  - [x] 4.5 解析响应：成功返回模型列表，失败返回错误详情

- [x] **Task 5: 创建测试连接 API 路由** (AC: #1, #2, #3, #4)
  - [x] 5.1 新建 `backend/src/api/routes/auth.rs`
  - [x] 5.2 实现 `POST /api/v1/auth/test-connection/dify` 端点
  - [x] 5.3 实现 `POST /api/v1/auth/test-connection/generic-llm` 端点
  - [x] 5.4 使用 `ApiResponse<T>` 统一响应格式 (AR1)：成功返回 `{ data: TestConnectionResult }`，失败返回 `{ error: { code, message } }`
  - [x] 5.5 错误处理：使用 thiserror 定义错误类型，anyhow 包装
  - [x] 5.6 **使用 correlationId**：handler 读取 middleware 已注入的 `x-correlation-id` header，用于业务日志（无需生成，`correlation_id_middleware` 已处理）

- [x] **Task 6: 注册路由到 Axum Router** (AC: #1, #2)
  - [x] 6.1 修改 `backend/src/api/routes/mod.rs`，添加 `pub mod auth;` 导出
  - [x] 6.2 修改 `backend/src/main.rs`，在现有 `.nest("/api/v1", health::router())` 后添加 `.nest("/api/v1/auth", auth::router())`
  - [x] 6.3 确保 `/api/v1/auth` 前缀正确

### 前端任务

- [x] **Task 7: 扩展凭证状态类型** (AC: #1, #2)
  - [x] 7.1 在 `frontend/src/types/credentials.ts` 中扩展 `CredentialStatus` 类型
  - [x] 7.2 新增状态：`testing` | `valid` | `invalid`
  - [x] 7.3 更新状态徽章颜色映射
  - [x] 7.4 **影响文件清单**（需同步修改）：
    - `frontend/src/stores/useCredentialStore.ts` - Store 状态处理
    - `frontend/src/features/api-config/DifyCredentialForm.tsx` - 徽章渲染
    - `frontend/src/features/api-config/GenericLlmCredentialForm.tsx` - 徽章渲染
    - 相关测试文件

- [x] **Task 8: 创建 API 服务函数** (AC: #1, #2)
  - [x] 8.1 在 `frontend/src/features/api-config/` 下新建 `services/credentialService.ts`（遵循 feature 目录结构）
  - [x] 8.2 使用 `frontend/src/lib/api.ts` 的 `post()` 函数发送请求
  - [x] 8.3 实现 `testDifyConnection(baseUrl: string, apiKey: string): Promise<ApiResponse<TestConnectionResult>>`
  - [x] 8.4 实现 `testGenericLlmConnection(baseUrl: string, apiKey: string, provider: string): Promise<ApiResponse<TestConnectionResult>>`
  - [x] 8.5 定义 `TestConnectionResult` 类型（message: string, models?: string[]）

- [x] **Task 9: 创建 TanStack Query Hook** (AC: #1, #2, #4)
  - [x] 9.1 创建 `frontend/src/features/api-config/hooks/useTestConnection.ts`
  - [x] 9.2 实现 `useTestDifyConnection()` mutation hook
  - [x] 9.3 实现 `useTestGenericLlmConnection()` mutation hook
  - [x] 9.4 处理 loading、success、error 状态

- [x] **Task 10: 修改 Dify 凭证表单组件** (AC: #1, #3, #5)
  - [x] 10.1 在 `DifyCredentialForm.tsx` 中添加“测试连接”按钮
  - [x] 10.2 按钮状态：填写完整时启用，未完整时禁用
  - [x] 10.3 测试中显示 loading 状态（按钮禁用 + Spinner 图标）
  - [x] 10.4 **复用 `useFeedback.ts` 和 `FeedbackAlert.tsx`** 显示测试结果反馈消息
  - [x] 10.5 更新 Store 中的凭证状态（testing → valid/invalid）

- [x] **Task 11: 修改通用大模型凭证表单组件** (AC: #2, #3, #5)
  - [x] 11.1 在 `GenericLlmCredentialForm.tsx` 中添加“测试连接”按钮
  - [x] 11.2 按钮状态：Provider 已选择且填写完整时启用
  - [x] 11.3 测试中显示 loading 状态（按钮禁用 + Spinner 图标）
  - [x] 11.4 **复用 `useFeedback.ts` 和 `FeedbackAlert.tsx`** 显示测试结果反馈消息
  - [x] 11.5 更新 Store 中的凭证状态

### 测试任务

- [x] **Task 12: 后端单元测试** (AC: #1, #2, #3)
  - [x] 12.1 测试日志脱敏函数（纯函数，无需 mock）：
    - API Key 长度 ≤ 8 时返回 `****`
    - API Key 长度 > 8 时返回 `xxxx****xxxx`
  - [x] 12.1b **HTTP 客户端测试使用 `wiremock`**（统一选型，减少临场决策）
  - [x] 12.2 测试 Dify 连接客户端：mock 成功/失败响应
  - [x] 12.3 测试通用 LLM 连接客户端：mock 成功/失败响应
  - [x] 12.4 测试 API 路由：验证请求参数和响应格式
  - [x] 12.5 **边界条件测试清单**：
    - 空 API Key 的处理
    - 无效 URL 格式的处理
    - 网络超时（60s）的处理
    - 401 Unauthorized 错误的处理 → 返回 `AUTH_INVALID_CREDENTIALS`
    - 403 Forbidden 错误的处理 → 返回 `AUTH_FORBIDDEN`
    - 500 Internal Server Error 的处理 → 返回 `UPSTREAM_ERROR`
    - 连接超时 → 返回 `AUTH_CONNECTION_TIMEOUT`

- [x] **Task 13: 前端测试** (AC: #1, #2, #5)
  - [x] 13.1 测试凭证状态扩展：新增状态正确定义
  - [x] 13.2 测试 useTestConnection hook：**使用 `msw` mock API 调用**（统一选型）
  - [x] 13.3 测试表单组件：按钮禁用/启用逻辑
  - [x] 13.4 测试表单组件：测试连接流程 UI 更新
  - [x] 13.5 **边界条件测试清单**：
    - 表单未填写完整时按钮禁用
    - 测试中按钮禁用 + loading 状态
    - 测试成功后状态切换为 valid
    - 测试失败后状态切换为 invalid + 显示错误消息

## Dev Notes

### ⚠️ Guardrails（必须遵循）

- **日志脱敏是硬性要求 (NFR11)**：API Key 不能以任何形式出现在日志中
- **遵循 ApiResponse<T> 规范 (AR1)**：data/error 互斥，不能同时存在
- **使用 TanStack Query**：不允许在组件中直接使用裸 fetch/axios
- **错误处理层级**：thiserror（库层 core/infra）+ anyhow（应用层 api）
- **本 Story 不持久化凭证**：凭证仍存于前端状态，持久化在 Story 1.5 实现
- **超时配置 (NFR23)**：API 调用超时 ≤ 60 秒
- **本 Story 不实现重试机制 (NFR8)**：重试逻辑在 Story 1.5 或统一 HTTP client 层实现

### 技术实现要点

**后端 HTTP 客户端封装（reqwest）：**

```rust
// backend/src/infra/external/http_client.rs
use reqwest::Client;
use std::time::Duration;

pub fn create_http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(60))      // NFR23: 总超时 60s
        .connect_timeout(Duration::from_secs(10)) // 连接超时 10s
        .build()
        .expect("Failed to create HTTP client")
}
```

**日志脱敏函数：**

```rust
// backend/src/shared/log_sanitizer.rs
pub fn sanitize_api_key(api_key: &str) -> String {
    let len = api_key.len();
    if len <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &api_key[..4], &api_key[len-4..])
}
```

**Dify 连接测试：**

> ⚠️ **重要说明**：
> 1. **本项目连接测试仅验证 Dify App API Key 的有效性**，不区分 Workflow/Chat/Completion 应用类型。
> 2. 实现上统一使用 `POST /v1/completion-messages` 作为验证端点（请求体最小、blocking 模式易判断成功/失败）。
> 3. 如果用户配置的是 Workflow 类型应用，该端点同样可以验证 API Key 是否有效（401 = 无效 Key）。

```rust
// backend/src/infra/external/dify_client.rs
// 
// 【Dify API 连接验证方案】
// 方案 A（推荐）：调用 POST /v1/completion-messages 发送空请求
//   - 请求：{ "inputs": {}, "response_mode": "blocking", "user": "test" }
//   - 成功：返回 200 + 响应体（即使内容为空也算成功）
//   - 失败：返回 401（无效 API Key）或其他错误码
//
// 方案 B（备选）：调用 POST /v1/parameters（如果 Dify 实例支持）
//   - 请求：GET {base_url}/v1/parameters
//   - 成功：返回应用参数信息
//   - 失败：返回 401/404
//
pub async fn test_connection(base_url: &str, api_key: &str) -> Result<TestConnectionResult> {
    let client = create_http_client();
    let url = format!("{}/v1/completion-messages", base_url.trim_end_matches('/'));
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "inputs": {},
            "response_mode": "blocking",
            "user": "connection-test"
        }))
        .send()
        .await?;
    
    match response.status().as_u16() {
        200..=299 => Ok(TestConnectionResult {
            message: "连接成功".to_string(),
            models: None,
        }),
        401 => Err(ConnectionError::InvalidCredentials),
        403 => Err(ConnectionError::Forbidden),
        _ => Err(ConnectionError::UpstreamError(response.status().to_string())),
    }
}
```

**Dify API 响应格式参考：**

```json
// 成功响应 (200)
{
  "answer": "...",
  "conversation_id": "...",
  "created_at": 1234567890
}

// 错误响应 (401)
{
  "code": "unauthorized",
  "message": "Invalid API key"
}
```

**通用大模型连接测试（OpenAI 兼容 API）：**

```rust
// backend/src/infra/external/llm_client.rs
// 硅基流动和魔搭社区均兼容 OpenAI API 格式
// 调用 /v1/models 端点获取模型列表验证凭证
pub async fn test_connection(
    base_url: &str, 
    api_key: &str, 
    provider: &str
) -> Result<TestConnectionResult, ConnectionError> {
    let client = create_http_client();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;
    
    match response.status().as_u16() {
        200..=299 => {
            let models: ModelsResponse = response.json().await?;
            // ✅ AR1 规范：不包含 success 字段，message 是 String 不是 Option<String>
            Ok(TestConnectionResult {
                message: format!("连接成功，可用模型: {}", models.data.len()),
                models: Some(models.data.iter().map(|m| m.id.clone()).collect()),
            })
        }
        401 => Err(ConnectionError::InvalidCredentials),
        403 => Err(ConnectionError::Forbidden),
        _ => Err(ConnectionError::UpstreamError(response.status().to_string())),
    }
}
```

**API 路由定义（符合 AR1 规范）：**

```rust
// backend/src/api/routes/auth.rs
use axum::{routing::post, Router, Json};
use serde::{Deserialize, Serialize};
use crate::api::response::ApiResponse;  // 使用统一的 ApiResponse<T>

#[derive(Deserialize)]
pub struct TestDifyConnectionRequest {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct TestGenericLlmConnectionRequest {
    pub base_url: String,
    pub api_key: String,
    pub provider: String, // "siliconflow" | "modelscope"
}

// ⚠️ 注意：不要在 data 中包含 success 字段！
// AR1 规范要求 data/error 互斥，成功时返回 data，失败时返回 error
#[derive(Serialize)]
pub struct TestConnectionResult {
    pub message: String,
    pub models: Option<Vec<String>>,  // 仅通用 LLM 返回模型列表
}

// 成功响应示例：{ "data": { "message": "连接成功", "models": [...] } }
// 失败响应示例：{ "error": { "code": "AUTH_INVALID_CREDENTIALS", "message": "无效的 API Key" } }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/test-connection/dify", post(test_dify_connection))
        .route("/test-connection/generic-llm", post(test_generic_llm_connection))
}

// Handler 返回类型使用 ApiResponse<TestConnectionResult>
async fn test_dify_connection(
    Json(req): Json<TestDifyConnectionRequest>,
) -> ApiResponse<TestConnectionResult> {
    // 实现略...
}
```

**错误码规范（error.code 命名）：**

| HTTP 状态 | error.code | error.message |
|-----------|------------|---------------|
| 401 | `AUTH_INVALID_CREDENTIALS` | 无效的 API Key |
| 403 | `AUTH_FORBIDDEN` | 访问被拒绝 |
| 408 | `AUTH_CONNECTION_TIMEOUT` | 连接超时 |
| 502 | `AUTH_UPSTREAM_ERROR` | 上游服务不可用 |
| 500 | `AUTH_INTERNAL_ERROR` | 内部错误 |

**前端状态扩展：**

```typescript
// frontend/src/types/credentials.ts
// 扩展 CredentialStatus 类型
export type CredentialStatus = 
  | 'empty'    // 未填写
  | 'filled'   // 已填写，待测试
  | 'testing'  // 测试中
  | 'valid'    // 连接成功
  | 'invalid'; // 连接失败

// UI 状态徽章映射
const statusBadgeMap: Record<CredentialStatus, { color: string; text: string }> = {
  empty: { color: 'gray', text: '未配置' },
  filled: { color: 'yellow', text: '已填写，待测试' },
  testing: { color: 'blue', text: '测试中...' },
  valid: { color: 'green', text: '连接成功' },
  invalid: { color: 'red', text: '连接失败' },
};
```

**TanStack Query Mutation Hook（符合 AR1 规范）：**

```typescript
// frontend/src/features/api-config/hooks/useTestConnection.ts
import { useMutation } from '@tanstack/react-query';
import { testDifyConnection } from '../services/credentialService';  // 从 feature 目录导入
import { useCredentialStore } from '@/stores/useCredentialStore';
import { isApiError } from '@/lib/api';  // 使用类型守卫

export function useTestDifyConnection() {
  const { setDifyStatus } = useCredentialStore();
  
  return useMutation({
    mutationFn: ({ baseUrl, apiKey }: { baseUrl: string; apiKey: string }) =>
      testDifyConnection(baseUrl, apiKey),
    onMutate: () => {
      setDifyStatus('testing');
    },
    onSuccess: (response) => {
      // AR1 规范：使用类型守卫判断 data/error
      if (isApiError(response)) {
        setDifyStatus('invalid');
      } else {
        setDifyStatus('valid');
      }
    },
    onError: () => {
      setDifyStatus('invalid');
    },
  });
}
```

### Provider 特定配置

| Provider | Base URL 默认值 | 备注 |
|----------|-----------------|------|
| 硅基流动 (siliconflow) | `https://api.siliconflow.cn` | OpenAI 兼容 API |
| 魔搭社区 (modelscope) | `https://api.modelscope.cn` | OpenAI 兼容 API |

### UI 状态展示规范

| 状态 | 徽章颜色 | 文案 | 按钮状态 |
|------|----------|------|----------|
| `empty` | 灰色 | 未配置 | 禁用 |
| `filled` | 黄色 (warning) | 已填写，待测试 | 启用 |
| `testing` | 蓝色 (info) | 测试中... | 禁用 + loading |
| `valid` | 绿色 (success) | 连接成功 | 启用（可重新测试） |
| `invalid` | 红色 (error) | 连接失败 | 启用（可重试） |

### 预期影响的文件

**后端（新建）：**
- `backend/src/infra/external/http_client.rs` - HTTP 客户端封装
- `backend/src/infra/external/dify_client.rs` - Dify 连接测试
- `backend/src/shared/log_sanitizer.rs` - 日志脱敏工具
- `backend/src/api/routes/auth.rs` - 测试连接 API 路由

**后端（修改/填充）：**
- `backend/Cargo.toml` - 添加 `reqwest = { version = "0.12", features = ["json", "rustls-tls"] }`
- `backend/src/main.rs` - 添加 `.nest("/api/v1/auth", auth::router())`
- `backend/src/api/routes/mod.rs` - 添加 `pub mod auth;` 导出
- `backend/src/infra/external/mod.rs` - 导出新模块
- `backend/src/infra/external/llm_client.rs` - **填充现有空壳文件**
- `backend/src/shared/mod.rs` - 导出 log_sanitizer

**前端（新建）：**
- `frontend/src/features/api-config/services/credentialService.ts` - API 服务函数（在 feature 目录下）
- `frontend/src/features/api-config/hooks/useTestConnection.ts` - TanStack Query hook

**前端（修改）：**
- `frontend/src/types/credentials.ts` - 扩展状态类型（新增 testing/valid/invalid）
- `frontend/src/stores/useCredentialStore.ts` - 添加状态更新方法
- `frontend/src/features/api-config/DifyCredentialForm.tsx` - 添加测试按钮，复用 FeedbackAlert
- `frontend/src/features/api-config/GenericLlmCredentialForm.tsx` - 添加测试按钮，复用 FeedbackAlert

### 与后续 Story 的关系

| Story | 依赖关系 |
|-------|----------|
| **1.5 凭证持久化** | 使用本 Story 的连接测试逻辑，扩展为"保存前必须测试通过" |
| **2.1 测试集管理** | 依赖 Dify API 连接来解析变量结构 (FR9) |

### References

- [Source: docs/epics.md#Story-1.4] - 验收标准原文
- [Source: docs/stories/1-3-generic-llm-api-credential-configuration.md] - 前置实现模式
- [Source: docs/architecture.md#API-Communication-Patterns] - API 响应格式规范
- [Source: docs/architecture.md#Error-Handling] - 错误处理层级规范
- [Source: Dify API 文档] - Dify 工作流 API 格式
- [Source: reqwest 文档] - Rust HTTP 客户端配置

## Dev Agent Record

### Agent Model Used

Claude 3.5 Sonnet (Cascade)

### Debug Log References

无

### Completion Notes List

- 实现后端 HTTP 客户端封装（reqwest，60s 超时，10s 连接超时）
- 实现日志脱敏工具（sanitize_api_key 函数）
- 实现 Dify 连接测试客户端（POST /v1/completion-messages）
- 实现通用大模型连接测试客户端（GET /v1/models）
- 创建 API 路由（/api/v1/auth/test-connection/dify 和 /api/v1/auth/test-connection/generic-llm）
- 扩展前端 CredentialStatus 类型（新增 testing/valid/invalid 状态）
- 创建 TanStack Query mutation hooks
- 修改表单组件添加"测试连接"按钮
- 后端测试 8 个全部通过
- 前端测试 82 个全部通过

### Change Log

- 2025-12-24: 完成 Story 1.4 全部任务实现
- 2025-12-24: Code Review 修复 - SSRF 防护、HTTP Client 复用、CORS expose_headers、输入验证、错误截断、provider 验证、correlationId 透传
- 2025-12-24: Code Review #2 修复 - modelscope Base URL 前后端统一、React act() 测试警告、超时集成测试、CORS 配置可配置化

### File List

**后端（新建）：**
- backend/src/infra/external/http_client.rs
- backend/src/infra/external/dify_client.rs
- backend/src/shared/log_sanitizer.rs
- backend/src/shared/url_validator.rs（Code Review 新增：SSRF 防护）
- backend/src/api/routes/auth.rs
- backend/tests/connection_test.rs（Code Review 新增：wiremock 集成测试）

**后端（修改）：**
- backend/Cargo.toml（添加 url crate）
- backend/Cargo.lock
- backend/src/main.rs（HTTP Client 复用、CORS expose_headers、CORS 配置可配置化）
- backend/src/api/routes/mod.rs
- backend/src/api/state.rs（添加 http_client 字段）
- backend/src/infra/external/mod.rs
- backend/src/infra/external/llm_client.rs（provider 验证、correlationId 透传）
- backend/src/shared/mod.rs
- backend/src/shared/config.rs（Code Review #2：添加 cors_origins 配置）
- backend/.env.example（Code Review #2：添加 CORS_ORIGINS 配置说明）

**前端（新建）：**
- frontend/src/features/api-config/services/credentialService.ts
- frontend/src/features/api-config/services/credentialService.test.ts（Code Review 新增）
- frontend/src/features/api-config/hooks/useTestConnection.ts
- frontend/src/features/api-config/hooks/useTestConnection.test.tsx（Code Review 新增）

**前端（修改）：**
- frontend/src/types/credentials.ts
- frontend/src/stores/useCredentialStore.ts（注释修正）
- frontend/src/features/api-config/DifyCredentialForm.tsx（展示 message）
- frontend/src/features/api-config/DifyCredentialForm.test.tsx
- frontend/src/features/api-config/GenericLlmCredentialForm.tsx（展示 message/models）
- frontend/src/features/api-config/GenericLlmCredentialForm.test.tsx
