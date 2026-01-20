import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'

import { TurningPointTimeline } from './TurningPointTimeline'
import type { TurningPoint } from '@/types/generated/models/TurningPoint'

const turningPoints: TurningPoint[] = [
  {
    round: 2,
    eventType: 'improvement',
    description: '提升',
    passRateBefore: 0.3,
    passRateAfter: 0.5,
    timestamp: '2025-01-01T00:00:00Z',
  },
]

describe('TurningPointTimeline', () => {
  it('应显示类型与颜色标识', () => {
    render(<TurningPointTimeline turningPoints={turningPoints} />)

    expect(screen.getAllByText('提升').length).toBeGreaterThan(0)
    const roundLabel = screen.getByText('第 2 轮')
    expect(roundLabel).toHaveClass('text-emerald-700')
  })
})
