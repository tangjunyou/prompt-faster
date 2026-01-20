import { describe, it, expect, beforeAll, afterAll, afterEach, beforeEach, vi } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { DiagnosticReport } from './DiagnosticReport'
import { useAuthStore } from '@/stores/useAuthStore'
import type { DiagnosticReport as DiagnosticReportType } from '@/types/generated/models/DiagnosticReport'
import type { FailedCaseDetail } from '@/types/generated/models/FailedCaseDetail'

const API_BASE = 'http://localhost:3000/api/v1'

let reportPayload: DiagnosticReportType | null = null
let caseDetailPayload: FailedCaseDetail | null = null

vi.mock('@monaco-editor/react', () => ({
  default: (props: { original?: string; modified?: string }) => (
    <div
      data-testid="monaco-editor-mock"
      data-original={props.original}
      data-modified={props.modified}
    />
  ),
  DiffEditor: (props: { original?: string; modified?: string }) => (
    <div
      data-testid="monaco-diff-mock"
      data-original={props.original}
      data-modified={props.modified}
    />
  ),
}))

const server = setupServer(
  http.get(`${API_BASE}/tasks/:taskId/diagnostic`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    if (!reportPayload) {
      return HttpResponse.json(
        { error: { code: 'NOT_FOUND', message: '任务不存在' } },
        { status: 404 }
      )
    }
    return HttpResponse.json({ data: reportPayload })
  }),
  http.get(`${API_BASE}/tasks/:taskId/diagnostic/cases/:caseId`, ({ request }) => {
    const auth = request.headers.get('authorization')
    if (auth !== 'Bearer test-token') {
      return HttpResponse.json(
        { error: { code: 'UNAUTHORIZED', message: '请先登录' } },
        { status: 401 }
      )
    }
    if (!caseDetailPayload) {
      return HttpResponse.json(
        { error: { code: 'NOT_FOUND', message: '失败用例不存在' } },
        { status: 404 }
      )
    }
    return HttpResponse.json({ data: caseDetailPayload })
  })
)

function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>)
}

describe('DiagnosticReport', () => {
  beforeAll(() => server.listen())
  afterEach(() => {
    server.resetHandlers()
    reportPayload = null
    caseDetailPayload = null
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

  it('应展示诊断报告摘要与转折点', async () => {
    reportPayload = {
      taskId: 'task-1',
      taskName: '任务A',
      status: 'completed',
      summary: {
        totalIterations: 3,
        failedIterations: 1,
        successIterations: 2,
        commonFailureReasons: [
          { reason: 'format', count: 2, percentage: 66.7 },
        ],
        naturalLanguageExplanation: '主要失败原因是 format',
      },
      turningPoints: [
        {
          round: 2,
          eventType: 'improvement',
          description: '通过率提升 20%',
          passRateBefore: 0.3,
          passRateAfter: 0.5,
          timestamp: '2025-01-01T00:00:00Z',
        },
      ],
      improvementSuggestions: ['补充输出格式示例'],
      failedCases: [
        {
          caseId: 'iter-1:case-1',
          inputPreview: 'input',
          failureReason: 'format',
          iterationRound: 1,
          testCaseId: 'case-1',
        },
      ],
    }

    renderWithQueryClient(<DiagnosticReport taskId="task-1" />)

    expect(await screen.findByText('诊断报告')).toBeInTheDocument()
    expect(await screen.findByText('失败原因摘要')).toBeInTheDocument()
    expect((await screen.findAllByText('format')).length).toBeGreaterThan(0)
    expect(await screen.findByText('关键转折点')).toBeInTheDocument()
    expect(await screen.findByText('提升')).toBeInTheDocument()
  })

  it('点击失败用例应加载对比对话框', async () => {
    reportPayload = {
      taskId: 'task-1',
      taskName: '任务A',
      status: 'completed',
      summary: {
        totalIterations: 1,
        failedIterations: 1,
        successIterations: 0,
        commonFailureReasons: [],
        naturalLanguageExplanation: '说明',
      },
      turningPoints: [],
      improvementSuggestions: [],
      failedCases: [
        {
          caseId: 'iter-1:case-1',
          inputPreview: 'input',
          failureReason: 'format',
          iterationRound: 1,
          testCaseId: 'case-1',
        },
      ],
    }

    caseDetailPayload = {
      caseId: 'iter-1:case-1',
      testCaseId: 'case-1',
      input: 'input',
      expectedOutput: 'expected',
      actualOutput: 'actual',
      failureReason: 'format',
      iterationRound: 1,
      promptUsed: 'prompt',
      diffSegments: [
        { segmentType: 'removed', content: 'expected', startIndex: 0, endIndex: 8 },
      ],
    }

    renderWithQueryClient(<DiagnosticReport taskId="task-1" />)

    const button = await screen.findByRole('button', { name: '查看对比' })
    fireEvent.click(button)

    await waitFor(() => {
      expect(screen.getByText('失败用例对比')).toBeInTheDocument()
      expect(screen.getByTestId('monaco-diff-mock')).toBeInTheDocument()
    })
  })

  it('无失败用例时应显示空态提示', async () => {
    reportPayload = {
      taskId: 'task-1',
      taskName: '任务A',
      status: 'completed',
      summary: {
        totalIterations: 1,
        failedIterations: 0,
        successIterations: 1,
        commonFailureReasons: [],
        naturalLanguageExplanation: '暂无失败原因',
      },
      turningPoints: [],
      improvementSuggestions: [],
      failedCases: [],
    }

    renderWithQueryClient(<DiagnosticReport taskId="task-1" />)

    expect(await screen.findByText('恭喜！所有用例都通过了')).toBeInTheDocument()
  })
})
