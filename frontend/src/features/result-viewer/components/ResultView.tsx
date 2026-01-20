/**
 * 结果查看组件
 */

import { lazy, Suspense, useEffect, useMemo, useState } from 'react'
import { Copy, Download } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { TaskResultView } from '@/types/generated/models/TaskResultView'
import { useResult } from '../hooks/useResult'
import { ExportDialog } from './ExportDialog'

const MonacoEditor = lazy(async () => import('@monaco-editor/react'))

export interface ResultViewProps {
  taskId: string
  enabled?: boolean
  staleTime?: number
}

const statusLabels: Record<string, { label: string; variant: 'default' | 'secondary' | 'outline' }> = {
  running: { label: '进行中', variant: 'secondary' },
  paused: { label: '已暂停', variant: 'secondary' },
  completed: { label: '已完成', variant: 'default' },
  terminated: { label: '已终止', variant: 'outline' },
  draft: { label: '草稿', variant: 'outline' },
}

function formatPassRate(rate?: number | null) {
  if (rate === null || rate === undefined) return '—'
  return `${(rate * 100).toFixed(1)}%`
}

function statusDisplay(status: string) {
  return statusLabels[status] ?? { label: status, variant: 'secondary' }
}

function renderPromptFallback() {
  return <div className="p-4 text-sm text-muted-foreground">暂无可用最佳 Prompt</div>
}

function renderIterationSummary(result: TaskResultView) {
  if (result.iterationSummary.length === 0) {
    return <div className="text-sm text-muted-foreground">暂无已完成迭代</div>
  }
  return (
    <div className="overflow-hidden rounded-md border">
      <table className="w-full text-sm">
        <thead className="bg-muted/60 text-muted-foreground">
          <tr>
            <th className="px-3 py-2 text-left font-medium">轮次</th>
            <th className="px-3 py-2 text-left font-medium">通过率</th>
            <th className="px-3 py-2 text-left font-medium">状态</th>
          </tr>
        </thead>
        <tbody>
          {result.iterationSummary.map((entry) => (
            <tr key={entry.round} className="border-t">
              <td className="px-3 py-2">第 {entry.round} 轮</td>
              <td className="px-3 py-2">{formatPassRate(entry.passRate)}</td>
              <td className="px-3 py-2 text-muted-foreground">{statusDisplay(entry.status).label}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

export function ResultView({ taskId, enabled = true, staleTime }: ResultViewProps) {
  const [copyStatus, setCopyStatus] = useState<'success' | 'error' | null>(null)
  const [isExportOpen, setIsExportOpen] = useState(false)
  const { data, isLoading, error, refetch } = useResult(taskId, { enabled, staleTime })

  const statusInfo = useMemo(() => {
    const status = data?.status ?? 'running'
    return statusDisplay(status)
  }, [data?.status])

  useEffect(() => {
    if (!copyStatus) return
    const timer = window.setTimeout(() => setCopyStatus(null), 2000)
    return () => window.clearTimeout(timer)
  }, [copyStatus])

  const handleCopy = async () => {
    if (!data?.bestPrompt) {
      setCopyStatus('error')
      return
    }
    try {
      await navigator.clipboard.writeText(data.bestPrompt)
      setCopyStatus('success')
    } catch {
      setCopyStatus('error')
    }
  }

  const editorFallback = (
    <div className="p-4 text-sm text-muted-foreground">正在加载预览...</div>
  )

  return (
    <Card className="border-primary/40">
      <CardHeader className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <CardTitle className="text-lg">优化结果</CardTitle>
          <p className="text-sm text-muted-foreground">
            {data?.taskName ? `任务：${data.taskName}` : '任务结果查看'}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant={statusInfo.variant}>{statusInfo.label}</Badge>
          <Button type="button" variant="outline" size="sm" onClick={() => setIsExportOpen(true)}>
            <Download className="mr-1 h-4 w-4" />
            导出
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <div className="text-sm text-muted-foreground">正在加载结果...</div>
        ) : error ? (
          <div className="space-y-2 text-sm text-destructive">
            <div>加载失败：{error.message}</div>
            <Button type="button" variant="outline" size="sm" onClick={() => refetch()}>
              重试
            </Button>
          </div>
        ) : data ? (
          <>
            <div className="grid gap-3 rounded-lg border bg-muted/30 p-3 text-sm sm:grid-cols-3">
              <div>
                <div className="text-xs text-muted-foreground">完成时间</div>
                <div className="font-medium">{data.completedAt ?? '—'}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">通过率</div>
                <div className="font-medium">{formatPassRate(data.passRate)}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">迭代轮次</div>
                <div className="font-medium">{data.totalIterations}</div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="text-sm font-medium">最佳 Prompt</div>
                <div className="flex items-center gap-2">
                  {copyStatus ? (
                    <span
                      className={`text-xs ${
                        copyStatus === 'success' ? 'text-green-600' : 'text-destructive'
                      }`}
                    >
                      {copyStatus === 'success' ? '已复制' : '复制失败'}
                    </span>
                  ) : null}
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={handleCopy}
                    disabled={!data.bestPrompt}
                    aria-label="复制 Prompt"
                  >
                    <Copy className="mr-1 h-4 w-4" />
                    复制
                  </Button>
                </div>
              </div>
              <div className="rounded-lg border">
                {data.bestPrompt ? (
                  <Suspense fallback={editorFallback}>
                    <MonacoEditor
                      height="240px"
                      language="markdown"
                      value={data.bestPrompt}
                      options={{
                        minimap: { enabled: false },
                        lineNumbers: 'off',
                        wordWrap: 'on',
                        fontSize: 14,
                        padding: { top: 12 },
                        readOnly: true,
                        domReadOnly: true,
                      }}
                      theme="vs-light"
                    />
                  </Suspense>
                ) : (
                  renderPromptFallback()
                )}
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">迭代摘要</div>
              {renderIterationSummary(data)}
            </div>
          </>
        ) : null}
      </CardContent>

      <ExportDialog taskId={taskId} open={isExportOpen} onOpenChange={setIsExportOpen} />
    </Card>
  )
}

export default ResultView
