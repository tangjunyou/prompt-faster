# System-Level Test Design - Prompt Faster

**Date:** 2025-12-21
**Author:** 耶稣
**Workflow:** `*test-design` (System-Level Mode - Phase 3)
**Status:** Draft

---

## Executive Summary

本文档是 Prompt Faster 项目的系统级测试设计，在 Solutioning 阶段（Phase 3）进行架构可测试性审查，为后续 Sprint 0 的测试框架搭建和实现阶段的 Epic 级测试计划提供指导。

**项目概览：**
- **技术栈：** Rust (Axum 0.8) 后端 + React 19 前端 + SQLite + WebSocket
- **核心功能：** AI Prompt 自动迭代优化系统
- **复杂度：** Medium-High（实时 WebSocket 流式输出 + 四层处理器算法 + 7 Trait 体系）

**可测试性评估结果：** ✅ **PASS with CONCERNS**

> **CONCERNS 原因：** 存在风险分=9 的 CRITICAL 项（ASR-001 优化成功率、WebSocket 重连状态同步），缓解产物将作为 Sprint 0 Gate 条件。本阶段不阻塞方案推进，但在 Sprint 0 Gate 前必须以产物缓解。

---

## Testability Assessment

### Controllability（可控性）

**评估结果：** ✅ **PASS**

| 评估项 | 状态 | 说明 |
|--------|------|------|
| 系统状态控制 | ✅ | 7 Trait 体系支持依赖注入；SQLite 支持内存数据库测试；Checkpoint 机制支持状态控制 |
| 外部依赖可 Mock | ✅ | `ExecutionTarget` 和 `TeacherModel` Trait 封装 LLM 调用；`llm_client.rs` 唯一外部调用点 |
| 错误条件可触发 | ✅ | `thiserror` + `anyhow` 错误体系；统一 `ApiResponse<T>` 响应结构 |

**架构支持细节：**

1. **Trait 体系设计**
   - 7 个核心 Trait 定义明确接口，支持 Mock 替换
   - `RuleEngine`, `PromptGenerator`, `Evaluator`, `FeedbackAggregator`, `Optimizer`, `TeacherModel`, `ExecutionTarget`

2. **数据访问抽象**
   - Repository 模式：`task_repo`, `test_case_repo`, `checkpoint_repo` 等
   - 唯一数据库访问点，易于测试隔离

3. **外部服务封装**
   - `infra/external/llm_client.rs` - 唯一 LLM 调用点
   - `infra/external/api_key_manager.rs` - 唯一敏感数据处理点

### Observability（可观测性）

**评估结果：** ✅ **PASS with CONCERNS**

| 评估项 | 状态 | 说明 |
|--------|------|------|
| 系统状态可检查 | ✅ | `tracing` + `tracing-subscriber` 结构化日志；correlationId 全链路透传 |
| 测试结果确定性 | ✅ | 统一 `ApiResponse<T>` 结构；时间戳统一为 Unix 毫秒 |
| NFR 验证能力 | ⚠️ | `observability.rs` 预留但未实现；缺少性能指标收集机制 |

**关注点：**

- **CONCERNS:** `observability.rs` 标注为“预留：Prometheus/OTel/tracing 指标上报（MVP 仅 trace 日志）”

**可观测性路径（短期/长期）：**
- **短期（Sprint 0）：** Server-Timing/等价字段 + correlationId 全链路透传 + E2E 校验（已任务化为 P1）
- **长期（MVP 后期）：** OpenTelemetry/Prometheus 指标上报，支持更完整的性能 NFR 验证

**MVP 最小可验证闭环：**
- 确保 `correlationId` 在 HTTP 响应头与 WebSocket 消息 payload 中可观测
- 关键 API 输出最小耗时信号（如 `Server-Timing` 或等价字段）
- `tracing` 日志包含 `correlationId` 与关键耗时字段
- **对应测试：** E2E 校验响应头/消息体包含 correlationId；性能基准记录并输出 p95/p99；日志字段存在性作为门禁检查项

### Reliability（可靠性）

**评估结果：** ✅ **PASS**

