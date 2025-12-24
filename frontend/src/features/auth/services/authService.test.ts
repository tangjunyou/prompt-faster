import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import {
  getSystemStatus,
  login,
  register,
  logout,
  getMe,
  type SystemStatusResponse,
  type AuthResponse,
  type UserInfo,
} from './authService'

const API_BASE = 'http://localhost:3000/api/v1'

const server = setupServer(
  http.get(`${API_BASE}/auth/status`, () => {
    const data: SystemStatusResponse = {
      has_users: true,
      requires_registration: false,
    }
    return HttpResponse.json({ data })
  }),

  http.post(`${API_BASE}/auth/register`, async ({ request }) => {
    const body = (await request.json()) as { username: string; password: string }
    const data: AuthResponse = {
      session_token: 'register-token',
      user: { id: 'u-register', username: body.username },
    }
    return HttpResponse.json({ data })
  }),

  http.post(`${API_BASE}/auth/login`, async ({ request }) => {
    const body = (await request.json()) as { username: string; password: string }
    const data: AuthResponse = {
      session_token: 'login-token',
      user: { id: 'u-login', username: body.username },
    }
    return HttpResponse.json({ data })
  }),

  http.post(`${API_BASE}/auth/logout`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        {
          error: {
            code: 'UNAUTHORIZED',
            message: '请先登录',
          },
        },
        { status: 401 }
      )
    }

    return HttpResponse.json({ data: { message: '登出成功' } })
  }),

  http.get(`${API_BASE}/auth/me`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        {
          error: {
            code: 'UNAUTHORIZED',
            message: '请先登录',
          },
        },
        { status: 401 }
      )
    }

    const data: UserInfo = {
      id: 'u-me',
      username: 'me-user',
    }
    return HttpResponse.json({ data })
  })
)

describe('authService', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('getSystemStatus 应返回系统状态', async () => {
    const res = await getSystemStatus()
    expect('data' in res).toBe(true)

    if ('data' in res) {
      expect(res.data.has_users).toBe(true)
      expect(res.data.requires_registration).toBe(false)
    }
  })

  it('register 应返回 session_token 和 user', async () => {
    const res = await register({ username: 'user1', password: 'pass123' })
    expect('data' in res).toBe(true)

    if ('data' in res) {
      expect(res.data.session_token).toBe('register-token')
      expect(res.data.user.username).toBe('user1')
    }
  })

  it('login 应返回 session_token 和 user', async () => {
    const res = await login({ username: 'user2', password: 'pass123' })
    expect('data' in res).toBe(true)

    if ('data' in res) {
      expect(res.data.session_token).toBe('login-token')
      expect(res.data.user.username).toBe('user2')
    }
  })

  it('logout 应通过 apiRequestWithAuth 注入 Authorization header', async () => {
    const res = await logout('test-token')
    expect('data' in res).toBe(true)

    if ('data' in res) {
      expect(res.data.message).toBe('登出成功')
    }
  })

  it('getMe 应通过 apiRequestWithAuth 注入 Authorization header', async () => {
    const res = await getMe('test-token')
    expect('data' in res).toBe(true)

    if ('data' in res) {
      expect(res.data.username).toBe('me-user')
    }
  })
})
