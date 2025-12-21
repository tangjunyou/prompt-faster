# Prompt Faster 测试套件

## 概述

本目录包含 Prompt Faster 的端到端测试和集成测试。

## 目录结构

```
tests/
├── e2e/                    # 端到端测试
│   └── health.spec.ts      # 健康检查测试
├── support/                # 测试支持文件
│   ├── fixtures/           # Playwright Fixtures
│   │   ├── index.ts        # Fixtures 入口
│   │   └── factories/      # 测试数据工厂
│   ├── helpers/            # 测试辅助函数
│   └── page-objects/       # 页面对象模型（可选）
└── README.md               # 本文件
```

## 运行测试

```bash
# 运行所有 E2E 测试
npm run test:e2e

# 使用 UI 模式运行
npm run test:e2e:ui

# 调试模式
npm run test:e2e:debug

# 运行特定测试文件
npm run test:e2e -- tests/e2e/health.spec.ts

# 生成测试报告
npm run test:e2e -- --reporter=html
```

## 测试数据工厂

使用 `UserFactory` 和 `WorkspaceFactory` 创建测试数据：

```typescript
import { test, expect } from '../support/fixtures';

test('示例测试', async ({ page, userFactory, workspaceFactory }) => {
  // 创建测试用户
  const user = await userFactory.createUser();
  
  // 创建工作区
  const workspace = await workspaceFactory.createWorkspace(user.id);
  
  // 测试完成后自动清理
});
```

## 最佳实践

1. **使用 data-testid**: 为可交互元素添加 `data-testid` 属性
2. **避免硬编码等待**: 使用 `waitFor*` 辅助函数
3. **隔离测试数据**: 使用工厂创建独立的测试数据
4. **自动清理**: Fixtures 会自动清理创建的数据

## 配置

测试配置文件: `playwright.config.ts`

环境变量:
- `BASE_URL`: 前端应用地址（默认: http://localhost:5173）
- `API_URL`: 后端 API 地址（默认: http://localhost:3000/api/v1）