| 评估项 | 状态 | 说明 |
|--------|------|------|
| 测试隔离 | ✅ | Rust `#[cfg(test)]` + `backend/tests/`；Vitest + `*.test.ts(x)` |
| 可重现失败 | ✅ | Checkpoint 断点续跑；SQLite WAL + FULL synchronous |
| 组件松耦合 | ✅ | 7 Trait 体系；后端 api → core → domain → infra → shared 分层 |

**测试位置约定：**

| 类型 | 位置 | 命名 |
|------|------|------|
| Rust 单元测试 | 同文件 `#[cfg(test)]` | — |
| Rust 集成测试 | `backend/tests/` | `test_*.rs` |
| 前端单元测试 | 同目录 | `*.test.ts(x)` |
| 前端 E2E 测试 | `frontend/tests/e2e/` | `*.spec.ts` |

---

## Architecturally Significant Requirements (ASRs)

从 PRD NFRs 和架构决策中识别的质量需求，按风险评分排序：

| ASR ID | 需求 | 来源 | 架构影响 | 概率 | 影响 | 风险分 | 测试策略 |
|--------|------|------|----------|------|------|--------|----------|
| ASR-001 | 优化成功率 ≥ 90% | NFR21 | 四层处理器 + 7 Trait 体系 | 3 | 3 | **9** | 官方基准测试集（10-20 个标准任务） |
| ASR-002 | API Key 加密存储 | NFR9 | AES-GCM + Argon2 派生密钥 | 2 | 3 | **6** | 单元测试验证加密实现 + E2E 安全测试 |
| ASR-003 | 断点恢复率 100% | NFR5 | SQLite WAL + Checkpoint | 1 | 3 | 3 | 集成测试覆盖 kill/断网/断电/跨版本 |
| ASR-004 | 系统延迟 < 100ms | NFR1 | Tokio 异步 + 内存状态管理 | 2 | 2 | 4 | Mock LLM 的性能基准测试 |
| ASR-005 | 流式首字节 < 500ms | NFR2 | WebSocket + 流式输出 | 2 | 2 | 4 | WebSocket 首字节时间测量 |

**高风险 ASR 详情：**

### ASR-001: 优化成功率 ≥ 90% (风险分 = 9, CRITICAL)

**风险描述：** 核心算法有效性是项目成败关键。如果算法成功率 < 70%，产品价值归零。

**成功判定标准（三层结构）：**

| 层级 | 来源 | 判定口径 |
|------|------|----------|
| **层 1：硬门禁** | FR33 + Success Criteria | 固定任务：测试集全部通过（或达到阈值通过率）；创意任务：约束满足（必含/禁止/格式）+ 评分达标 |
| **层 2：并行质量** | NFR22 | 并行 vs 串行成功率差异 < 5% |
| **层 3：失败可解释** | FR30 + FR31 | 汇总分析 + 失败档案（用于审计与学习，非成功定义本身） |

**固定任务成功判定（FR12）：**
- 结构化字段匹配（如 JSON Schema 验证）
- 关键字段完全一致或达到容差阈值
- 测试集通过率 ≥ 配置阈值（默认 100%）

**创意任务成功判定（FR13/FR14）：**
- 约束满足（必含关键词、禁止词、格式要求）
- 评分函数达标（如 ≥ 0.8）
- 人工抽检比例（如 10%，用于校准评分函数）

**测试策略：**
- 创建官方基准测试集（10-20 个标准优化任务）
- 覆盖固定任务（有标准输出）和创意任务（无标准输出）
- 每次发布前必跑基准测试
- 成功率阈值：≥ 90%（正式）、≥ 70%（早期验证）

**缓解措施：**
- 算法研究时间盒 6 周，必须产出可测试原型
- 借鉴 OPRO、Reflexion、DSPy SIMBA、PromptWizard 成熟方案

### ASR-002: API Key 加密存储 (风险分 = 6, HIGH)

**风险描述：** API Key 泄露可能导致用户经济损失和安全事件。

**测试策略：**
- 单元测试：验证 AES-GCM 加密/解密正确性
- 单元测试：验证 Argon2 密钥派生
- E2E 测试：验证 API Key 不在日志/响应/前端中泄露
- 安全扫描：npm audit / cargo audit

---

## Test Levels Strategy

