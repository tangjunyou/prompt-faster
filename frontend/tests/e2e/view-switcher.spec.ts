import { test, expect } from '../support/fixtures'

test.describe('视图切换快捷键', () => {
  test('Ctrl+1/2/3 切换 Run/Focus/Workspace 视图', async ({ page }) => {
    await page.goto('/run')

    await expect(page.getByTestId('run-view')).toBeVisible()

    await page.keyboard.press('Control+2')
    await expect(page.getByTestId('focus-view')).toBeVisible()

    await page.keyboard.press('Control+3')
    await expect(page.getByTestId('workspace-view')).toBeVisible()

    await page.keyboard.press('Control+1')
    await expect(page.getByTestId('run-view')).toBeVisible()
  })
})
