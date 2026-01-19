/**
 * 历史迭代面板组件
 * 显示历史迭代列表，支持展开查看详情
 */

import { useEffect, useMemo, useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Download, History, RefreshCw, PlayCircle } from 'lucide-react'
import { useTaskHistory } from './hooks/useTaskHistory'
import { getTaskHistory, getTimeline } from './services/taskHistoryService'
import { useConnectivity } from '@/features/checkpoint-recovery/hooks/useConnectivity'
import { IterationHistoryItem } from './IterationHistoryItem'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'
import { CheckpointList } from '@/features/checkpoint-recovery/components/CheckpointList'
import { RollbackConfirmDialog } from '@/features/checkpoint-recovery/components/RollbackConfirmDialog'
import { useRollback } from '@/features/checkpoint-recovery/hooks/useRollback'
import { useTimeline } from './hooks/useTimeline'
import { useExportHistory } from './hooks/useExportHistory'
import { HistoryFilter, type HistoryFilterValue } from './components/HistoryFilter'
import { TimelineView } from './components/TimelineView'
import type { TimelineEntry } from '@/types/generated/models/TimelineEntry'

export interface HistoryPanelProps {
  /** 任务 ID */
  taskId: string
  /** 开始优化回调（空状态 CTA） */
  onStartOptimization?: () => void
}

