/**
 * 迭代控制面板
 *
 * 集成增加轮数和终止任务功能的控制面板
 */

import { useState, useCallback } from 'react'
import { Plus, StopCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { AddRoundsDialog } from './AddRoundsDialog'
import { TerminateDialog } from './TerminateDialog'
import type { RunControlState } from '@/types/generated/models/RunControlState'

export interface IterationControlPanelProps {
  /** 任务 ID */
  taskId: string
  /** 工作区 ID（用于刷新任务数据） */
  workspaceId?: string
  /** 运行控制状态 */
  runControlState: RunControlState
  /** 当前最大轮数 */
  currentMaxIterations?: number
  /** 当前轮次 */
  currentRound?: number
  /** 自定义类名 */
  className?: string
  /** 状态变更回调 */
  onStatusChange?: () => void
}

/**
 * 迭代控制面板
 *
 * 在任务运行中或暂停状态下显示增加轮数和终止任务按钮
 */
export function IterationControlPanel({
  taskId,
  workspaceId,
  runControlState,
  currentMaxIterations = 0,
  currentRound = 0,
  className,
  onStatusChange,
}: IterationControlPanelProps) {
  const [addRoundsOpen, setAddRoundsOpen] = useState(false)
  const [terminateOpen, setTerminateOpen] = useState(false)

  // 仅在 Running/Paused 状态下可操作
  const canControl = runControlState === 'running' || runControlState === 'paused'

  const handleAddRoundsSuccess = useCallback(() => {
    onStatusChange?.()
  }, [onStatusChange])

  const handleTerminateSuccess = useCallback(() => {
    onStatusChange?.()
  }, [onStatusChange])

  if (!canControl) {
    const statusLabel =
      runControlState === 'idle'
        ? '任务未开始运行，暂不可增加轮数或终止。'
        : runControlState === 'stopped'
          ? '任务已停止，暂不可增加轮数或终止。'
          : '当前状态不支持迭代控制操作。'
    return (
      <div className={`text-xs text-muted-foreground ${className ?? ''}`}>
        {statusLabel}
      </div>
    )
  }

  return (
    <div className={`flex items-center gap-3 ${className ?? ''}`}>
      <div className="text-xs text-muted-foreground">
        当前轮次 {currentRound} / 最大 {currentMaxIterations}
      </div>
      <Button
        variant="outline"
        size="sm"
        onClick={() => setAddRoundsOpen(true)}
        className="min-w-[44px] min-h-[44px]"
        title="增加迭代轮数"
      >
        <Plus className="h-4 w-4 mr-1" />
        增加轮数
      </Button>

      <Button
        variant="outline"
        size="sm"
        onClick={() => setTerminateOpen(true)}
        className="min-w-[44px] min-h-[44px] text-destructive hover:text-destructive"
        title="终止优化任务"
      >
        <StopCircle className="h-4 w-4 mr-1" />
        终止任务
      </Button>

      <AddRoundsDialog
        taskId={taskId}
        workspaceId={workspaceId}
        open={addRoundsOpen}
        onOpenChange={setAddRoundsOpen}
        currentMaxIterations={currentMaxIterations}
        currentRound={currentRound}
        onSuccess={handleAddRoundsSuccess}
      />

      <TerminateDialog
        taskId={taskId}
        workspaceId={workspaceId}
        open={terminateOpen}
        onOpenChange={setTerminateOpen}
        onSuccess={handleTerminateSuccess}
      />
    </div>
  )
}

export default IterationControlPanel
