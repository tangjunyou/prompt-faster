import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { useAuthStore } from '@/stores/useAuthStore'
import type { CreateTestSetRequest } from '@/types/generated/api/CreateTestSetRequest'
import type { SaveAsTemplateRequest } from '@/types/generated/api/SaveAsTemplateRequest'
import type { DifyVariablesResponse } from '@/types/generated/api/DifyVariablesResponse'
import type { SaveDifyConfigRequest } from '@/types/generated/api/SaveDifyConfigRequest'
import type { SaveGenericConfigRequest } from '@/types/generated/api/SaveGenericConfigRequest'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { TestSetTemplateListItemResponse } from '@/types/generated/api/TestSetTemplateListItemResponse'
import type { TestSetTemplateResponse } from '@/types/generated/api/TestSetTemplateResponse'
import type { TestSetResponse } from '@/types/generated/api/TestSetResponse'
import type { UserInfo } from '@/types/generated/api/UserInfo'
import { TestSetsView } from './TestSetsView'

const API_BASE = 'http://localhost:3000/api/v1'

let lastSaveAsTemplateBody: SaveAsTemplateRequest | null = null
let lastSaveDifyConfigBody: SaveDifyConfigRequest | null = null
let lastSaveGenericConfigBody: SaveGenericConfigRequest | null = null
let lastCreateTestSetBody: CreateTestSetRequest | null = null
let deleteGenericCalled = false

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
      name: '测试集 1',
      description: 'd',
      cases: [
        {
          id: 'case-1',
          input: { foo: 'bar' },
          reference: { Exact: { expected: 'ok' } },
          split: null,
          metadata: null,
        },
      ],
      dify_config: null,
      generic_config: null,
      created_at: 1,
      updated_at: 2,
    }
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
    lastCreateTestSetBody = body
    const data: TestSetResponse = {
      id: 'ts-created',
      workspace_id: String(params.workspaceId),
      name: body.name ?? 'ts-created',
      description: body.description ?? null,
      cases: [],
      dify_config: null,
      generic_config: null,
      created_at: 10,
      updated_at: 10,
    }
    return HttpResponse.json({ data })
  }),

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
          {
            name: 'k',
            component: 'number',
            type: 'number',
            required: false,
            required_known: true,
            default_value: 3,
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

      lastSaveDifyConfigBody = (await request.json()) as SaveDifyConfigRequest
      return HttpResponse.json({
        data: {
          difyConfig: {
            targetPromptVariable: lastSaveDifyConfigBody.targetPromptVariable,
            bindings: lastSaveDifyConfigBody.bindings,
            parametersSnapshot: null,
          },
        },
      })
    }
  ),

  http.put(
    `${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/generic/config`,
    async ({ request }) => {
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      lastSaveGenericConfigBody = (await request.json()) as SaveGenericConfigRequest
      return HttpResponse.json({
        data: {
          genericConfig: {
            variables: lastSaveGenericConfigBody.variables,
          },
        },
      })
    }
  ),

  http.delete(`${API_BASE}/workspaces/:workspaceId/test-sets/:testSetId/generic/config`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    deleteGenericCalled = true
    return HttpResponse.json({ data: { message: '已清空' } })
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
      dify_config: null,
      generic_config: null,
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
          dify_config: null,
          generic_config: null,
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
    lastSaveDifyConfigBody = null
    lastSaveGenericConfigBody = null
    lastCreateTestSetBody = null
    deleteGenericCalled = false
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

  it('Dify 变量配置：刷新、选择优化目标、保存', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const editButton = await screen.findByRole('button', { name: '编辑' })
    fireEvent.click(editButton)

    await waitFor(() => {
      expect(screen.getByText('Dify 变量配置')).toBeInTheDocument()
    })

    const refreshButton = screen.getByRole('button', { name: '刷新/解析变量' })
    fireEvent.click(refreshButton)

    await waitFor(() => {
      expect(screen.getByText('system_prompt')).toBeInTheDocument()
      expect(screen.getByText('k')).toBeInTheDocument()
    })

    const targetSelect = screen.getByLabelText('待优化 system prompt 变量') as HTMLSelectElement
    fireEvent.change(targetSelect, { target: { value: 'system_prompt' } })

    const fixedSelect = screen.getAllByDisplayValue('未配置（使用默认值/省略）')[0] as HTMLSelectElement
    fireEvent.change(fixedSelect, { target: { value: 'fixed' } })

    const jsonEditor = await screen.findByPlaceholderText('例如："hello" / 123 / true / {"a":1} / [1,2]')
    fireEvent.change(jsonEditor, { target: { value: '5' } })

    const saveButton = screen.getByRole('button', { name: '保存 Dify 配置' })
    fireEvent.click(saveButton)

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(lastSaveDifyConfigBody?.targetPromptVariable).toBe('system_prompt')
    expect(lastSaveDifyConfigBody?.bindings?.k?.source).toBe('fixed')
  })

  it('Dify 变量配置：解析失败应展示友好错误并可重试', async () => {
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

    renderPage('/workspaces/ws-1/test-sets')

    const editButton = await screen.findByRole('button', { name: '编辑' })
    fireEvent.click(editButton)

    await waitFor(() => {
      expect(screen.getByText('Dify 变量配置')).toBeInTheDocument()
    })

    const refreshButton = screen.getByRole('button', { name: '刷新/解析变量' })
    fireEvent.click(refreshButton)

    await waitFor(() => {
      expect(screen.getByText('解析失败：Dify 服务异常，请稍后重试')).toBeInTheDocument()
    })

    const retryButton = screen.getByRole('button', { name: '重试' })
    expect(retryButton).toBeInTheDocument()
  })

  it('通用 API 自定义变量：新增变量并保存', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const editButton = await screen.findByRole('button', { name: '编辑' })
    fireEvent.click(editButton)

    await waitFor(() => {
      expect(screen.getByText('通用 API 自定义变量')).toBeInTheDocument()
    })

    const addButton = screen.getByRole('button', { name: '新增变量' })
    fireEvent.click(addButton)

    const nameInput = await screen.findByPlaceholderText('变量名（唯一）')
    fireEvent.change(nameInput, { target: { value: 'x' } })
    const row = nameInput.closest('tr')
    expect(row).not.toBeNull()
    const defaultInput = within(row as HTMLElement).getByPlaceholderText('可选')
    fireEvent.change(defaultInput, { target: { value: 'y' } })

    const saveButton = screen.getByRole('button', { name: '保存通用变量' })
    fireEvent.click(saveButton)

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(lastSaveGenericConfigBody?.variables?.[0]?.name).toBe('x')
    expect(lastSaveGenericConfigBody?.variables?.[0]?.valueType).toBe('string')
    expect(lastSaveGenericConfigBody?.variables?.[0]?.defaultValue).toBe('y')
  })

  it('通用 API 自定义变量：编辑态禁用并清空应调用 delete 接口', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const editButton = await screen.findByRole('button', { name: '编辑' })
    fireEvent.click(editButton)

    const addButton = await screen.findByRole('button', { name: '新增变量' })
    fireEvent.click(addButton)

    const nameInput = await screen.findByPlaceholderText('变量名（唯一）')
    fireEvent.change(nameInput, { target: { value: 'x' } })

    const originalConfirm = window.confirm
    window.confirm = () => true
    try {
      const disableButton = screen.getByRole('button', { name: '禁用并清空' })
      fireEvent.click(disableButton)

      await waitFor(() => {
        expect(screen.getByText('已禁用并清空')).toBeInTheDocument()
      })

      expect(deleteGenericCalled).toBe(true)
    } finally {
      window.confirm = originalConfirm
    }
  })

  it('创建测试集：通用变量（创建态）应随创建写入', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const nameInput = (await screen.findByLabelText('名称')) as HTMLInputElement
    fireEvent.change(nameInput, { target: { value: '新测试集' } })

    const addButton = screen.getByRole('button', { name: '新增变量' })
    fireEvent.click(addButton)

    const varNameInput = await screen.findByPlaceholderText('变量名（唯一）')
    fireEvent.change(varNameInput, { target: { value: 'x' } })
    const row = varNameInput.closest('tr')
    expect(row).not.toBeNull()
    const defaultInput = within(row as HTMLElement).getByPlaceholderText('可选')
    fireEvent.change(defaultInput, { target: { value: 'y' } })

    const createButton = screen.getByRole('button', { name: '创建测试集' })
    fireEvent.click(createButton)

    await waitFor(() => {
      expect(screen.getByText('创建成功（含通用变量）')).toBeInTheDocument()
    })

    expect(lastCreateTestSetBody?.name).toBe('新测试集')
    expect(lastCreateTestSetBody?.generic_config?.variables?.[0]?.name).toBe('x')
  })

  it('创建测试集：通用变量超 32KB 时应阻止创建并提示错误', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const nameInput = (await screen.findByLabelText('名称')) as HTMLInputElement
    fireEvent.change(nameInput, { target: { value: '超大配置测试集' } })

    const addButton = screen.getByRole('button', { name: '新增变量' })
    fireEvent.click(addButton)

    const varNameInput = await screen.findByPlaceholderText('变量名（唯一）')
    fireEvent.change(varNameInput, { target: { value: 'big' } })
    const row = varNameInput.closest('tr')
    expect(row).not.toBeNull()
    const defaultInput = within(row as HTMLElement).getByPlaceholderText('可选')
    fireEvent.change(defaultInput, { target: { value: 'a'.repeat(40 * 1024) } })

    const createButton = screen.getByRole('button', { name: '创建测试集' })
    fireEvent.click(createButton)

    await waitFor(() => {
      expect(screen.getByText('保存失败：配置过大：最大 32KB')).toBeInTheDocument()
    })

    expect(lastCreateTestSetBody).toBeNull()
    expect(lastSaveGenericConfigBody).toBeNull()
  })

  it('标准答案编辑：应写入 cases[*].reference.Exact.expected', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const editButton = await screen.findByRole('button', { name: '编辑' })
    fireEvent.click(editButton)

    const expectedEditor = await screen.findByPlaceholderText('输入标准答案（期望输出）')
    fireEvent.change(expectedEditor, { target: { value: 'new-expected' } })

    const casesTextarea = screen.getByLabelText('cases (JSON)') as HTMLTextAreaElement
    expect(casesTextarea.value).toContain('"expected": "new-expected"')
  })

  it('创意任务配置：应写回 casesJson 并在创建请求体中包含 core_request/constraints.params', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const nameInput = (await screen.findByLabelText('名称')) as HTMLInputElement
    fireEvent.change(nameInput, { target: { value: '创意测试集' } })

    const casesTextarea = screen.getByLabelText('cases (JSON)') as HTMLTextAreaElement
    fireEvent.change(casesTextarea, {
      target: {
        value: JSON.stringify(
          [
            {
              id: 'case-1',
              input: { prompt: '写欢迎文案' },
              reference: { Constrained: { constraints: [], quality_dimensions: [] } },
              split: null,
              metadata: null,
            },
            {
              id: 'case-2',
              input: { prompt: 'x' },
              reference: { Exact: { expected: 'ok' } },
              split: null,
              metadata: null,
            },
          ],
          null,
          2
        ),
      },
    })

    await screen.findByText('创意任务配置（Constrained）')

    const coreRequest = await screen.findByPlaceholderText('输入核心诉求（自然语言）')
    fireEvent.change(coreRequest, { target: { value: '友好、简洁、鼓励探索' } })

    const minChars = await screen.findByLabelText('最小字符数')
    fireEvent.change(minChars, { target: { value: '30' } })
    const maxChars = await screen.findByLabelText('最大字符数')
    fireEvent.change(maxChars, { target: { value: '120' } })

    const mustInclude = await screen.findByLabelText('必含关键词（每行一个）')
    fireEvent.change(mustInclude, { target: { value: '欢迎\\n一起' } })
    const mustExclude = await screen.findByLabelText('禁止内容（每行一个）')
    fireEvent.change(mustExclude, { target: { value: '政治\\n敏感' } })

    const format = await screen.findByLabelText('格式要求')
    fireEvent.change(format, { target: { value: 'markdown' } })

    expect(casesTextarea.value).toContain('"core_request": "友好、简洁、鼓励探索"')
    expect(casesTextarea.value).toContain('"name": "length"')
    expect(casesTextarea.value).toContain('"minChars": 30')
    expect(casesTextarea.value).toContain('"maxChars": 120')
    expect(casesTextarea.value).toContain('"name": "must_include"')
    expect(casesTextarea.value).toContain('"keywords": [')
    expect(casesTextarea.value).toContain('"name": "must_exclude"')
    expect(casesTextarea.value).toContain('"name": "format"')
    expect(casesTextarea.value).toContain('"format": "markdown"')

    const createButton = screen.getByRole('button', { name: '创建测试集' })
    fireEvent.click(createButton)

    await waitFor(() => {
      expect(screen.getByText('创建成功')).toBeInTheDocument()
    })

    type CreatedCase = {
      reference?: {
        Constrained?: {
          core_request?: unknown
          constraints?: Array<{ params?: unknown }>
        }
        Exact?: {
          expected?: unknown
        }
      }
    }

    const createdCases = lastCreateTestSetBody?.cases as unknown as CreatedCase[] | undefined
    expect(createdCases?.[0]?.reference?.Constrained?.core_request).toBe('友好、简洁、鼓励探索')
    expect(createdCases?.[0]?.reference?.Constrained?.constraints?.[0]?.params).toEqual({
      minChars: 30,
      maxChars: 120,
    })
    expect(createdCases?.[1]?.reference?.Exact?.expected).toBe('ok')
  })

  it('创意任务配置：当 params 非对象时应提示并允许用户选择是否覆盖', async () => {
    renderPage('/workspaces/ws-1/test-sets')

    const nameInput = (await screen.findByLabelText('名称')) as HTMLInputElement
    fireEvent.change(nameInput, { target: { value: '创意测试集' } })

    const casesTextarea = screen.getByLabelText('cases (JSON)') as HTMLTextAreaElement
    fireEvent.change(casesTextarea, {
      target: {
        value: JSON.stringify(
          [
            {
              id: 'case-1',
              input: { prompt: '写欢迎文案' },
              reference: {
                Constrained: {
                  core_request: '友好、简洁',
                  constraints: [
                    { name: 'length', description: '长度限制', params: ['bad'], weight: null },
                  ],
                  quality_dimensions: [],
                },
              },
              split: null,
              metadata: null,
            },
          ],
          null,
          2
        ),
      },
    })

    await screen.findByText('创意任务配置（Constrained）')

    const minChars = await screen.findByLabelText('最小字符数')

    const originalConfirm = window.confirm
    try {
      const confirmCalls: string[] = []
      window.confirm = (msg?: string) => {
        confirmCalls.push(String(msg ?? ''))
        return false
      }

      fireEvent.change(minChars, { target: { value: '30' } })
      expect(confirmCalls.length).toBe(1)
      expect(casesTextarea.value).toContain('"params": [')
      expect(casesTextarea.value).not.toContain('"minChars": 30')

      window.confirm = (msg?: string) => {
        confirmCalls.push(String(msg ?? ''))
        return true
      }

      fireEvent.change(minChars, { target: { value: '30' } })
      expect(confirmCalls.length).toBe(2)
      expect(casesTextarea.value).not.toContain('"params": [')
      expect(casesTextarea.value).toContain('"minChars": 30')
    } finally {
      window.confirm = originalConfirm
    }
  })

  it('模板创建后应自动写回通用变量配置', async () => {
    server.use(
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
          dify_config: null,
          generic_config: {
            variables: [{ name: 'x', valueType: 'string', defaultValue: 'y' }],
          },
          created_at: 10,
          updated_at: 10,
        }
        return HttpResponse.json({ data })
      })
    )

    renderPage('/workspaces/ws-1/test-sets')

    const openButton = await screen.findByRole('button', { name: '从模板创建' })
    fireEvent.click(openButton)

    const useButton = await screen.findByRole('button', { name: '使用' })
    fireEvent.click(useButton)

    await waitFor(() => {
      expect(screen.getByText('已从模板预填（含通用变量）')).toBeInTheDocument()
    })

    const createButton = await screen.findByRole('button', { name: '创建测试集' })
    fireEvent.click(createButton)

    await waitFor(() => {
      expect(screen.getByText('创建成功（含通用变量）')).toBeInTheDocument()
    })

    expect(lastCreateTestSetBody?.generic_config?.variables?.[0]?.name).toBe('x')
  })
})
