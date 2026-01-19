import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { CheckpointList } from './CheckpointList'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'

const activeCheckpoint: CheckpointSummary = {
  id: 'cp-1',
  taskId: 'task-1',
  iteration: 2,
  state: 'completed',
  passRateSummary: {
    totalCases: 10,
    passedCases: 8,
    passRate: 0.8,
  },
  createdAt: '2025-01-01T12:00:00Z',
  archivedAt: null,
  archiveReason: null,
  branchId: 'branch-1',
  parentId: null,
}

const archivedCheckpoint: CheckpointSummary = {
  id: 'cp-2',
  taskId: 'task-1',
  iteration: 1,
  state: 'completed',
  passRateSummary: null,
  createdAt: '2025-01-01T11:00:00Z',
  archivedAt: '2025-01-01T12:10:00Z',
  archiveReason: '回滚归档',
  branchId: 'branch-1',
  parentId: null,
}

describe('CheckpointList', () => {
  it('空列表应显示提示', () => {
    render(<CheckpointList checkpoints={[]} />)
    expect(screen.getByText('暂无 Checkpoint')).toBeInTheDocument()
  })

  it('应渲染通过率与归档信息', () => {
    const onSelect = vi.fn()
    render(
      <CheckpointList
        checkpoints={[activeCheckpoint, archivedCheckpoint]}
        onSelect={onSelect}
      />,
    )

    expect(screen.getByText('通过率 80% (8/10)')).toBeInTheDocument()
    expect(screen.getByText('已归档：回滚归档')).toBeInTheDocument()
    const rollbackButtons = screen.getAllByRole('button', { name: '回滚' })
    expect(rollbackButtons[0]).toBeEnabled()
    expect(rollbackButtons[1]).toBeDisabled()
  })

  it('点击回滚按钮应触发选择', () => {
    const onSelect = vi.fn()
    render(
      <CheckpointList
        checkpoints={[activeCheckpoint]}
        onSelect={onSelect}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: '回滚' }))
    expect(onSelect).toHaveBeenCalledTimes(1)
    expect(onSelect).toHaveBeenCalledWith(activeCheckpoint)
  })
})
