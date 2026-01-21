import { useEffect, useMemo, useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { useAuthStore } from '@/stores/useAuthStore'
import { useMetaOptimizationTasks } from '../hooks/useMetaOptimizationTasks'
import { usePromptPreview } from '../hooks/usePromptPreview'
import { getOptimizationTask } from '@/features/task-config/services/optimizationTaskService'
import { getTestSet } from '@/features/test-set-manager/services/testSetService'
import type { MetaOptimizationTaskSummary } from '@/types/generated/models/MetaOptimizationTaskSummary'
import type { PromptPreviewResponse } from '@/types/generated/models/PromptPreviewResponse'
import type { TestCase } from '@/types/generated/models/TestCase'

export interface PromptPreviewPanelProps {
  content: string
  onPreviewingChange?: (previewing: boolean) => void
}

function summarizeReference(testCase: TestCase) {
  const ref = testCase.reference
  if ('Exact' in ref) return 'Exact'
  if ('Constrained' in ref) return 'Constrained'
  if ('Hybrid' in ref) return 'Hybrid'
  return 'Unknown'
}

function summarizeInput(testCase: TestCase) {
  const keys = Object.keys(testCase.input ?? {})
  return `${keys.length} 个输入`
}

export function PromptPreviewPanel({ content, onPreviewingChange }: PromptPreviewPanelProps) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  const { data: tasks = [], isLoading, error } = useMetaOptimizationTasks({ limit: 50 })
  const previewMutation = usePromptPreview()

  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([])
  const [availableCases, setAvailableCases] = useState<TestCase[]>([])
  const [selectedCaseIds, setSelectedCaseIds] = useState<string[]>([])
  const [casesLoading, setCasesLoading] = useState(false)
  const [casesError, setCasesError] = useState<string | null>(null)
  const [previewResult, setPreviewResult] = useState<PromptPreviewResponse | null>(null)

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

  useEffect(() => {
    onPreviewingChange?.(previewMutation.isPending)
  }, [onPreviewingChange, previewMutation.isPending])

  useEffect(() => {
    setPreviewResult(null)
  }, [content, selectedTaskIds, selectedCaseIds])

  useEffect(() => {
    let cancelled = false

    async function loadCases() {
      if (!isAuthenticated || selectedTasks.length === 0) {
        setAvailableCases([])
        setSelectedCaseIds([])
        setCasesError(null)
        return
      }
      if (!workspaceId) {
        setAvailableCases([])
        setSelectedCaseIds([])
        setCasesError('请只选择同一工作区内的历史任务')
        return
      }

      setCasesLoading(true)
      setCasesError(null)
      try {
        const seen = new Set<string>()
        const orderedTestSetIds: string[] = []

        for (const task of selectedTasks) {
          const detail = await getOptimizationTask(task.workspaceId, task.id, sessionToken!)
          for (const testSetId of detail.test_set_ids) {
            if (!seen.has(testSetId)) {
              seen.add(testSetId)
              orderedTestSetIds.push(testSetId)
            }
          }
        }

        const nextCases: TestCase[] = []
        for (const testSetId of orderedTestSetIds) {
          const testSet = await getTestSet(workspaceId, testSetId, sessionToken!)
          nextCases.push(...testSet.cases)
        }

        if (!cancelled) {
          setAvailableCases(nextCases)
          setSelectedCaseIds((prev) =>
            prev.filter((id) => nextCases.some((tc) => tc.id === id))
          )
        }
      } catch (err) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : '加载测试用例失败'
          setCasesError(message)
          setAvailableCases([])
          setSelectedCaseIds([])
        }
      } finally {
        if (!cancelled) {
          setCasesLoading(false)
        }
      }
    }

    loadCases()
    return () => {
      cancelled = true
    }
  }, [isAuthenticated, selectedTasks, sessionToken, workspaceId])

  const handleToggleTask = (taskId: string) => {
    setSelectedTaskIds((prev) =>
      prev.includes(taskId) ? prev.filter((id) => id !== taskId) : [...prev, taskId]
    )
  }

  const handleToggleCase = (testCaseId: string) => {
    setSelectedCaseIds((prev) => {
      if (prev.includes(testCaseId)) {
        return prev.filter((id) => id !== testCaseId)
      }
      if (prev.length >= 3) return prev
      return [...prev, testCaseId]
    })
  }

  const canPreview =
    isAuthenticated &&
    content.trim().length > 0 &&
    selectedTaskIds.length > 0 &&
    !!workspaceId &&
    !casesError &&
    !casesLoading &&
    !previewMutation.isPending

  const handlePreview = () => {
    if (!canPreview) return
    previewMutation.mutate(
      {
        content,
        taskIds: selectedTaskIds,
        testCaseIds: selectedCaseIds,
      },
      {
        onSuccess: (data) => setPreviewResult(data),
      }
    )
  }

  const renderTaskOption = (task: MetaOptimizationTaskSummary) => (
    <label key={task.id} className="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={selectedTaskIds.includes(task.id)}
        onChange={() => handleToggleTask(task.id)}
      />
      <span>{task.name}</span>
      <span className="text-xs text-muted-foreground">({task.status})</span>
    </label>
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>预览效果</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 text-sm">
        {!isAuthenticated && (
          <div className="text-muted-foreground">登录后可使用预览功能。</div>
        )}

        <div className="space-y-2">
          <Label>选择历史任务（最多 3 条测试用例）</Label>
          {isLoading && <div className="text-muted-foreground">加载任务中...</div>}
          {error && (
            <div className="text-destructive">加载任务失败：{error.message}</div>
          )}
          {!isLoading && !error && tasks.length === 0 && (
            <div className="text-muted-foreground">暂无历史任务可选</div>
          )}
          {!isLoading && !error && tasks.length > 0 && (
            <div className="grid gap-2">{tasks.map(renderTaskOption)}</div>
          )}
        </div>

        <div className="space-y-2">
          <Label>选择测试用例（最多 3 条）</Label>
          {casesLoading && <div className="text-muted-foreground">加载测试用例中...</div>}
          {casesError && <div className="text-destructive">{casesError}</div>}
          {!casesLoading && !casesError && availableCases.length === 0 && selectedTaskIds.length > 0 && (
            <div className="text-muted-foreground">暂无可用测试用例</div>
          )}
          <div className="grid gap-2">
            {availableCases.map((testCase) => {
              const checked = selectedCaseIds.includes(testCase.id)
              const disabled = selectedCaseIds.length >= 3 && !checked
              return (
                <label key={testCase.id} className="flex items-center gap-2 text-xs">
                  <input
                    type="checkbox"
                    checked={checked}
                    disabled={disabled}
                    onChange={() => handleToggleCase(testCase.id)}
                  />
                  <span className="font-medium">{testCase.id}</span>
                  <span className="text-muted-foreground">
                    {summarizeReference(testCase)} · {summarizeInput(testCase)}
                  </span>
                </label>
              )
            })}
          </div>
          <div className="text-xs text-muted-foreground">
            不选择时将按历史任务的测试集顺序自动取前 3 条。
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            disabled={!canPreview}
            onClick={handlePreview}
          >
            {previewMutation.isPending ? '执行中...' : '预览效果'}
          </Button>
          {!canPreview && (
            <span className="text-xs text-muted-foreground">
              请先选择任务并填写 Prompt
            </span>
          )}
        </div>

        {previewMutation.error instanceof Error && (
          <div className="text-destructive">预览失败：{previewMutation.error.message}</div>
        )}

        {previewResult && (
          <div className="space-y-3">
            <div className="flex flex-wrap items-center gap-3 text-xs">
              <Badge variant="outline">通过 {previewResult.totalPassed}</Badge>
              <Badge variant="outline">失败 {previewResult.totalFailed}</Badge>
              <Badge variant="outline">
                总耗时 {String(previewResult.totalExecutionTimeMs)} ms
              </Badge>
            </div>
            <div className="space-y-3">
              {previewResult.results.map((result) => (
                <div key={result.testCaseId} className="rounded-md border p-3 space-y-2">
                  <div className="flex items-center justify-between">
                    <div className="text-sm font-medium">{result.testCaseId}</div>
                    <Badge variant={result.passed ? 'default' : 'destructive'}>
                      {result.passed ? '通过' : '失败'}
                    </Badge>
                  </div>
                  <div className="grid gap-2 text-xs">
                    <div>
                      <div className="text-muted-foreground">输入</div>
                      <pre className="whitespace-pre-wrap rounded-md bg-muted/30 p-2">
                        {JSON.stringify(result.input, null, 2)}
                      </pre>
                    </div>
                    <div>
                      <div className="text-muted-foreground">输出</div>
                      <pre className="whitespace-pre-wrap rounded-md bg-muted/30 p-2">
                        {result.actualOutput}
                      </pre>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
