/**
 * 历史迭代列表项组件
 * 显示单个迭代的摘要信息，支持展开查看详情
 */

import { ChevronDown, ChevronRight, CheckCircle2, XCircle, Clock, AlertCircle } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { IterationHistorySummary } from '@/types/generated/models/IterationHistorySummary'
import { HistoryDetailView } from './HistoryDetailView'
import { useIterationDetail } from './hooks/useIterationHistory'

export interface IterationHistoryItemProps {
  /** 任务 ID */
  taskId: string
  /** 迭代摘要 */
  summary: IterationHistorySummary
  /** 是否展开 */
  isExpanded: boolean
  /** 切换展开状态回调 */
  onToggle: () => void
}

/**
 * 格式化通过率为百分比字符串
 */
function formatPassRate(passRate: number): string {
  return `${(passRate * 100).toFixed(1)}%`
}

/**
 * 格式化时间戳为可读格式
 */
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
 * 获取状态图标
 */
function getStatusIcon(status: string) {
  switch (status) {
    case 'completed':
      return <CheckCircle2 className="h-4 w-4 text-green-500" />
    case 'failed':
      return <XCircle className="h-4 w-4 text-red-500" />
    case 'running':
      return <Clock className="h-4 w-4 text-blue-500 animate-pulse" />
    case 'terminated':
      return <AlertCircle className="h-4 w-4 text-yellow-500" />
    default:
      return <Clock className="h-4 w-4 text-muted-foreground" />
  }
}

/**
 * 获取状态文本
 */
function getStatusText(status: string): string {
  switch (status) {
    case 'completed':
      return '已完成'
    case 'failed':
      return '失败'
    case 'running':
      return '进行中'
    case 'terminated':
      return '已终止'
    default:
      return status
  }
}

/**
 * 历史迭代列表项
 */
export function IterationHistoryItem({
  taskId,
  summary,
  isExpanded,
  onToggle,
}: IterationHistoryItemProps) {
  // 仅在展开时获取详情
  const { data: detail, isLoading: isLoadingDetail } = useIterationDetail(
    taskId,
    isExpanded ? summary.id : null
  )

  return (
    <div className="border rounded-lg overflow-hidden">
      {/* 摘要行 - 可点击展开 */}
      <Button
        variant="ghost"
        className="w-full h-auto p-3 justify-start hover:bg-muted/50"
        onClick={onToggle}
      >
        <div className="flex items-center gap-3 w-full">
          {/* 展开/折叠图标 */}
          <div className="shrink-0">
            {isExpanded ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
          </div>

          {/* 轮次编号 */}
          <div className="shrink-0">
            <Badge variant="outline" className="font-mono">
              #{summary.round}
            </Badge>
          </div>

          {/* 状态图标 */}
          <div className="shrink-0">{getStatusIcon(summary.status)}</div>

          {/* 通过率 */}
          <div className="flex items-center gap-1 shrink-0">
            <span className="text-sm font-medium">
              {formatPassRate(summary.passRate)}
            </span>
            <span className="text-xs text-muted-foreground">
              ({summary.passedCases}/{summary.totalCases})
            </span>
          </div>

          {/* 时间戳 */}
          <div className="text-xs text-muted-foreground ml-auto shrink-0">
            {formatTime(summary.startedAt)}
          </div>

          {/* 状态文本 */}
          <Badge
            variant={summary.status === 'completed' ? 'default' : 'secondary'}
            className="shrink-0"
          >
            {getStatusText(summary.status)}
          </Badge>
        </div>
      </Button>

      {/* 展开的详情内容 */}
      {isExpanded && (
        <div className="border-t bg-muted/20 p-4">
          {isLoadingDetail ? (
            <div className="flex items-center justify-center py-4">
              <Clock className="h-4 w-4 animate-spin text-muted-foreground" />
              <span className="ml-2 text-sm text-muted-foreground">
                加载详情...
              </span>
            </div>
          ) : detail ? (
            <HistoryDetailView detail={detail} />
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              无法加载详情
            </p>
          )}
        </div>
      )}
    </div>
  )
}

export default IterationHistoryItem
