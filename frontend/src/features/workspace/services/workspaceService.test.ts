import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { UnauthorizedError } from '@/lib/api'
import {
  createWorkspace,
  deleteWorkspace,
  getWorkspace,
  listWorkspaces,
} from './workspaceService'
import type { CreateWorkspaceRequest } from '@/types/generated/api/CreateWorkspaceRequest'
import type { WorkspaceResponse } from '@/types/generated/api/WorkspaceResponse'

const API_BASE = 'http://localhost:3000/api/v1'

const server = setupServer(
  http.get(`${API_BASE}/workspaces`, ({ request }) => {
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

    const data: WorkspaceResponse[] = [
      {
        id: 'ws-1',
        name: '工作区 1',
        description: 'desc',
        created_at: 1,
        updated_at: 2,
      },
    ]

    return HttpResponse.json({ data })
  }),

  http.post(`${API_BASE}/workspaces`, async ({ request }) => {
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

    const body = (await request.json()) as CreateWorkspaceRequest
    const data: WorkspaceResponse = {
      id: 'ws-created',
      name: body.name,
      description: body.description ?? null,
      created_at: 10,
      updated_at: 10,
    }

    return HttpResponse.json({ data })
  }),

  http.get(`${API_BASE}/workspaces/:id`, ({ request, params }) => {
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

    const data: WorkspaceResponse = {
      id: String(params.id),
      name: '工作区详情',
      description: null,
      created_at: 10,
      updated_at: 11,
    }

    return HttpResponse.json({ data })
  }),

  http.delete(`${API_BASE}/workspaces/:id`, ({ request }) => {
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

    return HttpResponse.json({ data: { message: '删除成功' } })
  })
)

describe('workspaceService', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('listWorkspaces 应返回工作区列表', async () => {
    const res = await listWorkspaces('test-token')
    expect(res.length).toBe(1)
    expect(res[0].id).toBe('ws-1')
  })

  it('listWorkspaces 在 401 时应抛 UnauthorizedError', async () => {
    await expect(listWorkspaces('bad-token')).rejects.toBeInstanceOf(UnauthorizedError)
  })

  it('createWorkspace 应返回创建后的工作区', async () => {
    const res = await createWorkspace({ name: '新工作区', description: 'd' }, 'test-token')
    expect(res.id).toBe('ws-created')
    expect(res.name).toBe('新工作区')
  })

  it('getWorkspace 应返回工作区详情', async () => {
    const res = await getWorkspace('ws-1', 'test-token')
    expect(res.id).toBe('ws-1')
    expect(res.name).toBe('工作区详情')
  })

  it('deleteWorkspace 应返回删除成功消息', async () => {
    const res = await deleteWorkspace('ws-1', 'test-token')
    expect(res.message).toBe('删除成功')
  })
})
