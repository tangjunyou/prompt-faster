import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { AddRoundsDialog } from './AddRoundsDialog'

const mutateMock = vi.fn()

vi.mock('./hooks/useIterationControl', () => ({
  useAddRounds: () => ({
    mutate: mutateMock,
    isPending: false,
    error: null,
  }),
}))

describe('AddRoundsDialog', () => {
  beforeEach(() => {
    mutateMock.mockClear()
  })

  it('updates preview and submits additional rounds', () => {
    render(
      <AddRoundsDialog
        taskId="task-1"
        open
        onOpenChange={() => undefined}
        currentMaxIterations={10}
        currentRound={2}
      />
    )

    expect(screen.getByText('新的最大轮数:')).toBeInTheDocument()
    expect(screen.getByText('15')).toBeInTheDocument()

    const input = screen.getByRole('spinbutton', { name: '增加轮数' })
    fireEvent.change(input, { target: { value: '3' } })

    expect(screen.getByText('13')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '确认增加' }))
    expect(mutateMock).toHaveBeenCalledWith(3, expect.any(Object))
  })
})