基于架构特点（Rust 后端 + React 前端 + SQLite + WebSocket + LLM 集成），推荐测试分布：

### 推荐比例：60/25/15（单元/集成/E2E）

| 测试级别 | 比例 | 覆盖范围 | 工具 | 执行频率 |
|----------|------|----------|------|----------|
| **单元测试** | 60% | 7 Trait 实现、业务逻辑、数据转换、错误处理 | Rust `#[test]` + tokio-test, Vitest | 每次提交 |
| **集成测试** | 25% | API 端点、数据库操作、Trait 交互、Repository 层 | tokio-test + SQLite 内存库, API 测试 | 每次 PR |
| **E2E 测试** | 15% | 关键用户旅程、WebSocket 流程、用户介入操作 | Playwright | 每次 PR + Nightly |

### 选型理由

1. **单元测试占主导 (60%)**
   - 7 Trait 体系中的业务逻辑适合单元测试
   - Rust 类型系统 + 纯函数设计天然适合快速单元测试
   - 算法模块（RuleEngine, Evaluator, Optimizer）需要大量边界条件测试

2. **集成测试中等占比 (25%)**
   - API 合约验证（`/api/v1/*` 端点）
   - 数据库事务和 Checkpoint 完整性
   - Trait 组合交互（如 IterationEngine 编排多个 Trait）

3. **E2E 测试聚焦关键路径 (15%)**
   - WebSocket 实时流程（节点状态变化、流式输出）
   - 用户介入操作（暂停、编辑、继续）
   - 三视图模式切换（Run View, Focus View, Workspace View）

---

## NFR Testing Approach

### Security（安全性）

| NFR | 测试方法 | 工具 | 验收标准 |
|-----|----------|------|----------|
| NFR9 API Key 加密 | 单元测试 AES-GCM 实现 | Rust `#[test]` | 加密/解密往返测试通过 |
| NFR10 凭证仅本地 | 集成测试验证无外部传输 | 网络拦截 | 无外发请求包含凭证 |
| NFR11 日志脱敏 | 单元测试 + 日志审查 | grep 脚本 | 日志中不含 API Key / 密码 |
| NFR11a 用户认证 | E2E 测试登录流程 | Playwright | Argon2 哈希存储、通用错误提示 |
| NFR11b 数据隔离 | 集成测试 user_id 过滤 | API 测试 | 用户 A 无法访问用户 B 数据 |

**安全测试重点：**
- 认证绕过测试：未认证用户访问受保护路由 → 重定向登录
- 授权测试：用户只能访问自己的工作区/任务
- 密码安全：错误密码返回通用提示，不泄露用户存在性
- API Key 保护：日志、响应、前端均不暴露完整 Key

### Performance（性能）

| NFR | 测试方法 | 工具 | 验收标准 |
|-----|----------|------|----------|
| NFR1 系统延迟 < 100ms | 基准测试（Mock LLM） | k6 / Criterion (Rust) | p95 < 100ms |
| NFR2 流式首字节 < 500ms | WebSocket 时间测量 | Playwright + 自定义计时 | 首字节 < 500ms |
| NFR3 节点图渲染 60fps | 帧率测量 | Playwright Performance API | 稳定 60fps（合理节点规模） |
| NFR4 并行测试集线性加速 | 并行 vs 串行对比 | 集成测试 | 差异 < 5% (NFR22) |

**性能测试策略：**
- **系统延迟测试**：Mock LLM 响应，测量纯系统开销
- **流式输出测试**：WebSocket 连接后计时到首字节
- **负载测试**：k6 模拟多用户并发任务创建/查询

**性能测试脚本位置约定：**
| 类型 | 位置 | 触发方式 |
|------|------|----------|
| k6 负载测试 | `tests/perf/k6/*.js` | CI nightly / 手动 |
| Criterion benchmark | `backend/benches/*.rs` | `cargo bench` / CI nightly |
| Playwright 性能 | `frontend/tests/e2e/perf/*.spec.ts` | CI nightly / 手动 |

**性能基准负载条件（阈值有效前提）：**
| 指标 | 并发条件 | 数据规模 |
|------|----------|----------|
| NFR1 系统延迟 | 1/10/50 并发用户 | 单任务、测试集 ≤10 个用例 |
| NFR2 流式首字节 | 1 并发 | 单任务、单流式输出 |
| NFR3 节点图渲染 | 前端单用户 | 节点数 ≤50、边数 ≤100 |

