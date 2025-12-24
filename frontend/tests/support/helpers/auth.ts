/**
 * 认证测试辅助函数
 */
import { Page } from '@playwright/test';

/**
 * 执行登录操作
 */
export async function login(
  page: Page,
  username: string,
  password: string
): Promise<void> {
  await page.goto('/login');
  await page.fill('[data-testid="username-input"]', username);
  await page.fill('[data-testid="password-input"]', password);
  await page.click('[data-testid="login-button"]');
  
  // 等待登录完成
  await page.waitForURL('/');
}

/**
 * 执行登出操作
 */
export async function logout(page: Page): Promise<void> {
  await page.click('[data-testid="logout-button"]');
  
  // 等待登出完成
  await page.waitForURL('/login');
}

/**
 * 检查是否已登录
 */
export async function isLoggedIn(page: Page): Promise<boolean> {
  const userMenu = page.locator('[data-testid="user-menu"]');
  return await userMenu.isVisible();
}
