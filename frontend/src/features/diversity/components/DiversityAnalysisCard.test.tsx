import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { DiversityAnalysisCard } from './DiversityAnalysisCard'
import type { DiversityAnalysisResult } from '@/types/generated/models/DiversityAnalysisResult'

vi.mock('./DiversityMetricsChart', () => ({
  DiversityMetricsChart: () => <div data-testid="diversity-chart" />,
}))

describe('DiversityAnalysisCard', () => {
  const analysis: DiversityAnalysisResult = {
    metrics: {
      lexicalDiversity: 0.62,
      structuralDiversity: 0.48,
      semanticDiversity: 0.2,
      overallScore: 0.43,
    },
    baselineComparison: {
      overallDiff: -0.05,
      lexicalDiff: 0.01,
      structuralDiff: -0.08,
      semanticDiff: 0,
      trend: 'declined',
    },
    warnings: [
      {
        level: 'medium',
        message: '多样性下降',
        affectedMetrics: ['structural_diversity'],
      },
    ],
    suggestions: [
      {
        suggestionType: '提升表达变体',
        content: '考虑加入更多语气或结构变化。',
      },
    ],
    analyzedAt: '2025-01-01T00:00:00Z',
    sampleCount: 8,
  }

  it('应展示多样性分析核心信息', () => {
    render(<DiversityAnalysisCard analysis={analysis} />)

    expect(screen.getByText('多样性分析')).toBeInTheDocument()
    expect(screen.getByText('样本数：8 · 分析时间：2025-01-01T00:00:00Z')).toBeInTheDocument()
    expect(screen.getByText('整体分数')).toBeInTheDocument()
    expect(screen.getByText('43.0%')).toBeInTheDocument()
    expect(screen.getByTestId('diversity-chart')).toBeInTheDocument()
    expect(screen.getByText('整体下降')).toBeInTheDocument()
    expect(screen.getAllByText('多样性下降').length).toBeGreaterThan(0)
    expect(screen.getByText('提升表达变体')).toBeInTheDocument()
  })

  it('加载中时显示提示', () => {
    render(<DiversityAnalysisCard analysis={null} isLoading />)

    expect(screen.getByTestId('diversity-loading')).toBeInTheDocument()
  })

  it('失败时可重试', () => {
    const onRetry = vi.fn()
    render(
      <DiversityAnalysisCard
        analysis={null}
        error={new Error('网络错误')}
        onRetry={onRetry}
      />
    )

    expect(screen.getByText('多样性分析暂不可用')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: '重试' }))
    expect(onRetry).toHaveBeenCalledTimes(1)
  })

  it('语义多样性不可用时提示未启用', () => {
    const analysisWithSemanticWarning: DiversityAnalysisResult = {
      ...analysis,
      warnings: [
        {
          level: 'low',
          message: '语义多样性暂不可用（未提供 embedding）',
          affectedMetrics: ['semantic'],
        },
      ],
    }

    render(<DiversityAnalysisCard analysis={analysisWithSemanticWarning} />)
    expect(screen.getByText('未启用 embedding')).toBeInTheDocument()
  })
})
