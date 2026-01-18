import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { CandidatePromptList } from './CandidatePromptList'
import type { CandidatePromptSummary } from '@/types/generated/models/CandidatePromptSummary'

const candidates: CandidatePromptSummary[] = [
  {
    iterationId: 'iter-1',
    round: 1,
    passRate: 0.82,
    passedCases: 82,
    totalCases: 100,
    prompt: 'Prompt A',
    promptPreview: 'Prompt A preview',
    completedAt: '2025-01-01T12:00:00Z',
  },
  {
    iterationId: 'iter-2',
    round: 2,
    passRate: 0.9,
    passedCases: 90,
    totalCases: 100,
    prompt: 'Prompt B',
    promptPreview: 'Prompt B preview',
    completedAt: '2025-01-01T12:10:00Z',
  },
]

describe('CandidatePromptList', () => {
  it('renders candidates and handles selection', () => {
    const onSelect = vi.fn()
    const onCopy = vi.fn()

    render(
      <CandidatePromptList
        candidates={candidates}
        selectedId={null}
        copiedId={null}
        onSelect={onSelect}
        onCopy={onCopy}
      />
    )

    expect(screen.getByText('第 1 轮')).toBeInTheDocument()
    expect(screen.getByText('第 2 轮')).toBeInTheDocument()

    fireEvent.click(screen.getByText('Prompt A preview'))
    expect(onSelect).toHaveBeenCalledWith('iter-1')
  })

  it('triggers copy handler', () => {
    const onSelect = vi.fn()
    const onCopy = vi.fn()

    render(
      <CandidatePromptList
        candidates={candidates}
        selectedId={'iter-1'}
        copiedId={null}
        onSelect={onSelect}
        onCopy={onCopy}
      />
    )

    const copyButtons = screen.getAllByTitle('复制 Prompt')
    fireEvent.click(copyButtons[0])

    expect(onCopy).toHaveBeenCalledWith('Prompt A', 'iter-1')
  })

  it('expands and collapses full prompt', () => {
    const onSelect = vi.fn()
    const onCopy = vi.fn()

    render(
      <CandidatePromptList
        candidates={candidates}
        selectedId={null}
        copiedId={null}
        onSelect={onSelect}
        onCopy={onCopy}
      />
    )

    expect(screen.queryByText('Prompt A')).not.toBeInTheDocument()

    const expandButtons = screen.getAllByRole('button', { name: '展开全文' })
    fireEvent.click(expandButtons[0])
    expect(screen.getByText('Prompt A')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '收起' }))
    expect(screen.queryByText('Prompt A')).not.toBeInTheDocument()
  })
})
