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

const CANDIDATE_PROMPT_COUNT_MIN = 1
const CANDIDATE_PROMPT_COUNT_MAX = 10

const DIVERSITY_INJECTION_THRESHOLD_MIN = 1
const DIVERSITY_INJECTION_THRESHOLD_MAX = 10

function validateIntegerInRange(value: number, min: number, max: number, label: string) {
  if (!Number.isFinite(value)) {
    return `${label}必须为数字`
  }
  if (!Number.isInteger(value)) {
    return `${label}必须为整数`
  }
  if (value < min || value > max) {
    return `${label}仅允许 ${min}-${max}`
  }
  return null
}

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
            配置初始 Prompt、迭代终止条件与算法参数（候选 Prompt 数量 / 多样性注入阈值）。
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
  const [candidatePromptCount, setCandidatePromptCount] = useState(task.config.candidate_prompt_count)
  const [diversityInjectionThreshold, setDiversityInjectionThreshold] = useState(
    task.config.diversity_injection_threshold
  )
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

    const maxIterationsValue = Number(maxIterations)
    const passThresholdValue = Number(passThresholdPercent)
    const candidatePromptCountValue = Number(candidatePromptCount)
    const diversityInjectionThresholdValue = Number(diversityInjectionThreshold)
    const trainPercentValue = Number(trainPercent)
    const validationPercentValue = Number(validationPercent)

    const maxIterationsError = validateIntegerInRange(maxIterationsValue, 1, 100, '最大迭代轮数')
    if (maxIterationsError) {
      setLocalError(maxIterationsError)
      return
    }

    const passThresholdError = validateIntegerInRange(passThresholdValue, 1, 100, '通过率阈值（%）')
    if (passThresholdError) {
      setLocalError(passThresholdError)
      return
    }

    const candidatePromptCountError = validateIntegerInRange(
      candidatePromptCountValue,
      CANDIDATE_PROMPT_COUNT_MIN,
      CANDIDATE_PROMPT_COUNT_MAX,
      '候选 Prompt 生成数量'
    )
    if (candidatePromptCountError) {
      setLocalError(candidatePromptCountError)
      return
    }

    const diversityInjectionThresholdError = validateIntegerInRange(
      diversityInjectionThresholdValue,
      DIVERSITY_INJECTION_THRESHOLD_MIN,
      DIVERSITY_INJECTION_THRESHOLD_MAX,
      '多样性注入阈值'
    )
    if (diversityInjectionThresholdError) {
      setLocalError(diversityInjectionThresholdError)
      return
    }

    const trainPercentError = validateIntegerInRange(trainPercentValue, 0, 100, 'Train%')
    if (trainPercentError) {
      setLocalError(trainPercentError)
      return
    }

    const validationPercentError = validateIntegerInRange(validationPercentValue, 0, 100, 'Validation%')
    if (validationPercentError) {
      setLocalError(validationPercentError)
      return
    }

    const sum = trainPercentValue + validationPercentValue
    if (sum !== 100) {
      setLocalError('Train% + Validation% 必须等于 100')
      return
    }

    const payload: UpdateOptimizationTaskConfigRequest = {
      initial_prompt: initialPrompt.trim() === '' ? null : initialPrompt.trim(),
      max_iterations: maxIterationsValue,
      pass_threshold_percent: passThresholdValue,
      candidate_prompt_count: candidatePromptCountValue,
      diversity_injection_threshold: diversityInjectionThresholdValue,
      train_percent: trainPercentValue,
      validation_percent: validationPercentValue,
    }

    try {
      const updated = await updateConfig(payload)
      setInitialPrompt(updated.config.initial_prompt ?? '')
      setMaxIterations(updated.config.max_iterations)
      setPassThresholdPercent(updated.config.pass_threshold_percent)
      setCandidatePromptCount(updated.config.candidate_prompt_count)
      setDiversityInjectionThreshold(updated.config.diversity_injection_threshold)
      setTrainPercent(updated.config.data_split.train_percent)
      setValidationPercent(updated.config.data_split.validation_percent)
      setSuccessMessage('保存成功')
    } catch {
      // 错误由 mutation error 状态渲染（仅展示 message）
    }
  }

  return (
    <form className="flex flex-col gap-4" noValidate onSubmit={handleSubmit}>
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
        <Label htmlFor="candidate-prompt-count">候选 Prompt 生成数量</Label>
        <Input
          id="candidate-prompt-count"
          type="number"
          min={CANDIDATE_PROMPT_COUNT_MIN}
          max={CANDIDATE_PROMPT_COUNT_MAX}
          value={candidatePromptCount}
          onChange={(e) => setCandidatePromptCount(Number(e.target.value))}
        />
        <div className="text-xs text-muted-foreground">
          每轮迭代将生成多个候选 Prompt 供评估与选择；数量越多探索更充分，但耗时/成本更高（推荐 3-5，默认 5）。
        </div>
      </div>

      <div className="grid gap-2">
        <Label htmlFor="diversity-injection-threshold">多样性注入阈值（连续失败次数）</Label>
        <Input
          id="diversity-injection-threshold"
          type="number"
          min={DIVERSITY_INJECTION_THRESHOLD_MIN}
          max={DIVERSITY_INJECTION_THRESHOLD_MAX}
          value={diversityInjectionThreshold}
          onChange={(e) => setDiversityInjectionThreshold(Number(e.target.value))}
        />
        <div className="text-xs text-muted-foreground">
          当连续失败达到该次数后触发多样性注入，用于跳出“卡住”状态并扩大探索空间（默认推荐值：3）。
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
