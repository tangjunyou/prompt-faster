import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { MemoryRouter } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import App from './App'
import { useAuthStore } from '@/stores/useAuthStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { WorkspaceResponse } from '@/types/generated/api/WorkspaceResponse'
import type { OptimizationTaskListItemResponse } from '@/types/generated/api/OptimizationTaskListItemResponse'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { TestSetTemplateListItemResponse } from '@/types/generated/api/TestSetTemplateListItemResponse'

const API_BASE = 'http://localhost:3000/api/v1'

let workspaces: WorkspaceResponse[] = []
let tasksByWorkspace: Record<string, OptimizationTaskListItemResponse[]> = {}
let testSetsByWorkspace: Record<string, TestSetListItemResponse[]> = {}
let deleteWorkspaceCalls = 0

const server = setupServer(
  http.get(`${API_BASE}/auth/status`, () => {
    return HttpResponse.json({ data: { has_users: true, requires_registration: false } })
  }),

  http.get(`${API_BASE}/workspaces`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    return HttpResponse.json({ data: workspaces })
  }),

  http.post(`${API_BASE}/workspaces`, async ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const body = (await request.json()) as { name?: string; description?: string | null }
    const created: WorkspaceResponse = {
      id: `ws-${workspaces.length + 1}`,
      name: body.name ?? '新工作区',
      description: body.description ?? null,
      created_at: 1,
      updated_at: 1,
    }
    workspaces = [created, ...workspaces]
    return HttpResponse.json({ data: created })
  }),

  http.delete(`${API_BASE}/workspaces/:workspaceId`, ({ request, params }) => {
    deleteWorkspaceCalls += 1

    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const workspaceId = String(params.workspaceId)
    const exists = workspaces.some((ws) => ws.id === workspaceId)
    if (!exists) {
      return HttpResponse.json(
        { error: { code: 'WORKSPACE_NOT_FOUND', message: '工作区不存在' } },
        { status: 404 }
      )
    }

    workspaces = workspaces.filter((ws) => ws.id !== workspaceId)
    delete tasksByWorkspace[workspaceId]
    delete testSetsByWorkspace[workspaceId]

    return HttpResponse.json({ data: { message: '删除成功' } })
  }),

  http.get(`${API_BASE}/workspaces/:workspaceId/optimization-tasks`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    const workspaceId = String(params.workspaceId)
    return HttpResponse.json({ data: tasksByWorkspace[workspaceId] ?? [] })
  }),

  http.get(`${API_BASE}/workspaces/:workspaceId/test-sets`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    const workspaceId = String(params.workspaceId)
    return HttpResponse.json({ data: testSetsByWorkspace[workspaceId] ?? [] })
  }),

  http.get(`${API_BASE}/workspaces/:workspaceId/test-set-templates`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const data: TestSetTemplateListItemResponse[] = []
    return HttpResponse.json({ data })
  })
)

