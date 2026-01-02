# 前端快速上手

## 1. 环境准备

- Node.js（建议 18 或 20）
- npm（与项目现有 `package-lock.json` 保持一致）

## 2. 安装依赖

```bash
cd frontend
npm install
```

## 3. 启动开发服务器

```bash
npm run dev
```

## 4. 代码结构导读

建议按以下顺序浏览：

1. `frontend/src/main.tsx` - Provider 入口
2. `frontend/src/App.tsx` - 路由与布局
3. `frontend/src/pages/WorkspaceView/WorkspaceView.tsx` - 标准页面示例
4. `frontend/src/features/workspace/hooks/useWorkspaces.ts` - TanStack Query hooks
5. `frontend/src/features/workspace/services/workspaceService.ts` - API Service 层

## 5. 推荐开发顺序

1. 明确需求 → 确认对应的后端 DTO
2. 如 DTO 变更，运行 `cargo run --bin gen-types`
3. 在 `services/` 添加请求
4. 在 `hooks/` 封装 `useQuery/useMutation`
5. 在 `pages/` 实现页面 UI 与状态分支

## 6. 常用脚本

```bash
npm test
npm run lint
```
