import { useMemo, useState, type FormEvent } from 'react'
import { Link, useParams } from 'react-router'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  useOptimizationTask,
  useUpdateOptimizationTaskConfig,
} from '@/features/task-config/hooks/useOptimizationTasks'
import type { UpdateOptimizationTaskConfigRequest } from '@/types/generated/api/UpdateOptimizationTaskConfigRequest'
import type { OptimizationTaskResponse } from '@/types/generated/api/OptimizationTaskResponse'

const EMPTY_INITIAL_PROMPT_HINT =
  '留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt'

export function OptimizationTaskConfigView() {
  const workspaceId = useParams().id ?? ''
  const taskId = useParams().taskId ?? ''

  const { data: task, isLoading, error } = useOptimizationTask(workspaceId, taskId)

  const loadErrorMessage = error instanceof Error ? error.message : '加载失败'

  return (
    <section
      className="mx-auto flex max-w-5xl flex-col gap-6 px-4 py-6"
      data-testid="optimization-task-config-view"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h1 className="truncate text-2xl font-semibold">
            任务配置{task?.name ? `：${task.name}` : ''}
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            配置初始 Prompt 与迭代终止条件（最大轮数 / 通过率阈值 / 数据划分比例）。
          </p>
        </div>
        <div className="shrink-0">
          <Button asChild variant="outline" size="sm">
            <Link to={`/workspaces/${workspaceId}/tasks`}>返回任务列表</Link>
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>配置</CardTitle>
          <CardDescription>保存后将回显后端规范化后的配置（例如空初始 Prompt 会归一化为 null）。</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading && <div className="text-sm text-muted-foreground">加载中...</div>}

          {error && <div className="text-sm text-red-500">加载失败：{loadErrorMessage}</div>}

          {!isLoading && !error && task && (
            <OptimizationTaskConfigForm workspaceId={workspaceId} taskId={taskId} task={task} />
          )}
        </CardContent>
      </Card>
    </section>
  )
}

function OptimizationTaskConfigForm(props: {
  workspaceId: string
  taskId: string
  task: OptimizationTaskResponse
}) {
  const { workspaceId, taskId, task } = props

  const {
    mutateAsync: updateConfig,
    isPending: isSaving,
    error: saveError,
  } = useUpdateOptimizationTaskConfig(workspaceId, taskId)

  const [initialPrompt, setInitialPrompt] = useState(task.config.initial_prompt ?? '')
  const [maxIterations, setMaxIterations] = useState(task.config.max_iterations)
  const [passThresholdPercent, setPassThresholdPercent] = useState(task.config.pass_threshold_percent)
  const [trainPercent, setTrainPercent] = useState(task.config.data_split.train_percent)
  const [validationPercent, setValidationPercent] = useState(task.config.data_split.validation_percent)

  const [localError, setLocalError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)

  const saveErrorMessage = saveError instanceof Error ? saveError.message : null

  const showEmptyInitialPromptHint = useMemo(() => {
    return initialPrompt.trim() === ''
  }, [initialPrompt])

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setLocalError(null)
    setSuccessMessage(null)

    const sum = Number(trainPercent) + Number(validationPercent)
    if (sum !== 100) {
      setLocalError('Train% + Validation% 必须等于 100')
      return
    }

    const payload: UpdateOptimizationTaskConfigRequest = {
      initial_prompt: initialPrompt,
      max_iterations: Number(maxIterations),
      pass_threshold_percent: Number(passThresholdPercent),
      train_percent: Number(trainPercent),
      validation_percent: Number(validationPercent),
    }

    try {
      const updated = await updateConfig(payload)
      setInitialPrompt(updated.config.initial_prompt ?? '')
      setMaxIterations(updated.config.max_iterations)
      setPassThresholdPercent(updated.config.pass_threshold_percent)
      setTrainPercent(updated.config.data_split.train_percent)
      setValidationPercent(updated.config.data_split.validation_percent)
      setSuccessMessage('保存成功')
    } catch {
      // 错误由 mutation error 状态渲染（仅展示 message）
    }
  }

  return (
    <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
      <div className="grid gap-2">
        <Label htmlFor="initial-prompt">初始 Prompt（可空）</Label>
        <textarea
          id="initial-prompt"
          className="min-h-32 w-full rounded-md border bg-transparent px-3 py-2 text-sm"
          value={initialPrompt}
          onChange={(e) => setInitialPrompt(e.target.value)}
        />
        {showEmptyInitialPromptHint && (
          <div className="text-xs text-muted-foreground">{EMPTY_INITIAL_PROMPT_HINT}</div>
        )}
      </div>

      <div className="grid gap-2">
        <Label htmlFor="max-iterations">最大迭代轮数</Label>
        <Input
          id="max-iterations"
          type="number"
          min={1}
          max={100}
          value={maxIterations}
          onChange={(e) => setMaxIterations(Number(e.target.value))}
        />
        <div className="text-xs text-muted-foreground">合理范围：1-100（默认推荐值：10）。</div>
      </div>

      <div className="grid gap-2">
        <Label htmlFor="pass-threshold">通过率阈值（%）</Label>
        <Input
          id="pass-threshold"
          type="number"
          min={1}
          max={100}
          value={passThresholdPercent}
          onChange={(e) => setPassThresholdPercent(Number(e.target.value))}
        />
        <div className="text-xs text-muted-foreground">
          当 Validation 通过率达到该阈值时，认为已达标并停止迭代（默认推荐值：95%）。
        </div>
      </div>

      <div className="grid gap-2">
        <Label>数据划分策略（百分比）</Label>
        <div className="grid grid-cols-2 gap-3">
          <div className="grid gap-2">
            <Label htmlFor="train-percent">Train%</Label>
            <Input
              id="train-percent"
              type="number"
              min={0}
              max={100}
              value={trainPercent}
              onChange={(e) => setTrainPercent(Number(e.target.value))}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="validation-percent">Validation%</Label>
            <Input
              id="validation-percent"
              type="number"
              min={0}
              max={100}
              value={validationPercent}
              onChange={(e) => setValidationPercent(Number(e.target.value))}
            />
          </div>
        </div>
        <div className="text-xs text-muted-foreground">
          本 Story 仅暴露 Train/Validation；Holdout 固定为 0%（后续 Story 再开放）。
        </div>
      </div>

      {localError && <div className="text-sm text-red-500">{localError}</div>}
      {saveErrorMessage && <div className="text-sm text-red-500">保存失败：{saveErrorMessage}</div>}
      {successMessage && <div className="text-sm text-green-600">{successMessage}</div>}

      <div>
        <Button type="submit" disabled={isSaving}>
          {isSaving ? '保存中...' : '保存配置'}
        </Button>
      </div>
    </form>
  )
}
