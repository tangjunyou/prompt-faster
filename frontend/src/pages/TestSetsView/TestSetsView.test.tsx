import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { useAuthStore } from '@/stores/useAuthStore'
import type { SaveAsTemplateRequest } from '@/types/generated/api/SaveAsTemplateRequest'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { TestSetTemplateListItemResponse } from '@/types/generated/api/TestSetTemplateListItemResponse'
import type { TestSetTemplateResponse } from '@/types/generated/api/TestSetTemplateResponse'
import type { UserInfo } from '@/types/generated/api/UserInfo'
import { TestSetsView } from './TestSetsView'

const API_BASE = 'http://localhost:3000/api/v1'

let lastSaveAsTemplateBody: SaveAsTemplateRequest | null = null

const server = setupServer(
  http.get(`${API_BASE}/workspaces/:workspaceId/test-sets`, ({ request }) => {
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
        workspace_id: 'ws-1',
        name: '测试集 1',
        description: 'd',
        cases_count: 1,
        created_at: 1,
        updated_at: 2,
      },
    ]
    return HttpResponse.json({ data })
  }),

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
        cases_count: 1,
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
      name: '模板 1',
      description: 'desc',
      cases: [
        {
          id: 'case-1',
          input: { text: 'hi' },
          reference: { Exact: { expected: 'ok' } },
          split: null,
          metadata: null,
        },
      ],
      created_at: 10,
      updated_at: 10,
    }
    return HttpResponse.json({ data })
  })
  ,

  http.post(
    `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/save-as-template`,
    async ({ request }) => {
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      lastSaveAsTemplateBody = (await request.json()) as SaveAsTemplateRequest
      return HttpResponse.json({
        data: {
          id: 'tpl-created',
          workspace_id: 'ws-1',
          name: lastSaveAsTemplateBody.name,
          description: lastSaveAsTemplateBody.description,
          cases: [],
          created_at: 10,
          updated_at: 10,
        },
      })
    }
  )
)

function renderPage(initialEntry: string) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialEntry]}>
        <Routes>
          <Route path="/workspaces/:id/test-sets" element={<TestSetsView />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('TestSetsView templates', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  beforeEach(() => {
    const currentUser: UserInfo = { id: 'u1', username: 'user1' }
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser,
      requiresRegistration: null,
    })
    lastSaveAsTemplateBody = null
  })

  it('选择模板后应预填 name/description/cases', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const openButton = await screen.findByRole('button', { name: '从模板创建' })
    fireEvent.click(openButton)

    const useButton = await screen.findByRole('button', { name: '使用' })
    fireEvent.click(useButton)

    await waitFor(() => {
      expect(screen.getByText('已从模板预填')).toBeInTheDocument()
    })

    const nameInput = screen.getByLabelText('名称') as HTMLInputElement
    expect(nameInput.value).toBe('模板 1')

    const descInput = screen.getByLabelText('描述') as HTMLInputElement
    expect(descInput.value).toBe('desc')

    const casesTextarea = screen.getByLabelText('cases (JSON)') as HTMLTextAreaElement
    expect(casesTextarea.value).toContain('"id": "case-1"')
    expect(casesTextarea.value).toContain('"text": "hi"')
  })

  it('保存为模板应提交请求并显示成功提示', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const openSaveButton = await screen.findByRole('button', { name: '保存为模板' })
    fireEvent.click(openSaveButton)

    const nameInput = screen.getByLabelText('模板名称') as HTMLInputElement
    fireEvent.change(nameInput, { target: { value: '新模板' } })

    const saveButton = screen.getByRole('button', { name: '保存' })
    fireEvent.click(saveButton)

    await waitFor(() => {
      expect(screen.getByText('已保存为模板')).toBeInTheDocument()
    })

    expect(lastSaveAsTemplateBody).toEqual({ name: '新模板', description: 'd' })
  })
})
