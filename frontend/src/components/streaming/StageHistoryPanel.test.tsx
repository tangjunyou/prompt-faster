import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import type { StageHistoryItem } from '@/features/visualization/thinkingStages'
import { StageHistoryPanel } from './StageHistoryPanel'

describe('StageHistoryPanel', () => {
  const history: StageHistoryItem[] = [
    {
      stage: 'pattern',
      summary: '规律抽取摘要',
      text: '规律抽取完整内容',
      startSeq: 1,
      endSeq: 5,
    },
  ]

  it('无历史时显示空状态', () => {
    render(<StageHistoryPanel history={[]} />)

    expect(screen.getByText('暂无历史环节记录')).toBeInTheDocument()
  })

  it('应渲染历史摘要并支持展开收起', () => {
    render(<StageHistoryPanel history={history} />)

    expect(screen.getByText('规律抽取中')).toBeInTheDocument()
    expect(screen.getByText('规律抽取摘要')).toBeInTheDocument()

    const toggle = screen.getByTestId('stage-history-toggle-0')
    expect(toggle).toHaveAttribute('aria-expanded', 'false')

    fireEvent.click(toggle)

    expect(toggle).toHaveAttribute('aria-expanded', 'true')
    expect(screen.getByTestId('stage-history-content-0')).toHaveTextContent('规律抽取完整内容')
  })

  it('支持键盘 Enter/Space 展开收起', () => {
    render(<StageHistoryPanel history={history} />)

    const toggle = screen.getByTestId('stage-history-toggle-0')

    fireEvent.keyDown(toggle, { key: 'Enter' })
    expect(toggle).toHaveAttribute('aria-expanded', 'true')

    fireEvent.keyDown(toggle, { key: ' ' })
    expect(toggle).toHaveAttribute('aria-expanded', 'false')
  })
})
