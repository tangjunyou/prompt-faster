# Prompt Faster

> AI Prompt 自动迭代优化系统

## 📋 项目概述

Prompt Faster 是一个桌面端 AI Prompt 自动迭代优化工具，采用四层架构设计，通过规律抽取、Prompt 生成、质量评估和反思迭代的循环，自动优化用户的 Prompt。

## 🛠 技术栈

### 后端
- **语言**: Rust (Edition 2024)
- **框架**: Axum 0.8
- **数据库**: SQLite (WAL 模式)
- **ORM**: SQLx
- **API 文档**: utoipa + Swagger UI

### 前端
- **框架**: React 19 + TypeScript 5.x
- **路由**: React Router 7
- **状态管理**: TanStack Query
- **UI**: TailwindCSS
- **图标**: Lucide React

### DevOps
- **容器化**: Docker Compose
- **CI/CD**: GitHub Actions

## 🚀 快速开始

### 前置要求
- Rust 1.83+
- Node.js 22+
- Docker (可选)

### 本地开发

**后端:**
```bash
cd backend
cp .env.example .env
cargo run
```

**前端:**
```bash
cd frontend
npm install
npm run dev
```

**Docker Compose:**
```bash
docker compose up -d
docker compose ps

# 最小可用检查
curl -fsS http://localhost:3000/api/v1/health
curl -fsS http://localhost:5173
```

**Docker Compose 环境变量（以 `docker-compose.yml` 为准）：**
- `APP_ENV`: 后端运行环境（默认 `development`）
- `SERVER_HOST` / `SERVER_PORT`: 后端监听地址/端口（Compose 默认 `0.0.0.0:3000`）
- `DATABASE_URL`: SQLite 文件库路径（Compose 默认 `sqlite:data/prompt_faster.db?mode=rwc`，并通过 volume `backend-data` 持久化在容器内的 `/app/data`）
- `RUST_LOG`: 后端日志级别
- `VITE_API_URL`: 前端调用后端 API 的 base URL（Compose 默认 `http://localhost:3000/api/v1`）

### 访问地址
- 前端: http://localhost:5173
- 后端 API: http://localhost:3000/api/v1
- 健康检查: http://localhost:3000/api/v1/health

## 📁 项目结构

```
prompt-faster/
├── backend/                 # Rust 后端
│   ├── src/
│   │   ├── api/            # API 层（路由、处理器、中间件）
│   │   ├── core/           # 核心业务逻辑（7 Trait）
│   │   ├── domain/         # 领域模型
│   │   ├── infra/          # 基础设施（数据库、外部服务）
│   │   └── shared/         # 共享工具
│   └── migrations/         # 数据库迁移
├── frontend/               # React 前端
│   ├── src/
│   │   ├── components/     # 通用组件
│   │   ├── features/       # 功能模块
│   │   ├── pages/          # 页面组件
│   │   ├── lib/            # 工具库
│   │   ├── hooks/          # 自定义 Hooks
│   │   └── types/          # TypeScript 类型
│   └── tests/              # 测试文件
├── docs/                   # 项目文档
├── docker-compose.yml      # Docker 配置
└── .github/workflows/      # CI 配置
```

## 📖 文档

- [产品需求文档](docs/implementation-artifacts/prd.md)
- [架构设计](docs/implementation-artifacts/architecture.md)
- [UX 设计规范](docs/implementation-artifacts/ux-design-specification.md)
- [Epic 与 Story 分解](docs/implementation-artifacts/epics.md)
- [测试设计](docs/implementation-artifacts/test-design-system.md)

## 🧪 测试

**后端测试:**
```bash
cd backend
cargo test
```

**前端测试:**
```bash
cd frontend
npm run test
```

## 📝 License

MIT
