import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router'

import { RecoveryPrompt } from './RecoveryPrompt'
import { useAuthStore } from '@/stores/useAuthStore'
import type { UnfinishedTask } from '@/types/generated/models/UnfinishedTask'

const API_BASE = 'http://localhost:3000/api/v1'

let unfinishedTasks: UnfinishedTask[] = []
let recoverCalls = 0
let abortCalls = 0

const server = setupServer(
  http.get(`${API_BASE}/recovery/unfinished-tasks`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    return HttpResponse.json({
      data: {
        tasks: unfinishedTasks,
        total: unfinishedTasks.length,
      },
    })
  }),
  http.post(`${API_BASE}/recovery/tasks/:taskId/recover`, () => {
    recoverCalls += 1
    return HttpResponse.json({
      data: {
        success: true,
        taskId: 'task-1',
        checkpointId: 'cp-1',
        iteration: 1,
        state: 'running_tests',
        runControlState: 'running',
        message: 'ok',
      },
    })
  }),
  http.post(`${API_BASE}/recovery/tasks/:taskId/abort`, () => {
    abortCalls += 1
    return HttpResponse.json({
      data: { success: true, message: 'ok' },
    })
  })
)

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{ui}</MemoryRouter>
    </QueryClientProvider>
  )
}

describe('RecoveryPrompt', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  beforeEach(() => {
    unfinishedTasks = [
      {
        taskId: 'task-1',
        taskName: '示例任务',
        checkpointId: 'cp-1',
        lastCheckpointAt: '2026-01-01T12:00:00Z',
        iteration: 3,
        state: 'running_tests',
        runControlState: 'paused',
      },
    ]
    recoverCalls = 0
    abortCalls = 0
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
  })

  it('显示未完成任务并触发恢复操作', async () => {
    renderWithProviders(<RecoveryPrompt />)

    expect(await screen.findByText('检测到未完成任务')).toBeInTheDocument()
    expect(screen.getByText('示例任务')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '恢复' }))

    await waitFor(() => {
      expect(recoverCalls).toBe(1)
    })

    expect(await screen.findByText('恢复完成')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '继续迭代' })).toBeInTheDocument()
  })

  it('支持放弃恢复操作', async () => {
    renderWithProviders(<RecoveryPrompt />)

    expect(await screen.findByText('检测到未完成任务')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '放弃恢复' }))

    await waitFor(() => {
      expect(abortCalls).toBe(1)
    })
  })
})
