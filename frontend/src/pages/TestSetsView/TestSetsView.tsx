import { useMemo, useState, type FormEvent } from 'react'
import { Link, useParams } from 'react-router'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  useCreateTestSet,
  useDeleteTestSet,
  useTestSets,
  useUpdateTestSet,
} from '@/features/test-set-manager/hooks/useTestSets'
import { getTestSet } from '@/features/test-set-manager/services/testSetService'
import { useAuthStore } from '@/stores/useAuthStore'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { JsonValue } from '@/types/generated/serde_json/JsonValue'

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

export function TestSetsView() {
  const params = useParams()
  const workspaceId = params.id ?? ''

  const { data, isLoading, error } = useTestSets(workspaceId)
  const testSets: TestSetListItemResponse[] = data ?? []

  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  const [editingId, setEditingId] = useState<string | null>(null)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [casesJson, setCasesJson] = useState('[]')
  const [localCasesError, setLocalCasesError] = useState<string | null>(null)
  const [isLoadingEdit, setIsLoadingEdit] = useState(false)
  const [loadingEditId, setLoadingEditId] = useState<string | null>(null)

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

  const isSaving = isCreating || isUpdating
  const listErrorMessage = error instanceof Error ? error.message : '加载失败'
  const saveError = createError ?? updateError
  const saveErrorMessage = saveError instanceof Error ? saveError.message : null

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
    } catch (e) {
      setLocalCasesError(e instanceof Error ? e.message : '加载测试集失败')
    } finally {
      setIsLoadingEdit(false)
      setLoadingEditId(null)
    }
  }

  const resetForm = () => {
    setEditingId(null)
    setName('')
    setDescription('')
    setCasesJson('[]')
    setLocalCasesError(null)
  }

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!workspaceId) return
    if (!name.trim()) return

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

    const cases = parsed as JsonValue

    if (editingId) {
      await updateTestSet({
        name: name.trim(),
        description: description.trim() ? description.trim() : null,
        cases,
      })
      resetForm()
      return
    }

    await createTestSet({
      name: name.trim(),
      description: description.trim() ? description.trim() : null,
      cases,
    })
    resetForm()
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
          <CardTitle>{title}</CardTitle>
          <CardDescription>cases 使用 JSON 编辑（最小校验：id/input/reference）。</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
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

            <div className="flex items-center gap-2">
              <Button type="submit" disabled={isSaving || !workspaceId}>
                {isSaving ? '保存中...' : editingId ? '保存修改' : '创建测试集'}
              </Button>
              {editingId && (
                <Button type="button" variant="outline" onClick={resetForm} disabled={isSaving}>
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
