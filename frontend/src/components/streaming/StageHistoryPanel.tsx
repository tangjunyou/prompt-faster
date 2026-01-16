import { useState } from 'react'

import type { StageHistoryItem } from '@/features/visualization/thinkingStages'
import { STAGE_LABELS } from '@/features/visualization/thinkingStages'

export type StageHistoryPanelProps = {
  history: StageHistoryItem[]
  prefersReducedMotion?: boolean
  className?: string
}

export function StageHistoryPanel({ history, prefersReducedMotion = false, className = '' }: StageHistoryPanelProps) {
  const [openItems, setOpenItems] = useState<Set<number>>(() => new Set())

  const toggleItem = (index: number) => {
    setOpenItems((prev) => {
      const next = new Set(prev)
      if (next.has(index)) {
        next.delete(index)
      } else {
        next.add(index)
      }
      return next
    })
  }

  const transitionClassName = prefersReducedMotion ? '' : 'transition-colors duration-200'

  return (
    <div className={`flex flex-col ${className}`.trim()} data-testid="stage-history-panel">
      <div className="px-4 py-2 text-xs font-semibold uppercase text-muted-foreground">历史环节摘要</div>
      {history.length === 0 ? (
        <div className="px-4 pb-3 text-xs text-slate-500">暂无历史环节记录</div>
      ) : (
        <div className="flex flex-col gap-2 px-3 pb-3">
          {history.map((item, index) => {
            const isOpen = openItems.has(index)
            const label = STAGE_LABELS[item.stage]
            return (
              <div
                key={`${item.stage}-${item.startSeq}-${index}`}
                className="rounded-md border border-slate-200 bg-slate-50"
                data-testid={`stage-history-item-${index}`}
              >
                <div className="flex items-center justify-between gap-2 px-3 py-2">
                  <div className="flex flex-col gap-1">
                    <span className="text-xs font-medium text-slate-700">{label}</span>
                    <span className="text-xs text-slate-500">{item.summary}</span>
                  </div>
                  <button
                    type="button"
                    onClick={() => toggleItem(index)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' || event.key === ' ') {
                        event.preventDefault()
                        toggleItem(index)
                      }
                    }}
                    className={`rounded-md border border-slate-200 px-2 py-1 text-xs text-slate-600 hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 ${transitionClassName}`.trim()}
                    aria-expanded={isOpen}
                    aria-controls={`stage-history-content-${index}`}
                    aria-label={`${label} ${isOpen ? '收起' : '展开'}详情`}
                    data-testid={`stage-history-toggle-${index}`}
                  >
                    {isOpen ? '收起' : '展开'}
                  </button>
                </div>
                {isOpen ? (
                  <div
                    id={`stage-history-content-${index}`}
                    className="border-t border-slate-200 bg-white px-3 py-2 text-xs text-slate-600 whitespace-pre-wrap"
                    data-testid={`stage-history-content-${index}`}
                  >
                    {item.text}
                  </div>
                ) : null}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
