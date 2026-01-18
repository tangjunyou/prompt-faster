import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach, vi } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { HistoryPanel } from './HistoryPanel'
import { useAuthStore } from '@/stores/useAuthStore'
import type { IterationHistorySummary } from '@/types/generated/models/IterationHistorySummary'
import type { IterationHistoryDetail } from '@/types/generated/models/IterationHistoryDetail'
import type { CheckpointResponse } from '@/types/generated/models/CheckpointResponse'

const API_BASE = 'http://localhost:3000/api/v1'

let iterations: IterationHistorySummary[] = []
let detail: IterationHistoryDetail | null = null
let checkpoints: CheckpointResponse[] = []

vi.mock('@monaco-editor/react', () => ({
  default: (props: { value?: string }) => (
    <textarea data-testid="monaco-mock" value={props.value} readOnly />
  ),
}))

const server = setupServer(
  http.get(`${API_BASE}/tasks/:taskId/iterations`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    return HttpResponse.json({ data: iterations })
  }),
  http.get(`${API_BASE}/tasks/:taskId/iterations/:iterationId`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    if (!detail) {
      return HttpResponse.json(
        { error: { code: 'NOT_FOUND', message: '迭代不存在' } },
        { status: 404 }
      )
    }

    return HttpResponse.json({ data: detail })
  }),
  http.get(`${API_BASE}/tasks/:taskId/checkpoints`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }

    return HttpResponse.json({
      data: {
        checkpoints,
        total: checkpoints.length,
      },
    })
  })
)

function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
  )
}

describe('HistoryPanel', () => {
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
    checkpoints = []
    detail = null
  })

  it('空状态应显示提示与 CTA', async () => {
    iterations = []
    const onStart = vi.fn()

    renderWithQueryClient(<HistoryPanel taskId="task-1" onStartOptimization={onStart} />)

    expect(await screen.findByText('暂无历史记录')).toBeInTheDocument()
    expect(screen.getByText('开始优化任务以生成历史记录')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '开始优化' }))
    expect(onStart).toHaveBeenCalledTimes(1)
  })

  it('应渲染历史列表摘要信息', async () => {
    iterations = [
      {
        id: 'iter-1',
        round: 2,
        startedAt: '2025-01-01T12:00:00Z',
        completedAt: '2025-01-01T12:05:00Z',
        passRate: 0.85,
        totalCases: 10,
        passedCases: 8,
        status: 'completed',
      },
    ]
    checkpoints = [
      {
        id: 'cp-1',
        taskId: 'task-1',
        iteration: 2,
        state: 'paused',
        runControlState: 'pause',
        promptPreview: 'checkpoint-preview',
        hasArtifacts: true,
        hasUserGuidance: false,
        checksum: 'abc',
        integrityOk: true,
        createdAt: '2025-01-01T12:03:00Z',
      },
    ]

    renderWithQueryClient(<HistoryPanel taskId="task-1" />)

    expect(await screen.findByText('共 1 轮迭代')).toBeInTheDocument()
    expect(screen.getByText('#2')).toBeInTheDocument()
    expect(screen.getByText('85.0%')).toBeInTheDocument()
    expect(screen.getByText('checkpoint-preview')).toBeInTheDocument()
  })

  it('集成流程应支持从列表展开到查看评估与反思', async () => {
    iterations = [
      {
        id: 'iter-1',
        round: 2,
        startedAt: '2025-01-01T12:00:00Z',
        completedAt: '2025-01-01T12:05:00Z',
        passRate: 0.85,
        totalCases: 10,
        passedCases: 8,
        status: 'completed',
      },
    ]
    detail = {
      id: 'iter-1',
      round: 2,
      startedAt: '2025-01-01T12:00:00Z',
      completedAt: '2025-01-01T12:05:00Z',
      passRate: 0.85,
      totalCases: 10,
      passedCases: 8,
      status: 'completed',
      artifacts: {
        patterns: [
          { id: 'p1', pattern: 'pattern-1', source: 'system', confidence: 0.8 },
        ],
        candidatePrompts: [
          { id: 'c1', content: 'prompt-1', source: 'system', isBest: true, score: 0.9 },
        ],
        userGuidance: null,
        updatedAt: '2025-01-01T12:00:00Z',
      },
      evaluationResults: [
        { testCaseId: 'case-1', passed: true, score: 0.95, failureReason: null },
      ],
      reflectionSummary: '反思总结',
    }

    renderWithQueryClient(<HistoryPanel taskId="task-1" />)

    expect(await screen.findByText('共 1 轮迭代')).toBeInTheDocument()

    const summaryButton = screen.getByText('#2').closest('button')
    if (!summaryButton) {
      throw new Error('未找到迭代摘要按钮')
    }
    fireEvent.click(summaryButton)

    expect(await screen.findByText(/历史记录仅供查看/)).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '评估' }))
    expect(await screen.findByText('case-1')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '反思' }))
    expect(await screen.findByText('反思总结')).toBeInTheDocument()
  })
})
