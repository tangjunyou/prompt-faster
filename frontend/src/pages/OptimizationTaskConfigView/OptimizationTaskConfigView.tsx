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
import type { AdvancedDataSplitStrategy } from '@/types/generated/models/AdvancedDataSplitStrategy'
import type { EvaluatorType } from '@/types/generated/models/EvaluatorType'
import type { OutputStrategy } from '@/types/generated/models/OutputStrategy'
import type { SamplingStrategy } from '@/types/generated/models/SamplingStrategy'

const EMPTY_INITIAL_PROMPT_HINT =
  '留空时，系统将在首次迭代中基于优化目标和测试集自动生成初始 Prompt'

const CANDIDATE_PROMPT_COUNT_MIN = 1
const CANDIDATE_PROMPT_COUNT_MAX = 10

const DIVERSITY_INJECTION_THRESHOLD_MIN = 1
const DIVERSITY_INJECTION_THRESHOLD_MAX = 10

const CONFLICT_ALERT_THRESHOLD_MIN = 1
const CONFLICT_ALERT_THRESHOLD_MAX = 10

const K_FOLD_FOLDS_MIN = 2
const K_FOLD_FOLDS_MAX = 10

const SEMANTIC_SIMILARITY_THRESHOLD_MIN = 1
const SEMANTIC_SIMILARITY_THRESHOLD_MAX = 100

