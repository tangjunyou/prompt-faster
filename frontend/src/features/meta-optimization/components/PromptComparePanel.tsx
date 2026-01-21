import { useCallback, useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { useMetaOptimizationTasks } from '../hooks/useMetaOptimizationTasks'
import { usePromptVersions } from '../hooks/usePromptVersions'
import { usePromptCompare } from '../hooks/usePromptCompare'
import { CompareResultSummary } from './CompareResultSummary'
import { PromptDiffViewer } from './PromptDiffViewer'
import { CaseComparisonList } from './CaseComparisonList'
import type { TeacherPromptVersion } from '@/types/generated/models/TeacherPromptVersion'
import type { MetaOptimizationTaskSummary } from '@/types/generated/models/MetaOptimizationTaskSummary'

export function PromptComparePanel() {
  const { data: versions = [], isLoading: versionsLoading, error: versionsError } = usePromptVersions()
  const {
    data: tasks = [],
    isLoading: tasksLoading,
    error: tasksError,
  } = useMetaOptimizationTasks({ limit: 50 })

  const compareMutation = usePromptCompare()

  const [versionIdA, setVersionIdA] = useState('')
  const [versionIdB, setVersionIdB] = useState('')
  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([])

  const selectedTasks = useMemo(
    () => tasks.filter((task) => selectedTaskIds.includes(task.id)),
    [tasks, selectedTaskIds]
  )

  const workspaceId = useMemo(() => {
    if (selectedTasks.length === 0) return null
    const first = selectedTasks[0].workspaceId
    if (selectedTasks.some((task) => task.workspaceId !== first)) {
      return null
    }
    return first
  }, [selectedTasks])

  const workspaceError = selectedTasks.length > 0 && !workspaceId ? '请只选择同一工作区内的历史任务' : null

  const resetCompare = useCallback(() => {
    if (compareMutation.data || compareMutation.error) {
      compareMutation.reset()
    }
  }, [compareMutation.data, compareMutation.error, compareMutation.reset])

  useEffect(() => {
    resetCompare()
  }, [versionIdA, versionIdB, selectedTaskIds, resetCompare])

  const availableVersionsA = useMemo(
    () => versions.filter((version) => version.id !== versionIdB),
    [versions, versionIdB]
  )
  const availableVersionsB = useMemo(
    () => versions.filter((version) => version.id !== versionIdA),
    [versions, versionIdA]
  )

  const toggleTask = (taskId: string) => {
    setSelectedTaskIds((prev) =>
      prev.includes(taskId) ? prev.filter((id) => id !== taskId) : [...prev, taskId]
    )
  }

  const canCompare =
    !!versionIdA &&
    !!versionIdB &&
    versionIdA !== versionIdB &&
    selectedTaskIds.length > 0 &&
    !workspaceError &&
    !compareMutation.isPending

  const handleCompare = () => {
    if (!canCompare) return
    compareMutation.mutate({
      versionIdA,
      versionIdB,
      taskIds: selectedTaskIds,
      testCaseIds: [],
    })
  }

  const renderVersionOption = (version: TeacherPromptVersion) => (
    <option key={version.id} value={version.id}>
      v{version.version} {version.description ? `- ${version.description}` : ''}
    </option>
  )

  const renderTaskOption = (task: MetaOptimizationTaskSummary) => (
    <label key={task.id} className="flex items-start gap-2 rounded-lg border p-2 text-sm">
      <input
        type="checkbox"
        checked={selectedTaskIds.includes(task.id)}
        onChange={() => toggleTask(task.id)}
      />
      <div className="min-w-0 flex-1">
        <div className="flex items-center justify-between gap-2">
          <span className="font-medium">{task.name}</span>
          <Badge variant="outline">{task.status}</Badge>
        </div>
        <div className="mt-1 text-xs text-muted-foreground">
          Pass Rate：
          {task.passRate === null || task.passRate === undefined
            ? '—'
            : `${(task.passRate * 100).toFixed(1)}%`}
        </div>
      </div>
    </label>
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>版本对比</CardTitle>
      </CardHeader>
      <CardContent className="space-y-5 text-sm">
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="prompt-compare-version-a">版本 A（基准）</Label>
            <select
              id="prompt-compare-version-a"
              className="w-full rounded-md border px-3 py-2"
              value={versionIdA}
              onChange={(event) => setVersionIdA(event.target.value)}
              disabled={versionsLoading}
            >
              <option value="">请选择版本</option>
              {availableVersionsA.map(renderVersionOption)}
            </select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="prompt-compare-version-b">版本 B（对比）</Label>
            <select
              id="prompt-compare-version-b"
              className="w-full rounded-md border px-3 py-2"
              value={versionIdB}
              onChange={(event) => setVersionIdB(event.target.value)}
              disabled={versionsLoading}
            >
              <option value="">请选择版本</option>
              {availableVersionsB.map(renderVersionOption)}
            </select>
          </div>
        </div>

        {versionsError instanceof Error && (
          <div className="text-sm text-destructive">加载版本失败：{versionsError.message}</div>
        )}

        <div className="space-y-2">
          <Label>选择历史任务（最多 10 条测试用例）</Label>
          {tasksLoading && <div className="text-muted-foreground">加载任务中...</div>}
          {tasksError instanceof Error && (
            <div className="text-destructive">加载任务失败：{tasksError.message}</div>
          )}
          {!tasksLoading && !tasksError && tasks.length === 0 && (
            <div className="text-muted-foreground">暂无历史任务可选</div>
          )}
          {!tasksLoading && !tasksError && tasks.length > 0 && (
            <div className="grid gap-2">{tasks.map(renderTaskOption)}</div>
          )}
          {workspaceError && <div className="text-destructive">{workspaceError}</div>}
          <div className="text-xs text-muted-foreground">
            未选择测试用例时，将按任务测试集顺序取前 10 条。
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button type="button" disabled={!canCompare} onClick={handleCompare}>
            {compareMutation.isPending ? '对比执行中...' : '开始对比'}
          </Button>
          {!canCompare && (
            <span className="text-xs text-muted-foreground">
              请先选择两个版本与历史任务
            </span>
          )}
        </div>

        {compareMutation.error instanceof Error && (
          <div className="text-destructive">对比失败：{compareMutation.error.message}</div>
        )}

        {compareMutation.data && (
          <div className="space-y-4">
            <CompareResultSummary
              summary={compareMutation.data.summary}
              versionA={compareMutation.data.versionA}
              versionB={compareMutation.data.versionB}
            />
            <PromptDiffViewer
              versionA={{
                version: compareMutation.data.versionA.version,
                content: compareMutation.data.versionAContent,
              }}
              versionB={{
                version: compareMutation.data.versionB.version,
                content: compareMutation.data.versionBContent,
              }}
            />
            <CaseComparisonList comparisons={compareMutation.data.caseComparisons} />
          </div>
        )}
      </CardContent>
    </Card>
  )
}
