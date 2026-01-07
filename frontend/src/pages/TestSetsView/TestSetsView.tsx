import { useMemo, useRef, useState, type FormEvent } from 'react'
import { Link, useParams } from 'react-router'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  useCreateTestSet,
  useDeleteTestSet,
  useTestSets,
  useUpdateTestSet,
} from '@/features/test-set-manager/hooks/useTestSets'
import {
  useSaveAsTemplate,
  useTestSetTemplates,
} from '@/features/test-set-manager/hooks/useTestSetTemplates'
import {
  parseTestCasesJsonl,
  type JsonlParseError,
  type ParseStats,
} from '@/features/test-set-manager/services/parseTestCasesJsonl'
import { getTestSet } from '@/features/test-set-manager/services/testSetService'
import { getTestSetTemplate } from '@/features/test-set-manager/services/testSetTemplateService'
import { refreshDifyVariables, saveDifyConfig } from '@/features/test-set-manager/services/difyService'
import { deleteGenericConfig, saveGenericConfig } from '@/features/test-set-manager/services/genericConfigService'
import { useAuthStore } from '@/stores/useAuthStore'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { DifyBindingSource } from '@/types/generated/api/DifyBindingSource'
import type { DifyConfig } from '@/types/generated/api/DifyConfig'
import type { DifyInputVariable } from '@/types/generated/api/DifyInputVariable'
import type { SaveDifyConfigRequest } from '@/types/generated/api/SaveDifyConfigRequest'
import type { GenericConfig } from '@/types/generated/api/GenericConfig'
import type { GenericInputVariable } from '@/types/generated/api/GenericInputVariable'
import type { GenericValueType } from '@/types/generated/api/GenericValueType'
import type { SaveGenericConfigRequest } from '@/types/generated/api/SaveGenericConfigRequest'
import type { TestCase } from '@/types/generated/models/TestCase'
import type { JsonValue } from '@/types/generated/serde_json/JsonValue'

const JSONL_FORMAT_HELP = {
  title: '导入格式（JSON Lines / JSONL）',
  description: '仅支持 txt（UTF-8），一行一个 TestCase JSON；空行会被跳过。',
  example: `{"id":"case-1","input":{"question":"你好，帮我写一段自我介绍"},"reference":{"Exact":{"expected":"（此处填写期望输出）"}}}
{"id":"case-2","input":{"question":"用 JSON 输出一个用户对象"},"reference":{"Constrained":{"core_request":"必须是 JSON 且字段合理","constraints":[{"name":"format","description":"必须是 JSON","params":{"format":"json"},"weight":null}],"quality_dimensions":[]}}}
{"id":"case-3","input":{"prompt":"写一段欢迎文案"},"reference":{"Constrained":{"core_request":"友好、简洁、鼓励探索","constraints":[{"name":"length","description":"长度限制","params":{"minChars":30,"maxChars":120},"weight":null}],"quality_dimensions":[]}}}`,
} as const

function validateCasesJson(parsed: unknown): string | null {
  if (!Array.isArray(parsed)) return 'cases 必须是数组'

  for (const [index, item] of parsed.entries()) {
    if (typeof item !== 'object' || item === null) {
      return `cases[${index}] 必须是对象`
    }
    const record = item as Record<string, unknown>
    if (typeof record.id !== 'string' || record.id.trim() === '') {
      return `cases[${index}].id 必须是非空字符串`
    }
    if (typeof record.input !== 'object' || record.input === null || Array.isArray(record.input)) {
      return `cases[${index}].input 必须是对象`
    }
    const reference = record.reference
    if (typeof reference !== 'object' || reference === null || Array.isArray(reference)) {
      return `cases[${index}].reference 必须是对象`
    }

    const refRecord = reference as Record<string, unknown>
    const refKeys = Object.keys(refRecord)
    if (refKeys.length !== 1 || !['Exact', 'Constrained', 'Hybrid'].includes(refKeys[0]!)) {
      return `cases[${index}].reference 必须是 Exact / Constrained / Hybrid 之一`
    }

    const variant = refKeys[0]!
    const payload = refRecord[variant]
    if (typeof payload !== 'object' || payload === null || Array.isArray(payload)) {
      return `cases[${index}].reference.${variant} 必须是对象`
    }
    const payloadRecord = payload as Record<string, unknown>

    if (variant === 'Exact') {
      if (typeof payloadRecord.expected !== 'string') {
        return `cases[${index}].reference.Exact.expected 必须是字符串`
      }
    } else if (variant === 'Constrained') {
      if (
        'core_request' in payloadRecord &&
        payloadRecord.core_request !== null &&
        typeof payloadRecord.core_request !== 'string'
      ) {
        return `cases[${index}].reference.Constrained.core_request 必须是 string|null`
      }
      if (!Array.isArray(payloadRecord.constraints)) {
        return `cases[${index}].reference.Constrained.constraints 必须是数组`
      }
      if (!Array.isArray(payloadRecord.quality_dimensions)) {
        return `cases[${index}].reference.Constrained.quality_dimensions 必须是数组`
      }
    } else if (variant === 'Hybrid') {
      const exactParts = payloadRecord.exact_parts
      if (typeof exactParts !== 'object' || exactParts === null || Array.isArray(exactParts)) {
        return `cases[${index}].reference.Hybrid.exact_parts 必须是对象`
      }
      if (!Array.isArray(payloadRecord.constraints)) {
        return `cases[${index}].reference.Hybrid.constraints 必须是数组`
      }
    }
  }

  return null
}

function formatMillis(ms: number): string {
  if (!Number.isFinite(ms)) return '-'
  const date = new Date(ms)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString()
}

