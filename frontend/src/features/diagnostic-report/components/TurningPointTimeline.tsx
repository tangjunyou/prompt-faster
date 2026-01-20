/**
 * 关键转折点时间线
 */

import { Badge } from '@/components/ui/badge'
import { formatCheckpointTime } from '@/lib/formatters'
import type { TurningPoint } from '@/types/generated/models/TurningPoint'

export interface TurningPointTimelineProps {
  turningPoints: TurningPoint[]
}

const typeStyles: Record<
  string,
  { label: string; badge: 'default' | 'secondary' | 'outline'; textClass: string }
> = {
  improvement: { label: '提升', badge: 'default', textClass: 'text-emerald-700' },
  regression: { label: '回退', badge: 'outline', textClass: 'text-rose-600' },
  breakthrough: { label: '突破', badge: 'secondary', textClass: 'text-blue-600' },
}

function formatPassRate(rate?: number | null) {
  if (rate === null || rate === undefined) return '—'
  return `${(rate * 100).toFixed(1)}%`
}

export function TurningPointTimeline({ turningPoints }: TurningPointTimelineProps) {
  if (turningPoints.length === 0) {
    return <div className="text-sm text-muted-foreground">暂无关键转折点</div>
  }

  return (
    <div className="space-y-3">
      {turningPoints.map((point) => {
        const config = typeStyles[point.eventType] ?? {
          label: point.eventType,
          badge: 'secondary' as const,
          textClass: 'text-muted-foreground',
        }
        return (
          <div key={`${point.round}-${point.eventType}`} className="rounded-lg border p-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={config.badge}>{config.label}</Badge>
              <span className={`text-sm font-medium ${config.textClass}`}>
                第 {point.round} 轮
              </span>
              <span className="text-xs text-muted-foreground">
                {formatCheckpointTime(point.timestamp)}
              </span>
            </div>
            <div className="mt-2 text-sm text-muted-foreground">{point.description}</div>
            <div className="mt-2 text-xs text-muted-foreground">
              通过率：{formatPassRate(point.passRateBefore)} → {formatPassRate(point.passRateAfter)}
            </div>
          </div>
        )
      })}
    </div>
  )
}
