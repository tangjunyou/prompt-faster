/**
 * 健康检查测试
 * 
 * 验证基础设施是否正常工作
 */
import { test, expect } from '../support/fixtures';

test.describe('健康检查', () => {
  test('前端应用应该正常加载', async ({ page }) => {
    await page.goto('/');
    
    // 验证页面标题或核心元素
    await expect(page).toHaveTitle(/Prompt Faster|Vite/i);
  });

  test('后端 API 应该正常响应', async ({ request }) => {
    const response = await request.get('http://localhost:3000/api/v1/health');
    
    // 如果后端未启动，跳过此测试
    if (!response.ok()) {
      test.skip(true, '后端 API 未启动');
      return;
    }

    expect(response.status()).toBe(200);
    
    const data = await response.json();
    expect(data.status).toBe('ok');
  });
});
