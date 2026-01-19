import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect } from 'vitest'

import { TimelineView } from './TimelineView'
import type { TimelineEntry } from '@/types/generated/models/TimelineEntry'

const entries: TimelineEntry[] = [
  {
    id: 'evt-1',
    entryType: 'event',
    timestamp: '2025-01-01T12:00:00Z',
    iteration: 2,
    title: 'user_pause',
    description: '用户触发暂停',
    actor: 'user',
    details: { reason: 'manual' },
  },
]

describe('TimelineView', () => {
  it('渲染时间线并支持展开详情', () => {
    render(<TimelineView entries={entries} />)

    expect(screen.getByText('事件')).toBeInTheDocument()
    expect(screen.getByText('user_pause')).toBeInTheDocument()
    expect(screen.getByText('用户触发暂停')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '查看详情' }))
    expect(screen.getByText(/manual/)).toBeInTheDocument()
  })
})
