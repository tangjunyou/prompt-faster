/**
 * 历史迭代面板组件
 * 显示历史迭代列表，支持展开查看详情
 */

import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { History, RefreshCw, PlayCircle } from 'lucide-react'
import { useIterationHistory } from './hooks/useIterationHistory'
import { useCheckpoints } from '@/features/checkpoint-recovery/hooks/useCheckpoints'
import { useConnectivity } from '@/features/checkpoint-recovery/hooks/useConnectivity'
import { IterationHistoryItem } from './IterationHistoryItem'

export interface HistoryPanelProps {
  /** 任务 ID */
  taskId: string
  /** 开始优化回调（空状态 CTA） */
  onStartOptimization?: () => void
}

function formatTime(isoString: string): string {
  try {
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return isoString
  }
}

/**
 * 历史迭代面板
 * 显示历史迭代列表，按轮次倒序排列
 */
export function HistoryPanel({ taskId, onStartOptimization }: HistoryPanelProps) {
  const [expandedIterationId, setExpandedIterationId] = useState<string | null>(null)
  const { isOffline } = useConnectivity()

  const {
    data: iterations,
    isLoading,
    error,
    refetch,
  } = useIterationHistory(taskId)
  const {
    data: checkpointData,
    isLoading: isLoadingCheckpoints,
    error: checkpointError,
    refetch: refetchCheckpoints,
  } = useCheckpoints(taskId, { limit: 10 })

  // 切换展开状态
  const handleToggle = (iterationId: string) => {
    setExpandedIterationId((prev) => (prev === iterationId ? null : iterationId))
  }

  const handleRefresh = () => {
    refetch()
    refetchCheckpoints()
  }

  const checkpointCount = checkpointData?.total ?? 0

  const checkpointSection = (
    <div className="pt-4 mt-4 border-t">
      <div className="flex items-center justify-between">
        <p className="text-sm font-medium">Checkpoint</p>
        <span className="text-xs text-muted-foreground">
          共 {checkpointCount} 个
        </span>
      </div>
      {isLoadingCheckpoints ? (
        <div className="flex items-center py-3 text-xs text-muted-foreground">
          <RefreshCw className="h-3 w-3 animate-spin" />
          <span className="ml-2">加载中...</span>
        </div>
      ) : checkpointError ? (
        <p className="text-xs text-destructive mt-2">
          加载失败：{checkpointError.message}
        </p>
      ) : checkpointData && checkpointData.checkpoints.length > 0 ? (
        <div className="mt-2 space-y-2 max-h-[220px] overflow-y-auto">
          {checkpointData.checkpoints.map((checkpoint) => (
            <div
              key={checkpoint.id}
              className="flex items-start justify-between gap-3 border rounded-md p-2 text-xs"
            >
              <div className="flex flex-col gap-1 min-w-0">
                <span className="font-mono text-[11px] text-muted-foreground">
                  迭代 #{checkpoint.iteration}
                </span>
                <span className="truncate">
                  {checkpoint.promptPreview || '无 Prompt 摘要'}
                </span>
              </div>
              <span className="text-[11px] text-muted-foreground shrink-0">
                {formatTime(checkpoint.createdAt)}
              </span>
            </div>
          ))}
        </div>
      ) : (
        <p className="text-xs text-muted-foreground mt-2">
          暂无 Checkpoint
        </p>
      )}
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

  // 正常状态 - 显示历史列表
  return (
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
  )
}

export default HistoryPanel