### Reliability（可靠性）

| NFR | 测试方法 | 工具 | 验收标准 |
|-----|----------|------|----------|
| NFR5 断点恢复率 100% | 故障注入测试 | 集成测试 + kill/网络断开 | 4 种场景全部恢复 |
| NFR6 WAL + FULL sync | 数据库事务测试 | SQLite 测试 | 断电后数据完整 |
| NFR7 Checkpoint 完整性 | 集成测试 | API 测试 | 每步 Checkpoint 可加载 |
| NFR8 API 重试 3 次 | Mock 503 响应 | 集成测试 | 第 3 次成功时通过 |

**可靠性测试场景：**
- 进程被杀死（kill -9）后恢复
- 网络断开中途后重连
- 数据库写入中途断电
- 跨版本升级后恢复旧数据

### Maintainability（可维护性）

| NFR | 测试方法 | 工具 | 验收标准 |
|-----|----------|------|----------|
| NFR12-14 新增模块 < 4h | Trait 实现验证 | Code review | 实现 Trait 即可集成 |
| NFR15 算法替换仅影响算法模块 | 模块依赖分析 | cargo tree / 集成测试 | 无级联修改 |
| NFR19 核心流程 E2E 覆盖 ≥ 80% | 用例/旅程覆盖率 | Playwright 用例执行报告 | 关键路径用例覆盖 ≥ 80% |
| 代码行覆盖率（后端/前端） | 覆盖率报告 | cargo-tarpaulin + vitest coverage | 后端 ≥ 80% + 前端 ≥ 80%（CI Linux） |
| NFR20 模块回归 100% | CI 测试门禁 | GitHub Actions | 所有测试通过 |

**覆盖率约束：** `cargo-tarpaulin` 在非 Linux 环境可用性有限，覆盖率以 **CI（Linux）产物为准**；本地仅作为辅助，不作为门禁依据。

---

## Test Environment Requirements

### 本地开发环境

> **注意：** 以下为示意配置，实际端口/healthcheck/依赖就绪条件以 Sprint 0 框架落地为准。

```yaml
# docker-compose.yml
# 路径说明：DATABASE_URL 中的 ./data 指容器内路径，通过 volume 映射到宿主机 ./data
services:
  backend:
    build: ./backend
    working_dir: /app
    environment:
      - DATABASE_URL=sqlite:///data/test.db?mode=wal  # 容器内绝对路径
      - RUST_LOG=debug
    volumes:
      - ./backend:/app
      - ./data:/data  # 宿主机 ./data 映射到容器 /data

  frontend:
    build: ./frontend
    environment:
      - VITE_API_URL=http://localhost:3000
    volumes:
      - ./frontend:/app

  # 用于 E2E 测试
  # 版本要求：>= v1.48（支持 routeWebSocket），以仓库 lockfile 为准
  playwright:
    image: mcr.microsoft.com/playwright:v1.51.0
    depends_on:
      - backend
      - frontend
```

### 测试数据库策略

| 测试类型 | 数据库配置 | 说明 |
|----------|------------|------|
| 单元测试 | Mock / 无数据库 | 纯业务逻辑测试 |
| 集成测试（逻辑/事务） | SQLite 内存数据库 (`:memory:`) | 适用于 API 逻辑、事务边界测试 |
| 可靠性 NFR（NFR5/6/7） | **SQLite 文件库 + WAL + FULL sync** | 必须用于 kill/断网/断电/跨版本恢复验证（`:memory:` 无法覆盖 WAL 回放语义） |
| E2E 测试 | SQLite 文件数据库 | 建议按 worker 使用独立文件（避免并行互相污染），并提供 suite 级重置策略 |

### Mock 服务需求

| 外部依赖 | Mock 策略 |
|----------|-----------|
| LLM API (硅基流动/魔搭) | `MockTeacherModel` Trait 实现 |
| Dify 工作流 API | `MockExecutionTarget` Trait 实现 |
| WebSocket | Playwright `routeWebSocket`（v1.48+）拦截/修改/Mock；`page.on('websocket')` 用于监听与断言 |