const LLM_JUDGE_SAMPLES_MIN = 1
const LLM_JUDGE_SAMPLES_MAX = 5

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

  const [outputStrategy, setOutputStrategy] = useState<OutputStrategy>(task.config.output_config.strategy)
  const [conflictAlertThreshold, setConflictAlertThreshold] = useState(
    task.config.output_config.conflict_alert_threshold
  )
  const [autoRecommendOutput, setAutoRecommendOutput] = useState(task.config.output_config.auto_recommend)

  const [evaluatorType, setEvaluatorType] = useState<EvaluatorType>(task.config.evaluator_config.evaluator_type)
  const [caseSensitive, setCaseSensitive] = useState(task.config.evaluator_config.exact_match.case_sensitive)
  const [semanticThresholdPercent, setSemanticThresholdPercent] = useState(
    task.config.evaluator_config.semantic_similarity.threshold_percent
  )
  const [constraintStrict, setConstraintStrict] = useState(task.config.evaluator_config.constraint_check.strict)
  const [llmJudgeSamples, setLlmJudgeSamples] = useState(task.config.evaluator_config.teacher_model.llm_judge_samples)

  const [advancedDataSplitStrategy, setAdvancedDataSplitStrategy] = useState<AdvancedDataSplitStrategy>(
    task.config.advanced_data_split.strategy
  )
  const [kFoldFolds, setKFoldFolds] = useState(task.config.advanced_data_split.k_fold_folds)
  const [samplingStrategy, setSamplingStrategy] = useState<SamplingStrategy>(
    task.config.advanced_data_split.sampling_strategy
  )

  const [localError, setLocalError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)

  const saveErrorMessage = saveError instanceof Error ? saveError.message : null

  const showEmptyInitialPromptHint = useMemo(() => {
    return initialPrompt.trim() === ''
  }, [initialPrompt])

  const applyUpdatedConfig = (config: OptimizationTaskResponse['config']) => {
    setInitialPrompt(config.initial_prompt ?? '')
    setMaxIterations(config.max_iterations)
    setPassThresholdPercent(config.pass_threshold_percent)
    setCandidatePromptCount(config.candidate_prompt_count)
    setDiversityInjectionThreshold(config.diversity_injection_threshold)
    setTrainPercent(config.data_split.train_percent)
    setValidationPercent(config.data_split.validation_percent)
    setOutputStrategy(config.output_config.strategy)
    setConflictAlertThreshold(config.output_config.conflict_alert_threshold)
    setAutoRecommendOutput(config.output_config.auto_recommend)
    setEvaluatorType(config.evaluator_config.evaluator_type)
    setCaseSensitive(config.evaluator_config.exact_match.case_sensitive)
    setSemanticThresholdPercent(config.evaluator_config.semantic_similarity.threshold_percent)
    setConstraintStrict(config.evaluator_config.constraint_check.strict)
    setLlmJudgeSamples(config.evaluator_config.teacher_model.llm_judge_samples)
    setAdvancedDataSplitStrategy(config.advanced_data_split.strategy)
    setKFoldFolds(config.advanced_data_split.k_fold_folds)
    setSamplingStrategy(config.advanced_data_split.sampling_strategy)
  }

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
    const conflictAlertThresholdValue = Number(conflictAlertThreshold)
    const semanticThresholdValue = Number(semanticThresholdPercent)
    const llmJudgeSamplesValue = Number(llmJudgeSamples)
    const kFoldFoldsValue = Number(kFoldFolds)

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

    const conflictAlertThresholdError = validateIntegerInRange(
      conflictAlertThresholdValue,
      CONFLICT_ALERT_THRESHOLD_MIN,
      CONFLICT_ALERT_THRESHOLD_MAX,
      '冲突告警阈值'
    )
    if (conflictAlertThresholdError) {
      setLocalError(conflictAlertThresholdError)
      return
    }

    if (advancedDataSplitStrategy === 'k_fold') {
      const kFoldFoldsError = validateIntegerInRange(
        kFoldFoldsValue,
        K_FOLD_FOLDS_MIN,
        K_FOLD_FOLDS_MAX,
        '交叉验证折数'
      )
      if (kFoldFoldsError) {
        setLocalError(kFoldFoldsError)
        return
      }
    }

    if (evaluatorType === 'semantic_similarity') {
      const semanticThresholdError = validateIntegerInRange(
        semanticThresholdValue,
        SEMANTIC_SIMILARITY_THRESHOLD_MIN,
        SEMANTIC_SIMILARITY_THRESHOLD_MAX,
        '语义相似度阈值（%）'
      )
      if (semanticThresholdError) {
        setLocalError(semanticThresholdError)
        return
      }
    }

    if (evaluatorType === 'teacher_model') {
      const llmJudgeSamplesError = validateIntegerInRange(
        llmJudgeSamplesValue,
        LLM_JUDGE_SAMPLES_MIN,
        LLM_JUDGE_SAMPLES_MAX,
        '老师模型采样数'
      )
      if (llmJudgeSamplesError) {
        setLocalError(llmJudgeSamplesError)
        return
      }
    }

    const payload: UpdateOptimizationTaskConfigRequest = {
      initial_prompt: initialPrompt.trim() === '' ? null : initialPrompt.trim(),
      max_iterations: maxIterationsValue,
      pass_threshold_percent: passThresholdValue,
      candidate_prompt_count: candidatePromptCountValue,
      diversity_injection_threshold: diversityInjectionThresholdValue,
      train_percent: trainPercentValue,
      validation_percent: validationPercentValue,
      output_config: {
        strategy: outputStrategy,
        conflict_alert_threshold: conflictAlertThresholdValue,
        auto_recommend: autoRecommendOutput,
      },
      evaluator_config: {
        evaluator_type: evaluatorType,
        exact_match: { case_sensitive: caseSensitive },
        semantic_similarity: { threshold_percent: semanticThresholdValue },
        constraint_check: { strict: constraintStrict },
        teacher_model: { llm_judge_samples: llmJudgeSamplesValue },
      },
      advanced_data_split: {
        strategy: advancedDataSplitStrategy,
        k_fold_folds: kFoldFoldsValue,
        sampling_strategy: samplingStrategy,
      },
    }

    try {
      const updated = await updateConfig(payload)
      applyUpdatedConfig(updated.config)
      setSuccessMessage('保存成功')
    } catch {
      // 错误由 mutation error 状态渲染（仅展示 message）
    }
  }

  const resetAdvancedToDefaults = async () => {
    setLocalError(null)
    setSuccessMessage(null)

    const confirmed = window.confirm('确认将高级配置重置为默认值？（不会影响基础配置与初始 Prompt）')
    if (!confirmed) {
      return
    }

    const defaultOutputStrategy: OutputStrategy = 'single'
    const defaultConflictAlertThreshold = 3
    const defaultAutoRecommendOutput = true

    const defaultEvaluatorType: EvaluatorType = 'auto'
    const defaultCaseSensitive = false
    const defaultSemanticThresholdPercent = 85
    const defaultConstraintStrict = true
    const defaultLlmJudgeSamples = 1

    const defaultAdvancedDataSplitStrategy: AdvancedDataSplitStrategy = 'percent'
    const defaultKFoldFolds = 5
    const defaultSamplingStrategy: SamplingStrategy = 'random'

    const payload: UpdateOptimizationTaskConfigRequest = {
      initial_prompt: initialPrompt.trim() === '' ? null : initialPrompt.trim(),
      max_iterations: Number(maxIterations),
      pass_threshold_percent: Number(passThresholdPercent),
      candidate_prompt_count: Number(candidatePromptCount),
      diversity_injection_threshold: Number(diversityInjectionThreshold),
      train_percent: Number(trainPercent),
      validation_percent: Number(validationPercent),
      output_config: {
        strategy: defaultOutputStrategy,
        conflict_alert_threshold: defaultConflictAlertThreshold,
        auto_recommend: defaultAutoRecommendOutput,
      },
      evaluator_config: {
        evaluator_type: defaultEvaluatorType,
        exact_match: { case_sensitive: defaultCaseSensitive },
        semantic_similarity: { threshold_percent: defaultSemanticThresholdPercent },
        constraint_check: { strict: defaultConstraintStrict },
        teacher_model: { llm_judge_samples: defaultLlmJudgeSamples },
      },
      advanced_data_split: {
        strategy: defaultAdvancedDataSplitStrategy,
        k_fold_folds: defaultKFoldFolds,
        sampling_strategy: defaultSamplingStrategy,
      },
    }

    try {
      const updated = await updateConfig(payload)
      applyUpdatedConfig(updated.config)
      setSuccessMessage('保存成功')
    } catch {
      // 错误由 mutation error 状态渲染（仅展示 message）
    }
  }

  return (
    <form className="flex flex-col gap-4" noValidate onSubmit={handleSubmit}>
      <div className="text-sm font-medium">基础配置</div>
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

      <details className="rounded-md border p-4">
        <summary className="cursor-pointer text-sm font-medium">高级配置</summary>
        <div className="mt-4 grid gap-6">
          <div className="grid gap-3">
            <div className="text-sm font-medium">OutputConfig（输出配置）</div>
            <div className="grid gap-2">
              <Label htmlFor="output-strategy">输出策略</Label>
              <select
                id="output-strategy"
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
                value={outputStrategy}
                onChange={(e) => setOutputStrategy(e.target.value as OutputStrategy)}
              >
                <option value="single">single（单一输出）</option>
                <option value="adaptive">adaptive（自适应）</option>
                <option value="multi">multi（多输出）</option>
              </select>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="conflict-alert-threshold">冲突告警阈值</Label>
              <Input
                id="conflict-alert-threshold"
                type="number"
                min={CONFLICT_ALERT_THRESHOLD_MIN}
                max={CONFLICT_ALERT_THRESHOLD_MAX}
                value={conflictAlertThreshold}
                onChange={(e) => setConflictAlertThreshold(Number(e.target.value))}
              />
              <div className="text-xs text-muted-foreground">
                合理范围：{CONFLICT_ALERT_THRESHOLD_MIN}-{CONFLICT_ALERT_THRESHOLD_MAX}（默认推荐值：3）。
              </div>
            </div>

            <div className="flex items-center gap-2">
              <input
                id="auto-recommend-output"
                type="checkbox"
                checked={autoRecommendOutput}
                onChange={(e) => setAutoRecommendOutput(e.target.checked)}
              />
              <Label htmlFor="auto-recommend-output">自动推荐输出</Label>
            </div>
          </div>

          <div className="grid gap-3">
            <div className="text-sm font-medium">EvaluatorConfig（评估器）</div>
            <div className="grid gap-2">
              <Label htmlFor="evaluator-type">评估器类型</Label>
              <select
                id="evaluator-type"
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
                value={evaluatorType}
                onChange={(e) => setEvaluatorType(e.target.value as EvaluatorType)}
              >
                <option value="auto">auto（自动选择）</option>
                <option value="exact_match">精确匹配</option>
                <option value="semantic_similarity">语义相似度</option>
                <option value="constraint_check">约束检查</option>
                <option value="teacher_model">老师模型评估</option>
              </select>
              <div className="text-xs text-muted-foreground">
                `auto` 表示使用系统默认评估策略（本 Story 仅承载配置占位；实际自动选择逻辑在执行引擎阶段落地）。
              </div>
            </div>

            {evaluatorType === 'exact_match' && (
              <div className="flex items-center gap-2">
                <input
                  id="exact-match-case-sensitive"
                  type="checkbox"
                  checked={caseSensitive}
                  onChange={(e) => setCaseSensitive(e.target.checked)}
                />
                <Label htmlFor="exact-match-case-sensitive">大小写敏感</Label>
              </div>
            )}

            {evaluatorType === 'semantic_similarity' && (
              <div className="grid gap-2">
                <Label htmlFor="semantic-threshold-percent">语义相似度阈值（%）</Label>
                <Input
                  id="semantic-threshold-percent"
                  type="number"
                  min={SEMANTIC_SIMILARITY_THRESHOLD_MIN}
                  max={SEMANTIC_SIMILARITY_THRESHOLD_MAX}
                  value={semanticThresholdPercent}
                  onChange={(e) => setSemanticThresholdPercent(Number(e.target.value))}
                />
                <div className="text-xs text-muted-foreground">
                  合理范围：{SEMANTIC_SIMILARITY_THRESHOLD_MIN}-{SEMANTIC_SIMILARITY_THRESHOLD_MAX}（默认推荐值：85）。
                </div>
              </div>
            )}

            {evaluatorType === 'constraint_check' && (
              <div className="flex items-center gap-2">
                <input
                  id="constraint-check-strict"
                  type="checkbox"
                  checked={constraintStrict}
                  onChange={(e) => setConstraintStrict(e.target.checked)}
                />
                <Label htmlFor="constraint-check-strict">严格模式</Label>
              </div>
            )}

            {evaluatorType === 'teacher_model' && (
              <div className="grid gap-2">
                <Label htmlFor="llm-judge-samples">老师模型采样数</Label>
                <Input
                  id="llm-judge-samples"
                  type="number"
                  min={LLM_JUDGE_SAMPLES_MIN}
                  max={LLM_JUDGE_SAMPLES_MAX}
                  value={llmJudgeSamples}
                  onChange={(e) => setLlmJudgeSamples(Number(e.target.value))}
                />
                <div className="text-xs text-muted-foreground">
                  合理范围：{LLM_JUDGE_SAMPLES_MIN}-{LLM_JUDGE_SAMPLES_MAX}（默认推荐值：1）。评估使用系统既有老师模型配置。
                </div>
              </div>
            )}
          </div>

          <div className="grid gap-3">
            <div className="text-sm font-medium">AdvancedDataSplitConfig（高级数据划分）</div>
            <div className="grid gap-2">
              <Label htmlFor="advanced-data-split-strategy">高级数据划分策略</Label>
              <select
                id="advanced-data-split-strategy"
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
                value={advancedDataSplitStrategy}
                onChange={(e) => setAdvancedDataSplitStrategy(e.target.value as AdvancedDataSplitStrategy)}
              >
                <option value="percent">percent（百分比划分）</option>
                <option value="k_fold">k_fold（交叉验证）</option>
              </select>
              <div className="text-xs text-muted-foreground">
                选择 percent 时沿用基础配置中的 Train% / Validation%；选择 k_fold 时，执行阶段将忽略 Train% / Validation%。
              </div>
            </div>

            {advancedDataSplitStrategy === 'k_fold' && (
              <>
                <div className="grid gap-2">
                  <Label htmlFor="k-fold-folds">交叉验证折数</Label>
                  <Input
                    id="k-fold-folds"
                    type="number"
                    min={K_FOLD_FOLDS_MIN}
                    max={K_FOLD_FOLDS_MAX}
                    value={kFoldFolds}
                    onChange={(e) => setKFoldFolds(Number(e.target.value))}
                  />
                  <div className="text-xs text-muted-foreground">
                    合理范围：{K_FOLD_FOLDS_MIN}-{K_FOLD_FOLDS_MAX}（默认推荐值：5）。
                  </div>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="sampling-strategy">采样策略</Label>
                  <select
                    id="sampling-strategy"
                    className="h-9 rounded-md border bg-transparent px-3 text-sm"
                    value={samplingStrategy}
                    onChange={(e) => setSamplingStrategy(e.target.value as SamplingStrategy)}
                  >
                    <option value="random">random（随机）</option>
                    <option value="stratified">stratified（分层）</option>
                  </select>
                </div>
              </>
            )}
          </div>

          <div className="flex items-center justify-between gap-4">
            <div className="text-xs text-muted-foreground">
              “重置为默认值”仅作用于高级配置（OutputConfig / EvaluatorConfig / AdvancedDataSplitConfig）。
            </div>
            <Button type="button" variant="outline" onClick={resetAdvancedToDefaults} disabled={isSaving}>
              重置为默认值
            </Button>
          </div>
        </div>
      </details>

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
