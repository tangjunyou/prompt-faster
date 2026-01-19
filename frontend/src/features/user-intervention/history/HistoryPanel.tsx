/**
 * 历史迭代面板组件
 * 显示历史迭代列表，支持展开查看详情
 */

import { useEffect, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { History, RefreshCw, PlayCircle } from 'lucide-react'
import { useTaskHistory } from './hooks/useTaskHistory'
import { getTaskHistory } from './services/taskHistoryService'
import { useConnectivity } from '@/features/checkpoint-recovery/hooks/useConnectivity'
import { IterationHistoryItem } from './IterationHistoryItem'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'
import { CheckpointList } from '@/features/checkpoint-recovery/components/CheckpointList'
import { RollbackConfirmDialog } from '@/features/checkpoint-recovery/components/RollbackConfirmDialog'
import { useRollback } from '@/features/checkpoint-recovery/hooks/useRollback'

export interface HistoryPanelProps {
  /** 任务 ID */
  taskId: string
  /** 开始优化回调（空状态 CTA） */
  onStartOptimization?: () => void
}

const CHECKPOINT_PAGE_SIZE = 50

/**
 * 历史迭代面板
 * 显示历史迭代列表，按轮次倒序排列
 */
export function HistoryPanel({ taskId, onStartOptimization }: HistoryPanelProps) {
  const [expandedIterationId, setExpandedIterationId] = useState<string | null>(null)
  const [selectedCheckpoint, setSelectedCheckpoint] =
    useState<CheckpointSummary | null>(null)
  const [checkpointItems, setCheckpointItems] = useState<CheckpointSummary[]>([])
  const [checkpointTotal, setCheckpointTotal] = useState(0)
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  const [loadMoreError, setLoadMoreError] = useState<string | null>(null)
  const [rollbackFeedback, setRollbackFeedback] = useState<{
    type: 'success' | 'error'
    message: string
  } | null>(null)
  const { isOffline } = useConnectivity()
  const rollbackMutation = useRollback()

  const {
    data: historyData,
    isLoading,
    error,
    refetch,
  } = useTaskHistory(taskId, {
    checkpointsLimit: CHECKPOINT_PAGE_SIZE,
    includeArchived: true,
  })
  const iterations = historyData?.iterations ?? []
  const checkpointData = historyData?.checkpoints

  useEffect(() => {
    if (!checkpointData) {
      return
    }
    setCheckpointItems(checkpointData.checkpoints ?? [])
    setCheckpointTotal(checkpointData.total ?? 0)
    setLoadMoreError(null)
  }, [checkpointData])

  // 切换展开状态
  const handleToggle = (iterationId: string) => {
    setExpandedIterationId((prev) => (prev === iterationId ? null : iterationId))
  }

  const handleRefresh = () => {
    refetch()
  }

  const checkpointCount = checkpointTotal
  const handleConfirmRollback = async () => {
    if (!selectedCheckpoint) {
      return
    }
    try {
      const response = await rollbackMutation.mutateAsync({
        taskId,
        checkpointId: selectedCheckpoint.id,
      })
      setRollbackFeedback({ type: 'success', message: response.message })
      setSelectedCheckpoint(null)
      refetch()
    } catch (err) {
      const message =
        err instanceof Error ? err.message : '回滚失败，请稍后再试'
      setRollbackFeedback({ type: 'error', message })
    }
  }

  const handleCancelRollback = () => {
    setSelectedCheckpoint(null)
  }

  const handleLoadMore = async () => {
    if (isLoadingMore || checkpointItems.length >= checkpointTotal) {
      return
    }
    setIsLoadingMore(true)
    setLoadMoreError(null)
    try {
      const nextPage = await getTaskHistory(taskId, {
        includeArchived: true,
        checkpointsLimit: CHECKPOINT_PAGE_SIZE,
        checkpointsOffset: checkpointItems.length,
      })
      const nextItems = nextPage.checkpoints.checkpoints ?? []
      setCheckpointItems((prev) => [...prev, ...nextItems])
      setCheckpointTotal(nextPage.checkpoints.total ?? checkpointTotal)
    } catch (err) {
      const message =
        err instanceof Error ? err.message : '加载更多失败，请稍后再试'
      setLoadMoreError(message)
    } finally {
      setIsLoadingMore(false)
    }
  }

  const checkpointSection = (
    <div className="pt-4 mt-4 border-t">
      <div className="flex items-center justify-between">
        <p className="text-sm font-medium">Checkpoint</p>
        <span className="text-xs text-muted-foreground">
          共 {checkpointCount} 个
        </span>
      </div>
      <div className="mt-2">
        <CheckpointList
          checkpoints={checkpointItems}
          isLoading={isLoading}
          error={error}
          selectedCheckpointId={selectedCheckpoint?.id}
          onSelect={(checkpoint) => {
            setRollbackFeedback(null)
            setSelectedCheckpoint(checkpoint)
          }}
        />
      </div>
      {rollbackFeedback ? (
        <p
          className={`text-xs mt-2 ${
            rollbackFeedback.type === 'success'
              ? 'text-emerald-600'
              : 'text-destructive'
          }`}
        >
          {rollbackFeedback.message}
        </p>
      ) : null}
      {loadMoreError ? (
        <p className="text-xs text-destructive mt-2">{loadMoreError}</p>
      ) : null}
      {checkpointItems.length < checkpointTotal ? (
        <div className="mt-3 flex items-center justify-center">
          <Button
            variant="outline"
            size="sm"
            onClick={handleLoadMore}
            disabled={isLoadingMore}
            className="min-w-[44px] min-h-[44px]"
          >
            {isLoadingMore ? '加载中...' : '加载更多'}
          </Button>
        </div>
      ) : null}
    </div>
  )

  // 加载中状态
  if (isLoading) {
    return (
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <History className="h-5 w-5" />
            历史记录
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <RefreshCw className="h-5 w-5 animate-spin text-muted-foreground" />
            <span className="ml-2 text-sm text-muted-foreground">加载中...</span>
          </div>
          {checkpointSection}
        </CardContent>
      </Card>
    )
  }

  // 错误状态
  if (error) {
    return (
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <History className="h-5 w-5" />
            历史记录
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <p className="text-sm text-destructive mb-4">
              加载失败：{error.message}
            </p>
            <Button
              variant="outline"
              size="sm"
              onClick={handleRefresh}
              className="min-w-[44px] min-h-[44px]"
            >
              <RefreshCw className="h-4 w-4 mr-1" />
              重试
            </Button>
          </div>
          {checkpointSection}
        </CardContent>
      </Card>
    )
  }

  // 空状态
  if (!iterations || iterations.length === 0) {
    return (
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <History className="h-5 w-5" />
            历史记录
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <p className="text-sm text-muted-foreground mb-4">
              暂无历史记录
            </p>
            <p className="text-xs text-muted-foreground mb-4">
              开始优化任务以生成历史记录
            </p>
            {onStartOptimization && (
              <Button
                variant="default"
                size="sm"
                onClick={onStartOptimization}
                disabled={isOffline}
                className="min-w-[44px] min-h-[44px]"
                title={isOffline ? '当前离线，无法开始优化' : undefined}
              >
                <PlayCircle className="h-4 w-4 mr-1" />
                开始优化
              </Button>
            )}
          </div>
          {checkpointSection}
        </CardContent>
      </Card>
    )
  }

  // 正常状态 - 显示历史列表与回滚对话框
  return (
    <>
      <Card className="w-full">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg flex items-center gap-2">
              <History className="h-5 w-5" />
              历史记录
            </CardTitle>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleRefresh}
              className="min-w-[44px] min-h-[44px]"
              title="刷新"
            >
              <RefreshCw className="h-4 w-4" />
            </Button>
          </div>
          <p className="text-sm text-muted-foreground mt-1">
            共 {iterations.length} 轮迭代
          </p>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 max-h-[500px] overflow-y-auto">
            {iterations.map((iteration) => (
              <IterationHistoryItem
                key={iteration.id}
                taskId={taskId}
                summary={iteration}
                isExpanded={expandedIterationId === iteration.id}
                onToggle={() => handleToggle(iteration.id)}
              />
            ))}
          </div>
          {checkpointSection}
        </CardContent>
      </Card>
      <RollbackConfirmDialog
        open={Boolean(selectedCheckpoint)}
        checkpoint={selectedCheckpoint}
        isSubmitting={rollbackMutation.isPending}
        onCancel={handleCancelRollback}
        onConfirm={handleConfirmRollback}
      />
    </>
  )
}

export default HistoryPanel
