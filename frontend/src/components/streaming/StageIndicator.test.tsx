import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { StageIndicator } from './StageIndicator'

describe('StageIndicator', () => {
  it('未提供环节时显示等待开始', () => {
    render(<StageIndicator stage={null} />)

    expect(screen.getByTestId('stage-indicator')).toHaveTextContent('等待开始')
  })

  it('提供环节时显示对应中文标识', () => {
    render(<StageIndicator stage="pattern" />)

    expect(screen.getByText('规律抽取中')).toBeInTheDocument()
  })

  it('prefersReducedMotion=true 时不包含过渡类', () => {
    render(<StageIndicator stage="quality" prefersReducedMotion={true} />)

    const badge = screen.getByText('质量评估中')
    expect(badge.className).not.toContain('transition-colors')
  })
})
