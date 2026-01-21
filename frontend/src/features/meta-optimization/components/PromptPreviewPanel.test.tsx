import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PromptPreviewPanel } from './PromptPreviewPanel'
import { useAuthStore } from '@/stores/useAuthStore'

const mockTasks = [
  {
    id: 'task-1',
    workspaceId: 'ws-1',
    name: '历史任务 1',
    status: 'completed',
    passRate: null,
    createdAt: '2026-01-01T00:00:00Z',
  },
]

const previewResponse = {
  results: [
    {
      testCaseId: 'tc-1',
      input: { foo: 'bar' },
      reference: { Exact: { expected: 'ok' } },
      actualOutput: 'ok',
      passed: true,
      executionTimeMs: 5,
      errorMessage: null,
    },
  ],
  totalPassed: 1,
  totalFailed: 0,
  totalExecutionTimeMs: 5,
}

const mutate = vi.fn((_, options) => options?.onSuccess?.(previewResponse))

vi.mock('../hooks/useMetaOptimizationTasks', () => ({
  useMetaOptimizationTasks: () => ({
    data: mockTasks,
    isLoading: false,
    error: null,
  }),
}))

vi.mock('../hooks/usePromptPreview', () => ({
  usePromptPreview: () => ({
    mutate,
    isPending: false,
    error: null,
  }),
}))

vi.mock('@/features/task-config/services/optimizationTaskService', () => ({
  getOptimizationTask: vi.fn().mockResolvedValue({ test_set_ids: ['ts-1'] } as any),
}))

vi.mock('@/features/test-set-manager/services/testSetService', () => ({
  getTestSet: vi.fn().mockResolvedValue({
    id: 'ts-1',
    workspace_id: 'ws-1',
    name: '测试集',
    description: null,
    cases: [
      {
        id: 'tc-1',
        input: { foo: 'bar' },
        reference: { Exact: { expected: 'ok' } },
        split: null,
        metadata: null,
      },
    ],
    dify_config: null,
    generic_config: null,
    created_at: 0,
    updated_at: 0,
  }),
}))

describe('PromptPreviewPanel', () => {
  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
    mutate.mockClear()
  })

  it('loads cases and triggers preview', async () => {
    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <PromptPreviewPanel content="hello" />
      </QueryClientProvider>
    )

    fireEvent.click(screen.getByLabelText(/历史任务 1/))

    await waitFor(() => {
      expect(screen.getByText('tc-1')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByLabelText(/tc-1/))
    fireEvent.click(screen.getByRole('button', { name: '预览效果' }))

    expect(mutate).toHaveBeenCalledWith(
      {
        content: 'hello',
        taskIds: ['task-1'],
        testCaseIds: ['tc-1'],
      },
      expect.any(Object)
    )

    expect(await screen.findByText('通过 1')).toBeInTheDocument()
    expect(screen.getByText('失败 0')).toBeInTheDocument()
  })
})
