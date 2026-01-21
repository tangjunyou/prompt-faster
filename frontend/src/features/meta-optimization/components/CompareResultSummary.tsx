import type { CompareSummary } from '@/types/generated/models/CompareSummary'
import type { VersionCompareResult } from '@/types/generated/models/VersionCompareResult'

export interface CompareResultSummaryProps {
  summary: CompareSummary
  versionA: VersionCompareResult
  versionB: VersionCompareResult
}

function formatRate(rate: number) {
  return `${(rate * 100).toFixed(1)}%`
}

export function CompareResultSummary({ summary, versionA, versionB }: CompareResultSummaryProps) {
  const diff = summary.passRateDiff
  const diffLabel = `${diff >= 0 ? '+' : ''}${(diff * 100).toFixed(1)}%`
  const diffClass = diff > 0 ? 'text-green-600' : diff < 0 ? 'text-red-600' : 'text-muted-foreground'

  return (
    <div className="rounded-lg border p-4 text-sm">
      <div className="flex flex-wrap items-center gap-4">
        <div>
          <div className="text-muted-foreground">版本 A 通过率</div>
          <div className="font-medium">{formatRate(versionA.passRate)}</div>
        </div>
        <div>
          <div className="text-muted-foreground">版本 B 通过率</div>
          <div className="font-medium">{formatRate(versionB.passRate)}</div>
        </div>
        <div>
          <div className="text-muted-foreground">通过率差异</div>
          <div className={`font-medium ${diffClass}`}>{diffLabel}</div>
        </div>
        <div>
          <div className="text-muted-foreground">改进/退化/输出差异/无变化</div>
          <div className="font-medium">
            {summary.improvedCases}/{summary.regressedCases}/{summary.outputDiffCases}/
            {summary.unchangedCases}
          </div>
        </div>
        <div>
          <div className="text-muted-foreground">总耗时</div>
          <div className="font-medium">{summary.totalExecutionTimeMs} ms</div>
        </div>
      </div>
    </div>
  )
}
