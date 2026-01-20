/**
 * 失败用例列表
 */

import { Button } from '@/components/ui/button'
import type { FailedCaseSummary } from '@/types/generated/models/FailedCaseSummary'

export interface FailedCaseListProps {
  cases: FailedCaseSummary[]
  onCaseClick: (caseId: string) => void
}

export function FailedCaseList({ cases, onCaseClick }: FailedCaseListProps) {
  if (cases.length === 0) {
    return (
      <div className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
        恭喜！所有用例都通过了
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {cases.map((item) => (
        <div
          key={item.caseId}
          className="flex flex-col gap-2 rounded-lg border p-3 sm:flex-row sm:items-center sm:justify-between"
        >
          <div>
            <div className="text-sm font-medium">第 {item.iterationRound} 轮失败用例</div>
            <div className="mt-1 text-xs text-muted-foreground">{item.failureReason}</div>
            <div className="mt-1 text-xs text-muted-foreground">{item.inputPreview}</div>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => onCaseClick(item.caseId)}
          >
            查看对比
          </Button>
        </div>
      ))}
    </div>
  )
}
