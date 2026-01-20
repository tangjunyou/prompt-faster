import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { TeacherPrompt } from '@/types/generated/models/TeacherPrompt'
import type { TeacherPromptStats } from '@/types/generated/models/TeacherPromptStats'

export interface PromptVersionDetailProps {
  prompt: TeacherPrompt | null
  stats?: TeacherPromptStats | null
  isLoading?: boolean
  error?: Error | null
}

function formatRate(rate: number | null | undefined) {
  if (rate === null || rate === undefined) return '—'
  return `${(rate * 100).toFixed(1)}%`
}

function formatDate(value?: string | null) {
  if (!value) return '—'
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function PromptVersionDetail({ prompt, stats, isLoading, error }: PromptVersionDetailProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>版本详情</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <div className="text-sm text-muted-foreground">加载版本详情中...</div>
        ) : error ? (
          <div className="text-sm text-destructive">加载失败：{error.message}</div>
        ) : !prompt ? (
          <div className="text-sm text-muted-foreground">请选择一个版本查看详情。</div>
        ) : (
          <>
            <div className="space-y-1">
              <div className="text-sm text-muted-foreground">版本号</div>
              <div className="text-base font-semibold">v{prompt.version}</div>
            </div>

            <div className="grid gap-3 text-sm sm:grid-cols-2">
              <div>
                <div className="text-muted-foreground">创建时间</div>
                <div className="text-foreground">{formatDate(prompt.createdAt)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">更新时间</div>
                <div className="text-foreground">{formatDate(prompt.updatedAt)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">成功率</div>
                <div className="text-foreground">{formatRate(stats?.successRate)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">平均通过率</div>
                <div className="text-foreground">{formatRate(stats?.averagePassRate)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">任务数</div>
                <div className="text-foreground">{stats?.totalTasks ?? 0}</div>
              </div>
              <div>
                <div className="text-muted-foreground">成功任务</div>
                <div className="text-foreground">{stats?.successfulTasks ?? 0}</div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">成功率图表</div>
              {stats?.successRate === null || stats?.successRate === undefined ? (
                <div className="text-sm text-muted-foreground">暂无统计数据</div>
              ) : (
                <div className="h-2 w-full rounded-full bg-muted">
                  <div
                    className="h-2 rounded-full bg-primary"
                    style={{ width: `${Math.round(stats.successRate * 100)}%` }}
                  />
                </div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">Prompt 内容</div>
              <pre className="whitespace-pre-wrap rounded-lg border bg-muted/20 p-3 text-xs text-muted-foreground">
                {prompt.content}
              </pre>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
