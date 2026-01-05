import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { UnauthorizedError } from '@/lib/api'
import {
  getTestSetTemplate,
  listTestSetTemplates,
  saveAsTemplate,
} from './testSetTemplateService'
import type { SaveAsTemplateRequest } from '@/types/generated/api/SaveAsTemplateRequest'
import type { TestSetTemplateListItemResponse } from '@/types/generated/api/TestSetTemplateListItemResponse'
import type { TestSetTemplateResponse } from '@/types/generated/api/TestSetTemplateResponse'

const API_BASE = 'http://localhost:3000/api/v1'

const server = setupServer(
  http.get(`${API_BASE}/workspaces/:workspaceId/test-set-templates`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const data: TestSetTemplateListItemResponse[] = [
      {
        id: 'tpl-1',
        workspace_id: String(params.workspaceId),
        name: '模板 1',
        description: 'desc',
        cases_count: 2,
        created_at: 1,
        updated_at: 2,
      },
    ]
    return HttpResponse.json({ data })
  }),

  http.get(`${API_BASE}/workspaces/:workspaceId/test-set-templates/:id`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const data: TestSetTemplateResponse = {
      id: String(params.id),
      workspace_id: String(params.workspaceId),
      name: '模板详情',
      description: null,
      cases: [],
      dify_config: null,
      generic_config: null,
      created_at: 10,
      updated_at: 11,
    }
    return HttpResponse.json({ data })
  }),

  http.post(
    `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/save-as-template`,
    async ({ request, params }) => {
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      const body = (await request.json()) as SaveAsTemplateRequest
      const data: TestSetTemplateResponse = {
        id: 'tpl-created',
        workspace_id: String(params.workspaceId),
        name: body.name,
        description: body.description ?? null,
        cases: [],
        dify_config: null,
        generic_config: null,
        created_at: 20,
        updated_at: 20,
      }
      return HttpResponse.json({ data })
    }
  )
)

describe('testSetTemplateService', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('listTestSetTemplates 应返回模板列表', async () => {
    const res = await listTestSetTemplates('ws-1', 'test-token')
    expect(res.length).toBe(1)
    expect(res[0].id).toBe('tpl-1')
  })

  it('listTestSetTemplates 在 401 时应抛 UnauthorizedError', async () => {
    await expect(listTestSetTemplates('ws-1', 'bad-token')).rejects.toBeInstanceOf(UnauthorizedError)
  })

  it('getTestSetTemplate 应返回模板详情', async () => {
    const res = await getTestSetTemplate('ws-1', 'tpl-1', 'test-token')
    expect(res.id).toBe('tpl-1')
    expect(res.name).toBe('模板详情')
  })

  it('saveAsTemplate 应返回创建后的模板', async () => {
    const res = await saveAsTemplate(
      'ws-1',
      'ts-1',
      { name: '新模板', description: null },
      'test-token'
    )
    expect(res.id).toBe('tpl-created')
    expect(res.workspace_id).toBe('ws-1')
    expect(res.name).toBe('新模板')
  })

  it('saveAsTemplate 在 400 时应抛出错误', async () => {
    server.use(
      http.post(
        `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/save-as-template`,
        ({ request }) => {
          const auth = request.headers.get('authorization')
          if (auth !== 'Bearer test-token') {
            return HttpResponse.json(
              { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
              { status: 401 }
            )
          }

          return HttpResponse.json(
            { error: { code: 'VALIDATION_ERROR', message: '模板名称不能为空' } },
            { status: 400 }
          )
        }
      )
    )

    await expect(
      saveAsTemplate('ws-1', 'ts-1', { name: '', description: null }, 'test-token')
    ).rejects.toThrow('模板名称不能为空')
  })
})