---

## Testability Concerns

### ⚠️ CONCERNS（需关注但不阻塞当前阶段）

> **治理说明：** 以下项目在 Phase 3 不阻塞方案推进，但在 Sprint 0 Gate 前必须以产物缓解。风险分=9 的项目已纳入 Sprint 0 任务表并标为 P0。

#### 1. LLM 依赖测试复杂度

**问题：** 外部 LLM API 调用使测试依赖网络和 API 可用性，且响应不确定。

**缓解措施：**
- `ExecutionTarget` / `TeacherModel` Trait 支持 Mock 实现
- 创建 `MockTeacherModel` 返回预定义响应
- 创建 `MockExecutionTarget` 模拟 Dify/直连响应

**Mock Conformance/契约套件（确保 Mock 与真实行为一致）：**
- **记录-回放（record/replay）：** 从真实 Dify/LLM 录制若干交互（脱敏后）作为 fixture
- **差分测试（differential）：** 同一输入同时跑 Real 与 Mock，对齐关键字段（状态机转移、错误码、延迟分布范围等）
- **明确边界：** token 级流式可抽象为 chunk 级，但事件序列必须一致

**契约套件最小验收标准：**
- 至少覆盖：成功响应、限流/超时、模型返回格式错误、流式中断
- 差分对齐字段：状态机转移、错误码、关键事件序列（start/chunk/end/error）
- 延迟分布：Mock 响应时间应在真实响应时间的 p50~p95 范围内

**建议实现：**
```rust
// backend/src/core/teacher_model/mock_impl.rs
pub struct MockTeacherModel {
    responses: HashMap<String, String>,
}

impl TeacherModel for MockTeacherModel {
    async fn generate(&self, prompt: &str) -> Result<String> {
        self.responses.get(prompt)
            .cloned()
            .ok_or_else(|| anyhow!("No mock response for prompt"))
    }
}
```

#### 2. WebSocket 流式测试

**问题：** 实时流式输出测试需要特殊处理时序和事件序列。

**缓解措施：**
- Playwright `routeWebSocket`（v1.48+）支持拦截/修改/Mock WebSocket 消息
- `page.on('websocket')` 用于监听与断言事件序列
- 定义标准 WebSocket 事件序列用于确定性验证

**关键约束（官方文档强调）：**
- `routeWebSocket` 只对“调用之后新建的 WebSocket”生效
- 推荐在导航前调用（或更早：在 context 创建/新 page 前调用）

**工程注意事项：**
- 如果页面侧/服务端侧任一边设置了消息处理器，**自动转发可能被覆盖**
- 需要显式把消息转发到另一侧（page→server 和 server→page 都需要处理）
- 示例中已处理 page→server，若需处理 server→page 需注册 `server.onMessage`

**建议实现（拦截/Mock）：**
```typescript
// frontend/tests/e2e/helpers/ws-mock.ts
import { Page } from '@playwright/test';

// 使用 routeWebSocket 拦截并 Mock WebSocket 消息
export async function mockWebSocket(page: Page, responses: Record<string, string>) {
  await page.routeWebSocket('/ws', ws => {
    ws.onMessage(message => {
      const response = responses[message];
      if (response) {
        ws.send(response);
      }
    });
  });
}

// 使用 routeWebSocket 拦截并转发到真实服务器（双向转发 + 可修改消息）
export async function interceptWebSocket(
  page: Page, 
  transformToServer: (msg: string) => string,
  transformToPage: (msg: string) => string
) {
  await page.routeWebSocket('/ws', ws => {
    const server = ws.connectToServer();
    // page → server：拦截并转换后转发
    ws.onMessage(message => {
      server.send(transformToServer(message));
    });
    // server → page：必须显式注册，否则自动转发可能被覆盖
    server.onMessage(message => {
      ws.send(transformToPage(message));
    });
  });
}
```

**建议实现（监听/断言）：**
```typescript
// frontend/tests/e2e/helpers/ws-helpers.ts
import { Page } from '@playwright/test';

// 使用 page.on('websocket') 监听并断言事件序列
export async function captureWsMessages(page: Page, urlPattern: string): Promise<string[]> {
  const messages: string[] = [];
  
  page.on('websocket', ws => {
    if (ws.url().includes(urlPattern)) {
      ws.on('framereceived', event => messages.push(event.payload.toString()));
    }
  });
  
  return messages;
}
```

