import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

import { IterationControlPanel } from './IterationControlPanel'

vi.mock('./AddRoundsDialog', () => ({
  AddRoundsDialog: () => <div data-testid="add-rounds-dialog" />,
}))

vi.mock('./TerminateDialog', () => ({
  TerminateDialog: () => <div data-testid="terminate-dialog" />,
}))

describe('IterationControlPanel', () => {
  it('renders controls when running', () => {
    render(
      <IterationControlPanel
        taskId="task-1"
        runControlState="running"
        currentMaxIterations={10}
        currentRound={2}
      />
    )

    expect(screen.getByText('当前轮次 2 / 最大 10')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '增加轮数' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '终止任务' })).toBeInTheDocument()
  })

  it('returns null when idle', () => {
    const { container } = render(
      <IterationControlPanel
        taskId="task-1"
        runControlState="idle"
        currentMaxIterations={10}
        currentRound={2}
      />
    )

    expect(container).not.toBeEmptyDOMElement()
    expect(screen.getByText('任务未开始运行，暂不可增加轮数或终止。')).toBeInTheDocument()
  })
})
