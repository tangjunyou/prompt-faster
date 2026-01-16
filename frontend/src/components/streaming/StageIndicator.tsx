import type { StageType } from '@/features/visualization/thinkingStages'
import { STAGE_COLORS, STAGE_LABELS } from '@/features/visualization/thinkingStages'

export type StageIndicatorProps = {
  stage: StageType | null
  prefersReducedMotion?: boolean
  className?: string
}

export function StageIndicator({ stage, prefersReducedMotion = false, className = '' }: StageIndicatorProps) {
  const label = stage ? STAGE_LABELS[stage] : '等待开始'
  const colors = stage ? STAGE_COLORS[stage] : null
  const badgeClassName = colors
    ? `border ${colors.badge}`
    : 'border-slate-200 bg-slate-100 text-slate-600'
  const dotClassName = colors ? colors.dot : 'bg-slate-400'
  const transitionClassName = prefersReducedMotion ? '' : 'transition-colors duration-300'

  return (
    <div
      className={`flex items-center gap-2 ${className}`.trim()}
      aria-label={`当前环节：${label}`}
      data-testid="stage-indicator"
    >
      <span className={`h-2.5 w-2.5 rounded-full ${dotClassName}`} aria-hidden="true" />
      <span
        className={`inline-flex items-center rounded-full px-3 py-1 text-xs font-medium ${badgeClassName} ${transitionClassName}`.trim()}
      >
        {label}
      </span>
    </div>
  )
}