#### 3. 算法有效性验证

**问题：** NFR21（≥90% 成功率）需要官方基准测试集，目前不存在。

**风险等级：** 高（风险分 = 9，详见 ASR-001）

**缓解措施（详见 ASR-001 测试策略）：**
- Sprint 0 创建 10-20 个标准优化任务作为基准
- 覆盖固定任务和创意任务两种模式
- 定义明确的成功判定标准

**基准测试集结构建议：**
```
tests/
├── benchmark/
│   ├── v1/                    # 版本化基准集
│   │   ├── fixed-tasks/       # 固定任务（有标准输出）
│   │   │   ├── contract-extraction.json
│   │   │   ├── data-structuring.json
│   │   │   └── ...
│   │   ├── creative-tasks/    # 创意任务（无标准输出）
│   │   │   ├── copywriting.json
│   │   │   ├── summarization.json
│   │   │   └── ...
│   │   └── manifest.json      # 版本元信息
│   └── benchmark-runner.rs    # 基准测试执行器
```

**基准测试集版本治理：**
- Gate 绑定固定版本（如 `benchmarks/v1`），发布前必跑该版本
- 新增用例只增不改；改动旧用例需要新版本并给迁移说明
- 每个用例保存元信息（任务类型 fixed/creative、来源、期望、约束、评分方式）

**用例元信息示例：**
```json
{
  "id": "fixed-001",
  "type": "fixed",
  "name": "合同关键信息提取",
  "source": "internal",
  "input": { "prompt": "...", "test_cases": [...] },
  "expected": { "schema": "contract_v1.json", "threshold": 1.0 },
  "scoring": "exact_match"
}
```

#### 4. WebSocket 重连状态同步

**问题：** 网络断开后 WebSocket 重连，状态如何对齐？

**重连协议测试矩阵：**

| 场景 | 断言点 | 测试方式 |
|------|--------|----------|
| 重连后自动重新订阅 | 重连后是否收到之前订阅的事件 | E2E 模拟 offline/online |
| 丢失事件补偿 | 断网期间事件是否在重连后补发 | `routeWebSocket` 注入丢包 |
| 幂等性 | 重复消息是否被正确处理 | `routeWebSocket` 注入重复消息 |
| sequenceId/lastAck | 是否有序列号机制确保顺序 | **待设计**：若采用则需增加协议字段；否则需明确替代机制（server 重放窗口/客户端补偿请求/订阅重放 token） |
| 乱序处理 | 乱序消息是否被正确排序/拒绝 | `routeWebSocket` 注入乱序消息 |

**风险等级：** 高（风险分 = 9）

**缓解措施（与 Phase 3 Gate 缓解计划对齐）：**
- **重连协议最小字段/机制定义**：Sprint 0 完成协议设计（replay token / 重放窗口 / ack 机制择一）
- **最小 E2E 故障注入用例**：offline/online 模拟 + 丢包/重复/乱序注入的最小闭环
- **协议文档更新**：在 architecture.md 中补充重连协议字段定义

---

## Recommendations for Sprint 0

### 1. 测试基础设施搭建

| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| 配置 Rust 测试框架（tokio-test） | P0 | 2h |
| 配置 Vitest + React Testing Library | P0 | 2h |
| 配置 Playwright E2E 环境 | P0 | 4h |
| 创建 CI 流水线（lint + test + build） | P0 | 4h |
| 创建 Docker Compose 测试环境 | P1 | 4h |

### 2. Mock 实现

| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| 实现 `MockTeacherModel` | P0 | 4h |
| 实现 `MockExecutionTarget` | P0 | 4h |
| 创建测试数据工厂（Rust + TypeScript） | P1 | 8h |
| 配置 SQLite 内存数据库测试 | P1 | 2h |

### 3. 基准测试集（⭐ 风险分=9 缓解产物）

