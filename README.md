# Prompt 自动优化（规范开发）

这是一个基于 BMAD（BMad Method）框架的 AI 工作流自动化项目，专注于提示词工程的自动优化和规范化开发。

## 🚀 项目概述

本项目利用 BMAD 框架的完整 AI 驱动敏捷开发流程，包含：

- **12 个专业化 AI 智能体**：从产品经理到架构师，从开发到测试
- **34 个工作流**：覆盖分析、规划、解决方案和实施的完整生命周期
- **自适应系统**：根据项目复杂度自动调整（Level 0-4）
- **多智能体协作**：支持群体决策和创意头脑风暴

## 📁 项目结构

```
prompt 自动优化（规范开发）/
├── .bmad/                    # BMAD 框架核心
│   ├── core/                # 核心模块
│   ├── bmm/                 # BMad Method 模块
│   ├── cis/                 # 创新与战略模块
│   └── _cfg/                # 配置文件
├── .windsurf/               # Windsurf 工作流配置
├── .claude/                 # Claude AI 配置
├── docs/                    # 项目文档和 sprint 产物
├── .gitignore               # Git 忽略文件
└── README.md               # 项目说明
```

## 🎯 Git 追踪配置

本项目配置了 Git 版本控制，追踪以下重要目录：
- `.bmad/` - BMAD 框架配置和工作流
- `.claude/` - Claude AI 配置
- `.windsurf/workflows/bmad/` - Windsurf 工作流配置
- `docs/` - 项目文档（除了 sprint-artifacts）

忽略的文件：
- `.bmad-user-memory/` - 个人用户记忆
- `docs/sprint-artifacts/` - 临时 sprint 产物
- 各种临时文件和缓存

## 🛠️ 快速开始

### 环境要求
- Node.js 18+
- 支持 BMAD 框架的 IDE（推荐 Windsurf 或 Claude Code）

### 初始化步骤
1. 克隆或下载本项目
2. 确保 BMAD 框架已安装（版本 6.0.0-alpha.16）
3. 在 IDE 中加载 Analyst 智能体
4. 运行 `*workflow-init` 初始化项目

---

**准备好开始构建了吗？** → [从快速开始指南开始](./.bmad/bmm/docs/quick-start.md)
