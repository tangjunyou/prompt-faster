import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { OptimizationTaskConfigView } from './OptimizationTaskConfigView'
import { useAuthStore } from '@/stores/useAuthStore'
import type { UserInfo } from '@/types/generated/api/UserInfo'
import type { OptimizationTaskResponse } from '@/types/generated/api/OptimizationTaskResponse'
import type { UpdateOptimizationTaskConfigRequest } from '@/types/generated/api/UpdateOptimizationTaskConfigRequest'

const API_BASE = 'http://localhost:3000/api/v1'

let task: OptimizationTaskResponse | null = null

const server = setupServer(
  http.get(`${API_BASE}/workspaces/:workspaceId/optimization-tasks/:taskId`, ({ request, params }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    const workspaceId = String(params.workspaceId)
    const taskId = String(params.taskId)

    if (!task || task.workspace_id !== workspaceId || task.id !== taskId) {
      return HttpResponse.json(
        { error: { code: 'OPTIMIZATION_TASK_NOT_FOUND', message: '优化任务不存在' } },
        { status: 404 }
      )
    }

    return HttpResponse.json({ data: task })
  }),

  http.put(
    `${API_BASE}/workspaces/:workspaceId/optimization-tasks/:taskId/config`,
    async ({ request }) => {
      const auth = request.headers.get('authorization')
      if (auth !== 'Bearer test-token') {
        return HttpResponse.json(
          { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
          { status: 401 }
        )
      }

      const body = (await request.json()) as UpdateOptimizationTaskConfigRequest

      const normalizedInitialPrompt =
        body.initial_prompt && body.initial_prompt.trim() ? body.initial_prompt.trim() : null

      const now = 1700000001234
      if (task) {
        task = {
          ...task,
          config: {
            ...task.config,
            schema_version: 1,
            initial_prompt: normalizedInitialPrompt,
            max_iterations: body.max_iterations,
            pass_threshold_percent: body.pass_threshold_percent,
            data_split: {
              train_percent: body.train_percent,
              validation_percent: body.validation_percent,
              holdout_percent: 0,
            },
          },
          updated_at: now,
        }
      }

      return HttpResponse.json({ data: task })
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
          <Route path="/workspaces/:id/tasks/:taskId" element={<OptimizationTaskConfigView />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('OptimizationTaskConfigView', () => {
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

    task = {
      id: 'task-1',
      workspace_id: 'ws-1',
      name: '任务 1',
      description: null,
      goal: 'g',
      execution_target_type: 'dify',
      task_mode: 'fixed',
      status: 'draft',
      test_set_ids: ['ts-1'],
      config: {
        schema_version: 1,
        initial_prompt: null,
        max_iterations: 10,
        pass_threshold_percent: 95,
        data_split: { train_percent: 80, validation_percent: 20, holdout_percent: 0 },
      },
      created_at: 1700000000000,
      updated_at: 1700000000001,
    }
  })

  it('应渲染默认值并在 initial_prompt 为 null 时展示留空提示', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    expect(await screen.findByText('任务配置：任务 1')).toBeInTheDocument()
    expect(
      await screen.findByText('留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt')
    ).toBeInTheDocument()

    expect(screen.getByLabelText('最大迭代轮数')).toHaveValue(10)
    expect(screen.getByLabelText('通过率阈值（%）')).toHaveValue(95)
    expect(screen.getByLabelText('Train%')).toHaveValue(80)
    expect(screen.getByLabelText('Validation%')).toHaveValue(20)
  })

  it('保存成功后应提示成功并回显后端归一化配置（空 prompt → null）', async () => {
    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')

    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    await waitFor(() => {
      expect(screen.getByText('保存成功')).toBeInTheDocument()
    })

    expect(
      screen.getByText('留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt')
    ).toBeInTheDocument()
  })

  it('保存失败时仅展示 message（不展示 details）', async () => {
    server.use(
      http.put(
        `${API_BASE}/workspaces/:workspaceId/optimization-tasks/:taskId/config`,
        async () => {
          return HttpResponse.json(
            { error: { code: 'VALIDATION_ERROR', message: '最大迭代轮数仅允许 1-100' } },
            { status: 400 }
          )
        }
      )
    )

    renderPage('/workspaces/ws-1/tasks/task-1')

    await screen.findByText('任务配置：任务 1')
    fireEvent.click(screen.getByRole('button', { name: '保存配置' }))

    expect(await screen.findByText('保存失败：最大迭代轮数仅允许 1-100')).toBeInTheDocument()
  })
})
