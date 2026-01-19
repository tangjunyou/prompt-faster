import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { RollbackConfirmDialog } from './RollbackConfirmDialog'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'

const checkpoint: CheckpointSummary = {
  id: 'cp-1',
  taskId: 'task-1',
  iteration: 3,
  state: 'completed',
  passRateSummary: {
    totalCases: 10,
    passedCases: 7,
    passRate: 0.7,
  },
  createdAt: '2025-01-01T12:00:00Z',
  archivedAt: null,
  archiveReason: null,
  branchId: 'branch-1',
  parentId: null,
}

describe('RollbackConfirmDialog', () => {
  it('未选择 Checkpoint 时禁用确认按钮', () => {
    render(
      <RollbackConfirmDialog
        open
        checkpoint={null}
        onCancel={() => undefined}
        onConfirm={() => undefined}
      />,
    )

    expect(screen.getByText('未选择 Checkpoint')).toBeInTheDocument()
    const confirmButton = screen.getByRole('button', { name: '确认回滚' })
    expect(confirmButton).toBeDisabled()
  })

  it('应展示 Checkpoint 信息并触发确认', () => {
    const onConfirm = vi.fn()
    render(
      <RollbackConfirmDialog
        open
        checkpoint={checkpoint}
        onCancel={() => undefined}
        onConfirm={onConfirm}
      />,
    )

    expect(screen.getByText('迭代 #3')).toBeInTheDocument()
    expect(screen.getByText('通过率 70% (7/10)')).toBeInTheDocument()
    expect(
      screen.getByText('回滚后，该 Checkpoint 之后的状态将被归档'),
    ).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '确认回滚' }))
    expect(onConfirm).toHaveBeenCalledTimes(1)
  })
})
