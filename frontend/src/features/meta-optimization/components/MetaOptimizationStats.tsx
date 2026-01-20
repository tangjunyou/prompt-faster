import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { TeacherPromptStats } from '@/types/generated/models/TeacherPromptStats'

export interface MetaOptimizationStatsProps {
  stats: TeacherPromptStats[]
}

function formatRate(rate: number | null | undefined) {
  if (rate === null || rate === undefined) return '—'
  return `${(rate * 100).toFixed(1)}%`
}

export function MetaOptimizationStats({ stats }: MetaOptimizationStatsProps) {
  if (stats.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>成功率对比</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-muted-foreground">暂无统计数据</div>
        </CardContent>
      </Card>
    )
  }

  const maxRate = Math.max(
    1,
    ...stats.map((item) => (item.successRate ?? 0) * 100)
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>成功率对比</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {stats.map((item) => {
          const percent = (item.successRate ?? 0) * 100
          const width = Math.round((percent / maxRate) * 100)

          return (
            <div key={item.versionId} className="space-y-1">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>v{item.version}</span>
                <span>{formatRate(item.successRate)}</span>
              </div>
              <div className="h-2 w-full rounded-full bg-muted">
                <div
                  className="h-2 rounded-full bg-primary transition-all"
                  style={{ width: `${width}%` }}
                />
              </div>
            </div>
          )}
        )}
      </CardContent>
    </Card>
  )
}