export function TestSetsView() {
  const params = useParams()
  const workspaceId = params.id ?? ''

  const { data, isLoading, isFetching, error } = useTestSets(workspaceId)
  const testSets: TestSetListItemResponse[] = data ?? []

  const [isSaveAsTemplateOpen, setIsSaveAsTemplateOpen] = useState(false)
  const [saveAsTemplateSource, setSaveAsTemplateSource] = useState<TestSetListItemResponse | null>(null)
  const [templateName, setTemplateName] = useState('')
  const [templateDescription, setTemplateDescription] = useState('')
  const [localTemplateError, setLocalTemplateError] = useState<string | null>(null)

  const [isTemplatePickerOpen, setIsTemplatePickerOpen] = useState(false)
  const [isApplyingTemplate, setIsApplyingTemplate] = useState(false)
  const [localTemplatePickerError, setLocalTemplatePickerError] = useState<string | null>(null)

  const {
    data: templatesData,
    isLoading: isLoadingTemplates,
    error: templatesError,
  } = useTestSetTemplates(workspaceId, { enabled: isTemplatePickerOpen })
  const templates = templatesData ?? []

  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  const [editingId, setEditingId] = useState<string | null>(null)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [casesJson, setCasesJson] = useState('[]')
  const [localCasesError, setLocalCasesError] = useState<string | null>(null)
  const [saveSuccessMessage, setSaveSuccessMessage] = useState<string | null>(null)
  const [isLoadingEdit, setIsLoadingEdit] = useState(false)
  const [loadingEditId, setLoadingEditId] = useState<string | null>(null)

  type DifyBindingDraft = {
    source: '' | DifyBindingSource
    fixedJsonText: string
    inputKey: string
  }

  type GenericVariableDraft = {
    id: string
    name: string
    valueType: GenericValueType
    defaultValueText: string
  }

  const [difyVariables, setDifyVariables] = useState<DifyInputVariable[] | null>(null)
  const [difyVariablesError, setDifyVariablesError] = useState<string | null>(null)
  const [isRefreshingDifyVariables, setIsRefreshingDifyVariables] = useState(false)

  const [difyTargetPromptVariable, setDifyTargetPromptVariable] = useState('')
  const [difyBindingDrafts, setDifyBindingDrafts] = useState<Record<string, DifyBindingDraft>>({})
  const [difySaveError, setDifySaveError] = useState<string | null>(null)
  const [difySaveSuccess, setDifySaveSuccess] = useState<string | null>(null)

  const [pendingTemplateDifyConfig, setPendingTemplateDifyConfig] = useState<SaveDifyConfigRequest | null>(null)

  const [genericVariableDrafts, setGenericVariableDrafts] = useState<GenericVariableDraft[]>([])
  const [genericSaveError, setGenericSaveError] = useState<string | null>(null)
  const [genericSaveSuccess, setGenericSaveSuccess] = useState<string | null>(null)
  const [isGenericConfigEnabled, setIsGenericConfigEnabled] = useState(false)

  const [importFileName, setImportFileName] = useState<string | null>(null)
  const [importFileError, setImportFileError] = useState<string | null>(null)
  const [importErrors, setImportErrors] = useState<JsonlParseError[]>([])
  const [importTruncatedErrors, setImportTruncatedErrors] = useState(false)
  const [importCases, setImportCases] = useState<TestCase[]>([])
  const [importStats, setImportStats] = useState<ParseStats | null>(null)
  const [isParsingImport, setIsParsingImport] = useState(false)
  const importParseTokenRef = useRef(0)

  const {
    mutateAsync: createTestSet,
    isPending: isCreating,
    error: createError,
  } = useCreateTestSet(workspaceId)
  const { mutateAsync: deleteTestSet, isPending: isDeleting } = useDeleteTestSet(workspaceId)

  const {
    mutateAsync: updateTestSet,
    isPending: isUpdating,
    error: updateError,
  } = useUpdateTestSet(workspaceId, editingId ?? '')

  const {
    mutateAsync: saveAsTemplateMutate,
    isPending: isSavingTemplate,
    error: saveTemplateError,
    reset: resetSaveTemplateMutation,
  } = useSaveAsTemplate(workspaceId)

  const isSaving = isCreating || isUpdating
  const listErrorMessage = error instanceof Error ? error.message : '加载失败'
  const saveError = createError ?? updateError
  const saveErrorMessage = saveError instanceof Error ? saveError.message : null
  const saveTemplateErrorMessage = saveTemplateError instanceof Error ? saveTemplateError.message : null

  const title = useMemo(() => (editingId ? '编辑测试集' : '创建测试集'), [editingId])

  const applyDifyConfigToDrafts = (config: DifyConfig | null) => {
    setDifyTargetPromptVariable(config?.targetPromptVariable ?? '')

    const nextDrafts: Record<string, DifyBindingDraft> = {}
    const bindings = config?.bindings ?? {}
    for (const [name, binding] of Object.entries(bindings)) {
      if (!binding) continue
      if (binding.source === 'fixed') {
        nextDrafts[name] = {
          source: 'fixed',
          fixedJsonText: JSON.stringify(binding.value, null, 2),
          inputKey: '',
        }
      } else {
        nextDrafts[name] = {
          source: 'testCaseInput',
          fixedJsonText: '',
          inputKey: binding.inputKey ?? '',
        }
      }
    }
    setDifyBindingDrafts(nextDrafts)
    setDifySaveError(null)
    setDifySaveSuccess(null)
  }

  const makeLocalId = () =>
    typeof crypto !== 'undefined' && 'randomUUID' in crypto
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`

  const formatGenericDefaultValueText = (valueType: GenericValueType, value: JsonValue | null): string => {
    if (value === null) return ''
    if (valueType === 'json') return JSON.stringify(value, null, 2)
    if (valueType === 'string') return typeof value === 'string' ? value : ''
    if (valueType === 'number') return typeof value === 'number' ? String(value) : ''
    if (valueType === 'boolean') return typeof value === 'boolean' ? (value ? 'true' : 'false') : ''
    return ''
  }

  const applyGenericConfigToDrafts = (config: GenericConfig | null) => {
    const variables = config?.variables ?? []
    setIsGenericConfigEnabled(Boolean(config))
    setGenericVariableDrafts(
      variables.map((v) => ({
        id: makeLocalId(),
        name: v.name ?? '',
        valueType: (v.valueType ?? 'string') as GenericValueType,
        defaultValueText: formatGenericDefaultValueText((v.valueType ?? 'string') as GenericValueType, v.defaultValue),
      }))
    )
    setGenericSaveError(null)
    setGenericSaveSuccess(null)
  }

  const startEdit = async (ts: TestSetListItemResponse) => {
    if (!workspaceId) return
    if (authStatus !== 'authenticated' || !sessionToken) return

    setIsLoadingEdit(true)
    setLoadingEditId(ts.id)
    try {
      const full = await getTestSet(workspaceId, ts.id, sessionToken)
      setEditingId(full.id)
      setName(full.name)
      setDescription(full.description ?? '')
      setCasesJson(JSON.stringify(full.cases, null, 2))
      setDifyVariables(null)
      setDifyVariablesError(null)
      applyDifyConfigToDrafts(full.dify_config)
      setPendingTemplateDifyConfig(null)
      applyGenericConfigToDrafts(full.generic_config)
      setLocalCasesError(null)
      setSaveSuccessMessage(null)
    } catch (e) {
      setLocalCasesError(e instanceof Error ? e.message : '加载测试集失败')
    } finally {
      setIsLoadingEdit(false)
      setLoadingEditId(null)
    }
  }

  const resetForm = (options?: { keepSuccessMessage?: boolean }) => {
    setEditingId(null)
    setName('')
    setDescription('')
    setCasesJson('[]')
    setLocalCasesError(null)
    setDifyVariables(null)
    setDifyVariablesError(null)
    setIsRefreshingDifyVariables(false)
    setDifyTargetPromptVariable('')
    setDifyBindingDrafts({})
    setDifySaveError(null)
    setDifySaveSuccess(null)
    setPendingTemplateDifyConfig(null)
    setGenericVariableDrafts([])
    setGenericSaveError(null)
    setGenericSaveSuccess(null)
    setIsGenericConfigEnabled(false)
    if (!options?.keepSuccessMessage) setSaveSuccessMessage(null)
  }

  const openSaveAsTemplate = (ts: TestSetListItemResponse) => {
    resetSaveTemplateMutation()
    setSaveAsTemplateSource(ts)
    setTemplateName(ts.name)
    setTemplateDescription(ts.description ?? '')
    setLocalTemplateError(null)
    setSaveSuccessMessage(null)
    setIsSaveAsTemplateOpen(true)
  }

  const closeSaveAsTemplate = () => {
    resetSaveTemplateMutation()
    setIsSaveAsTemplateOpen(false)
    setSaveAsTemplateSource(null)
    setLocalTemplateError(null)
  }

  const handleConfirmSaveAsTemplate = async () => {
    if (!workspaceId) return
    if (authStatus !== 'authenticated' || !sessionToken) return
    if (!saveAsTemplateSource) return

    const trimmedName = templateName.trim()
    if (!trimmedName) {
      setLocalTemplateError('模板名称不能为空')
      return
    }
    if (Array.from(trimmedName).length > 128) {
      setLocalTemplateError('模板名称不能超过 128 个字符')
      return
    }

    setLocalTemplateError(null)
    setSaveSuccessMessage(null)

    try {
      await saveAsTemplateMutate({
        testSetId: saveAsTemplateSource.id,
        params: {
          name: trimmedName,
          description: templateDescription.trim() ? templateDescription.trim() : null,
        },
      })
      setSaveSuccessMessage('已保存为模板')
      closeSaveAsTemplate()
    } catch {
      // 错误由 saveTemplateErrorMessage 渲染；保持弹窗打开便于用户修正。
    }
  }

  const closeTemplatePicker = () => {
    setIsTemplatePickerOpen(false)
    setLocalTemplatePickerError(null)
  }

  const applyTemplateToForm = async (templateId: string) => {
    if (!workspaceId) return
    if (authStatus !== 'authenticated' || !sessionToken) return

    setIsApplyingTemplate(true)
    setLocalTemplatePickerError(null)
    setLocalCasesError(null)
    setSaveSuccessMessage(null)
    try {
      const tpl = await getTestSetTemplate(workspaceId, templateId, sessionToken)
      clearImport()
      resetForm()
      setName(tpl.name)
      setDescription(tpl.description ?? '')
      setCasesJson(JSON.stringify(tpl.cases, null, 2))
      applyGenericConfigToDrafts(tpl.generic_config)

      const hasDify = Boolean(tpl.dify_config)
      const hasGeneric = Boolean(tpl.generic_config)

      if (tpl.dify_config) {
        setPendingTemplateDifyConfig({
          targetPromptVariable: tpl.dify_config.targetPromptVariable,
          bindings: tpl.dify_config.bindings,
        })
      } else {
        setPendingTemplateDifyConfig(null)
      }

      if (hasDify && hasGeneric) setSaveSuccessMessage('已从模板预填（含 Dify 配置/通用变量）')
      else if (hasDify) setSaveSuccessMessage('已从模板预填（含 Dify 配置）')
      else if (hasGeneric) setSaveSuccessMessage('已从模板预填（含通用变量）')
      else setSaveSuccessMessage('已从模板预填')

      closeTemplatePicker()
    } catch (e) {
      setLocalTemplatePickerError(e instanceof Error ? e.message : '加载模板失败')
    } finally {
      setIsApplyingTemplate(false)
    }
  }

  const cancelImportParsing = () => {
    importParseTokenRef.current += 1
  }

  const clearImport = () => {
    cancelImportParsing()
    setImportFileName(null)
    setImportFileError(null)
    setImportErrors([])
    setImportTruncatedErrors(false)
    setImportCases([])
    setImportStats(null)
    setIsParsingImport(false)
  }

  const parseImportFile = async (file: File) => {
    cancelImportParsing()
    const token = importParseTokenRef.current
    const isActive = () => token === importParseTokenRef.current

    setImportFileName(file.name)
    setImportFileError(null)
    setImportErrors([])
    setImportTruncatedErrors(false)
    setImportCases([])
    setImportStats(null)
    setSaveSuccessMessage(null)
    setLocalCasesError(null)

    const MAX_BYTES = 5 * 1024 * 1024
    if (file.size > MAX_BYTES) {
      setImportFileError('文件过大：最大支持 5MB')
      return
    }
    if (!file.name.toLowerCase().endsWith('.txt')) {
      setImportFileError('仅支持 .txt 文件（JSONL）')
      return
    }

    setIsParsingImport(true)
    try {
      const text = await file.text()
      const res = await parseTestCasesJsonl(text, {
        progressEvery: 100,
        yieldEvery: 100,
        onProgress: (stats) => {
          if (!isActive()) return
          setImportStats(stats)
        },
      })
      if (!isActive()) return
      setImportCases(res.cases)
      setImportErrors(res.errors)
      setImportTruncatedErrors(res.truncatedErrors)
      setImportStats(res.stats)
    } catch (e) {
      if (!isActive()) return
      setImportFileError(e instanceof Error ? e.message : '解析失败')
    } finally {
      if (isActive()) setIsParsingImport(false)
    }
  }

  const importHasBlockingError =
    Boolean(importFileError) || importErrors.length > 0 || (importFileName !== null && isParsingImport)

  const applyImportToCases = async () => {
    if (importCases.length === 0) return
    if (importHasBlockingError) return

    let shouldOverwrite = true
    try {
      const parsed = JSON.parse(casesJson) as unknown
      if (Array.isArray(parsed) && parsed.length > 0) {
        shouldOverwrite = window.confirm('当前 cases 不为空，确定要覆盖为导入结果吗？')
      }
    } catch {
      shouldOverwrite = window.confirm('当前 cases 不是合法 JSON，确定要覆盖为导入结果吗？')
    }

    if (!shouldOverwrite) return
    setCasesJson(JSON.stringify(importCases, null, 2))
    setLocalCasesError(null)
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!workspaceId) return
    if (!name.trim()) return

    if (importFileName !== null) {
      if (isParsingImport) {
        setLocalCasesError('批量导入正在解析中，请稍候...')
        return
      }
      if (importFileError || importErrors.length > 0) {
        setLocalCasesError('批量导入解析失败，请先修复文件或清除导入结果后再保存。')
        return
      }
    }

    let parsed: unknown
    try {
      parsed = JSON.parse(casesJson)
    } catch {
      setLocalCasesError('JSON 无法解析')
      return
    }

    const validationError = validateCasesJson(parsed)
    if (validationError) {
      setLocalCasesError(validationError)
      return
    }

    setLocalCasesError(null)
    setSaveSuccessMessage(null)

    const cases = parsed as JsonValue

    const builtGeneric = isGenericConfigEnabled
      ? buildGenericConfigRequestFromDrafts(genericVariableDrafts)
      : null

    if (builtGeneric && !builtGeneric.ok) {
      setGenericSaveError(builtGeneric.error)
      setGenericSaveSuccess(null)
      return
    }

    if (editingId) {
      await updateTestSet({
        name: name.trim(),
        description: description.trim() ? description.trim() : null,
        cases,
      })
      setSaveSuccessMessage('保存成功')
      resetForm({ keepSuccessMessage: true })
      return
    }

    const includeDify = Boolean(pendingTemplateDifyConfig)
    const includeGeneric = Boolean(builtGeneric && builtGeneric.ok)

    await createTestSet({
      name: name.trim(),
      description: description.trim() ? description.trim() : null,
      cases,
      dify_config: pendingTemplateDifyConfig,
      generic_config: builtGeneric && builtGeneric.ok ? builtGeneric.req : null,
    })

    if (!includeDify && !includeGeneric) setSaveSuccessMessage('创建成功')
    else if (includeDify && includeGeneric) setSaveSuccessMessage('创建成功（含 Dify 配置/通用变量）')
    else if (includeDify) setSaveSuccessMessage('创建成功（含 Dify 配置）')
    else setSaveSuccessMessage('创建成功（含通用变量）')
    resetForm({ keepSuccessMessage: true })
  }

  const handleDelete = async (ts: TestSetListItemResponse) => {
    if (!workspaceId) return
    const confirmed = window.confirm(`确定删除测试集「${ts.name}」？`)
    if (!confirmed) return
    await deleteTestSet(ts.id)
    if (editingId === ts.id) resetForm()
  }

  const casesForExpectedEditor = useMemo(() => {
    try {
      const parsed = JSON.parse(casesJson) as unknown
      const validationError = validateCasesJson(parsed)
      if (validationError) return null
      return parsed as TestCase[]
    } catch {
      return null
    }
  }, [casesJson])

  const updateExactExpectedAtIndex = (index: number, expected: string) => {
    setCasesJson((prev) => {
      try {
        const parsed = JSON.parse(prev) as unknown
        if (!Array.isArray(parsed)) return prev
        if (index < 0 || index >= parsed.length) return prev
        const next = parsed.map((item, i) => {
          if (i !== index) return item
          if (typeof item !== 'object' || item === null) return item
          const record = item as Record<string, unknown>
          const reference = record.reference
          if (typeof reference !== 'object' || reference === null || Array.isArray(reference)) return item
          const refRecord = reference as Record<string, unknown>
          const exact = refRecord.Exact
          if (typeof exact !== 'object' || exact === null || Array.isArray(exact)) return item
          return {
            ...record,
            reference: {
              ...refRecord,
              Exact: { ...(exact as Record<string, unknown>), expected },
            },
          }
        })
        return JSON.stringify(next, null, 2)
      } catch {
        return prev
      }
    })
  }

  const isPlainObject = (value: unknown): value is Record<string, unknown> =>
    typeof value === 'object' && value !== null && !Array.isArray(value)

  const parseOptionalNonNegativeInt = (raw: string): number | undefined => {
    const trimmed = raw.trim()
    if (!trimmed) return undefined
    const num = Number(trimmed)
    if (!Number.isFinite(num)) return undefined
    const floored = Math.floor(num)
    if (floored < 0) return undefined
    return floored
  }

  const parseKeywords = (raw: string): string[] =>
    raw
      .split(/[\n,，]+/g)
      .map((s) => s.trim())
      .filter((s) => s.length > 0)

  const updateConstrainedAtIndex = (
    index: number,
    updater: (constrained: Record<string, unknown>) => Record<string, unknown>
  ) => {
    setCasesJson((prev) => {
      try {
        const parsed = JSON.parse(prev) as unknown
        if (!Array.isArray(parsed)) return prev
        if (index < 0 || index >= parsed.length) return prev

        const next = parsed.map((item, i) => {
          if (i !== index) return item
          if (!isPlainObject(item)) return item
          const reference = item.reference
          if (!isPlainObject(reference)) return item

          const constrained = reference.Constrained
          if (!isPlainObject(constrained)) return item

          return {
            ...item,
            reference: {
              ...reference,
              Constrained: updater(constrained),
            },
          }
        })

        return JSON.stringify(next, null, 2)
      } catch {
        return prev
      }
    })
  }

  const updateCoreRequestAtIndex = (index: number, coreRequest: string) => {
    updateConstrainedAtIndex(index, (constrained) => {
      const trimmed = coreRequest.trim()
      if (!trimmed) {
        const next = { ...constrained }
        delete next.core_request
        return next
      }
      return { ...constrained, core_request: coreRequest }
    })
  }

  const updateConstraintAtIndex = (
    index: number,
    name: string,
    description: string,
    paramsUpdater: (prevParams: Record<string, unknown> | null) => Record<string, unknown> | null
  ) => {
    updateConstrainedAtIndex(index, (constrained) => {
      const rawConstraints = constrained.constraints
      const constraints = Array.isArray(rawConstraints) ? rawConstraints.slice() : []

      const firstMatchIndex = constraints.findIndex(
        (c) => isPlainObject(c) && c.name === name
      )

      const existing = firstMatchIndex >= 0 && isPlainObject(constraints[firstMatchIndex])
        ? (constraints[firstMatchIndex] as Record<string, unknown>)
        : null

      const prevParamsRaw = existing?.params
      const prevParams = isPlainObject(prevParamsRaw) ? (prevParamsRaw as Record<string, unknown>) : null
      const nextParams = paramsUpdater(prevParams)

      if (nextParams === null) {
        if (firstMatchIndex >= 0) constraints.splice(firstMatchIndex, 1)
        return { ...constrained, constraints }
      }

      const existingDescription =
        typeof existing?.description === 'string' ? existing.description : undefined
      const base = {
        name,
        description: existingDescription ?? description,
        params: nextParams,
      }

      if (firstMatchIndex >= 0) {
        if (existing) constraints[firstMatchIndex] = { ...existing, ...base }
        return { ...constrained, constraints }
      }

      constraints.push({ ...base, weight: null })
      return { ...constrained, constraints }
    })
  }

  const updateLengthMinAtIndex = (index: number, raw: string) => {
    const nextMin = parseOptionalNonNegativeInt(raw)
    updateConstraintAtIndex(index, 'length', '长度限制', (prevParams) => {
      const base = prevParams ? { ...prevParams } : {}
      const prevMax = typeof base.maxChars === 'number' ? base.maxChars : undefined

      if (nextMin === undefined) delete base.minChars
      else base.minChars = nextMin

      if (prevMax === undefined) delete base.maxChars
      else base.maxChars = prevMax

      const min = typeof base.minChars === 'number' ? base.minChars : undefined
      const max = typeof base.maxChars === 'number' ? base.maxChars : undefined
      if (min === undefined && max === undefined) return null
      return base
    })
  }

  const updateLengthMaxAtIndex = (index: number, raw: string) => {
    const nextMax = parseOptionalNonNegativeInt(raw)
    updateConstraintAtIndex(index, 'length', '长度限制', (prevParams) => {
      const base = prevParams ? { ...prevParams } : {}
      const prevMin = typeof base.minChars === 'number' ? base.minChars : undefined

      if (prevMin === undefined) delete base.minChars
      else base.minChars = prevMin

      if (nextMax === undefined) delete base.maxChars
      else base.maxChars = nextMax

      const min = typeof base.minChars === 'number' ? base.minChars : undefined
      const max = typeof base.maxChars === 'number' ? base.maxChars : undefined
      if (min === undefined && max === undefined) return null
      return base
    })
  }

  const updateMustIncludeAtIndex = (index: number, raw: string) => {
    const keywords = parseKeywords(raw)
    updateConstraintAtIndex(
      index,
      'must_include',
      '必含关键词',
      (prevParams) => (keywords.length ? { ...(prevParams ?? {}), keywords } : null)
    )
  }

  const updateMustExcludeAtIndex = (index: number, raw: string) => {
    const keywords = parseKeywords(raw)
    updateConstraintAtIndex(
      index,
      'must_exclude',
      '禁止内容',
      (prevParams) => (keywords.length ? { ...(prevParams ?? {}), keywords } : null)
    )
  }

  const updateFormatAtIndex = (index: number, raw: string) => {
    const trimmed = raw.trim()
    updateConstraintAtIndex(
      index,
      'format',
      '格式要求',
      (prevParams) => (trimmed ? { ...(prevParams ?? {}), format: trimmed } : null)
    )
  }

  const sampleInputKeys = useMemo(() => {
    try {
      const parsed = JSON.parse(casesJson) as unknown
      if (!Array.isArray(parsed)) return []
      const first = parsed[0] as unknown
      if (typeof first !== 'object' || first === null) return []
      const record = first as Record<string, unknown>
      const input = record.input
      if (typeof input !== 'object' || input === null || Array.isArray(input)) return []
      return Object.keys(input as Record<string, unknown>).sort()
    } catch {
      return []
    }
  }, [casesJson])

  const handleRefreshDifyVariables = async () => {
    if (!workspaceId || !editingId) return
    if (authStatus !== 'authenticated' || !sessionToken) return

    setIsRefreshingDifyVariables(true)
    setDifyVariablesError(null)
    setDifySaveError(null)
    setDifySaveSuccess(null)
    try {
      const res = await refreshDifyVariables(workspaceId, editingId, sessionToken)
      setDifyVariables(res.variables)
    } catch (e) {
      setDifyVariablesError(e instanceof Error ? e.message : '解析变量失败')
    } finally {
      setIsRefreshingDifyVariables(false)
    }
  }

  const handleSaveDifyConfig = async () => {
    if (!workspaceId || !editingId) return
    if (authStatus !== 'authenticated' || !sessionToken) return

    const target = difyTargetPromptVariable.trim()
    if (!target) {
      setDifySaveError('请选择待优化 system prompt 变量')
      setDifySaveSuccess(null)
      return
    }

    const bindings: SaveDifyConfigRequest['bindings'] = {}
    for (const [name, draft] of Object.entries(difyBindingDrafts)) {
      if (!draft.source) continue
      if (name === target) continue

      if (draft.source === 'fixed') {
        try {
          const parsed = JSON.parse(draft.fixedJsonText)
          bindings[name] = { source: 'fixed', value: parsed as JsonValue, inputKey: null }
        } catch {
          setDifySaveError(`变量 ${name} 的固定值不是合法 JSON`)
          setDifySaveSuccess(null)
          return
        }
      } else {
        const key = draft.inputKey.trim()
        if (!key) {
          setDifySaveError(`变量 ${name} 的 inputKey 不能为空`)
          setDifySaveSuccess(null)
          return
        }
        bindings[name] = { source: 'testCaseInput', value: null, inputKey: key }
      }
    }

    setDifySaveError(null)
    setDifySaveSuccess(null)
    try {
      const res = await saveDifyConfig(
        workspaceId,
        editingId,
        { targetPromptVariable: target, bindings },
        sessionToken
      )
      applyDifyConfigToDrafts(res.difyConfig)
      setDifySaveSuccess('保存成功')
    } catch (e) {
      setDifySaveError(e instanceof Error ? e.message : '保存失败')
    }
  }

  const handleAddGenericVariable = () => {
    setIsGenericConfigEnabled(true)
    setGenericVariableDrafts((prev) => [
      ...prev,
      { id: makeLocalId(), name: '', valueType: 'string', defaultValueText: '' },
    ])
    setGenericSaveError(null)
    setGenericSaveSuccess(null)
  }

  const buildGenericConfigRequestFromDrafts = (
    drafts: GenericVariableDraft[]
  ): { ok: true; req: SaveGenericConfigRequest } | { ok: false; error: string } => {
    const seen = new Set<string>()
    const variables: GenericInputVariable[] = []

    for (const draft of drafts) {
      const name = draft.name.trim()
      if (!name) return { ok: false, error: '变量名不能为空' }
      if (Array.from(name).length > 128) return { ok: false, error: '变量名不能超过 128 个字符' }
      if (seen.has(name)) return { ok: false, error: '变量名必须唯一' }
      seen.add(name)

      const raw = draft.defaultValueText.trim()
      let defaultValue: JsonValue | null = null

      if (raw) {
        if (draft.valueType === 'string') {
          defaultValue = draft.defaultValueText
        } else if (draft.valueType === 'number') {
          const num = Number(raw)
          if (!Number.isFinite(num)) return { ok: false, error: `变量 ${name} 的默认值必须是数字` }
          defaultValue = num
        } else if (draft.valueType === 'boolean') {
          if (raw !== 'true' && raw !== 'false') return { ok: false, error: `变量 ${name} 的默认值必须是 true/false` }
          defaultValue = raw === 'true'
        } else {
          try {
            defaultValue = JSON.parse(draft.defaultValueText) as JsonValue
          } catch {
            return { ok: false, error: `变量 ${name} 的默认值不是合法 JSON` }
          }
        }
      }

      variables.push({ name, valueType: draft.valueType, defaultValue })
    }

    const req: SaveGenericConfigRequest = { variables }
    const bytes = new TextEncoder().encode(JSON.stringify(req)).length
    if (bytes > 32 * 1024) return { ok: false, error: '配置过大：最大 32KB' }

    return { ok: true, req }
  }

  const handleSaveGenericConfig = async () => {
    if (!workspaceId) return

    if (!isGenericConfigEnabled) {
      setGenericSaveError(null)
      setGenericSaveSuccess('未启用通用变量配置')
      return
    }

    const built = buildGenericConfigRequestFromDrafts(genericVariableDrafts)
    if (!built.ok) {
      setGenericSaveError(built.error)
      setGenericSaveSuccess(null)
      return
    }

    setGenericSaveError(null)
    setGenericSaveSuccess(null)

    if (!editingId) {
      setGenericSaveSuccess('已校验：将在创建测试集时写入')
      return
    }

    if (authStatus !== 'authenticated' || !sessionToken) return

    try {
      const res = await saveGenericConfig(workspaceId, editingId, built.req, sessionToken)
      applyGenericConfigToDrafts(res.genericConfig)
      setGenericSaveSuccess('保存成功')
    } catch (e) {
      setGenericSaveError(e instanceof Error ? e.message : '保存失败')
    }
  }

  const handleDisableGenericConfig = async () => {
    const confirmed = window.confirm('确定禁用并清空通用变量配置？')
    if (!confirmed) return

    if (!editingId) {
      setIsGenericConfigEnabled(false)
      setGenericVariableDrafts([])
      setGenericSaveError(null)
      setGenericSaveSuccess('已禁用')
      return
    }

    if (authStatus !== 'authenticated' || !sessionToken) return

    try {
      await deleteGenericConfig(workspaceId, editingId, sessionToken)
      setIsGenericConfigEnabled(false)
      setGenericVariableDrafts([])
      setGenericSaveError(null)
      setGenericSaveSuccess('已禁用并清空')
    } catch (e) {
      setGenericSaveError(e instanceof Error ? e.message : '清空失败')
      setGenericSaveSuccess(null)
    }
  }

  return (
    <section className="mx-auto flex max-w-5xl flex-col gap-6 px-4 py-6" data-testid="test-sets-view">
      {!isLoading && isFetching && testSets.length > 0 && (
        <div className="text-xs text-muted-foreground">加载中...</div>
      )}
      {isSaveAsTemplateOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          role="dialog"
          aria-modal="true"
        >
          <Card className="w-full max-w-md">
            <CardHeader>
              <CardTitle>保存为模板</CardTitle>
              <CardDescription>
                来源：{saveAsTemplateSource?.name ?? '-'}
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-3">
              <div className="grid gap-2">
                <Label htmlFor="template-name">模板名称</Label>
                <Input
                  id="template-name"
                  value={templateName}
                  onChange={(e) => setTemplateName(e.target.value)}
                  placeholder="例如：客服对话模板"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="template-description">描述（可选）</Label>
                <Input
                  id="template-description"
                  value={templateDescription}
                  onChange={(e) => setTemplateDescription(e.target.value)}
                  placeholder="可选"
                />
              </div>

              {localTemplateError && (
                <div className="text-sm text-red-500">校验失败：{localTemplateError}</div>
              )}
              {saveTemplateErrorMessage && (
                <div className="text-sm text-red-500">保存失败：{saveTemplateErrorMessage}</div>
              )}
            </CardContent>
            <CardFooter className="justify-end gap-2">
              <Button type="button" variant="outline" onClick={closeSaveAsTemplate} disabled={isSavingTemplate}>
                取消
              </Button>
              <Button type="button" onClick={() => void handleConfirmSaveAsTemplate()} disabled={isSavingTemplate}>
                {isSavingTemplate ? '保存中...' : '保存'}
              </Button>
            </CardFooter>
          </Card>
        </div>
      )}

      {isTemplatePickerOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          role="dialog"
          aria-modal="true"
        >
          <Card className="w-full max-w-2xl">
            <CardHeader>
              <CardTitle>从模板创建</CardTitle>
              <CardDescription>选择一个模板后，将自动预填 name/description/cases。</CardDescription>
            </CardHeader>
            <CardContent>
              {isLoadingTemplates && (
                <div className="text-sm text-muted-foreground">加载模板中...</div>
              )}
              {templatesError && (
                <div className="text-sm text-red-500">
                  加载失败：{templatesError instanceof Error ? templatesError.message : '加载模板失败'}
                </div>
              )}
              {localTemplatePickerError && (
                <div className="text-sm text-red-500">操作失败：{localTemplatePickerError}</div>
              )}

              {!isLoadingTemplates && !templatesError && templates.length === 0 && (
                <div className="text-sm text-muted-foreground">暂无模板，请先从一个测试集「保存为模板」。</div>
              )}

              {!isLoadingTemplates && !templatesError && templates.length > 0 && (
                <ul className="space-y-2 text-sm">
                  {templates.map((tpl) => (
                    <li key={tpl.id} className="flex items-start justify-between gap-3 rounded-md border px-3 py-2">
                      <div className="min-w-0">
                        <div className="font-medium">{tpl.name}</div>
                        <div className="text-muted-foreground">
                          {tpl.description || '暂无描述'} · {tpl.cases_count} 条用例 · {formatMillis(tpl.created_at)}
                        </div>
                      </div>
                      <div className="shrink-0">
                        <Button
                          type="button"
                          size="sm"
                          onClick={() => void applyTemplateToForm(tpl.id)}
                          disabled={isApplyingTemplate}
                        >
                          {isApplyingTemplate ? '加载中...' : '使用'}
                        </Button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </CardContent>
            <CardFooter className="justify-end">
              <Button type="button" variant="outline" onClick={closeTemplatePicker} disabled={isApplyingTemplate}>
                关闭
              </Button>
            </CardFooter>
          </Card>
        </div>
      )}

      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">测试集管理</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Workspace：{workspaceId || '(missing)'}（<Link className="underline" to="/workspace">返回工作区列表</Link>）
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>测试集列表</CardTitle>
          <CardDescription>在当前 workspace 下创建/编辑/删除测试集。</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading && <div className="text-sm text-muted-foreground">加载中...</div>}

          {error && (
            <div className="text-sm text-red-500">
              加载失败：{listErrorMessage}
            </div>
          )}

          {!isLoading && !error && testSets.length === 0 && (
            <div className="text-sm text-muted-foreground">暂无测试集，请先创建一个。</div>
          )}

          {!isLoading && !error && testSets.length > 0 && (
            <ul className="space-y-2 text-sm">
              {testSets.map((ts) => (
                <li key={ts.id} className="flex items-start justify-between gap-3 rounded-md border px-3 py-2">
                  <div className="min-w-0">
                    <div className="font-medium">{ts.name}</div>
                    <div className="text-muted-foreground">
                      {ts.description || '暂无描述'} · {ts.cases_count} 条用例
                    </div>
                  </div>
                  <div className="flex shrink-0 gap-2">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      onClick={() => startEdit(ts)}
                      disabled={isSaving || isDeleting || isLoadingEdit}
                    >
                      {isLoadingEdit && loadingEditId === ts.id ? '加载中...' : '编辑'}
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      onClick={() => openSaveAsTemplate(ts)}
                      disabled={isSaving || isDeleting || isLoadingEdit}
                    >
                      保存为模板
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="destructive"
                      onClick={() => handleDelete(ts)}
                      disabled={isSaving || isDeleting || isLoadingEdit}
                    >
                      删除
                    </Button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <CardTitle>{title}</CardTitle>
              <CardDescription>cases 使用 JSON 编辑（最小校验：id/input/reference）。</CardDescription>
            </div>
            <div className="shrink-0">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => {
                  setSaveSuccessMessage(null)
                  setLocalTemplatePickerError(null)
                  setIsTemplatePickerOpen(true)
                }}
                disabled={!workspaceId || authStatus !== 'authenticated' || !sessionToken}
              >
                从模板创建
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
            <div className="rounded-md border p-3">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="font-medium">批量导入（txt）</div>
                  <div className="mt-1 text-sm text-muted-foreground">
                    {JSONL_FORMAT_HELP.description}（最大 5MB）
                  </div>
                </div>
                <div className="shrink-0">
                  <Button type="button" variant="outline" size="sm" onClick={clearImport}>
                    清除
                  </Button>
                </div>
              </div>

              <div
                className="mt-3 rounded-md border border-dashed p-3 text-sm"
                onDragOver={(e) => e.preventDefault()}
                onDrop={(e) => {
                  e.preventDefault()
                  const file = e.dataTransfer.files?.[0]
                  if (file) void parseImportFile(file)
                }}
              >
                <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                  <div className="text-muted-foreground">拖拽 .txt 到这里，或使用文件选择</div>
                  <Input
                    type="file"
                    accept=".txt,text/plain"
                    onChange={(e) => {
                      const file = e.target.files?.[0]
                      if (file) void parseImportFile(file)
                    }}
                  />
                </div>

                {importFileName && (
                  <div className="mt-2 text-xs text-muted-foreground">
                    当前文件：{importFileName}
                  </div>
                )}

                {importStats && (
                  <div className="mt-2 text-xs text-muted-foreground">
                    进度：{importStats.processedLines}/{importStats.totalLines} 行 · 成功 {importStats.ok} 条 · 失败{' '}
                    {importStats.failed} 条
                  </div>
                )}

                {importFileError && (
                  <div className="mt-2 text-sm text-red-500">导入失败：{importFileError}</div>
                )}

                {importErrors.length > 0 && (
                  <div className="mt-2">
                    <div className="text-sm text-red-500">格式错误（按行）：</div>
                    <ul className="mt-1 max-h-48 overflow-auto rounded-md border bg-muted/30 p-2 text-xs">
                      {importErrors.map((err) => (
                        <li key={`${err.line}-${err.message}`} className="rounded px-1 py-0.5 hover:bg-muted">
                          第 {err.line} 行：{err.message}
                        </li>
                      ))}
                    </ul>
                    {importTruncatedErrors && (
                      <div className="mt-1 text-xs text-muted-foreground">仅显示前 50 条错误，文件中还有更多。</div>
                    )}

                    <div className="mt-3 text-sm font-medium">{JSONL_FORMAT_HELP.title}</div>
                    <div className="mt-1 text-xs text-muted-foreground">
                      `reference` 必须是单 key 变体对象（Exact / Constrained / Hybrid）。
                    </div>
                    <pre className="mt-2 overflow-auto rounded-md border bg-muted/30 p-2 text-xs">
                      {JSONL_FORMAT_HELP.example}
                    </pre>
                  </div>
                )}

                {!importHasBlockingError && importCases.length > 0 && (
                  <div className="mt-3">
                    <div className="text-sm text-muted-foreground">
                      预览：共 {importCases.length} 条（仅展示前 3 条）
                    </div>
                    <ul className="mt-2 space-y-1 text-xs">
                      {importCases.slice(0, 3).map((tc) => (
                        <li key={tc.id} className="rounded-md border bg-muted/20 p-2">
                          <div className="font-mono">id: {tc.id}</div>
                          <details className="mt-1">
                            <summary className="cursor-pointer text-muted-foreground">查看 JSON</summary>
                            <pre className="mt-1 overflow-auto rounded-md border bg-muted/30 p-2">
                              {JSON.stringify(tc, null, 2)}
                            </pre>
                          </details>
                        </li>
                      ))}
                    </ul>

                    <div className="mt-3 flex items-center gap-2">
                      <Button type="button" onClick={() => void applyImportToCases()} disabled={isParsingImport}>
                        应用到 cases(JSON)
                      </Button>
                      <div className="text-xs text-muted-foreground">默认覆盖当前 cases</div>
                    </div>
                  </div>
                )}
              </div>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="test-set-name">名称</Label>
              <Input
                id="test-set-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="例如：客服对话测试集"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="test-set-description">描述</Label>
              <Input
                id="test-set-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder="可选"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="test-set-cases">cases (JSON)</Label>
              <textarea
                id="test-set-cases"
                className="min-h-48 w-full rounded-md border border-input bg-background px-3 py-2 font-mono text-xs shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                value={casesJson}
                onChange={(event) => setCasesJson(event.target.value)}
                placeholder="[]"
              />
              {localCasesError && (
                <div className="text-sm text-red-500">校验失败：{localCasesError}</div>
              )}
            </div>

            {casesForExpectedEditor && casesForExpectedEditor.some((c) => 'Exact' in c.reference) && (
              <div className="grid gap-3 rounded-md border border-input p-3">
                <div className="grid gap-1">
                  <div className="text-sm font-medium">标准答案（固定任务）</div>
                  <div className="text-xs text-muted-foreground">
                    将写入 <span className="font-mono">cases[*].reference.Exact.expected</span>（JSON 编辑仍为最终数据源）。
                  </div>
                </div>
                <div className="grid gap-3">
                  {casesForExpectedEditor.map((c, index) => (
                    <div key={c.id} className="grid gap-2">
                      <div className="text-xs text-muted-foreground">{c.id}</div>
                      {'Exact' in c.reference ? (
                        <textarea
                          className="min-h-16 w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                          value={c.reference.Exact.expected}
                          onChange={(e) => updateExactExpectedAtIndex(index, e.target.value)}
                          placeholder="输入标准答案（期望输出）"
                        />
                      ) : (
                        <div className="text-xs text-muted-foreground">
                          该用例 reference 不是 Exact（不会修改）
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {casesForExpectedEditor &&
              casesForExpectedEditor.some((c) => 'Constrained' in c.reference) && (
                <div className="grid gap-3 rounded-md border border-input p-3">
                  <div className="grid gap-1">
                    <div className="text-sm font-medium">创意任务配置（Constrained）</div>
                    <div className="text-xs text-muted-foreground">
                      将写入{' '}
                      <span className="font-mono">
                        cases[*].reference.Constrained.core_request / constraints[*].params
                      </span>
                      （JSON 编辑仍为最终数据源）。
                    </div>
                  </div>
                  <div className="grid gap-4">
                    {casesForExpectedEditor.map((c, index) => {
                      if (!('Constrained' in c.reference)) {
                        return (
                          <div key={c.id} className="grid gap-2">
                            <div className="text-xs text-muted-foreground">{c.id}</div>
                            <div className="text-xs text-muted-foreground">
                              该用例 reference 不是 Constrained（不会修改）
                            </div>
                          </div>
                        )
                      }

                      const constrained = c.reference.Constrained as unknown as Record<string, unknown>
                      const constraints = Array.isArray(constrained.constraints)
                        ? (constrained.constraints as unknown[])
                        : []

                      const findConstraint = (name: string) =>
                        constraints.find(
                          (it) => isPlainObject(it) && it.name === name
                        ) as Record<string, unknown> | undefined

                      const getParamsRaw = (name: string) => findConstraint(name)?.params
                      const getParamsObject = (name: string) => {
                        const raw = getParamsRaw(name)
                        return isPlainObject(raw) ? (raw as Record<string, unknown>) : null
                      }
                      const hasNonObjectParams = (name: string) => {
                        const raw = getParamsRaw(name)
                        return raw !== undefined && raw !== null && !isPlainObject(raw)
                      }

                      const confirmOverwriteNonObjectParams = (label: string) =>
                        window.confirm(
                          `检测到「${label}」的 params 不是对象（可能是数组/字符串）。继续编辑将覆盖原值并可能丢失数据。是否继续？`
                        )

                      const lengthHasNonObjectParams = hasNonObjectParams('length')
                      const includeHasNonObjectParams = hasNonObjectParams('must_include')
                      const excludeHasNonObjectParams = hasNonObjectParams('must_exclude')
                      const formatHasNonObjectParams = hasNonObjectParams('format')

                      const lengthParams = getParamsObject('length')
                      const minChars =
                        typeof lengthParams?.minChars === 'number'
                          ? String(lengthParams.minChars)
                          : ''
                      const maxChars =
                        typeof lengthParams?.maxChars === 'number'
                          ? String(lengthParams.maxChars)
                          : ''

                      const includeParams = getParamsObject('must_include')
                      const includeKeywords = Array.isArray(includeParams?.keywords)
                        ? includeParams?.keywords.filter((k) => typeof k === 'string').join('\n')
                        : ''

                      const excludeParams = getParamsObject('must_exclude')
                      const excludeKeywords = Array.isArray(excludeParams?.keywords)
                        ? excludeParams?.keywords.filter((k) => typeof k === 'string').join('\n')
                        : ''

                      const formatParams = getParamsObject('format')
                      const formatValue = typeof formatParams?.format === 'string' ? formatParams.format : ''

                      const coreRequestValue =
                        typeof constrained.core_request === 'string' ? constrained.core_request : ''

                      return (
                        <div key={c.id} className="grid gap-3 rounded-md border bg-muted/10 p-3">
                          <div className="text-xs text-muted-foreground">{c.id}</div>

                          <div className="grid gap-2">
                            <Label htmlFor={`core-request-${c.id}`}>核心诉求</Label>
                            <textarea
                              id={`core-request-${c.id}`}
                              className="min-h-16 w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                              value={coreRequestValue}
                              onChange={(e) => updateCoreRequestAtIndex(index, e.target.value)}
                              placeholder="输入核心诉求（自然语言）"
                            />
                            <div className="text-xs text-muted-foreground">
                              提示：trim 为空会删除 <span className="font-mono">core_request</span> 字段（不强制填写）。
                            </div>
                          </div>

                          <div className="grid gap-3">
                            <div className="text-sm font-medium">结构化约束</div>
                            {(lengthHasNonObjectParams ||
                              includeHasNonObjectParams ||
                              excludeHasNonObjectParams ||
                              formatHasNonObjectParams) && (
                              <div className="rounded-md border border-yellow-300 bg-yellow-50 px-3 py-2 text-xs text-yellow-900">
                                检测到部分约束的 <span className="font-mono">params</span> 不是对象（例如数组/字符串）。
                                使用表单编辑这些约束时会提示是否覆盖原值。
                              </div>
                            )}

                            <div className="grid gap-2">
                              <div className="text-xs font-medium">长度限制</div>
                              <div className="grid grid-cols-2 gap-3">
                                <div className="grid gap-1">
                                  <Label htmlFor={`min-chars-${c.id}`}>最小字符数</Label>
                                  <Input
                                    id={`min-chars-${c.id}`}
                                    type="number"
                                    value={minChars}
                                    onChange={(e) => {
                                      const nextMin = parseOptionalNonNegativeInt(e.target.value)
                                      const curMax = parseOptionalNonNegativeInt(maxChars)
                                      const willRemove = nextMin === undefined && curMax === undefined
                                      if (!willRemove && lengthHasNonObjectParams) {
                                        if (!confirmOverwriteNonObjectParams('长度限制')) return
                                      }
                                      updateLengthMinAtIndex(index, e.target.value)
                                    }}
                                    placeholder="例如 30"
                                  />
                                </div>
                                <div className="grid gap-1">
                                  <Label htmlFor={`max-chars-${c.id}`}>最大字符数</Label>
                                  <Input
                                    id={`max-chars-${c.id}`}
                                    type="number"
                                    value={maxChars}
                                    onChange={(e) => {
                                      const nextMax = parseOptionalNonNegativeInt(e.target.value)
                                      const curMin = parseOptionalNonNegativeInt(minChars)
                                      const willRemove = curMin === undefined && nextMax === undefined
                                      if (!willRemove && lengthHasNonObjectParams) {
                                        if (!confirmOverwriteNonObjectParams('长度限制')) return
                                      }
                                      updateLengthMaxAtIndex(index, e.target.value)
                                    }}
                                    placeholder="例如 120"
                                  />
                                </div>
                              </div>
                              <div className="text-xs text-muted-foreground">两项都清空时将移除 length 约束。</div>
                            </div>

                            <div className="grid gap-2">
                              <Label htmlFor={`must-include-${c.id}`}>必含关键词（每行一个）</Label>
                              <textarea
                                id={`must-include-${c.id}`}
                                className="min-h-16 w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                value={includeKeywords}
                                onChange={(e) => {
                                  const keywords = parseKeywords(e.target.value)
                                  const willRemove = keywords.length === 0
                                  if (!willRemove && includeHasNonObjectParams) {
                                    if (!confirmOverwriteNonObjectParams('必含关键词')) return
                                  }
                                  updateMustIncludeAtIndex(index, e.target.value)
                                }}
                                placeholder="例如：欢迎\n一起"
                              />
                            </div>

                            <div className="grid gap-2">
                              <Label htmlFor={`must-exclude-${c.id}`}>禁止内容（每行一个）</Label>
                              <textarea
                                id={`must-exclude-${c.id}`}
                                className="min-h-16 w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                value={excludeKeywords}
                                onChange={(e) => {
                                  const keywords = parseKeywords(e.target.value)
                                  const willRemove = keywords.length === 0
                                  if (!willRemove && excludeHasNonObjectParams) {
                                    if (!confirmOverwriteNonObjectParams('禁止内容')) return
                                  }
                                  updateMustExcludeAtIndex(index, e.target.value)
                                }}
                                placeholder="例如：政治\n敏感"
                              />
                            </div>

                            <div className="grid gap-2">
                              <Label htmlFor={`format-${c.id}`}>格式要求</Label>
                              <Input
                                id={`format-${c.id}`}
                                value={formatValue}
                                onChange={(e) => {
                                  const trimmed = e.target.value.trim()
                                  const willRemove = trimmed.length === 0
                                  if (!willRemove && formatHasNonObjectParams) {
                                    if (!confirmOverwriteNonObjectParams('格式要求')) return
                                  }
                                  updateFormatAtIndex(index, e.target.value)
                                }}
                                placeholder="例如：json / markdown / plain_text"
                              />
                            </div>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </div>
              )}

            {saveErrorMessage && (
              <div className="text-sm text-red-500">保存失败：{saveErrorMessage}</div>
            )}

            {saveSuccessMessage && (
              <div className="text-sm text-green-600">{saveSuccessMessage}</div>
            )}

            <div className="flex items-center gap-2">
              <Button type="submit" disabled={isSaving || !workspaceId || importHasBlockingError}>
                {isSaving ? '保存中...' : editingId ? '保存修改' : '创建测试集'}
              </Button>
              {editingId && (
                <Button type="button" variant="outline" onClick={() => resetForm()} disabled={isSaving}>
                  取消编辑
                </Button>
              )}
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>通用 API 自定义变量</CardTitle>
          <CardDescription>
            在测试集维度维护通用 API 的额外输入变量（不包含任何明文 API Key）。
            {!editingId && <span className="ml-1">创建测试集时将自动写入。</span>}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="flex items-center justify-between gap-2">
            <div className="text-xs text-muted-foreground">大小上限：32KB（超限将保存失败）。</div>
            <Button type="button" variant="outline" onClick={handleAddGenericVariable}>
              新增变量
            </Button>
          </div>

          {!isGenericConfigEnabled ? (
            <div className="text-sm text-muted-foreground">未启用。点击“新增变量”开始编辑。</div>
          ) : genericVariableDrafts.length === 0 ? (
            <div className="text-sm text-muted-foreground">已启用但未配置变量。</div>
          ) : (
            <div className="overflow-x-auto rounded-md border border-input">
              <table className="w-full text-left text-sm">
                <thead className="bg-muted/50">
                  <tr>
                    <th className="px-3 py-2 font-medium">name</th>
                    <th className="px-3 py-2 font-medium">valueType</th>
                    <th className="px-3 py-2 font-medium">defaultValue</th>
                    <th className="px-3 py-2 font-medium" />
                  </tr>
                </thead>
                <tbody>
                  {genericVariableDrafts.map((v) => (
                    <tr key={v.id} className="border-t">
                      <td className="px-3 py-2 align-top">
                        <Input
                          value={v.name}
                          onChange={(e) => {
                            setGenericVariableDrafts((prev) =>
                              prev.map((it) => (it.id === v.id ? { ...it, name: e.target.value } : it))
                            )
                            setGenericSaveError(null)
                            setGenericSaveSuccess(null)
                          }}
                          placeholder="变量名（唯一）"
                        />
                      </td>
                      <td className="px-3 py-2 align-top">
                        <select
                          className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                          value={v.valueType}
                          onChange={(e) => {
                            const next = e.target.value as GenericValueType
                            setGenericVariableDrafts((prev) =>
                              prev.map((it) => (it.id === v.id ? { ...it, valueType: next } : it))
                            )
                            setGenericSaveError(null)
                            setGenericSaveSuccess(null)
                          }}
                        >
                          <option value="string">string</option>
                          <option value="number">number</option>
                          <option value="boolean">boolean</option>
                          <option value="json">json</option>
                        </select>
                      </td>
                      <td className="px-3 py-2 align-top">
                        {v.valueType === 'json' ? (
                          <textarea
                            className="min-h-20 w-full rounded-md border border-input bg-background px-3 py-2 font-mono text-xs shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                            value={v.defaultValueText}
                            onChange={(e) => {
                              setGenericVariableDrafts((prev) =>
                                prev.map((it) =>
                                  it.id === v.id ? { ...it, defaultValueText: e.target.value } : it
                                )
                              )
                              setGenericSaveError(null)
                              setGenericSaveSuccess(null)
                            }}
                            placeholder='例如："hello" / 123 / true / {"a":1} / [1,2]'
                          />
                        ) : v.valueType === 'boolean' ? (
                          <select
                            className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                            value={v.defaultValueText.trim()}
                            onChange={(e) => {
                              setGenericVariableDrafts((prev) =>
                                prev.map((it) =>
                                  it.id === v.id ? { ...it, defaultValueText: e.target.value } : it
                                )
                              )
                              setGenericSaveError(null)
                              setGenericSaveSuccess(null)
                            }}
                          >
                            <option value="">（空）</option>
                            <option value="true">true</option>
                            <option value="false">false</option>
                          </select>
                        ) : (
                          <Input
                            value={v.defaultValueText}
                            onChange={(e) => {
                              setGenericVariableDrafts((prev) =>
                                prev.map((it) =>
                                  it.id === v.id ? { ...it, defaultValueText: e.target.value } : it
                                )
                              )
                              setGenericSaveError(null)
                              setGenericSaveSuccess(null)
                            }}
                            placeholder={v.valueType === 'number' ? '例如：3.14' : '可选'}
                          />
                        )}
                      </td>
                      <td className="px-3 py-2 align-top">
                        <Button
                          type="button"
                          variant="outline"
                          onClick={() => {
                            setGenericVariableDrafts((prev) => prev.filter((it) => it.id !== v.id))
                            setGenericSaveError(null)
                            setGenericSaveSuccess(null)
                          }}
                        >
                          删除
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {genericSaveError && <div className="text-sm text-red-500">保存失败：{genericSaveError}</div>}
          {genericSaveSuccess && <div className="text-sm text-green-600">{genericSaveSuccess}</div>}

          <div className="flex items-center gap-2">
            <Button type="button" onClick={() => void handleSaveGenericConfig()}>
              {editingId ? '保存通用变量' : '校验通用变量'}
            </Button>
            {isGenericConfigEnabled && (
              <Button type="button" variant="outline" onClick={() => void handleDisableGenericConfig()}>
                禁用并清空
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {editingId && (
        <>
          <Card>
          <CardHeader>
            <CardTitle>Dify 变量配置</CardTitle>
            <CardDescription>
              刷新 Dify 输入变量结构，并指定待优化的 system prompt 变量与其他变量的取值来源。
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <div className="flex items-center gap-2">
              <Button type="button" onClick={() => void handleRefreshDifyVariables()} disabled={isRefreshingDifyVariables}>
                {isRefreshingDifyVariables ? '解析中...' : '刷新/解析变量'}
              </Button>
              <div className="text-xs text-muted-foreground">仅后端使用已保存的 Dify 凭证；前端不会接触明文 API Key。</div>
            </div>

            {difyVariablesError && (
              <div className="flex items-center justify-between gap-2 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700">
                <div>解析失败：{difyVariablesError}</div>
                <Button type="button" variant="outline" onClick={() => void handleRefreshDifyVariables()} disabled={isRefreshingDifyVariables}>
                  重试
                </Button>
              </div>
            )}

            {difyVariables ? (
              <>
                <div className="grid gap-2">
                  <Label htmlFor="dify-target-variable">待优化 system prompt 变量</Label>
                  <select
                    id="dify-target-variable"
                    className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
                    value={difyTargetPromptVariable}
                    onChange={(e) => {
                      const next = e.target.value
                      const selected = difyVariables.find((v) => v.name === next)
                      if (selected?.type === 'unknown') {
                        const ok = window.confirm('该变量类型为 unknown，可能不是字符串。仍要选择为优化目标吗？')
                        if (!ok) return
                      }
                      setDifyTargetPromptVariable(next)
                      setDifySaveError(null)
                      setDifySaveSuccess(null)
                    }}
                  >
                    <option value="" disabled>
                      请选择...
                    </option>
                    {difyVariables
                      .filter((v) => v.type === 'string' || v.type === 'unknown')
                      .map((v) => (
                        <option key={v.name} value={v.name}>
                          {v.name} ({v.type})
                        </option>
                      ))}
                  </select>
                </div>

                <div className="overflow-x-auto rounded-md border">
                  <table className="w-full text-left text-sm">
                    <thead className="bg-muted/50">
                      <tr>
                        <th className="p-2">变量名</th>
                        <th className="p-2">类型</th>
                        <th className="p-2">组件</th>
                        <th className="p-2">必填</th>
                        <th className="p-2">默认值</th>
                        <th className="p-2">来源配置</th>
                      </tr>
                    </thead>
                    <tbody>
                      {difyVariables.map((v) => {
                        const isTarget = v.name === difyTargetPromptVariable
                        const draft =
                          difyBindingDrafts[v.name] ?? { source: '', fixedJsonText: '', inputKey: '' }
                        const defaultText = v.default_value === null ? '-' : JSON.stringify(v.default_value)

                        return (
                          <tr key={v.name} className="border-t">
                            <td className="p-2 font-mono text-xs">
                              {v.name}
                              {isTarget && (
                                <span className="ml-2 rounded bg-blue-600 px-2 py-0.5 text-[10px] text-white">
                                  优化目标
                                </span>
                              )}
                            </td>
                            <td className="p-2">{v.type}</td>
                            <td className="p-2">{v.component}</td>
                            <td className="p-2">
                              {v.required_known ? (v.required ? '是' : '否') : '未知'}
                            </td>
                            <td className="p-2 font-mono text-xs">{defaultText}</td>
                            <td className="p-2">
                              {isTarget ? (
                                <div className="text-xs text-muted-foreground">由优化引擎运行时注入</div>
                              ) : (
                                <div className="flex flex-col gap-2">
                                  <select
                                    className="h-9 w-48 rounded-md border border-input bg-background px-2 text-sm"
                                    value={draft.source}
                                    onChange={(e) => {
                                      const nextSource = e.target.value as '' | DifyBindingSource
                                      setDifyBindingDrafts((prev) => {
                                        const current =
                                          prev[v.name] ?? { source: '', fixedJsonText: '', inputKey: '' }
                                        const nextDraft = { ...current, source: nextSource }
                                        if (nextSource === 'fixed' && !nextDraft.fixedJsonText) {
                                          nextDraft.fixedJsonText = JSON.stringify(v.default_value, null, 2)
                                        }
                                        if (nextSource === 'testCaseInput' && !nextDraft.inputKey) {
                                          nextDraft.inputKey = v.name
                                        }
                                        return { ...prev, [v.name]: nextDraft }
                                      })
                                      setDifySaveError(null)
                                      setDifySaveSuccess(null)
                                    }}
                                  >
                                    <option value="">未配置（使用默认值/省略）</option>
                                    <option value="fixed">固定默认值</option>
                                    <option value="testCaseInput">关联测试用例字段</option>
                                  </select>

                                  {draft.source === 'fixed' && (
                                    <textarea
                                      className="min-h-20 w-full rounded-md border border-input bg-background px-3 py-2 font-mono text-xs"
                                      value={draft.fixedJsonText}
                                      onChange={(e) =>
                                        setDifyBindingDrafts((prev) => ({
                                          ...prev,
                                          [v.name]: { ...draft, fixedJsonText: e.target.value },
                                        }))
                                      }
                                      placeholder='例如："hello" / 123 / true / {"a":1} / [1,2]'
                                    />
                                  )}

                                  {draft.source === 'testCaseInput' && (
                                    <div className="flex items-center gap-2">
                                      <Input
                                        value={draft.inputKey}
                                        onChange={(e) =>
                                          setDifyBindingDrafts((prev) => ({
                                            ...prev,
                                            [v.name]: { ...draft, inputKey: e.target.value },
                                          }))
                                        }
                                        placeholder="TestCase.input 的 key"
                                        list={`dify-input-keys-${v.name}`}
                                      />
                                      <datalist id={`dify-input-keys-${v.name}`}>
                                        {sampleInputKeys.map((k) => (
                                          <option key={k} value={k} />
                                        ))}
                                      </datalist>
                                    </div>
                                  )}

                                  {!draft.source && (
                                    <div className="text-xs text-muted-foreground">
                                      {v.default_value === null ? '未配置：将按 Dify 语义省略该输入字段' : '未配置：将使用默认值'}
                                    </div>
                                  )}
                                </div>
                              )}
                            </td>
                          </tr>
                        )
                      })}
                    </tbody>
                  </table>
                </div>

                {difySaveError && <div className="text-sm text-red-500">保存失败：{difySaveError}</div>}
                {difySaveSuccess && <div className="text-sm text-green-600">{difySaveSuccess}</div>}

                <div className="flex items-center gap-2">
                  <Button type="button" onClick={() => void handleSaveDifyConfig()}>
                    保存 Dify 配置
                  </Button>
                </div>
              </>
            ) : (
              <div className="text-sm text-muted-foreground">
                {difyTargetPromptVariable
                  ? `已保存配置：优化目标变量为 ${difyTargetPromptVariable}。点击“刷新/解析变量”以编辑或校验绑定。`
                  : '点击“刷新/解析变量”以加载 Dify 输入变量。'}
              </div>
            )}
          </CardContent>
        </Card>
        </>
      )}
    </section>
  )
}
