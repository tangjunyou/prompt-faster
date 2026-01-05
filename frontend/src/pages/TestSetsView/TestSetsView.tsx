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
import { useAuthStore } from '@/stores/useAuthStore'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { TestCase } from '@/types/generated/models/TestCase'
import type { JsonValue } from '@/types/generated/serde_json/JsonValue'

const JSONL_FORMAT_HELP = {
  title: '导入格式（JSON Lines / JSONL）',
  description: '仅支持 txt（UTF-8），一行一个 TestCase JSON；空行会被跳过。',
  example: `{"id":"case-1","input":{"question":"你好，帮我写一段自我介绍"},"reference":{"Exact":{"expected":"（此处填写期望输出）"}}}
{"id":"case-2","input":{"question":"用 JSON 输出一个用户对象"},"reference":{"Constrained":{"constraints":[{"name":"format","description":"必须是 JSON","weight":1.0}],"quality_dimensions":[{"name":"correctness","description":"字段合理且可解析","weight":1.0}]}}}`,
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
    const refKeys = Object.keys(reference as Record<string, unknown>)
    if (refKeys.length !== 1 || !['Exact', 'Constrained', 'Hybrid'].includes(refKeys[0]!)) {
      return `cases[${index}].reference 必须是 Exact / Constrained / Hybrid 之一`
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

  const { data, isLoading, error } = useTestSets(workspaceId)
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
      setSaveSuccessMessage('已从模板预填')
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

    await createTestSet({
      name: name.trim(),
      description: description.trim() ? description.trim() : null,
      cases,
    })
    setSaveSuccessMessage('创建成功')
    resetForm({ keepSuccessMessage: true })
  }

  const handleDelete = async (ts: TestSetListItemResponse) => {
    if (!workspaceId) return
    const confirmed = window.confirm(`确定删除测试集「${ts.name}」？`)
    if (!confirmed) return
    await deleteTestSet(ts.id)
    if (editingId === ts.id) resetForm()
  }

  return (
    <section className="mx-auto flex max-w-5xl flex-col gap-6 px-4 py-6" data-testid="test-sets-view">
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
    </section>
  )
}
