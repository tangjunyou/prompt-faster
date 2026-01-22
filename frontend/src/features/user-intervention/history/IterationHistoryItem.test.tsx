import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach, vi } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { IterationHistoryItem } from './IterationHistoryItem'
import { useAuthStore } from '@/stores/useAuthStore'
import type { IterationHistorySummary } from '@/types/generated/models/IterationHistorySummary'
import type { IterationHistoryDetail } from '@/types/generated/models/IterationHistoryDetail'

vi.mock('@monaco-editor/react', () => ({
  default: (props: { value?: string }) => (
    <textarea data-testid="monaco-mock" value={props.value} readOnly />
  ),
}))

const API_BASE = 'http://localhost:3000/api/v1'

let detail: IterationHistoryDetail | null = null

const server = setupServer(
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

describe('IterationHistoryItem', () => {
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

    detail = {
      id: 'iter-1',
      round: 1,
      startedAt: '2025-01-01T12:00:00Z',
      completedAt: '2025-01-01T12:05:00Z',
      passRate: 0.9,
      totalCases: 10,
      passedCases: 9,
      status: 'completed',
      artifacts: {
        patterns: [
          { id: 'p1', pattern: 'pattern-1', source: 'system', confidence: 0.8 },
        ],
        candidatePrompts: [
          { id: 'c1', content: 'prompt-1', source: 'system', isBest: true, score: 0.9 },
        ],
        userGuidance: null,
        failureArchive: null,
        diversityAnalysis: null,
        updatedAt: '2025-01-01T12:00:00Z',
      },
      evaluationResults: [
        { testCaseId: 'case-1', passed: true, score: 0.95, failureReason: null },
      ],
      reflectionSummary: '反思总结',
    }
  })

  it('展开后应加载详情并显示只读提示', async () => {
    const summary: IterationHistorySummary = {
      id: 'iter-1',
      round: 1,
      startedAt: '2025-01-01T12:00:00Z',
      completedAt: '2025-01-01T12:05:00Z',
      passRate: 0.9,
      totalCases: 10,
      passedCases: 9,
      status: 'completed',
    }

    renderWithQueryClient(
      <IterationHistoryItem
        taskId="task-1"
        summary={summary}
        isExpanded
        onToggle={() => {}}
      />
    )

    expect(await screen.findByText(/历史记录仅供查看/)).toBeInTheDocument()
    expect(screen.getByText('#1')).toBeInTheDocument()
    expect(screen.getByText('90.0%')).toBeInTheDocument()
  })
})
