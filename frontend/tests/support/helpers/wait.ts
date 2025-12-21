/**
 * 等待辅助函数
 */
import { Page } from '@playwright/test';

/**
 * 等待 API 请求完成
 */
export async function waitForApiResponse(
  page: Page,
  urlPattern: string | RegExp
): Promise<void> {
  await page.waitForResponse(
    (response) => {
      const url = response.url();
      if (typeof urlPattern === 'string') {
        return url.includes(urlPattern);
      }
      return urlPattern.test(url);
    },
    { timeout: 30000 }
  );
}

/**
 * 等待加载状态消失
 */
export async function waitForLoadingComplete(page: Page): Promise<void> {
  const loading = page.locator('[data-testid="loading"]');
  await loading.waitFor({ state: 'hidden', timeout: 30000 });
}

/**
 * 等待 Toast 消息出现
 */
export async function waitForToast(
  page: Page,
  message: string | RegExp
): Promise<void> {
  const toast = page.locator('[data-testid="toast"]');
  await toast.waitFor({ state: 'visible' });
  
  if (typeof message === 'string') {
    await toast.filter({ hasText: message }).waitFor();
  } else {
    await toast.filter({ hasText: message }).waitFor();
  }
}
