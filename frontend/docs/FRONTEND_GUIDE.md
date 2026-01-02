# 前端开发指南

## 技术栈

- React 19
- React Router 7
- TanStack Query 5
- ts-rs 类型生成（后端 Rust → 前端 TypeScript）
- Zustand（全局状态）

## 项目结构约定

```
frontend/src/
├── App.tsx                  # 路由配置
├── main.tsx                 # Provider 入口
├── pages/                   # 页面级组件
├── features/                # 业务功能模块
│   ├── */services/          # 纯函数 Service（禁止 hooks）
│   └── */hooks/             # TanStack Query hooks
├── components/              # 通用组件
├── lib/                     # API 与 QueryClient 等基础能力
├── stores/                  # Zustand 状态
└── types/                   # 类型导出入口（含 ts-rs 生成类型）
```

> 规则：业务组件中禁止直接 `fetch/axios`，必须通过 `services` + `hooks`。

## 标准页面开发流程

1. 页面组件只负责 UI 与状态分支（loading / error / empty / success）。
2. 数据获取/变更使用 `features/*/hooks` 暴露的 `useQuery/useMutation`。
3. 类型从 `@/types/generated/*` 引入，避免手写 DTO。

示例（参考 `WorkspaceView`）：

```tsx
const { data, isLoading, error } = useWorkspaces()
const { mutateAsync, isPending } = useCreateWorkspace()
const workspaces: WorkspaceResponse[] = data ?? []
```

## TanStack Query 使用指南

- **查询**：在 hooks 内统一使用 `useQuery`，页面只消费返回值。
- **变更**：使用 `useMutation`，成功后 `invalidateQueries` 触发刷新。
- **错误处理**：只展示 `error.message`；不要在 UI 中展示 `error.details`。
- **重试策略**：已在 `lib/query-client.ts` 统一配置，401 不重试。

## ts-rs 类型生成流程

1. 后端 DTO 增加 `#[derive(TS)]` 标注。
2. 在后端执行：

```bash
cargo run --bin gen-types
```

3. 生成文件输出到：

```
frontend/src/types/generated/
├── api/
└── models/
```

## 常见问题

**Q: 为什么页面组件不能直接调用 fetch？**

A: 统一走 `services` + `hooks`，便于缓存、错误处理、测试和复用。

**Q: 如何新增一个 API 模块？**

A: 先在 `services/` 添加纯函数请求，再在 `hooks/` 封装 TanStack Query。

**Q: 什么时候需要重新生成类型？**

A: 任何后端 DTO 变更后都需要执行 `cargo run --bin gen-types`。
