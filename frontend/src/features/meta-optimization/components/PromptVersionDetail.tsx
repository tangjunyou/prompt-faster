import { useMemo, useRef, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useAuthStore } from '@/stores/useAuthStore'
import type { TeacherPrompt } from '@/types/generated/models/TeacherPrompt'
import type { TeacherPromptStats } from '@/types/generated/models/TeacherPromptStats'
import type { TeacherPromptVersion } from '@/types/generated/models/TeacherPromptVersion'
import {
  activatePromptVersion,
  createPromptVersion,
  getPromptVersion,
  validatePrompt,
} from '../services/metaOptimizationService'
import { META_OPTIMIZATION_OVERVIEW_QUERY_KEY } from '../hooks/useMetaOptimizationOverview'
import { PROMPT_VERSIONS_QUERY_KEY } from '../hooks/usePromptVersions'
import { PromptEditor } from './PromptEditor'
import { PromptPreviewPanel } from './PromptPreviewPanel'

const MAX_PROMPT_BYTES = 100 * 1024

export interface PromptVersionDetailProps {
  prompt: TeacherPrompt | null
  stats?: TeacherPromptStats | null
  versions?: TeacherPromptVersion[]
  isLoading?: boolean
  error?: Error | null
  onSelectVersion?: (id: string) => void
}

function formatRate(rate: number | null | undefined) {
  if (rate === null || rate === undefined) return '—'
  return `${(rate * 100).toFixed(1)}%`
}

