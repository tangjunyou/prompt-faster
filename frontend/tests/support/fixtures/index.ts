/**
 * Playwright 测试 Fixtures
 * 
 * 提供可复用的测试上下文和工厂
 */
import { test as base } from '@playwright/test';
import { UserFactory } from './factories/user-factory';
import { WorkspaceFactory } from './factories/workspace-factory';

/**
 * 自定义测试 Fixtures 类型
 */
type TestFixtures = {
  userFactory: UserFactory;
  workspaceFactory: WorkspaceFactory;
};

/**
 * 扩展的 test 对象，包含自定义 fixtures
 */
export const test = base.extend<TestFixtures>({
  userFactory: async ({}, use) => {
    const factory = new UserFactory();
    await use(factory);
    await factory.cleanup();
  },

  workspaceFactory: async ({}, use) => {
    const factory = new WorkspaceFactory();
    await use(factory);
    await factory.cleanup();
  },
});

export { expect } from '@playwright/test';
