import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { UnauthorizedError } from '@/lib/api'
import { refreshDifyVariables, saveDifyConfig } from './difyService'
import type { DifyVariablesResponse } from '@/types/generated/api/DifyVariablesResponse'
import type { SaveDifyConfigRequest } from '@/types/generated/api/SaveDifyConfigRequest'
import type { SaveDifyConfigResponse } from '@/types/generated/api/SaveDifyConfigResponse'

const API_BASE = 'http://localhost:3000/api/v1'

const server = setupServer(
  http.post(
    `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/dify/variables/refresh`,
    ({ request }) => {
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      const data: DifyVariablesResponse = {
        variables: [
          {
            name: 'system_prompt',
            component: 'text-input',
            type: 'string',
            required: true,
            required_known: true,
            default_value: null,
            raw: null,
          },
        ],
      }
      return HttpResponse.json({ data })
    }
  ),

  http.put(
    `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/dify/config`,
    async ({ request }) => {
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      const body = (await request.json()) as SaveDifyConfigRequest
      const data: SaveDifyConfigResponse = {
        difyConfig: {
          targetPromptVariable: body.targetPromptVariable,
          bindings: body.bindings,
          parametersSnapshot: null,
        },
      }
      return HttpResponse.json({ data })
    }
  )
)

describe('difyService', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('refreshDifyVariables 应返回 variables', async () => {
    const res = await refreshDifyVariables('ws-1', 'ts-1', 'test-token')
    expect(res.variables.length).toBe(1)
    expect(res.variables[0].name).toBe('system_prompt')
  })

  it('refreshDifyVariables 在 401 时应抛 UnauthorizedError', async () => {
    await expect(refreshDifyVariables('ws-1', 'ts-1', 'bad-token')).rejects.toBeInstanceOf(
      UnauthorizedError
    )
  })

  it('saveDifyConfig 应返回 difyConfig', async () => {
    const res = await saveDifyConfig(
      'ws-1',
      'ts-1',
      { targetPromptVariable: 'system_prompt', bindings: {} },
      'test-token'
    )
    expect(res.difyConfig.targetPromptVariable).toBe('system_prompt')
  })

  it('refreshDifyVariables 在 502 时应抛出上游错误 message', async () => {
    server.use(
      http.post(
        `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/dify/variables/refresh`,
        ({ request }) => {
          const auth = request.headers.get('authorization')
          if (auth !== 'Bearer test-token') {
            return HttpResponse.json(
              { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
              { status: 401 }
            )
          }

          return HttpResponse.json(
            { error: { code: 'UPSTREAM_ERROR', message: 'Dify 服务异常，请稍后重试' } },
            { status: 502 }
          )
        }
      )
    )

    await expect(refreshDifyVariables('ws-1', 'ts-1', 'test-token')).rejects.toThrow(
      'Dify 服务异常，请稍后重试'
    )
  })
})
