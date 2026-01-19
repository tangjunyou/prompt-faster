import { useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ChevronDown, ChevronUp, Clock } from 'lucide-react'
import type { TimelineEntry } from '@/types/generated/models/TimelineEntry'

interface TimelineViewProps {
  entries: TimelineEntry[]
}

const ENTRY_LABELS: Record<string, string> = {
  iteration: '迭代',
  checkpoint: 'Checkpoint',
  event: '事件',
}

function formatTimestamp(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return value
  }
  return date.toLocaleString()
}

export function TimelineView({ entries }: TimelineViewProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null)

  return (
    <div className="space-y-4">
      {entries.map((entry) => {
        const isExpanded = expandedId === entry.id
        const entryLabel = ENTRY_LABELS[entry.entryType] ?? entry.entryType
        const actorLabel = entry.actor === 'system' ? '系统' : entry.actor === 'user' ? '用户' : null
        return (
          <div key={entry.id} className="relative pl-6 pb-4 border-l border-muted">
            <span className="absolute left-[-6px] top-2 h-3 w-3 rounded-full bg-primary" />
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="outline">{entryLabel}</Badge>
              <span className="text-sm font-medium">{entry.title}</span>
              {entry.iteration !== null ? (
                <Badge variant="secondary"># {entry.iteration}</Badge>
              ) : null}
            </div>
            <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              <Clock className="h-3 w-3" />
              <span>{formatTimestamp(entry.timestamp)}</span>
              {actorLabel ? <span>· {actorLabel}</span> : null}
            </div>
            {entry.description ? (
              <p className="text-sm text-muted-foreground mt-2">{entry.description}</p>
            ) : null}
            {entry.details ? (
              <div className="mt-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    setExpandedId(isExpanded ? null : entry.id)
                  }
                  className="px-2"
                >
                  {isExpanded ? (
                    <ChevronUp className="h-4 w-4 mr-1" />
                  ) : (
                    <ChevronDown className="h-4 w-4 mr-1" />
                  )}
                  {isExpanded ? '收起详情' : '查看详情'}
                </Button>
                {isExpanded ? (
                  <pre className="mt-2 rounded-md bg-muted p-3 text-xs overflow-auto max-h-[240px]">
                    {JSON.stringify(entry.details, null, 2)}
                  </pre>
                ) : null}
              </div>
            ) : null}
          </div>
        )
      })}
    </div>
  )
}
