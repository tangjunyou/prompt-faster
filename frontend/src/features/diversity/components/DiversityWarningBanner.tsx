/**
 * 多样性告警提示条
 */

import type { DiversityWarning } from '@/types/generated/models/DiversityWarning'
import { formatAffectedMetrics } from '../utils/diversityWarning'

const levelMeta: Record<
  DiversityWarning['level'],
  { label: string; className: string }
> = {
  low: {
    label: '低风险',
    className: 'border-amber-200 bg-amber-50 text-amber-900',
  },
  medium: {
    label: '中等风险',
    className: 'border-orange-200 bg-orange-50 text-orange-900',
  },
  high: {
    label: '高风险',
    className: 'border-red-200 bg-red-50 text-red-900',
  },
}

export interface DiversityWarningBannerProps {
  warning: DiversityWarning
  suggestionHref?: string
}

export function DiversityWarningBanner({
  warning,
  suggestionHref = '#diversity-suggestions',
}: DiversityWarningBannerProps) {
  const meta = levelMeta[warning.level] ?? levelMeta.low
  const affected = formatAffectedMetrics(warning.affectedMetrics)

  return (
    <div className={`rounded-lg border px-4 py-3 text-sm ${meta.className}`} role="status">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="font-medium">{meta.label} · 多样性告警</div>
        <a
          href={suggestionHref}
          className="text-xs font-medium underline underline-offset-2"
        >
          查看优化建议
        </a>
      </div>
      <div className="mt-2 text-sm">{warning.message}</div>
      {affected ? (
        <div className="mt-2 text-xs text-muted-foreground">涉及指标：{affected}</div>
      ) : null}
    </div>
  )
}

export default DiversityWarningBanner
