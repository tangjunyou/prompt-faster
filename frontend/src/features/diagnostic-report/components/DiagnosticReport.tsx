/**
 * 诊断报告主组件
 */

import { useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { FailedCaseDetail } from '@/types/generated/models/FailedCaseDetail'
import { useDiagnostic } from '../hooks/useDiagnostic'
import { useFailedCaseDetail } from '../hooks/useFailedCaseDetail'
import { TurningPointTimeline } from './TurningPointTimeline'
import { FailedCaseList } from './FailedCaseList'
import { CaseComparisonDialog } from './CaseComparisonDialog'

export interface DiagnosticReportProps {
  taskId: string
  enabled?: boolean
  staleTime?: number
  failedCasesLimit?: number
}

function formatCount(value?: number | null) {
  if (value === null || value === undefined) return '—'
  return value
}

export function DiagnosticReport({
  taskId,
  enabled = true,
  staleTime,
  failedCasesLimit,
}: DiagnosticReportProps) {
  const [selectedCaseId, setSelectedCaseId] = useState<string | null>(null)
  const { data, isLoading, error, refetch } = useDiagnostic(taskId, {
    enabled,
    staleTime,
    failedCasesLimit,
  })

  const caseDetailQuery = useFailedCaseDetail(taskId, selectedCaseId, {
    enabled: !!selectedCaseId,
  })

  const selectedCase: FailedCaseDetail | null = caseDetailQuery.data ?? null

  const failureReasons = useMemo(() => {
    if (!data?.summary.commonFailureReasons?.length) return []
    return data.summary.commonFailureReasons
  }, [data?.summary.commonFailureReasons])

  return (
    <Card className="border-primary/40">
      <CardHeader className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <CardTitle className="text-lg">诊断报告</CardTitle>
          <p className="text-sm text-muted-foreground">
            {data?.taskName ? `任务：${data.taskName}` : '诊断报告概览'}
          </p>
        </div>
        <Badge variant="outline">{data?.status ?? 'unknown'}</Badge>
      </CardHeader>
      <CardContent className="space-y-6">
        {isLoading ? (
          <div className="text-sm text-muted-foreground">正在加载诊断报告...</div>
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
                <div className="text-xs text-muted-foreground">迭代总数</div>
                <div className="font-medium">{formatCount(data.summary.totalIterations)}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">失败迭代</div>
                <div className="font-medium">{formatCount(data.summary.failedIterations)}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">成功迭代</div>
                <div className="font-medium">{formatCount(data.summary.successIterations)}</div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">失败原因摘要</div>
              {failureReasons.length === 0 ? (
                <div className="text-sm text-muted-foreground">暂无失败原因统计</div>
              ) : (
                <div className="space-y-2">
                  {failureReasons.map((reason) => (
                    <div
                      key={reason.reason}
                      className="flex items-center justify-between rounded-lg border p-2 text-sm"
                    >
                      <span>{reason.reason}</span>
                      <span className="text-xs text-muted-foreground">
                        {reason.count} 次 · {reason.percentage.toFixed(1)}%
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">自然语言解释</div>
              <div className="rounded-lg border bg-muted/20 p-3 text-sm text-muted-foreground">
                {data.summary.naturalLanguageExplanation}
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">关键转折点</div>
              <TurningPointTimeline turningPoints={data.turningPoints} />
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">改进建议</div>
              {data.improvementSuggestions.length === 0 ? (
                <div className="text-sm text-muted-foreground">暂无建议</div>
              ) : (
                <div className="space-y-2">
                  {data.improvementSuggestions.map((item) => (
                    <div key={item} className="rounded-lg border p-2 text-sm text-muted-foreground">
                      {item}
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">失败用例列表</div>
              <FailedCaseList
                cases={data.failedCases}
                onCaseClick={(caseId) => setSelectedCaseId(caseId)}
              />
            </div>
          </>
        ) : null}
      </CardContent>

      <CaseComparisonDialog
        open={!!selectedCaseId}
        caseDetail={selectedCase}
        isLoading={caseDetailQuery.isLoading}
        error={caseDetailQuery.error ?? null}
        onClose={() => setSelectedCaseId(null)}
      />
    </Card>
  )
}
