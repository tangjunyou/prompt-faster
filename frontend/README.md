# Prompt Faster Frontend

## 技术栈

- React 19
- React Router 7
- TanStack Query 5
- ts-rs（后端 Rust 类型生成）
- Zustand

## 常用脚本

```bash
npm run dev
npm run build
npm run lint
npm test
```

## 路由约定

- `/run`：Run View（默认视图）
- `/focus`：Focus View
- `/workspace`：Workspace View
- `/login`：登录/注册
- `/settings/api`：API 配置

## 类型生成

在后端执行：

```bash
cd backend
cargo run --bin gen-types
```

生成文件输出到：

```
frontend/src/types/generated/
```

## 参考文档

- `frontend/docs/FRONTEND_GUIDE.md`
- `frontend/docs/ONBOARDING.md`
