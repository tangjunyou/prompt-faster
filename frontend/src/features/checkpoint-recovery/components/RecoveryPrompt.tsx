/**
 * 恢复提示对话框
 */

import { useMemo, useState } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useUnfinishedTasks } from '../hooks/useUnfinishedTasks'
import { useAbortRecovery, useRecoverTask } from '../hooks/useRecovery'
import { useLocation, useNavigate } from 'react-router'
import { useAuthStore } from '@/stores/useAuthStore'
import type { RecoveryResponse } from '@/types/generated/models/RecoveryResponse'

export function RecoveryPrompt() {
  const location = useLocation()
  const navigate = useNavigate()
  const authStatus = useAuthStore((state) => state.authStatus)
  const isLoginPage = location.pathname === '/login'
  const { data, isLoading } = useUnfinishedTasks({
    enabled: authStatus === 'authenticated' && !isLoginPage,
  })
  const tasks = useMemo(() => data?.tasks ?? [], [data?.tasks])
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null)
  const [recoveryResult, setRecoveryResult] = useState<RecoveryResponse | null>(null)
  const [dismissedSignature, setDismissedSignature] = useState<string | null>(null)

  const recoverMutation = useRecoverTask()
  const abortMutation = useAbortRecovery()

  const taskSignature = useMemo(
    () => tasks.map((task) => task.taskId).join('|'),
    [tasks],
  )
  const resolvedSelectedTaskId = useMemo(() => {
    if (tasks.length === 0) return null
    const exists = tasks.some((task) => task.taskId === selectedTaskId)
    return exists ? selectedTaskId : tasks[0]?.taskId ?? null
  }, [tasks, selectedTaskId])
  const isDialogOpen = tasks.length > 0 && dismissedSignature !== taskSignature

  const selectedTask = useMemo(
    () => tasks.find((task) => task.taskId === resolvedSelectedTaskId) ?? tasks[0],
    [tasks, resolvedSelectedTaskId],
  )

  if (isLoginPage || authStatus !== 'authenticated') {
    return null
  }

  if (!tasks.length || isLoading) {
    return null
  }

  const isBusy = recoverMutation.isPending || abortMutation.isPending

  const handleRecover = async () => {
    if (!selectedTask) return
    try {
      const result = await recoverMutation.mutateAsync({
        taskId: selectedTask.taskId,
        checkpointId: selectedTask.checkpointId,
      })
      setRecoveryResult(result)
    } catch {
      // 失败时保持对话框打开，便于重试或切换任务
    }
  }

  const handleAbort = async () => {
    if (!selectedTask) return
    await abortMutation.mutateAsync({ taskId: selectedTask.taskId })
    setRecoveryResult(null)
    setDismissedSignature(taskSignature)
  }

  const handleContinue = () => {
    if (!selectedTask) return
    navigate(`/run?taskId=${selectedTask.taskId}&resume=1`)
    setRecoveryResult(null)
    setDismissedSignature(taskSignature)
  }

  const handleViewPaused = () => {
    if (!selectedTask) return
    navigate(`/run?taskId=${selectedTask.taskId}`)
    setRecoveryResult(null)
    setDismissedSignature(taskSignature)
  }

  return (
    <Dialog
      open={isDialogOpen}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) {
          setRecoveryResult(null)
          setDismissedSignature(taskSignature)
        } else {
          setDismissedSignature(null)
        }
      }}
    >
      <DialogContent className="sm:max-w-[560px]">
        <DialogHeader>
          <DialogTitle>检测到未完成任务</DialogTitle>
          <DialogDescription>
            系统检测到上次未完成的任务，是否恢复？
          </DialogDescription>
        </DialogHeader>

        {!recoveryResult ? (
          <div className="space-y-3 max-h-[240px] overflow-y-auto">
            {tasks.map((task) => (
              <button
                key={task.taskId}
                type="button"
                onClick={() => setSelectedTaskId(task.taskId)}
                className={`w-full rounded-md border px-3 py-2 text-left text-sm transition ${
                  resolvedSelectedTaskId === task.taskId
                    ? 'border-primary bg-primary/5'
                    : 'border-border hover:border-primary/60'
                }`}
              >
                <div className="font-medium">{task.taskName}</div>
                <div className="text-xs text-muted-foreground">
                  最近 Checkpoint：{task.lastCheckpointAt}
                </div>
                <div className="text-xs text-muted-foreground">
                  迭代：{task.iteration} · 状态：{task.state} · 运行控制：
                  {task.runControlState === 'paused'
                    ? '已暂停'
                    : task.runControlState === 'running'
                      ? '运行中'
                      : task.runControlState}
                </div>
              </button>
            ))}
          </div>
        ) : (
          <div className="rounded-md border bg-muted/40 p-3 text-sm">
            <div className="font-medium">恢复完成</div>
            <div className="mt-1 text-xs text-muted-foreground">
              任务：{selectedTask?.taskName ?? recoveryResult.taskId}
            </div>
            <div className="mt-1 text-xs text-muted-foreground">
              迭代：{recoveryResult.iteration} · 状态：{recoveryResult.state} · 运行控制：
              {recoveryResult.runControlState === 'paused'
                ? '已暂停'
                : recoveryResult.runControlState === 'running'
                  ? '运行中'
                  : recoveryResult.runControlState}
            </div>
          </div>
        )}

        {(recoverMutation.error || abortMutation.error) && (
          <div className="text-sm text-red-600">
            {(recoverMutation.error ?? abortMutation.error)?.message ?? '操作失败'}
          </div>
        )}

        <DialogFooter className="gap-2 sm:gap-0">
          {!recoveryResult ? (
            <>
              <Button
                type="button"
                variant="outline"
                onClick={handleAbort}
                disabled={isBusy}
              >
                放弃恢复
              </Button>
              <Button
                type="button"
                onClick={handleRecover}
                disabled={isBusy}
              >
                恢复
              </Button>
            </>
          ) : (
            <>
              <Button
                type="button"
                variant="outline"
                onClick={handleViewPaused}
              >
                暂停查看
              </Button>
              <Button type="button" onClick={handleContinue}>
                继续迭代
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default RecoveryPrompt
