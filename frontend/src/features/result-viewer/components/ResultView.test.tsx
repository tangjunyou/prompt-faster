import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach, vi } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { ResultView } from './ResultView'
import { useAuthStore } from '@/stores/useAuthStore'
import type { TaskResultView } from '@/types/generated/models/TaskResultView'

const API_BASE = 'http://localhost:3000/api/v1'

let resultPayload: TaskResultView | null = null

vi.mock('@monaco-editor/react', () => ({
  default: (props: { value?: string }) => (
    <textarea data-testid="monaco-mock" value={props.value} readOnly />
  ),
}))

const server = setupServer(
  http.get(`${API_BASE}/tasks/:taskId/result`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    if (!resultPayload) {
      return HttpResponse.json(
        { error: { code: 'NOT_FOUND', message: '任务未开始' } },
        { status: 404 }
      )
    }
    return HttpResponse.json({ data: resultPayload })
  })
)

function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>)
}

describe('ResultView', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
  })

  it('应展示结果信息与最佳 Prompt', async () => {
    resultPayload = {
      taskId: 'task-1',
      taskName: '任务A',
      status: 'completed',
      bestPrompt: '最佳 Prompt 内容',
      passRate: 0.85,
      totalIterations: 3,
      completedAt: '2025-01-01T12:00:00Z',
      createdAt: '2025-01-01T11:00:00Z',
      iterationSummary: [
        { round: 3, passRate: 0.85, status: 'completed' },
      ],
    }

    renderWithQueryClient(<ResultView taskId="task-1" />)

    expect(await screen.findByText('优化结果')).toBeInTheDocument()
    expect(await screen.findByText('任务：任务A')).toBeInTheDocument()
    expect(await screen.findByText('最佳 Prompt')).toBeInTheDocument()
    expect(await screen.findByTestId('monaco-mock')).toHaveValue('最佳 Prompt 内容')
    expect(await screen.findByText('第 3 轮')).toBeInTheDocument()
  })

  it('复制成功后应提示已复制', async () => {
    const mockWriteText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, {
      clipboard: { writeText: mockWriteText },
    })
    resultPayload = {
      taskId: 'task-1',
      taskName: '任务A',
      status: 'completed',
      bestPrompt: '复制内容',
      passRate: 0.8,
      totalIterations: 2,
      completedAt: null,
      createdAt: '2025-01-01T11:00:00Z',
      iterationSummary: [],
    }

    renderWithQueryClient(<ResultView taskId="task-1" />)
    const copyButton = await screen.findByRole('button', { name: '复制 Prompt' })
    fireEvent.click(copyButton)

    await waitFor(() => {
      expect(mockWriteText).toHaveBeenCalledWith('复制内容')
      expect(screen.getByText('已复制')).toBeInTheDocument()
    })
  })

  it('无完成迭代时应显示空态提示', async () => {
    resultPayload = {
      taskId: 'task-1',
      taskName: '任务A',
      status: 'running',
      bestPrompt: null,
      passRate: null,
      totalIterations: 1,
      completedAt: null,
      createdAt: '2025-01-01T11:00:00Z',
      iterationSummary: [],
    }

    renderWithQueryClient(<ResultView taskId="task-1" />)

    expect(await screen.findByText('暂无已完成迭代')).toBeInTheDocument()
  })
})
