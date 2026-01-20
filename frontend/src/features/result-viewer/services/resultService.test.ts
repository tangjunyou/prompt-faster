import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { exportResult } from './resultService'
import { useAuthStore } from '@/stores/useAuthStore'

const API_BASE = 'http://localhost:3000/api/v1'

const server = setupServer(
  http.get(`${API_BASE}/tasks/:taskId/result/export`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    return HttpResponse.json({
      data: {
        content: '{"ok":true}',
        format: 'json',
        filename: '任务:导出?结果<>.json',
      },
    })
  })
)

describe('resultService', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
  })

  it('应对文件名进行安全化处理', async () => {
    const { filename } = await exportResult('task-1', 'json')
    expect(filename).not.toMatch(/[\\/:*?"<>|]/)
    expect(filename.endsWith('.json')).toBe(true)
  })
})
