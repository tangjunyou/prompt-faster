/**
 * 终止任务对话框
 *
 * 允许用户选择候选 Prompt 并终止任务
 */

import { useState, useCallback } from 'react'
import { AlertTriangle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useCandidates, useTerminateTask } from './hooks/useIterationControl'
import { CandidatePromptList } from './CandidatePromptList'

export interface TerminateDialogProps {
  /** 任务 ID */
  taskId: string
  /** 工作区 ID（用于刷新任务数据） */
  workspaceId?: string
  /** 是否打开 */
  open: boolean
  /** 关闭回调 */
  onOpenChange: (open: boolean) => void
  /** 成功回调 */
  onSuccess?: () => void
}

type TerminateDialogBodyProps = {
  taskId: string
  workspaceId?: string
  onOpenChange: (open: boolean) => void
  onSuccess?: () => void
}

function TerminateDialogBody({
  taskId,
  workspaceId,
  onOpenChange,
  onSuccess,
}: TerminateDialogBodyProps) {
  const [selectedIterationId, setSelectedIterationId] = useState<string | undefined>()
  const [copiedId, setCopiedId] = useState<string | null>(null)

  const pageSize = 20

  const {
    candidates,
    isLoading: isLoadingCandidates,
    isFetchingNextPage,
    fetchNextPage,
    hasMore,
  } = useCandidates(taskId, true, { limit: pageSize })
  const { mutate: terminate, isPending: isTerminating, error } = useTerminateTask(
    taskId,
    workspaceId
  )

  const handleSelect = useCallback((iterationId: string) => {
    setSelectedIterationId((prev) => (prev === iterationId ? undefined : iterationId))
  }, [])

  const handleCopyPrompt = useCallback(async (prompt: string, iterationId: string) => {
    try {
      await navigator.clipboard.writeText(prompt)
      setCopiedId(iterationId)
      setTimeout(() => setCopiedId(null), 2000)
    } catch {
      // 忽略复制失败
    }
  }, [])

  const handleTerminate = useCallback(() => {
    terminate(selectedIterationId, {
      onSuccess: () => {
        onOpenChange(false)
        setSelectedIterationId(undefined)
        onSuccess?.()
      },
    })
  }, [selectedIterationId, terminate, onOpenChange, onSuccess])

  return (
    <DialogContent className="sm:max-w-[600px] max-h-[80vh] overflow-hidden flex flex-col">
      <DialogHeader>
        <DialogTitle className="flex items-center gap-2">
          <AlertTriangle className="h-5 w-5 text-destructive" />
          终止优化任务
        </DialogTitle>
        <DialogDescription>
          选择一个候选 Prompt 作为最终结果。仅终止当前任务，且不可撤销。
        </DialogDescription>
      </DialogHeader>

      <div className="flex-1 overflow-y-auto py-4 space-y-3">
        {isLoadingCandidates ? (
          <div className="text-center text-muted-foreground py-8">加载候选列表中...</div>
        ) : candidates.length === 0 ? (
          <div className="text-center text-muted-foreground py-8">
            暂无候选 Prompt，将直接终止任务。
          </div>
        ) : (
          <>
            <CandidatePromptList
              candidates={candidates}
              selectedId={selectedIterationId ?? null}
              copiedId={copiedId}
              onSelect={handleSelect}
              onCopy={handleCopyPrompt}
            />
            {hasMore ? (
              <div className="pt-2 flex justify-center">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => fetchNextPage()}
                  disabled={isFetchingNextPage}
                >
                  {isFetchingNextPage ? '加载中...' : '加载更多'}
                </Button>
              </div>
            ) : (
              <div className="pt-2 text-xs text-muted-foreground text-center">已加载全部候选</div>
            )}
          </>
        )}

        {error && (
          <div className="text-sm text-destructive text-center space-y-2">
            <div>{error.message}</div>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleTerminate}
              disabled={isTerminating}
            >
              重试
            </Button>
          </div>
        )}
      </div>

      <DialogFooter className="border-t pt-4">
        <Button
          type="button"
          variant="outline"
          onClick={() => onOpenChange(false)}
          disabled={isTerminating}
        >
          取消
        </Button>
        <Button
          type="button"
          variant="destructive"
          onClick={handleTerminate}
          disabled={isTerminating}
        >
          {isTerminating ? '终止中...' : selectedIterationId ? '确认终止并保存' : '直接终止'}
        </Button>
      </DialogFooter>
    </DialogContent>
  )
}

export function TerminateDialog({
  taskId,
  workspaceId,
  open,
  onOpenChange,
  onSuccess,
}: TerminateDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      {open ? (
        <TerminateDialogBody
          taskId={taskId}
          workspaceId={workspaceId}
          onOpenChange={onOpenChange}
          onSuccess={onSuccess}
        />
      ) : null}
    </Dialog>
  )
}

export default TerminateDialog
