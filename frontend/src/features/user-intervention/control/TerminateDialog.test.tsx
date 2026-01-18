import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { TerminateDialog } from './TerminateDialog'
import type { CandidatePromptSummary } from '@/types/generated/models/CandidatePromptSummary'

const candidates: CandidatePromptSummary[] = [
  {
    iterationId: 'iter-1',
    round: 1,
    passRate: 0.75,
    passedCases: 15,
    totalCases: 20,
    prompt: 'Prompt A',
    promptPreview: 'Prompt A preview',
    completedAt: '2025-01-01T12:00:00Z',
  },
  {
    iterationId: 'iter-2',
    round: 2,
    passRate: 0.9,
    passedCases: 18,
    totalCases: 20,
    prompt: 'Prompt B',
    promptPreview: 'Prompt B preview',
    completedAt: '2025-01-01T12:10:00Z',
  },
]

const terminateMock = vi.fn()

vi.mock('./hooks/useIterationControl', () => ({
  useCandidates: () => ({
    candidates,
    isLoading: false,
    isFetching: false,
    isFetchingNextPage: false,
    fetchNextPage: vi.fn(),
    hasMore: false,
  }),
  useTerminateTask: () => ({ mutate: terminateMock, isPending: false, error: null }),
}))

describe('TerminateDialog', () => {
  beforeEach(() => {
    terminateMock.mockClear()
  })

  it('selects candidate and submits terminate request', () => {
    render(
      <TerminateDialog taskId="task-1" open onOpenChange={() => undefined} />
    )

    expect(screen.getByText('终止优化任务')).toBeInTheDocument()
    fireEvent.click(screen.getByText('Prompt A preview'))

    expect(screen.getByRole('button', { name: '确认终止并保存' })).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '确认终止并保存' }))

    expect(terminateMock).toHaveBeenCalledWith('iter-1', expect.any(Object))
  })
})