| 任务 | 优先级 | 预估工时 | 备注 |
|------|--------|----------|------|
| 设计基准测试集结构 | P0 | 4h | Gate 强制 |
| 创建 5 个固定任务测试用例 | **P0** | 8h | Gate 要求 ≥10 个任务 |
| 创建 5 个创意任务测试用例 | **P0** | 8h | Gate 要求 ≥10 个任务 |
| 实现基准测试执行器 + 成功判定逻辑 | **P0** | 8h | Gate 强制 |

### 4. WebSocket 重连缓解产物（⭐ 风险分=9 缓解产物）

| 任务 | 优先级 | 预估工时 | 备注 |
|------|--------|----------|------|
| 重连协议字段定义（replay token/重放窗口/ack 择一） | **P0** | 4h | Gate 强制 |
| 更新 architecture.md 重连协议文档 | **P0** | 2h | Gate 强制 |
| 最小 E2E 故障注入用例（offline/online + 丢包/重复/乱序） | **P0** | 8h | Gate 强制 |

### 5. 最小可观测性闭环

| 任务 | 优先级 | 预估工时 | 备注 |
|------|--------|----------|------|
| 实现 correlationId 全链路透传（HTTP 响应头 + WS payload） | P1 | 4h | NFR1/2 证据链前提 |
| 实现关键 API 最小耗时信号（Server-Timing 或等价字段） | P1 | 4h | NFR1/2 证据链前提 |
| E2E 校验 correlationId/耗时字段存在性 | P1 | 2h | 门禁检查项 |

### 6. 文档与规范

| 任务 | 优先级 | 预估工时 |
|------|--------|----------|
| 编写测试编码规范 | P1 | 4h |
| 创建测试示例模板 | P1 | 4h |
| 配置覆盖率报告 | P2 | 2h |

---

## Quality Gate Criteria

### Phase 3 Gate（Solutioning 完成）

- [x] 架构可测试性评估完成
- [ ] **CONCERNS**: 存在风险分=9 的 CRITICAL 项（见下方说明）
- [x] ASRs 识别并风险评分
- [x] 测试级别策略定义
- [x] NFR 测试方法明确
- [x] Sprint 0 建议制定

**风险分=9 缓解计划（作为 Sprint 0 Gate 条件）：**
- **ASR-001 优化成功率 ≥90%**：Sprint 0 必须完成基准测试集 v1（≥10 个任务）+ 成功判定逻辑实现
- **WebSocket 重连状态同步**：Sprint 0 必须完成重连协议字段定义 + 最小 E2E 注入测试

### Sprint 0 Gate（测试框架就绪）

- [ ] CI 流水线配置完成（lint + test + build）
- [ ] Mock Trait 实现可用
- [ ] 测试数据工厂可用
- [ ] 基准测试集初版完成（≥ 10 个任务）
- [ ] E2E 环境可运行
- [ ] 测试编码规范/模板完成（确定性/无硬等待/并行安全）
- [ ] 风险分=9 缓解产物完成（见 Phase 3 Gate 说明）

### Implementation Gate（每个 Epic 完成时）

- [ ] **后端行覆盖率 ≥ 80%** + **前端行覆盖率 ≥ 80%**（CI Linux 产物为准）
- [ ] **核心流程 E2E 覆盖率 ≥ 80%**（按用例/旅程覆盖，对齐 NFR19）
- [ ] 所有 P0 测试通过
- [ ] 无高风险（≥6）未缓解项
- [ ] 相关 NFR 验证通过（性能类 NFR1/2/3 默认 nightly 跑，PR 仅跑 smoke 基准）

---

## Appendix

### Knowledge Base References

- `nfr-criteria.md` - NFR 验证标准（安全、性能、可靠性、可维护性）
- `test-levels-framework.md` - 测试级别选择指南
- `risk-governance.md` - 风险评分与治理
- `test-quality.md` - 测试质量完成定义

### Related Documents

- PRD: `docs/project-planning-artifacts/prd.md`
- Architecture: `docs/project-planning-artifacts/architecture.md`
- Epics: `docs/project-planning-artifacts/epics.md`
- UX Design: `docs/project-planning-artifacts/ux-design-specification.md`

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `_bmad/bmm/workflows/testarch/test-design` (System-Level Mode)
**Version**: 4.0 (BMad v6)
