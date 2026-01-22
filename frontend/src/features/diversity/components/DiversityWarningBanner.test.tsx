import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { DiversityWarningBanner } from './DiversityWarningBanner'
import type { DiversityWarning } from '@/types/generated/models/DiversityWarning'

describe('DiversityWarningBanner', () => {
  it('应展示告警级别、原因与建议链接', () => {
    const warning: DiversityWarning = {
      level: 'high',
      message: '输出过于单一，建议增加表达方式',
      affectedMetrics: ['lexical_diversity', 'structural_diversity'],
    }

    render(<DiversityWarningBanner warning={warning} />)

    expect(screen.getByText('高风险 · 多样性告警')).toBeInTheDocument()
    expect(screen.getByText(warning.message)).toBeInTheDocument()
    expect(screen.getByText('涉及指标：词汇多样性、结构多样性')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: '查看优化建议' })).toBeInTheDocument()
  })
})
