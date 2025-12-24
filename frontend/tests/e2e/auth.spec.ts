/**
 * 认证流程 E2E 测试
 * 
 * 覆盖 Story 1.6 AC #2/#4：
 * - 登录后可访问受保护页面
 * - 点击“退出登录”后清理会话并跳回登录页
 */
import { test, expect } from '../support/fixtures';
import { logout } from '../support/helpers/auth';

function randomUsername(): string {
  return `e2e_user_${Date.now()}`;
}

test.describe('认证流程', () => {
  test('登录 -> 访问受保护页面 -> 退出登录 -> 回到登录页', async ({ page, request }) => {
    // 后端未启动则跳过（与 health.spec.ts 保持一致）
    let health;
    try {
      health = await request.get('http://localhost:3000/api/v1/health');
    } catch {
      test.skip(true, '后端 API 未启动');
      return;
    }

    if (!health.ok()) {
      test.skip(true, '后端 API 未启动');
      return;
    }

    const username = randomUsername();
    const password = 'TestPass123!';

    // 统一走“注册”流程，保证系统已有用户时也稳定通过
    await page.goto('/login');

    // 等待登录页加载完成
    await page.locator('[data-testid="username-input"]').waitFor();

    const confirmPasswordInput = page.locator('[data-testid="confirm-password-input"]');
    const toggleAuthModeButton = page.locator('[data-testid="toggle-auth-mode"]');

    // 如果当前是登录模式（没有确认密码输入框），则切换到注册模式
    if (!(await confirmPasswordInput.isVisible())) {
      await toggleAuthModeButton.click();
      await expect(confirmPasswordInput).toBeVisible();
    }

    await page.fill('[data-testid="username-input"]', username);
    await page.fill('[data-testid="password-input"]', password);

    await page.fill('[data-testid="confirm-password-input"]', password);

    await page.click('[data-testid="login-button"]');

    // 登录/注册成功后跳转首页
    await page.waitForURL('/');

    // 顶栏显示当前用户（AC #2）
    await expect(page.locator('[data-testid="user-menu"]')).toContainText(username);

    // 访问受保护页面
    await page.goto('/settings/api');
    await expect(page.getByText('API 配置')).toBeVisible();

    // 退出登录（AC #4）
    await logout(page);
    await expect(page.locator('[data-testid="username-input"]')).toBeVisible();

    // 未登录访问受保护页面应跳转登录页
    await page.goto('/settings/api');
    await page.waitForURL('/login');
  });
});
