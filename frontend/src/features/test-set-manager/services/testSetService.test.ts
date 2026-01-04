import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { UnauthorizedError } from '@/lib/api'
import {
  createTestSet,
  deleteTestSet,
  getTestSet,
  listTestSets,
  updateTestSet,
} from './testSetService'
import type { CreateTestSetRequest } from '@/types/generated/api/CreateTestSetRequest'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { UpdateTestSetRequest } from '@/types/generated/api/UpdateTestSetRequest'
import type { TestSetResponse } from '@/types/generated/api/TestSetResponse'

const API_BASE = 'http://localhost:3000/api/v1'

const server = setupServer(
  http.get(`${API_BASE}/workspaces/:workspaceId/test-sets`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const data: TestSetListItemResponse[] = [
      {
        id: 'ts-1',
        workspace_id: String(params.workspaceId),
        name: '测试集 1',
        description: 'desc',
        cases_count: 0,
        created_at: 1,
        updated_at: 2,
      },
    ]
    return HttpResponse.json({ data })
  }),

  http.post(`${API_BASE}/workspaces/:workspaceId/test-sets`, async ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const body = (await request.json()) as CreateTestSetRequest
    const data: TestSetResponse = {
      id: 'ts-created',
      workspace_id: String(params.workspaceId),
      name: body.name,
      description: body.description ?? null,
      cases: [],
      created_at: 10,
      updated_at: 10,
    }
    return HttpResponse.json({ data })
  }),

  http.get(`${API_BASE}/workspaces/:workspaceId/test-sets/:id`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const data: TestSetResponse = {
      id: String(params.id),
      workspace_id: String(params.workspaceId),
      name: '测试集详情',
      description: null,
      cases: [],
      created_at: 10,
      updated_at: 11,
    }
    return HttpResponse.json({ data })
  }),

  http.put(`${API_BASE}/workspaces/:workspaceId/test-sets/:id`, async ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const body = (await request.json()) as UpdateTestSetRequest
    const data: TestSetResponse = {
      id: String(params.id),
      workspace_id: String(params.workspaceId),
      name: body.name,
      description: body.description ?? null,
      cases: [],
      created_at: 10,
      updated_at: 12,
    }
    return HttpResponse.json({ data })
  }),

  http.delete(`${API_BASE}/workspaces/:workspaceId/test-sets/:id`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    return HttpResponse.json({ data: { message: '删除成功' } })
  })
)

describe('testSetService', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('listTestSets 应返回测试集列表', async () => {
    const res = await listTestSets('ws-1', 'test-token')
    expect(res.length).toBe(1)
    expect(res[0].id).toBe('ts-1')
  })

  it('listTestSets 在 401 时应抛 UnauthorizedError', async () => {
    await expect(listTestSets('ws-1', 'bad-token')).rejects.toBeInstanceOf(UnauthorizedError)
  })

  it('createTestSet 应返回创建后的测试集', async () => {
    const res = await createTestSet('ws-1', { name: '新测试集', description: null, cases: [] }, 'test-token')
    expect(res.id).toBe('ts-created')
    expect(res.workspace_id).toBe('ws-1')
  })

  it('getTestSet 应返回测试集详情', async () => {
    const res = await getTestSet('ws-1', 'ts-1', 'test-token')
    expect(res.id).toBe('ts-1')
    expect(res.name).toBe('测试集详情')
  })

  it('updateTestSet 应返回更新后的测试集', async () => {
    const res = await updateTestSet('ws-1', 'ts-1', { name: '更新', description: null, cases: [] }, 'test-token')
    expect(res.id).toBe('ts-1')
    expect(res.name).toBe('更新')
  })

  it('deleteTestSet 应返回删除成功消息', async () => {
    const res = await deleteTestSet('ws-1', 'ts-1', 'test-token')
    expect(res.message).toBe('删除成功')
  })
})
