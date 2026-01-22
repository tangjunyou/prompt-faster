import type { DiversityWarning } from '@/types/generated/models/DiversityWarning'

const levelOrder: Record<DiversityWarning['level'], number> = {
  high: 3,
  medium: 2,
  low: 1,
}

const metricLabels: Record<string, string> = {
  lexical: '词汇多样性',
  lexical_diversity: '词汇多样性',
  structural: '结构多样性',
  structural_diversity: '结构多样性',
  semantic: '语义多样性',
  semantic_diversity: '语义多样性',
  overall: '整体多样性',
  overall_score: '整体多样性',
}

export function formatAffectedMetrics(metrics?: string[]) {
  if (!metrics?.length) return null
  return metrics.map((metric) => metricLabels[metric] ?? metric).join('、')
}

export function pickHighestWarning(warnings: DiversityWarning[]) {
  if (warnings.length === 0) return null
  return warnings.reduce((highest, current) => {
    if (!highest) return current
    return levelOrder[current.level] > levelOrder[highest.level] ? current : highest
  }, warnings[0] ?? null)
}