const renderWithProviders = (initialEntry: string) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialEntry]}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('App routes', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'unauthenticated',
      sessionToken: null,
      currentUser: null,
      requiresRegistration: null,
    })
    useWorkspaceStore.getState().reset()

    workspaces = [
      { id: 'ws-1', name: '工作区 1', description: null, created_at: 1, updated_at: 1 },
      { id: 'ws-2', name: '工作区 2', description: null, created_at: 1, updated_at: 1 },
    ]
    tasksByWorkspace = {
      'ws-1': [
        {
          id: 'task-1',
          workspace_id: 'ws-1',
          name: '任务 ws-1',
          goal: 'g',
          execution_target_type: 'dify',
          task_mode: 'fixed',
          status: 'draft',
          teacher_model_display_name: '系统默认',
          created_at: 1,
          updated_at: 1,
        },
      ],
      'ws-2': [
        {
          id: 'task-2',
          workspace_id: 'ws-2',
          name: '任务 ws-2',
          goal: 'g',
          execution_target_type: 'dify',
          task_mode: 'fixed',
          status: 'draft',
          teacher_model_display_name: '系统默认',
          created_at: 1,
          updated_at: 1,
        },
      ],
    }
    testSetsByWorkspace = {
      'ws-1': [
        {
          id: 'ts-1',
          workspace_id: 'ws-1',
          name: '测试集 ws-1',
          description: null,
          cases_count: 1,
          created_at: 1,
          updated_at: 1,
        },
      ],
      'ws-2': [
        {
          id: 'ts-2',
          workspace_id: 'ws-2',
          name: '测试集 ws-2',
          description: null,
          cases_count: 1,
          created_at: 1,
          updated_at: 1,
        },
      ],
    }
    deleteWorkspaceCalls = 0
  })

  it('渲染 /run 路由', () => {
    renderWithProviders('/run')
    expect(screen.getByTestId('run-view')).toBeInTheDocument()
  })

  it('渲染 /focus 路由', () => {
    renderWithProviders('/focus')
    expect(screen.getByTestId('focus-view')).toBeInTheDocument()
  })

  it('渲染 /workspace 路由', () => {
    renderWithProviders('/workspace')
    expect(screen.getByTestId('workspace-view')).toBeInTheDocument()
  })

  it('默认路由应跳转到 /run', () => {
    renderWithProviders('/')
    expect(screen.getByTestId('run-view')).toBeInTheDocument()
  })

  it('认证态下 Header 出现工作区选择器', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/run')

    expect(await screen.findByTestId('workspace-selector-trigger')).toBeInTheDocument()
  })

  it('未登录时不显示工作区选择器', async () => {
    renderWithProviders('/run')
    expect(screen.queryByTestId('workspace-selector-trigger')).not.toBeInTheDocument()
  })

  it('登出后应清理 lastWorkspaceId（避免本地残留）', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: null,
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
    useWorkspaceStore.getState().setLastWorkspaceId('u1', 'ws-1')

    renderWithProviders('/run')

    fireEvent.click(await screen.findByTestId('logout-button'))

    await waitFor(() => {
      expect(useWorkspaceStore.getState().lastWorkspaceIdByUser.u1).toBeUndefined()
    })
  })

  it('选择工作区后路由跳转（保持 section 规则：test-sets）', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/workspaces/ws-1/test-sets')
    expect(await screen.findByTestId('test-sets-view')).toBeInTheDocument()
    expect(await screen.findByText('测试集 ws-1')).toBeInTheDocument()

    fireEvent.click(await screen.findByTestId('workspace-selector-trigger'))
    fireEvent.click(await screen.findByTestId('workspace-option-ws-2'))

    await waitFor(() => {
      expect(screen.getByText('测试集 ws-2')).toBeInTheDocument()
    })
  })

  it('切换过程平滑：切换瞬间不清空旧任务列表，并显示轻量 loading', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    server.use(
      http.get(`${API_BASE}/workspaces/:workspaceId/optimization-tasks`, async ({ request, params }) => {
        const auth = request.headers.get('authorization')
        if (auth !== 'Bearer test-token') {
          return HttpResponse.json(
            { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
            { status: 401 }
          )
        }

        const workspaceId = String(params.workspaceId)
        if (workspaceId === 'ws-2') {
          await new Promise((resolve) => setTimeout(resolve, 80))
        }
        return HttpResponse.json({ data: tasksByWorkspace[workspaceId] ?? [] })
      })
    )

    renderWithProviders('/workspaces/ws-1/tasks')
    expect(await screen.findByTestId('optimization-tasks-view')).toBeInTheDocument()
    expect(await screen.findByText('任务 ws-1')).toBeInTheDocument()

    fireEvent.click(await screen.findByTestId('workspace-selector-trigger'))
    fireEvent.click(await screen.findByTestId('workspace-option-ws-2'))

    // keep previous list while fetching new workspace
    expect(screen.getByText('任务 ws-1')).toBeInTheDocument()
    expect(await screen.findByText('加载中...')).toBeInTheDocument()

    await waitFor(() => {
      expect(screen.getByText('任务 ws-2')).toBeInTheDocument()
    })
  })

  it('创建工作区成功后自动切换并导航到 /workspaces/:id/tasks', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    workspaces = []

    renderWithProviders('/run')

    fireEvent.click(await screen.findByTestId('workspace-selector-trigger'))
    expect(await screen.findByText('暂无工作区')).toBeInTheDocument()

    fireEvent.click(await screen.findByRole('button', { name: '新建工作区' }))
    fireEvent.change(await screen.findByLabelText('名称'), { target: { value: '我的工作区' } })
    fireEvent.click(screen.getByRole('button', { name: '创建' }))

    expect(await screen.findByTestId('optimization-tasks-view')).toBeInTheDocument()
    expect(await screen.findByText('我的工作区')).toBeInTheDocument()
  })

  it('新建工作区：前端应做 name 本地校验（trim 非空、长度 ≤ 128）', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    workspaces = []
    renderWithProviders('/run')

    fireEvent.click(await screen.findByTestId('workspace-selector-trigger'))
    fireEvent.click(await screen.findByRole('button', { name: '新建工作区' }))

    fireEvent.change(await screen.findByLabelText('名称'), { target: { value: '   ' } })
    fireEvent.click(screen.getByRole('button', { name: '创建' }))
    expect(await screen.findByText('名称不能为空')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('名称'), { target: { value: 'a'.repeat(129) } })
    fireEvent.click(screen.getByRole('button', { name: '创建' }))
    expect(await screen.findByText('名称长度不能超过 128')).toBeInTheDocument()
  })

  it('新建工作区失败：只展示 error.message（不展示 error.details）', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    workspaces = []
    server.use(
      http.post(`${API_BASE}/workspaces`, () => {
        return HttpResponse.json(
          {
            error: {
              code: 'VALIDATION_ERROR',
              message: '名字不合法',
              details: 'should-not-render',
            },
          },
          { status: 400 }
        )
      })
    )

    renderWithProviders('/run')

    fireEvent.click(await screen.findByTestId('workspace-selector-trigger'))
    fireEvent.click(await screen.findByRole('button', { name: '新建工作区' }))

    fireEvent.change(await screen.findByLabelText('名称'), { target: { value: 'bad' } })
    fireEvent.click(screen.getByRole('button', { name: '创建' }))

    expect(await screen.findByText('创建失败：名字不合法')).toBeInTheDocument()
    expect(screen.queryByText('should-not-render')).not.toBeInTheDocument()
  })

  it('lastWorkspaceId 无效时应回退到第一个 workspace', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
    useWorkspaceStore.getState().setLastWorkspaceId('u1', 'ws-invalid')

    renderWithProviders('/run')

    const trigger = await screen.findByTestId('workspace-selector-trigger')
    await waitFor(() => {
      expect(trigger).toHaveTextContent('工作区 1')
    })
  })

  it('Workspace View 中点击删除 → 弹出确认对话框', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/workspace')
    expect(await within(screen.getByTestId('workspace-view')).findByText('工作区 1')).toBeInTheDocument()

    fireEvent.click(screen.getByTestId('workspace-delete-ws-1'))

    expect(await screen.findByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText('删除工作区')).toBeInTheDocument()
  })

  it('取消删除 → 不触发删除请求', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/workspace')
    expect(await within(screen.getByTestId('workspace-view')).findByText('工作区 1')).toBeInTheDocument()

    fireEvent.click(screen.getByTestId('workspace-delete-ws-1'))
    expect(await screen.findByRole('dialog')).toBeInTheDocument()

    fireEvent.click(screen.getByTestId('workspace-delete-cancel'))

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    })
    expect(deleteWorkspaceCalls).toBe(0)
  })

  it('确认删除成功 → 列表刷新且移除该工作区', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/workspace')

    const view = within(screen.getByTestId('workspace-view'))
    expect(await view.findByText('工作区 1')).toBeInTheDocument()
    expect(view.getByText('工作区 2')).toBeInTheDocument()

    fireEvent.click(view.getByTestId('workspace-delete-ws-1'))
    fireEvent.click(await screen.findByTestId('workspace-delete-confirm'))

    await waitFor(() => {
      expect(view.queryByText('工作区 1')).not.toBeInTheDocument()
    })
    expect(view.getByText('工作区 2')).toBeInTheDocument()
    expect(view.getByText('已删除工作区：工作区 1')).toBeInTheDocument()
  })

  it('删除当前工作区时 → 自动导航到其它 workspace', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
    useWorkspaceStore.getState().setLastWorkspaceId('u1', 'ws-1')

    renderWithProviders('/workspace')

    const view = within(screen.getByTestId('workspace-view'))
    expect(await view.findByText('工作区 1')).toBeInTheDocument()

    fireEvent.click(view.getByTestId('workspace-delete-ws-1'))
    fireEvent.click(await screen.findByTestId('workspace-delete-confirm'))

    expect(await screen.findByTestId('optimization-tasks-view')).toBeInTheDocument()
    expect(await screen.findByText('已删除工作区：工作区 1')).toBeInTheDocument()
    expect(await screen.findByText('任务 ws-2')).toBeInTheDocument()
  })

  it('删除失败 → 仅展示 error.message（不展示 error.details）', async () => {
    server.use(
      http.delete(`${API_BASE}/workspaces/:workspaceId`, ({ request }) => {
        const auth = request.headers.get('authorization')
        if (auth !== 'Bearer test-token') {
          return HttpResponse.json(
            { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
            { status: 401 }
          )
        }
        return HttpResponse.json(
          {
            error: {
              code: 'DATABASE_ERROR',
              message: '删除失败：数据库错误',
              details: { internal: 'do-not-leak' },
            },
          },
          { status: 500 }
        )
      })
    )

    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/workspace')

    const view = within(screen.getByTestId('workspace-view'))
    expect(await view.findByText('工作区 1')).toBeInTheDocument()

    fireEvent.click(view.getByTestId('workspace-delete-ws-1'))
    fireEvent.click(await screen.findByTestId('workspace-delete-confirm'))

    expect(await screen.findByText('删除失败：数据库错误')).toBeInTheDocument()
    expect(screen.queryByText('do-not-leak')).not.toBeInTheDocument()
  })

  it('确认按钮 loading 禁用：快速连续点击不会触发重复请求', async () => {
    server.use(
      http.delete(`${API_BASE}/workspaces/:workspaceId`, async ({ request, params }) => {
        deleteWorkspaceCalls += 1

        const auth = request.headers.get('authorization')
        if (auth !== 'Bearer test-token') {
          return HttpResponse.json(
            { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
            { status: 401 }
          )
        }

        await new Promise((resolve) => setTimeout(resolve, 50))

        const workspaceId = String(params.workspaceId)
        workspaces = workspaces.filter((ws) => ws.id !== workspaceId)
        return HttpResponse.json({ data: { message: '删除成功' } })
      })
    )

    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })

    renderWithProviders('/workspace')

    const view = within(screen.getByTestId('workspace-view'))
    expect(await view.findByText('工作区 1')).toBeInTheDocument()

    fireEvent.click(view.getByTestId('workspace-delete-ws-1'))

    const confirm = await screen.findByTestId('workspace-delete-confirm')
    fireEvent.click(confirm)
    fireEvent.click(confirm)

    await waitFor(() => {
      expect(deleteWorkspaceCalls).toBe(1)
    })
  })

  it('删除最后一个工作区：仍停留在 /workspace 并看到“创建工作区”入口（不出现空白页）', async () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
    useWorkspaceStore.getState().setLastWorkspaceId('u1', 'ws-1')

    workspaces = [{ id: 'ws-1', name: '工作区 1', description: null, created_at: 1, updated_at: 1 }]
    tasksByWorkspace = { 'ws-1': [] }
    testSetsByWorkspace = { 'ws-1': [] }

    renderWithProviders('/workspace')

    const view = within(await screen.findByTestId('workspace-view'))
    expect(await view.findByText('工作区 1')).toBeInTheDocument()

    fireEvent.click(view.getByTestId('workspace-delete-ws-1'))
    fireEvent.click(await screen.findByTestId('workspace-delete-confirm'))

    // 等待删除请求真正完成（避免在 CI + coverage 下出现时序抖动）
    await waitFor(() => {
      expect(workspaces).toHaveLength(0)
    })

    const updatedView = within(await screen.findByTestId('workspace-view'))
    expect(await updatedView.findByText('暂无工作区，请先创建一个。')).toBeInTheDocument()
    expect(updatedView.getByRole('button', { name: '创建工作区' })).toBeInTheDocument()
  })
})