function formatDate(value?: string | null) {
  if (!value) return '—'
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

function byteLength(value: string) {
  return new TextEncoder().encode(value).length
}

export function PromptVersionDetail({
  prompt,
  stats,
  versions = [],
  isLoading,
  error,
  onSelectVersion,
}: PromptVersionDetailProps) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  const [isEditing, setIsEditing] = useState(false)
  const [editContent, setEditContent] = useState('')
  const [changeNote, setChangeNote] = useState('')
  const [formError, setFormError] = useState<string | null>(null)
  const [validationErrors, setValidationErrors] = useState<string[]>([])
  const [isSaveDialogOpen, setIsSaveDialogOpen] = useState(false)
  const [isRollbackDialogOpen, setIsRollbackDialogOpen] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [isRollbacking, setIsRollbacking] = useState(false)
  const [isPreviewing, setIsPreviewing] = useState(false)

  const editorRef = useRef<any>(null)

  const previousVersion = useMemo(() => {
    if (!prompt || versions.length < 2) return null
    const sorted = [...versions].sort((a, b) => b.version - a.version)
    return sorted.find((version) => version.version < prompt.version) ?? null
  }, [prompt, versions])

  const handleStartEdit = () => {
    if (!prompt) return
    setEditContent(prompt.content)
    setChangeNote('')
    setFormError(null)
    setValidationErrors([])
    setIsEditing(true)
  }

  const handleCancelEdit = () => {
    setIsEditing(false)
    setEditContent('')
    setChangeNote('')
    setFormError(null)
    setValidationErrors([])
  }

  const handleFormat = () => {
    const editor = editorRef.current
    if (editor?.getAction) {
      editor.getAction('editor.action.formatDocument')?.run()
    }
  }

  const handleConfirmSave = async () => {
    setFormError(null)
    setValidationErrors([])

    if (!isAuthenticated || !sessionToken) {
      setFormError('未登录')
      return
    }
    if (!editContent.trim()) {
      setFormError('Prompt 内容不能为空')
      return
    }
    if (byteLength(editContent) > MAX_PROMPT_BYTES) {
      setFormError('Prompt 内容不能超过 100KB')
      return
    }
    if (!changeNote.trim()) {
      setFormError('请填写变更说明')
      return
    }

    setIsSaving(true)
    try {
      const validation = await validatePrompt({ content: editContent }, sessionToken)
      if (!validation.isValid) {
        setValidationErrors(validation.errors)
        return
      }
      const version = await createPromptVersion(
        {
          content: editContent,
          description: changeNote.trim(),
          activate: true,
        },
        sessionToken
      )
      setIsSaveDialogOpen(false)
      setIsEditing(false)
      setEditContent('')
      setChangeNote('')
      onSelectVersion?.(version.id)
      queryClient.invalidateQueries({ queryKey: PROMPT_VERSIONS_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: META_OPTIMIZATION_OVERVIEW_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'prompt', version.id] })
    } catch (err) {
      setFormError(err instanceof Error ? err.message : '保存失败')
    } finally {
      setIsSaving(false)
    }
  }

  const handleConfirmRollback = async () => {
    if (!previousVersion) return
    if (!isAuthenticated || !sessionToken) {
      setFormError('未登录')
      return
    }
    setIsRollbacking(true)
    try {
      await activatePromptVersion(previousVersion.id, sessionToken)
      const refreshed = await getPromptVersion(previousVersion.id, sessionToken)
      setEditContent(refreshed.content)
      setIsRollbackDialogOpen(false)
      setIsEditing(true)
      onSelectVersion?.(previousVersion.id)
      queryClient.invalidateQueries({ queryKey: PROMPT_VERSIONS_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: META_OPTIMIZATION_OVERVIEW_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'prompt', previousVersion.id] })
    } catch (err) {
      setFormError(err instanceof Error ? err.message : '回滚失败')
    } finally {
      setIsRollbacking(false)
    }
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between gap-2">
        <CardTitle>版本详情</CardTitle>
        {!isLoading && !error && prompt && !isEditing && (
          <Button type="button" size="sm" onClick={handleStartEdit}>
            编辑
          </Button>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <div className="text-sm text-muted-foreground">加载版本详情中...</div>
        ) : error ? (
          <div className="text-sm text-destructive">加载失败：{error.message}</div>
        ) : !prompt ? (
          <div className="text-sm text-muted-foreground">请选择一个版本查看详情。</div>
        ) : (
          <>
            <div className="space-y-1">
              <div className="text-sm text-muted-foreground">版本号</div>
              <div className="text-base font-semibold">v{prompt.version}</div>
            </div>

            <div className="grid gap-3 text-sm sm:grid-cols-2">
              <div>
                <div className="text-muted-foreground">创建时间</div>
                <div className="text-foreground">{formatDate(prompt.createdAt)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">更新时间</div>
                <div className="text-foreground">{formatDate(prompt.updatedAt)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">成功率</div>
                <div className="text-foreground">{formatRate(stats?.successRate)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">平均通过率</div>
                <div className="text-foreground">{formatRate(stats?.averagePassRate)}</div>
              </div>
              <div>
                <div className="text-muted-foreground">任务数</div>
                <div className="text-foreground">{stats?.totalTasks ?? 0}</div>
              </div>
              <div>
                <div className="text-muted-foreground">成功任务</div>
                <div className="text-foreground">{stats?.successfulTasks ?? 0}</div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">成功率图表</div>
              {stats?.successRate === null || stats?.successRate === undefined ? (
                <div className="text-sm text-muted-foreground">暂无统计数据</div>
              ) : (
                <div className="h-2 w-full rounded-full bg-muted">
                  <div
                    className="h-2 rounded-full bg-primary"
                    style={{ width: `${Math.round(stats.successRate * 100)}%` }}
                  />
                </div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">Prompt 内容</div>
              {isEditing ? (
                <div className="space-y-4">
                  <PromptEditor value={editContent} onChange={setEditContent} onMount={(editor) => {
                    editorRef.current = editor
                  }} />
                  <div className="flex flex-wrap items-center gap-2">
                    <Button
                      type="button"
                      onClick={() => setIsSaveDialogOpen(true)}
                      disabled={isSaving || isPreviewing}
                    >
                      {isSaving ? '保存中...' : '保存为新版本'}
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={handleCancelEdit}
                      disabled={isSaving}
                    >
                      取消
                    </Button>
                    <Button type="button" variant="ghost" onClick={handleFormat}>
                      格式化
                    </Button>
                    <Button
                      type="button"
                      variant="secondary"
                      onClick={() => setIsRollbackDialogOpen(true)}
                      disabled={!previousVersion || isRollbacking}
                    >
                      回滚到上一版本
                    </Button>
                    {!previousVersion && (
                      <span className="text-xs text-muted-foreground">
                        当前仅有 1 个版本
                      </span>
                    )}
                  </div>

                  {(formError || validationErrors.length > 0) && (
                    <div className="space-y-1 text-sm text-destructive">
                      {formError && <div>{formError}</div>}
                      {validationErrors.map((err) => (
                        <div key={err}>{err}</div>
                      ))}
                    </div>
                  )}

                  <PromptPreviewPanel
                    content={editContent}
                    onPreviewingChange={setIsPreviewing}
                  />
                </div>
              ) : (
                <PromptEditor value={prompt.content} readOnly height="320px" />
              )}
            </div>
          </>
        )}
      </CardContent>

      <Dialog open={isSaveDialogOpen} onOpenChange={setIsSaveDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>保存为新版本</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 text-sm">
            <div className="grid gap-2">
              <Label htmlFor="change-note">变更说明 *</Label>
              <Input
                id="change-note"
                value={changeNote}
                onChange={(event) => setChangeNote(event.target.value)}
                placeholder="例如：调整老师模型语气/增加结构化输出"
              />
            </div>
            <div className="text-xs text-muted-foreground">
              保存将创建一个新版本并自动切换为活跃版本。
            </div>
            {(formError || validationErrors.length > 0) && (
              <div className="space-y-1 text-sm text-destructive">
                {formError && <div>{formError}</div>}
                {validationErrors.map((err) => (
                  <div key={err}>{err}</div>
                ))}
              </div>
            )}
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsSaveDialogOpen(false)}
              disabled={isSaving}
            >
              取消
            </Button>
            <Button type="button" onClick={handleConfirmSave} disabled={isSaving || isPreviewing}>
              {isSaving ? '保存中...' : '确认保存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={isRollbackDialogOpen} onOpenChange={setIsRollbackDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认回滚</DialogTitle>
          </DialogHeader>
          <div className="space-y-2 text-sm">
            <div>回滚将切换到上一版本并丢弃当前编辑内容。</div>
            {previousVersion ? (
              <div className="text-xs text-muted-foreground">
                目标版本：v{previousVersion.version}
              </div>
            ) : (
              <div className="text-xs text-muted-foreground">没有可用的上一版本。</div>
            )}
            {formError && <div className="text-destructive">{formError}</div>}
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsRollbackDialogOpen(false)}
              disabled={isRollbacking}
            >
              取消
            </Button>
            <Button
              type="button"
              onClick={handleConfirmRollback}
              disabled={!previousVersion || isRollbacking}
            >
              {isRollbacking ? '回滚中...' : '确认回滚'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  )
}
