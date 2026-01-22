import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render } from '@testing-library/react'
import { DiversityMetricsChart } from './DiversityMetricsChart'

const radarChartSpy = vi.fn()

vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive">{children}</div>
  ),
  RadarChart: ({ data, children }: { data: unknown; children: React.ReactNode }) => {
    radarChartSpy(data)
    return <div data-testid="radar-chart">{children}</div>
  },
  Radar: () => <div data-testid="radar" />,
  PolarGrid: () => null,
  PolarAngleAxis: () => null,
  PolarRadiusAxis: () => null,
  Tooltip: () => null,
}))

describe('DiversityMetricsChart', () => {
  beforeEach(() => {
    radarChartSpy.mockClear()
  })

  it('应将多样性指标绑定到雷达图数据', () => {
    render(
      <DiversityMetricsChart
        metrics={{
          lexicalDiversity: 0.6,
          structuralDiversity: 0.3,
          semanticDiversity: 0.1,
          overallScore: 0.4,
        }}
      />,
    )

    expect(radarChartSpy).toHaveBeenCalledTimes(1)
    expect(radarChartSpy).toHaveBeenCalledWith([
      { label: '词汇多样性', score: 0.6 },
      { label: '结构多样性', score: 0.3 },
      { label: '语义多样性', score: 0.1 },
    ])
  })
})
