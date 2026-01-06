import { useMemo, useState, type FormEvent } from 'react'
import { useParams } from 'react-router'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useTestSets } from '@/features/test-set-manager/hooks/useTestSets'
import { useCreateOptimizationTask, useOptimizationTasks } from '@/features/task-config/hooks/useOptimizationTasks'
import type { CreateOptimizationTaskRequest } from '@/types/generated/api/CreateOptimizationTaskRequest'
import type { OptimizationTaskListItemResponse } from '@/types/generated/api/OptimizationTaskListItemResponse'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'

function formatTime(ts: number) {
  if (!Number.isFinite(ts) || ts <= 0) return '-'
  return new Date(ts).toLocaleString()
}

export function OptimizationTasksView() {
  const workspaceId = useParams().id ?? ''

  const { data: tasksData, isLoading: isLoadingTasks, error: tasksError } = useOptimizationTasks(workspaceId)
  const tasks: OptimizationTaskListItemResponse[] = tasksData ?? []

  const { data: testSetsData, isLoading: isLoadingTestSets, error: testSetsError } = useTestSets(workspaceId)
  const testSets: TestSetListItemResponse[] = testSetsData ?? []

  const {
    mutateAsync: createTask,
    isPending: isCreating,
    error: createError,
  } = useCreateOptimizationTask(workspaceId)

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [goal, setGoal] = useState('')
  const [executionTargetType, setExecutionTargetType] = useState<'dify' | 'generic'>('dify')
  const [taskMode, setTaskMode] = useState<'fixed' | 'creative'>('fixed')
  const [selectedTestSetIds, setSelectedTestSetIds] = useState<string[]>([])
  const [localError, setLocalError] = useState<string | null>(null)

  const createErrorMessage = createError instanceof Error ? createError.message : null
  const listErrorMessage = tasksError instanceof Error ? tasksError.message : '加载失败'
  const testSetsErrorMessage = testSetsError instanceof Error ? testSetsError.message : '加载失败'

  const executionTargetHelp = useMemo(() => {
    if (executionTargetType === 'dify') {
      return '已选择 Dify 工作流（本 Story 仅要求可选择并持久化；细节字段在后续 Story 补齐）。'
    }
    return '已选择 通用 API（直连模型）（本 Story 仅要求可选择并持久化；细节字段在后续 Story 补齐）。'
  }, [executionTargetType])

  const taskModeHelp = useMemo(() => {
    if (taskMode === 'fixed') {
      return 'Fixed：以 reference.Exact / reference.Hybrid 为主（有“标准答案”语义）。'
    }
    return 'Creative：以 reference.Constrained / reference.Hybrid 为主（无标准答案，靠约束/维度评估）。'
  }, [taskMode])

  const toggleTestSet = (id: string) => {
    setSelectedTestSetIds((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]))
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setLocalError(null)

    if (!name.trim()) {
      setLocalError('任务名称不能为空')
      return
    }
    if (!goal.trim()) {
      setLocalError('优化目标不能为空')
      return
    }
    if (selectedTestSetIds.length === 0) {
      setLocalError('请至少选择 1 个测试集')
      return
    }

    const payload: CreateOptimizationTaskRequest = {
      name: name.trim(),
      description: description.trim() ? description.trim() : null,
      goal: goal.trim(),
      execution_target_type: executionTargetType,
      task_mode: taskMode,
      test_set_ids: selectedTestSetIds,
    }

    try {
      await createTask(payload)
      setName('')
      setDescription('')
      setGoal('')
      setExecutionTargetType('dify')
      setTaskMode('fixed')
      setSelectedTestSetIds([])
    } catch {
      // 错误由 mutation error 状态渲染（仅展示 message）
    }
  }

  return (
    <section className="mx-auto flex max-w-5xl flex-col gap-6 px-4 py-6" data-testid="optimization-tasks-view">
      <div>
        <h1 className="text-2xl font-semibold">优化任务</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          创建优化任务并完成最小可用的基础配置（执行目标/任务模式/优化目标/关联测试集）。
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>新建优化任务</CardTitle>
          <CardDescription>本 Story：只要求可创建 + 可持久化 + 可回显。</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-2">
              <Label htmlFor="task-name">任务名称 *</Label>
              <Input
                id="task-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="例如：系统提示词简化优化"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="task-description">任务描述</Label>
              <Input
                id="task-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder="可选"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="task-goal">优化目标 *</Label>
              <Input
                id="task-goal"
                value={goal}
                onChange={(event) => setGoal(event.target.value)}
                placeholder="例如：让 system prompt 更简洁、稳定，并提高测试通过率"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="execution-target-type">执行目标 *</Label>
              <select
                id="execution-target-type"
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
                value={executionTargetType}
                onChange={(event) => setExecutionTargetType(event.target.value as 'dify' | 'generic')}
              >
                <option value="dify">Dify 工作流</option>
                <option value="generic">通用 API（直连模型）</option>
              </select>
              <div className="text-xs text-muted-foreground">{executionTargetHelp}</div>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="task-mode">任务模式 *</Label>
              <select
                id="task-mode"
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
                value={taskMode}
                onChange={(event) => setTaskMode(event.target.value as 'fixed' | 'creative')}
              >
                <option value="fixed">固定任务（Fixed）</option>
                <option value="creative">创意任务（Creative）</option>
              </select>
              <div className="text-xs text-muted-foreground">{taskModeHelp}</div>
            </div>

            <div className="grid gap-2">
              <Label>关联测试集 *</Label>
              {isLoadingTestSets && <div className="text-sm text-muted-foreground">加载测试集...</div>}
              {testSetsError && (
                <div className="text-sm text-red-500">加载测试集失败：{testSetsErrorMessage}</div>
              )}
              {!isLoadingTestSets && !testSetsError && testSets.length === 0 && (
                <div className="text-sm text-muted-foreground">当前工作区暂无测试集，请先创建测试集。</div>
              )}
              {!isLoadingTestSets && !testSetsError && testSets.length > 0 && (
                <ul className="space-y-2 text-sm">
                  {testSets.map((ts) => (
                    <li key={ts.id} className="flex items-start gap-2">
                      <input
                        type="checkbox"
                        aria-label={`选择测试集 ${ts.name}`}
                        checked={selectedTestSetIds.includes(ts.id)}
                        onChange={() => toggleTestSet(ts.id)}
                      />
                      <div className="min-w-0">
                        <div className="font-medium">{ts.name}</div>
                        <div className="text-muted-foreground">
                          {ts.description || '暂无描述'} · 用例数：{ts.cases_count}
                        </div>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            {localError && <div className="text-sm text-red-500">{localError}</div>}
            {createErrorMessage && (
              <div className="text-sm text-red-500">创建失败：{createErrorMessage}</div>
            )}

            <div>
              <Button type="submit" disabled={isCreating}>
                {isCreating ? '创建中...' : '创建任务'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>任务列表</CardTitle>
          <CardDescription>至少包含：名称、优化目标摘要、执行目标类型、任务模式、创建/更新时间。</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoadingTasks && <div className="text-sm text-muted-foreground">加载中...</div>}

          {tasksError && (
            <div className="text-sm text-red-500">加载失败：{listErrorMessage}</div>
          )}

          {!isLoadingTasks && !tasksError && tasks.length === 0 && (
            <div className="text-sm text-muted-foreground">暂无任务，请先创建一个。</div>
          )}

          {!isLoadingTasks && !tasksError && tasks.length > 0 && (
            <ul className="space-y-2 text-sm">
              {tasks.map((task) => (
                <li key={task.id} className="rounded-md border px-3 py-2">
                  <div className="flex flex-col gap-1">
                    <div className="font-medium">{task.name}</div>
                    <div className="text-muted-foreground">目标：{task.goal}</div>
                    <div className="text-muted-foreground">
                      执行目标：{task.execution_target_type} · 模式：{task.task_mode} · 状态：{task.status}
                    </div>
                    <div className="text-muted-foreground">
                      创建：{formatTime(task.created_at)} · 更新：{formatTime(task.updated_at)}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </section>
  )
}

