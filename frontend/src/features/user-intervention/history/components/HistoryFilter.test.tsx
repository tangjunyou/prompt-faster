import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'

import { HistoryFilter, type HistoryFilterValue } from './HistoryFilter'

const baseValue: HistoryFilterValue = {
  eventTypes: [],
  actor: '',
  iterationMin: '',
  iterationMax: '',
  timeStart: '',
  timeEnd: '',
}

describe('HistoryFilter', () => {
  it('切换事件类型后触发 onChange', () => {
    const onChange = vi.fn()
    render(<HistoryFilter value={baseValue} onChange={onChange} />)

    fireEvent.click(screen.getByRole('button', { name: '迭代开始' }))

    expect(onChange).toHaveBeenCalled()
    const payload = onChange.mock.calls[0][0] as HistoryFilterValue
    expect(payload.eventTypes).toContain('iteration_started')
  })

  it('点击清空会重置筛选条件', () => {
    const onChange = vi.fn()
    const value: HistoryFilterValue = {
      eventTypes: ['user_pause'],
      actor: 'user',
      iterationMin: '1',
      iterationMax: '3',
      timeStart: '2025-01-01T10:00',
      timeEnd: '2025-01-02T10:00',
    }

    render(<HistoryFilter value={value} onChange={onChange} />)

    fireEvent.click(screen.getByRole('button', { name: '清空' }))

    const payload = onChange.mock.calls[0][0] as HistoryFilterValue
    expect(payload.eventTypes).toHaveLength(0)
    expect(payload.actor).toBe('')
    expect(payload.iterationMin).toBe('')
    expect(payload.timeStart).toBe('')
  })
})
