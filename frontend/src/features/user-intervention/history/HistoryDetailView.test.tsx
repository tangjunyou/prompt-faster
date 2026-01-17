import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { HistoryDetailView } from './HistoryDetailView'
import type { IterationHistoryDetail } from '@/types/generated/models/IterationHistoryDetail'

vi.mock('@monaco-editor/react', () => ({
  default: (props: { value?: string }) => (
    <textarea data-testid="monaco-mock" value={props.value} readOnly />
  ),
}))

const detail: IterationHistoryDetail = {
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
    updatedAt: '2025-01-01T12:00:00Z',
  },
  evaluationResults: [
    { testCaseId: 'case-1', passed: true, score: 0.95, failureReason: null },
  ],
  reflectionSummary: '反思总结',
}

describe('HistoryDetailView', () => {
  it('应展示评估与反思内容', async () => {
    render(<HistoryDetailView detail={detail} />)

    expect(await screen.findByText(/历史记录仅供查看/)).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '评估' }))
    expect(await screen.findByText('case-1')).toBeInTheDocument()
    expect(screen.getByText('分数: 0.95')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '反思' }))
    expect(await screen.findByText('反思总结')).toBeInTheDocument()
  })
})
