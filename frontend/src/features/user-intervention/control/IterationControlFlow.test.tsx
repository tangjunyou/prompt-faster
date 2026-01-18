import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

import { IterationControlPanel } from './IterationControlPanel'
import { useAuthStore } from '@/stores/useAuthStore'

const API_BASE = 'http://localhost:3000/api/v1'

let addRoundsBody: { additionalRounds?: number } | null = null
let terminateBody: { selectedIterationId?: string | null } | null = null

const server = setupServer(
  http.patch(`${API_BASE}/tasks/:taskId/config`, async ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    const body = (await request.json()) as { additionalRounds?: number }
    addRoundsBody = body
    return HttpResponse.json({
      data: {
        previousMaxIterations: 10,
        newMaxIterations: 10 + (body.additionalRounds ?? 0),
        currentRound: 2,
      },
    })
  }),
  http.get(`${API_BASE}/tasks/:taskId/candidates`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    return HttpResponse.json({
      data: {
        candidates: [
          {
            iterationId: 'iter-1',
            round: 1,
            passRate: 0.9,
            passedCases: 9,
            totalCases: 10,
            prompt: 'Final Prompt',
            promptPreview: 'Final Prompt preview',
            completedAt: '2025-01-01T12:00:00Z',
          },
        ],
        total: 1,
      },
    })
  }),
  http.post(`${API_BASE}/tasks/:taskId/terminate`, async ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    const body = (await request.json()) as { selectedIterationId?: string | null }
    terminateBody = body
    return HttpResponse.json({
      data: {
        taskId: 'task-1',
        terminatedAt: '2025-01-01T12:10:00Z',
        finalPrompt: 'Final Prompt',
        selectedRound: 1,
      },
    })
  })
)

function renderWithClient(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>)
}

describe('Iteration control integration flow', () => {
  beforeAll(() => server.listen())
  afterEach(() => {
    server.resetHandlers()
    addRoundsBody = null
    terminateBody = null
  })
  afterAll(() => server.close())

  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
  })

  it('completes add rounds flow', async () => {
    renderWithClient(
      <IterationControlPanel
        taskId="task-1"
        runControlState="running"
        currentMaxIterations={10}
        currentRound={2}
      />
    )

    fireEvent.click(screen.getByRole('button', { name: '增加轮数' }))

    const input = await screen.findByRole('spinbutton', { name: '增加轮数' })
    fireEvent.change(input, { target: { value: '3' } })

    fireEvent.click(screen.getByRole('button', { name: '确认增加' }))

    await waitFor(() => {
      expect(screen.queryByText('增加迭代轮数')).not.toBeInTheDocument()
    })

    expect(addRoundsBody?.additionalRounds).toBe(3)
  })

  it('completes terminate flow', async () => {
    renderWithClient(
      <IterationControlPanel
        taskId="task-1"
        runControlState="running"
        currentMaxIterations={10}
        currentRound={2}
      />
    )

    fireEvent.click(screen.getByRole('button', { name: '终止任务' }))

    await screen.findByText('Final Prompt preview')
    fireEvent.click(screen.getByText('Final Prompt preview'))

    fireEvent.click(screen.getByRole('button', { name: '确认终止并保存' }))

    await waitFor(() => {
      expect(screen.queryByText('终止优化任务')).not.toBeInTheDocument()
    })

    expect(terminateBody?.selectedIterationId).toBe('iter-1')
  })
})
