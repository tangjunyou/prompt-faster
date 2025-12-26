import { test, expect } from '../support/fixtures'

function randomUsername(prefix: string): string {
  return `${prefix}_${Date.now()}_${Math.floor(Math.random() * 1000)}`
}

test.describe('工作区访问控制', () => {
  test('用户 B 无法通过 ID 访问用户 A 的工作区', async ({ request }) => {
    let health
    try {
      health = await request.get('http://localhost:3000/api/v1/health')
    } catch {
      test.skip(true, '后端 API 未启动')
      return
    }

    if (!health.ok()) {
      test.skip(true, '后端 API 未启动')
      return
    }

    const password = 'TestPass123!'

    const usernameA = randomUsername('e2e_ws_user_a')
    const usernameB = randomUsername('e2e_ws_user_b')

    const registerA = await request.post('http://localhost:3000/api/v1/auth/register', {
      data: { username: usernameA, password },
    })
    expect(registerA.ok()).toBe(true)
    const registerAJson = await registerA.json()
    const tokenA = registerAJson.data.session_token as string

    const registerB = await request.post('http://localhost:3000/api/v1/auth/register', {
      data: { username: usernameB, password },
    })
    expect(registerB.ok()).toBe(true)
    const registerBJson = await registerB.json()
    const tokenB = registerBJson.data.session_token as string

    const createWs = await request.post('http://localhost:3000/api/v1/workspaces', {
      data: { name: 'A Workspace', description: 'desc' },
      headers: { Authorization: `Bearer ${tokenA}` },
    })
    expect(createWs.ok()).toBe(true)
    const createWsJson = await createWs.json()
    const workspaceId = createWsJson.data.id as string

    const getByB = await request.get(`http://localhost:3000/api/v1/workspaces/${workspaceId}`, {
      headers: { Authorization: `Bearer ${tokenB}` },
    })

    expect(getByB.status()).toBe(404)
  })
})
