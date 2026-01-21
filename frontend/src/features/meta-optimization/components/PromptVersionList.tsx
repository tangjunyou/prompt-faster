import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { TeacherPromptVersion } from '@/types/generated/models/TeacherPromptVersion'

export interface PromptVersionListProps {
  versions: TeacherPromptVersion[]
  selectedId?: string | null
  onSelect?: (id: string) => void
  onActivate?: (id: string) => void
  isActivating?: boolean
  onCompare?: () => void
}

function formatRate(rate: number | null | undefined) {
  if (rate === null || rate === undefined) return '—'
  return `${(rate * 100).toFixed(1)}%`
}

function formatDate(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function PromptVersionList({
  versions,
  selectedId,
  onSelect,
  onActivate,
  isActivating = false,
  onCompare,
}: PromptVersionListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between gap-2">
        <CardTitle>版本列表</CardTitle>
        <Button type="button" size="sm" variant="outline" onClick={onCompare} disabled={!onCompare}>
          版本对比
        </Button>
      </CardHeader>
      <CardContent className="space-y-3">
        {versions.length === 0 ? (
          <div className="text-sm text-muted-foreground">暂无版本，请先创建一个 Prompt 版本。</div>
        ) : (
          versions.map((version) => {
            const isActive = version.isActive
            const isSelected = selectedId === version.id

            return (
              <div
                key={version.id}
                className={`rounded-lg border p-3 transition ${isActive ? 'border-primary/60 bg-primary/5' : 'border-border'} ${
                  isSelected ? 'ring-1 ring-primary/40' : ''
                }`}
                role="button"
                tabIndex={0}
                onClick={() => onSelect?.(version.id)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' || event.key === ' ') {
                    event.preventDefault()
                    onSelect?.(version.id)
                  }
                }}
              >
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-semibold">v{version.version}</span>
                      {isActive && <Badge variant="outline">当前使用</Badge>}
                    </div>
                    <div className="mt-1 text-sm text-muted-foreground">
                      {version.description ?? '无描述'}
                    </div>
                  </div>
                  <Button
                    type="button"
                    variant={isActive ? 'secondary' : 'outline'}
                    size="sm"
                    disabled={isActive || isActivating}
                    onClick={(event) => {
                      event.stopPropagation()
                      onActivate?.(version.id)
                    }}
                  >
                    {isActive ? '已激活' : '设为活跃'}
                  </Button>
                </div>

                <div className="mt-3 grid gap-2 text-xs text-muted-foreground sm:grid-cols-3">
                  <div>
                    成功率：<span className="text-foreground">{formatRate(version.successRate)}</span>
                  </div>
                  <div>
                    任务数：<span className="text-foreground">{version.taskCount}</span>
                  </div>
                  <div>
                    创建时间：<span className="text-foreground">{formatDate(version.createdAt)}</span>
                  </div>
                </div>
              </div>
            )
          })
        )}
      </CardContent>
    </Card>
  )
}
