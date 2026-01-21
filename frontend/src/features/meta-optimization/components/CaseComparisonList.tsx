import { useMemo, useState } from 'react'
import { Badge } from '@/components/ui/badge'
import type { CaseComparisonResult } from '@/types/generated/models/CaseComparisonResult'

export interface CaseComparisonListProps {
  comparisons: CaseComparisonResult[]
}

export function CaseComparisonList({ comparisons }: CaseComparisonListProps) {
  const [showOnlyDiff, setShowOnlyDiff] = useState(false)

  const sortedComparisons = useMemo(() => {
    return [...comparisons].sort((a, b) => {
      if (a.isDifferent && !b.isDifferent) return -1
      if (!a.isDifferent && b.isDifferent) return 1
      return 0
    })
  }, [comparisons])

  const filteredComparisons = showOnlyDiff
    ? sortedComparisons.filter((item) => item.isDifferent)
    : sortedComparisons

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">用例对比详情</h3>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={showOnlyDiff}
            onChange={(event) => setShowOnlyDiff(event.target.checked)}
          />
          只看差异
        </label>
      </div>

      {filteredComparisons.length === 0 ? (
        <div className="rounded-md border border-dashed p-4 text-sm text-muted-foreground">
          暂无可展示的差异用例。
        </div>
      ) : (
        filteredComparisons.map((comparison) => (
          <CaseComparisonCard key={comparison.testCaseId} comparison={comparison} />
        ))
      )}
    </div>
  )
}

function CaseComparisonCard({ comparison }: { comparison: CaseComparisonResult }) {
  const isImproved = !comparison.versionAPassed && comparison.versionBPassed
  const isRegressed = comparison.versionAPassed && !comparison.versionBPassed
  const isOutputDiff = comparison.isDifferent && !isImproved && !isRegressed

  const highlightClass = isImproved
    ? 'border-green-300 bg-green-50'
    : isRegressed
    ? 'border-red-300 bg-red-50'
    : isOutputDiff
    ? 'border-amber-300 bg-amber-50'
    : 'border-border'

  return (
    <div className={`rounded-lg border p-4 ${highlightClass}`}>
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
        <span>用例 ID: {comparison.testCaseId}</span>
        <div className="flex flex-wrap items-center gap-2">
          {isImproved && <Badge variant="outline">↑ 改进</Badge>}
          {isRegressed && <Badge variant="outline">↓ 退化</Badge>}
          {isOutputDiff && <Badge variant="outline">~ 输出差异</Badge>}
          {!comparison.isDifferent && <Badge variant="outline">无差异</Badge>}
        </div>
      </div>

      <div className="mb-3 text-xs">
        <div className="text-muted-foreground">输入</div>
        <pre className="mt-1 max-h-32 overflow-auto rounded bg-muted/40 p-2">
          {JSON.stringify(comparison.input, null, 2)}
        </pre>
      </div>

      <div className="mb-3 text-xs">
        <div className="text-muted-foreground">参考答案</div>
        <pre className="mt-1 max-h-32 overflow-auto rounded bg-muted/40 p-2">
          {JSON.stringify(comparison.reference, null, 2)}
        </pre>
      </div>

      <div className="grid gap-4 text-xs md:grid-cols-2">
        <div>
          <div className="flex items-center gap-2">
            <span className="font-medium">版本 A 输出</span>
            <span className={comparison.versionAPassed ? 'text-green-600' : 'text-red-600'}>
              {comparison.versionAPassed ? '✓ 通过' : '✗ 失败'}
            </span>
          </div>
          <pre className="mt-1 max-h-40 overflow-auto rounded bg-muted/40 p-2">
            {comparison.versionAOutput}
          </pre>
        </div>
        <div>
          <div className="flex items-center gap-2">
            <span className="font-medium">版本 B 输出</span>
            <span className={comparison.versionBPassed ? 'text-green-600' : 'text-red-600'}>
              {comparison.versionBPassed ? '✓ 通过' : '✗ 失败'}
            </span>
          </div>
          <pre className="mt-1 max-h-40 overflow-auto rounded bg-muted/40 p-2">
            {comparison.versionBOutput}
          </pre>
        </div>
      </div>

      {comparison.differenceNote && (
        <div className="mt-2 text-xs text-muted-foreground">
          💡 {comparison.differenceNote}
        </div>
      )}
    </div>
  )
}
