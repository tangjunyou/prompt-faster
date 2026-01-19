/**
 * Checkpoint 列表组件
 */

import { Button } from '@/components/ui/button'
import { RefreshCw } from 'lucide-react'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'
import { formatCheckpointTime, formatPassRateSummary } from '@/lib/formatters'

export interface CheckpointListProps {
  checkpoints: CheckpointSummary[]
  selectedCheckpointId?: string | null
  isLoading?: boolean
  error?: Error | null
  onRetry?: () => void
  onSelect?: (checkpoint: CheckpointSummary) => void
}

export function CheckpointList({
  checkpoints,
  selectedCheckpointId,
  isLoading,
  error,
  onRetry,
  onSelect,
}: CheckpointListProps) {
  if (isLoading) {
    return (
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <RefreshCw className="h-3 w-3 animate-spin" />
        <span>加载中...</span>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-2 text-xs text-destructive">
        <p>加载失败：{error.message}</p>
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry}>
            重试
          </Button>
        )}
      </div>
    )
  }

  if (!checkpoints || checkpoints.length === 0) {
    return <p className="text-xs text-muted-foreground">暂无 Checkpoint</p>
  }

  return (
    <div className="space-y-3">
      {checkpoints.map((checkpoint) => {
        const isArchived = Boolean(checkpoint.archivedAt)
        const isSelected = selectedCheckpointId === checkpoint.id
        return (
          <div
            key={checkpoint.id}
            className={`flex items-start justify-between gap-3 border rounded-md p-3 text-xs ${
              isArchived ? 'opacity-60' : ''
            } ${isSelected ? 'ring-1 ring-primary border-primary/60' : ''}`.trim()}
          >
            <div className="flex flex-col gap-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-mono text-[11px] text-muted-foreground">
                  迭代 #{checkpoint.iteration}
                </span>
                <span className="text-[11px] text-muted-foreground">
                  {formatCheckpointTime(checkpoint.createdAt)}
                </span>
              </div>
              <span className="text-[11px] text-muted-foreground">
                {formatPassRateSummary(checkpoint.passRateSummary)}
              </span>
              {checkpoint.archivedAt ? (
                <span className="text-[11px] text-muted-foreground truncate">
                  已归档：{checkpoint.archiveReason || '被回滚归档'}
                </span>
              ) : null}
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={isArchived || !onSelect}
              onClick={() => onSelect?.(checkpoint)}
            >
              回滚
            </Button>
          </div>
        )
      })}
    </div>
  )
}

export default CheckpointList
