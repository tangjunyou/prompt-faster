/**
 * 多样性分析卡片
 */

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import type { DiversityAnalysisResult } from '@/types/generated/models/DiversityAnalysisResult'
import type { BaselineComparison } from '@/types/generated/models/BaselineComparison'
import { DiversityMetricsChart } from './DiversityMetricsChart'
import { DiversityWarningBanner } from './DiversityWarningBanner'
import { formatAffectedMetrics, pickHighestWarning } from '../utils/diversityWarning'

export interface DiversityAnalysisCardProps {
  analysis: DiversityAnalysisResult | null
  isLoading?: boolean
  error?: Error | null
  onRetry?: () => void
}

function formatPercent(value?: number | null) {
  if (value === null || value === undefined) return '—'
  return `${(value * 100).toFixed(1)}%`
}

function formatDiff(value?: number | null) {
  if (value === null || value === undefined) return '—'
  const sign = value >= 0 ? '+' : ''
  return `${sign}${(value * 100).toFixed(1)}%`
}

function trendLabel(trend?: BaselineComparison['trend'] | null) {
  switch (trend) {
    case 'improved':
      return '整体提升'
    case 'declined':
      return '整体下降'
    case 'stable':
      return '整体稳定'
    default:
      return '暂无趋势'
  }
}

export function DiversityAnalysisCard({
  analysis,
  isLoading = false,
  error,
  onRetry,
}: DiversityAnalysisCardProps) {
  const primaryWarning = analysis ? pickHighestWarning(analysis.warnings ?? []) : null
  const semanticUnavailable = Boolean(
    analysis?.warnings?.some((warning) => warning.message.includes('语义多样性暂不可用'))
  )

  return (
    <Card className="border-primary/30">
      <CardHeader className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <CardTitle className="text-lg">多样性分析</CardTitle>
          <p className="text-sm text-muted-foreground">创意任务输出的多维度多样性评估</p>
        </div>
        {analysis ? (
          <div className="text-xs text-muted-foreground">
            样本数：{analysis.sampleCount} · 分析时间：{analysis.analyzedAt}
          </div>
        ) : null}
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <div className="space-y-3" data-testid="diversity-loading">
            <Skeleton className="h-4 w-32" />
            <div className="grid gap-3 sm:grid-cols-4">
              {Array.from({ length: 4 }).map((_, idx) => (
                <Skeleton key={`diversity-metric-${idx}`} className="h-16 w-full" />
              ))}
            </div>
            <Skeleton className="h-52 w-full" />
          </div>
        ) : error ? (
          <div className="space-y-2 text-sm text-destructive">
            <div>多样性分析暂不可用</div>
            {onRetry ? (
              <Button type="button" variant="outline" size="sm" onClick={onRetry}>
                重试
              </Button>
            ) : null}
          </div>
        ) : analysis ? (
          <>
            <div className="grid gap-3 rounded-lg border bg-muted/20 p-3 text-sm sm:grid-cols-4">
              <div>
                <div className="text-xs text-muted-foreground">整体分数</div>
                <div className="text-base font-semibold">{formatPercent(analysis.metrics.overallScore)}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">词汇多样性</div>
                <div className="font-medium">{formatPercent(analysis.metrics.lexicalDiversity)}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">结构多样性</div>
                <div className="font-medium">{formatPercent(analysis.metrics.structuralDiversity)}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground">语义多样性</div>
                <div className="font-medium">{formatPercent(analysis.metrics.semanticDiversity)}</div>
                {semanticUnavailable ? (
                  <div className="mt-1 text-xs text-muted-foreground">未启用 embedding</div>
                ) : null}
              </div>
            </div>

            <DiversityMetricsChart metrics={analysis.metrics} />

            {primaryWarning ? <DiversityWarningBanner warning={primaryWarning} /> : null}

            <div className="grid gap-3 rounded-lg border bg-white p-3 text-sm">
              <div className="text-sm font-medium">基准线对比</div>
              {analysis.baselineComparison ? (
                <div className="grid gap-2 sm:grid-cols-2">
                  <div className="rounded-md border bg-muted/20 p-2">
                    <div className="text-xs text-muted-foreground">趋势</div>
                    <div className="font-medium">{trendLabel(analysis.baselineComparison.trend)}</div>
                  </div>
                  <div className="rounded-md border bg-muted/20 p-2">
                    <div className="text-xs text-muted-foreground">整体变化</div>
                    <div className="font-medium">{formatDiff(analysis.baselineComparison.overallDiff)}</div>
                  </div>
                  <div className="rounded-md border bg-muted/20 p-2">
                    <div className="text-xs text-muted-foreground">词汇变化</div>
                    <div className="font-medium">{formatDiff(analysis.baselineComparison.lexicalDiff)}</div>
                  </div>
                  <div className="rounded-md border bg-muted/20 p-2">
                    <div className="text-xs text-muted-foreground">结构变化</div>
                    <div className="font-medium">{formatDiff(analysis.baselineComparison.structuralDiff)}</div>
                  </div>
                  <div className="rounded-md border bg-muted/20 p-2">
                    <div className="text-xs text-muted-foreground">语义变化</div>
                    <div className="font-medium">{formatDiff(analysis.baselineComparison.semanticDiff)}</div>
                  </div>
                </div>
              ) : (
                <div className="text-sm text-muted-foreground">暂无基准线对比数据。</div>
              )}
            </div>

            {analysis.warnings?.length ? (
              <div className="space-y-2">
                <div className="text-sm font-medium">告警详情</div>
                <div className="space-y-2">
                  {analysis.warnings.map((warning, idx) => {
                    const affected = formatAffectedMetrics(warning.affectedMetrics)
                    return (
                      <div key={`${warning.level}-${idx}`} className="rounded-md border p-2 text-sm">
                        <div className="font-medium">{warning.message}</div>
                        {affected ? (
                          <div className="text-xs text-muted-foreground">
                            受影响指标：{affected}
                          </div>
                        ) : null}
                      </div>
                    )
                  })}
                </div>
              </div>
            ) : null}

            <div id="diversity-suggestions" className="space-y-2">
              <div className="text-sm font-medium">优化建议</div>
              {analysis.suggestions?.length ? (
                <ul className="space-y-2 text-sm">
                  {analysis.suggestions.map((suggestion, idx) => (
                    <li key={`${suggestion.suggestionType}-${idx}`} className="rounded-md border p-2">
                      <div className="font-medium">{suggestion.suggestionType}</div>
                      <div className="text-xs text-muted-foreground">{suggestion.content}</div>
                    </li>
                  ))}
                </ul>
              ) : (
                <div className="text-sm text-muted-foreground">暂无优化建议。</div>
              )}
            </div>
          </>
        ) : (
          <div className="text-sm text-muted-foreground">暂无多样性分析数据。</div>
        )}
      </CardContent>
    </Card>
  )
}

export default DiversityAnalysisCard