const CHECKPOINT_PAGE_SIZE = 50
const TIMELINE_PAGE_SIZE = 50

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
  const [viewMode, setViewMode] = useState<'list' | 'timeline'>('list')
  const [filterState, setFilterState] = useState<HistoryFilterValue>({
    eventTypes: [],
    actor: '',
    iterationMin: '',
    iterationMax: '',
    timeStart: '',
    timeEnd: '',
  })
  const [timelineEntries, setTimelineEntries] = useState<TimelineEntry[]>([])
  const [timelineTotal, setTimelineTotal] = useState(0)
  const [timelineLoadError, setTimelineLoadError] = useState<string | null>(null)
  const [timelineLoadingMore, setTimelineLoadingMore] = useState(false)
  const [exportError, setExportError] = useState<string | null>(null)
  const [rollbackFeedback, setRollbackFeedback] = useState<{
    type: 'success' | 'error'
    message: string
  } | null>(null)
  const { isOffline } = useConnectivity()
  const rollbackMutation = useRollback()
  const exportMutation = useExportHistory()

  const {
    data: historyData,
    isLoading,
    error,
    refetch,
  } = useTaskHistory(taskId, {
    checkpointsLimit: CHECKPOINT_PAGE_SIZE,
    includeArchived: true,
  })

  const timelineFilter = useMemo(() => {
    const filter: {
      eventTypes?: string[]
      actor?: 'system' | 'user'
      iterationMin?: number
      iterationMax?: number
      timeStart?: number
      timeEnd?: number
    } = {}
    if (filterState.eventTypes.length > 0) {
      filter.eventTypes = filterState.eventTypes
    }
    if (filterState.actor === 'system' || filterState.actor === 'user') {
      filter.actor = filterState.actor
    }
    if (filterState.iterationMin.trim() !== '') {
      const iterationMin = Number(filterState.iterationMin)
      if (!Number.isNaN(iterationMin)) {
        filter.iterationMin = iterationMin
      }
    }
    if (filterState.iterationMax.trim() !== '') {
      const iterationMax = Number(filterState.iterationMax)
      if (!Number.isNaN(iterationMax)) {
        filter.iterationMax = iterationMax
      }
    }
    if (filterState.timeStart) {
      const ts = new Date(filterState.timeStart).getTime()
      if (!Number.isNaN(ts)) {
        filter.timeStart = ts
      }
    }
    if (filterState.timeEnd) {
      const ts = new Date(filterState.timeEnd).getTime()
      if (!Number.isNaN(ts)) {
        filter.timeEnd = ts
      }
    }
    return filter
  }, [filterState])
  const eventFiltersActive =
    filterState.eventTypes.length > 0 || filterState.actor !== ''

  const {
    data: timelineData,
    isLoading: timelineLoading,
    error: timelineError,
    refetch: refetchTimeline,
  } = useTimeline(taskId, timelineFilter, {
    limit: TIMELINE_PAGE_SIZE,
    offset: 0,
    enabled: viewMode === 'timeline',
  })
  const iterations = useMemo(
    () => historyData?.iterations ?? [],
    [historyData?.iterations]
  )
  const checkpointData = historyData?.checkpoints

  useEffect(() => {
    if (!checkpointData) {
      return
    }
    setCheckpointItems(checkpointData.checkpoints ?? [])
    setCheckpointTotal(checkpointData.total ?? 0)
    setLoadMoreError(null)
  }, [checkpointData])

  const timelineFilterKey = useMemo(
    () => JSON.stringify(timelineFilter),
    [timelineFilter]
  )

  useEffect(() => {
    if (!timelineData) {
      return
    }
    setTimelineEntries(timelineData.entries ?? [])
    setTimelineTotal(timelineData.total ?? 0)
    setTimelineLoadError(null)
  }, [timelineData])

  useEffect(() => {
    setTimelineEntries([])
    setTimelineTotal(0)
    setTimelineLoadError(null)
    if (viewMode === 'timeline') {
      refetchTimeline()
    }
  }, [timelineFilterKey, taskId, viewMode, refetchTimeline])

  const prunedTimelineEntries = useMemo(() => {
    if (timelineEntries.length === 0) {
      return timelineEntries
    }
    const iterationSet = new Set<number>()
    const checkpointIds = new Set<string>()
    for (const entry of timelineEntries) {
      if (entry.entryType === 'iteration' && entry.iteration !== null) {
        iterationSet.add(entry.iteration)
      }
      if (entry.entryType === 'checkpoint') {
        checkpointIds.add(entry.id)
      }
    }

    const getCheckpointId = (details: TimelineEntry['details']) => {
      if (!details || typeof details !== 'object' || Array.isArray(details)) {
        return null
      }
      const raw = (details as Record<string, unknown>).checkpoint_id
      return typeof raw === 'string' ? raw : null
    }

    return timelineEntries.filter((entry) => {
      if (entry.entryType !== 'event') {
        return true
      }
      if (
        (entry.title === 'iteration_started' || entry.title === 'iteration_completed') &&
        entry.iteration !== null &&
        iterationSet.has(entry.iteration)
      ) {
        return false
      }
      if (entry.title === 'checkpoint_saved') {
        const checkpointId = getCheckpointId(entry.details)
        if (checkpointId && checkpointIds.has(checkpointId)) {
          return false
        }
      }
      return true
    })
  }, [timelineEntries])

  // 切换展开状态
  const handleToggle = (iterationId: string) => {
    setExpandedIterationId((prev) => (prev === iterationId ? null : iterationId))
  }

  const handleRefresh = () => {
    refetch()
    if (viewMode === 'timeline') {
      refetchTimeline()
    }
  }

  const filteredIterations = useMemo(() => {
    return iterations.filter((iteration) => {
      if (
        timelineFilter.iterationMin !== undefined &&
        iteration.round < timelineFilter.iterationMin
      ) {
        return false
      }
      if (
        timelineFilter.iterationMax !== undefined &&
        iteration.round > timelineFilter.iterationMax
      ) {
        return false
      }
      const startedAtMs = Date.parse(iteration.startedAt)
      if (
        timelineFilter.timeStart !== undefined &&
        !Number.isNaN(startedAtMs) &&
        startedAtMs < timelineFilter.timeStart
      ) {
        return false
      }
      if (
        timelineFilter.timeEnd !== undefined &&
        !Number.isNaN(startedAtMs) &&
        startedAtMs > timelineFilter.timeEnd
      ) {
        return false
      }
      return true
    })
  }, [iterations, timelineFilter])

  const filteredCheckpoints = useMemo(() => {
    return checkpointItems.filter((checkpoint) => {
      if (
        timelineFilter.iterationMin !== undefined &&
        checkpoint.iteration < timelineFilter.iterationMin
      ) {
        return false
      }
      if (
        timelineFilter.iterationMax !== undefined &&
        checkpoint.iteration > timelineFilter.iterationMax
      ) {
        return false
      }
      const createdAtMs = Date.parse(checkpoint.createdAt)
      if (
        timelineFilter.timeStart !== undefined &&
        !Number.isNaN(createdAtMs) &&
        createdAtMs < timelineFilter.timeStart
      ) {
        return false
      }
      if (
        timelineFilter.timeEnd !== undefined &&
        !Number.isNaN(createdAtMs) &&
        createdAtMs > timelineFilter.timeEnd
      ) {
        return false
      }
      return true
    })
  }, [checkpointItems, timelineFilter])

  const checkpointCount = filteredCheckpoints.length
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

  const handleLoadMoreTimeline = async () => {
    if (timelineLoadingMore || timelineEntries.length >= timelineTotal) {
      return
    }
    setTimelineLoadingMore(true)
    setTimelineLoadError(null)
    try {
      const nextPage = await getTimeline(taskId, timelineFilter, {
        limit: TIMELINE_PAGE_SIZE,
        offset: timelineEntries.length,
      })
      const nextEntries = nextPage.entries ?? []
      setTimelineEntries((prev) => [...prev, ...nextEntries])
      setTimelineTotal(nextPage.total ?? timelineTotal)
    } catch (err) {
      const message =
        err instanceof Error ? err.message : '加载时间线失败，请稍后再试'
      setTimelineLoadError(message)
    } finally {
      setTimelineLoadingMore(false)
    }
  }

  const handleExport = async () => {
    setExportError(null)
    try {
      const { blob, filename } = await exportMutation.mutateAsync({ taskId })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = filename
      link.click()
      URL.revokeObjectURL(url)
    } catch (err) {
      const message =
        err instanceof Error ? err.message : '导出失败，请稍后再试'
      setExportError(message)
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
          checkpoints={filteredCheckpoints}
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

  const listContent = () => {
    if (isLoading) {
      return (
        <div className="flex items-center justify-center py-8">
          <RefreshCw className="h-5 w-5 animate-spin text-muted-foreground" />
          <span className="ml-2 text-sm text-muted-foreground">加载中...</span>
        </div>
      )
    }
    if (error) {
      return (
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
      )
    }
    if (!iterations || iterations.length === 0) {
      return (
        <div className="text-center py-8">
          <p className="text-sm text-muted-foreground mb-4">暂无历史记录</p>
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
      )
    }
    if (filteredIterations.length === 0) {
      return (
        <div className="text-center py-8">
          <p className="text-sm text-muted-foreground mb-2">未找到匹配记录</p>
          <p className="text-xs text-muted-foreground">请调整筛选条件</p>
        </div>
      )
    }
    return (
      <div className="space-y-2 max-h-[500px] overflow-y-auto">
        {filteredIterations.map((iteration) => (
          <IterationHistoryItem
            key={iteration.id}
            taskId={taskId}
            summary={iteration}
            isExpanded={expandedIterationId === iteration.id}
            onToggle={() => handleToggle(iteration.id)}
          />
        ))}
      </div>
    )
  }

  const timelineContent = () => {
    if (timelineLoading) {
      return (
        <div className="flex items-center justify-center py-8">
          <RefreshCw className="h-5 w-5 animate-spin text-muted-foreground" />
          <span className="ml-2 text-sm text-muted-foreground">加载中...</span>
        </div>
      )
    }
    if (timelineError) {
      return (
        <div className="text-center py-8">
          <p className="text-sm text-destructive mb-4">
            加载失败：{timelineError.message}
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
      )
    }
    if (prunedTimelineEntries.length === 0) {
      return (
        <div className="text-center py-8">
          <p className="text-sm text-muted-foreground mb-2">暂无时间线记录</p>
          <p className="text-xs text-muted-foreground">请尝试调整筛选条件</p>
        </div>
      )
    }
    return (
      <>
        <TimelineView entries={prunedTimelineEntries} />
        {timelineLoadError ? (
          <p className="text-xs text-destructive mt-2">{timelineLoadError}</p>
        ) : null}
        {timelineEntries.length < timelineTotal ? (
          <div className="mt-3 flex items-center justify-center">
            <Button
              variant="outline"
              size="sm"
              onClick={handleLoadMoreTimeline}
              disabled={timelineLoadingMore}
              className="min-w-[44px] min-h-[44px]"
            >
              {timelineLoadingMore ? '加载中...' : '加载更多'}
            </Button>
          </div>
        ) : null}
      </>
    )
  }

  return (
    <>
      <Card className="w-full">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg flex items-center gap-2">
              <History className="h-5 w-5" />
              历史记录
            </CardTitle>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleExport}
                disabled={exportMutation.isPending}
                className="min-w-[44px] min-h-[44px]"
                title="导出"
              >
                <Download className="h-4 w-4 mr-1" />
                导出
              </Button>
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
          </div>
          <div className="flex flex-col gap-1 mt-2">
            <p className="text-sm text-muted-foreground">
              共 {filteredIterations.length} 轮迭代
            </p>
            {exportError ? (
              <p className="text-xs text-destructive">{exportError}</p>
            ) : null}
          </div>
        </CardHeader>
        <CardContent>
          <HistoryFilter
            value={filterState}
            onChange={setFilterState}
            disableEventFilters={viewMode === 'list'}
            disableActorFilter={viewMode === 'list'}
          />
          {viewMode === 'list' && eventFiltersActive ? (
            <p className="text-xs text-muted-foreground mt-2">
              事件/操作者筛选仅在时间线视图生效
            </p>
          ) : null}
          <Tabs value={viewMode} onValueChange={(value) => setViewMode(value as 'list' | 'timeline')} className="mt-4">
            <TabsList className="grid grid-cols-2 w-full">
              <TabsTrigger value="list">列表视图</TabsTrigger>
              <TabsTrigger value="timeline">时间线</TabsTrigger>
            </TabsList>
            <TabsContent value="list">
              {listContent()}
              {checkpointSection}
            </TabsContent>
            <TabsContent value="timeline">{timelineContent()}</TabsContent>
          </Tabs>
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
