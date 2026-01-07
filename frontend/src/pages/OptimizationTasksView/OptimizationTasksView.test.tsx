import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { OptimizationTasksView } from './OptimizationTasksView'
import { useAuthStore } from '@/stores/useAuthStore'
import type { UserInfo } from '@/types/generated/api/UserInfo'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { OptimizationTaskListItemResponse } from '@/types/generated/api/OptimizationTaskListItemResponse'
import type { CreateOptimizationTaskRequest } from '@/types/generated/api/CreateOptimizationTaskRequest'

const API_BASE = 'http://localhost:3000/api/v1'

let tasks: OptimizationTaskListItemResponse[] = []
let testSets: TestSetListItemResponse[] = []

const server = setupServer(
  http.get(`${API_BASE}/workspaces/:workspaceId/optimization-tasks`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const workspaceId = String(params.workspaceId)
    return HttpResponse.json({
      data: tasks.filter((t) => t.workspace_id === workspaceId),
    })
  }),

  http.post(`${API_BASE}/workspaces/:workspaceId/optimization-tasks`, async ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const workspaceId = String(params.workspaceId)
    const body = (await request.json()) as CreateOptimizationTaskRequest

    if (body.task_mode === 'fixed' && body.test_set_ids.includes('ts-constrained')) {
      return HttpResponse.json(
        { error: { code: 'VALIDATION_ERROR', message: 'Fixed 模式不允许关联包含 Constrained reference 的测试集' } },
        { status: 400 }
      )
    }

    const now = 1700000000000
    const created: OptimizationTaskListItemResponse = {
      id: `task-${tasks.length + 1}`,
      workspace_id: workspaceId,
      name: body.name,
      goal: body.goal,
      execution_target_type: (body.execution_target_type as 'dify' | 'generic') ?? 'dify',
      task_mode: (body.task_mode as 'fixed' | 'creative') ?? 'fixed',
      status: 'draft',
      teacher_model_display_name: '系统默认',
      created_at: now,
      updated_at: now,
    }

    tasks = [created, ...tasks]

    return HttpResponse.json({
      data: {
        ...created,
        description: body.description ?? null,
        test_set_ids: body.test_set_ids,
      },
    })
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
    return HttpResponse.json({
      data: testSets.filter((t) => t.workspace_id === workspaceId),
    })
  })
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
          <Route path="/workspaces/:id/tasks" element={<OptimizationTasksView />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('OptimizationTasksView', () => {
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

    testSets = [
      {
        id: 'ts-1',
        workspace_id: 'ws-1',
        name: '测试集 1',
        description: 'desc',
        cases_count: 1,
        created_at: 1,
        updated_at: 2,
      },
      {
        id: 'ts-constrained',
        workspace_id: 'ws-1',
        name: '测试集 Constrained',
        description: null,
        cases_count: 1,
        created_at: 1,
        updated_at: 2,
      },
    ]

    tasks = [
      {
        id: 'task-a',
        workspace_id: 'ws-1',
        name: '已有任务',
        goal: '已有目标',
        execution_target_type: 'dify',
        task_mode: 'fixed',
        status: 'draft',
        teacher_model_display_name: '系统默认',
        created_at: 10,
        updated_at: 11,
      },
    ]
  })

  it('创建向导应做必填校验', async () => {
    renderPage('/workspaces/ws-1/tasks')

    const submitButton = await screen.findByRole('button', { name: '创建任务' })
    fireEvent.click(submitButton)
    expect(await screen.findByText('任务名称不能为空')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('任务名称 *'), { target: { value: 't1' } })
    fireEvent.click(submitButton)
    expect(await screen.findByText('优化目标不能为空')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('优化目标 *'), { target: { value: 'g1' } })
    fireEvent.click(submitButton)
    expect(await screen.findByText('请至少选择 1 个测试集')).toBeInTheDocument()
  })

  it('提交成功后列表应刷新', async () => {
    renderPage('/workspaces/ws-1/tasks')

    expect(await screen.findByText('已有任务')).toBeInTheDocument()
    expect(screen.getByText('老师模型：系统默认')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('任务名称 *'), { target: { value: '新任务' } })
    fireEvent.change(screen.getByLabelText('优化目标 *'), { target: { value: '新目标' } })

    const checkbox = await screen.findByLabelText('选择测试集 测试集 1')
    fireEvent.click(checkbox)

    fireEvent.click(screen.getByRole('button', { name: '创建任务' }))

    await waitFor(() => {
      expect(screen.getByText('新任务')).toBeInTheDocument()
    })
    expect(screen.getAllByText('老师模型：系统默认').length).toBeGreaterThan(0)
  })

  it('后端模式校验错误应展示 message（不展示 details）', async () => {
    renderPage('/workspaces/ws-1/tasks')

    fireEvent.change(screen.getByLabelText('任务名称 *'), { target: { value: 't1' } })
    fireEvent.change(screen.getByLabelText('优化目标 *'), { target: { value: 'g1' } })

    const checkbox = await screen.findByLabelText('选择测试集 测试集 Constrained')
    fireEvent.click(checkbox)

    fireEvent.click(screen.getByRole('button', { name: '创建任务' }))

    expect(
      await screen.findByText('创建失败：Fixed 模式不允许关联包含 Constrained reference 的测试集')
    ).toBeInTheDocument()
  })
})
