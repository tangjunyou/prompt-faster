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

## 🎯 核心功能

### 智能体团队
- **PM**：产品管理
- **Analyst**：业务分析
- **Architect**：系统架构
- **DEV**：开发实现
- **TEA**：测试与质量保证
- **UX Designer**：用户体验设计
- **Technical Writer**：技术文档
- **SM**：Scrum 管理

### 工作流阶段
1. **分析阶段**（可选）：市场调研、竞争分析、技术趋势
2. **规划阶段**（必需）：PRD、UX 设计、架构决策
3. **解决方案阶段**（Level 3-4）：史诗和用户故事创建
4. **实施阶段**（迭代）：代码开发、评审、部署
5. **测试阶段**（并行）：自动化测试、性能测试

## 🛠️ 快速开始

### 环境要求
- Node.js 18+
- 支持 BMAD 框架的 IDE（推荐 Windsurf 或 Claude Code）

### 初始化步骤
1. 克隆或下载本项目
2. 确保 BMAD 框架已安装（版本 6.0.0-alpha.16）
3. 在 IDE 中加载 Analyst 智能体
4. 运行 `*workflow-init` 初始化项目

### 开发流程
```bash
# 新项目
*workflow-init

# 现有项目（棕地）
*document-project
*workflow-init
```

## 📚 文档资源

- [BMAD 完整文档](./.bmad/bmm/docs/README.md)
- [快速开始指南](./.bmad/bmm/docs/quick-start.md)
- [智能体指南](./.bmad/bmm/docs/agents-guide.md)
- [自适应系统](./.bmad/bmm/docs/scale-adaptive-system.md)
- [常见问题](./.bmad/bmm/docs/faq.md)

## 🎮 特殊功能

### Party Mode
启用多智能体协作模式，用于：
- 战略决策
- 创意头脑风暴
- 复杂问题解决

### Scale-Adaptive System
- **Level 0-1**：快速规范流程（错误修复、小功能）
- **Level 2**：PRD + 可选架构
- **Level 3-4**：完整 PRD + 综合架构

## 🔧 配置说明

### BMAD 配置
- 版本：6.0.0-alpha.16
- 语言：中文
- 输出目录：`./docs`
- 用户记忆：`./.bmad-user-memory`

### IDE 集成
- Windsurf：完全支持
- Claude Code：通过 MCP 服务器支持
- 其他 IDE：通过标准工具集成

## 🤝 贡献指南

1. 使用 BMAD 标准工作流
2. 遵循既定的智能体协作模式
3. 在 docs/sprint-artifacts/ 中记录 sprint 产物
4. 保持配置文件同步

## 📄 许可证

本项目遵循 BMAD 框架的许可证条款。

---

**准备好开始构建了吗？** → [从快速开始指南开始](./.bmad/bmm/docs/quick-start.md)