import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { PromptVersionDetail, PromptVersionList, MetaOptimizationStats } from '@/features/meta-optimization'
import { usePromptVersions } from '@/features/meta-optimization/hooks/usePromptVersions'
import { useMetaOptimizationOverview } from '@/features/meta-optimization/hooks/useMetaOptimizationOverview'
import { useMetaOptimizationTasks } from '@/features/meta-optimization/hooks/useMetaOptimizationTasks'
import {
  activatePromptVersion,
  createPromptVersion,
  getPromptVersion,
} from '@/features/meta-optimization/services/metaOptimizationService'
import { useAuthStore } from '@/stores/useAuthStore'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'

function formatDate(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function MetaOptimizationPage() {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  const { data: versions = [], isLoading: versionsLoading, error: versionsError } = usePromptVersions()
  const { data: overview } = useMetaOptimizationOverview()
  const {
    data: tasks = [],
    isLoading: tasksLoading,
    error: tasksError,
  } = useMetaOptimizationTasks()

  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(null)
  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([])
  const [newPromptContent, setNewPromptContent] = useState('')
  const [newPromptDescription, setNewPromptDescription] = useState('')
  const [activateNewPrompt, setActivateNewPrompt] = useState(true)

  const effectiveSelectedId = useMemo(() => {
    if (selectedVersionId && versions.some((v) => v.id === selectedVersionId)) {
      return selectedVersionId
    }
    const active = versions.find((v) => v.isActive) ?? versions[0]
    return active?.id ?? null
  }, [selectedVersionId, versions])

  const promptQuery = useQuery({
    queryKey: ['metaOptimization', 'prompt', effectiveSelectedId],
    queryFn: () => getPromptVersion(effectiveSelectedId!, sessionToken!),
    enabled: authStatus === 'authenticated' && !!sessionToken && !!effectiveSelectedId,
  })

  const activateMutation = useMutation({
    mutationFn: async (versionId: string) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return activatePromptVersion(versionId, sessionToken)
    },
    onSuccess: (_, versionId) => {
      setSelectedVersionId(versionId)
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'promptVersions'] })
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'overview'] })
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'prompt', versionId] })
    },
  })

  const createMutation = useMutation({
    mutationFn: async () => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return createPromptVersion(
        {
          content: newPromptContent,
          description: newPromptDescription.trim() ? newPromptDescription.trim() : null,
          activate: activateNewPrompt,
        },
        sessionToken
      )
    },
    onSuccess: (version) => {
      setNewPromptContent('')
      setNewPromptDescription('')
      setActivateNewPrompt(true)
      setSelectedVersionId(version.id)
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'promptVersions'] })
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'overview'] })
      queryClient.invalidateQueries({ queryKey: ['metaOptimization', 'prompt', version.id] })
    },
  })

  const selectedStats = overview?.stats.find((item) => item.versionId === effectiveSelectedId) ?? null

  const activeVersionLabel = overview?.activeVersion
    ? `v${overview.activeVersion.version}`
    : '暂无活跃版本'

  const toggleTask = (taskId: string) => {
    setSelectedTaskIds((prev) =>
      prev.includes(taskId) ? prev.filter((id) => id !== taskId) : [...prev, taskId]
    )
  }

  return (
    <section className="mx-auto flex max-w-6xl flex-col gap-6 px-4 py-6" data-testid="meta-optimization-page">
      <div className="space-y-2">
        <h1 className="text-2xl font-semibold">元优化：老师模型 Prompt</h1>
        <p className="text-sm text-muted-foreground">
          维护老师模型 Prompt 版本，追踪成功率，选择历史任务作为测试集。
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-[1.2fr_1fr]">
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>创建 Prompt 版本</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4 text-sm">
              <div className="grid gap-2">
                <Label htmlFor="prompt-content">Prompt 内容 *</Label>
                <Textarea
                  id="prompt-content"
                  value={newPromptContent}
                  onChange={(event) => setNewPromptContent(event.target.value)}
                  placeholder="输入新的老师模型 Prompt"
                  rows={6}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="prompt-description">变更说明</Label>
                <Input
                  id="prompt-description"
                  value={newPromptDescription}
                  onChange={(event) => setNewPromptDescription(event.target.value)}
                  placeholder="例如：优化格式约束/减少冗余"
                />
              </div>
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={activateNewPrompt}
                  onChange={(event) => setActivateNewPrompt(event.target.checked)}
                />
                创建后设为活跃版本
              </label>
              <div>
                <Button
                  type="button"
                  disabled={!newPromptContent.trim() || createMutation.isPending}
                  onClick={() => createMutation.mutate()}
                >
                  {createMutation.isPending ? '保存中...' : '保存新版本'}
                </Button>
              </div>
              {createMutation.error instanceof Error && (
                <div className="text-sm text-destructive">
                  保存失败：{createMutation.error.message}
                </div>
              )}
            </CardContent>
          </Card>
          <PromptVersionList
            versions={versions}
            selectedId={effectiveSelectedId}
            onSelect={setSelectedVersionId}
            onActivate={(id) => activateMutation.mutate(id)}
            isActivating={activateMutation.isPending}
          />

          <PromptVersionDetail
            prompt={promptQuery.data ?? null}
            stats={selectedStats}
            versions={versions}
            isLoading={promptQuery.isLoading || versionsLoading}
            error={promptQuery.error as Error | null}
            onSelectVersion={setSelectedVersionId}
          />
        </div>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>概览</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">总版本数</span>
                <span className="font-medium">{overview?.totalVersions ?? 0}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">当前活跃版本</span>
                <span className="font-medium">
                  {overview?.activeVersion ? `v${overview.activeVersion.version}` : '—'}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">最佳成功率版本</span>
                <span className="font-medium">
                  {overview?.bestVersion ? `v${overview.bestVersion.version}` : '—'}
                </span>
              </div>
            </CardContent>
          </Card>

          <MetaOptimizationStats stats={overview?.stats ?? []} />

          <Card>
            <CardHeader>
              <CardTitle>元优化入口</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <div className="rounded-md border p-3">
                <div className="text-sm text-muted-foreground">优化目标</div>
                <div className="mt-1 font-medium">老师模型 Prompt（{activeVersionLabel}）</div>
                <div className="mt-2 text-xs text-muted-foreground">
                  MVP：仅提供历史任务选择入口，不执行真实元优化流程。
                </div>
              </div>

              {tasksLoading ? (
                <div className="text-muted-foreground">加载历史任务中...</div>
              ) : tasksError instanceof Error ? (
                <div className="text-destructive">加载失败：{tasksError.message}</div>
              ) : tasks.length === 0 ? (
                <div className="text-muted-foreground">暂无历史任务。</div>
              ) : (
                <div className="space-y-2">
                  {tasks.map((task) => (
                    <label
                      key={task.id}
                      className="flex items-start gap-2 rounded-lg border p-2"
                    >
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
                          Pass Rate：{task.passRate === null || task.passRate === undefined
                            ? '—'
                            : `${(task.passRate * 100).toFixed(1)}%`}
                        </div>
                        <div className="mt-1 text-xs text-muted-foreground">
                          创建时间：{formatDate(task.createdAt)}
                        </div>
                      </div>
                    </label>
                  ))}
                </div>
              )}

              <div className="text-xs text-muted-foreground">
                已选择 {selectedTaskIds.length} 个历史任务
              </div>
              <Button type="button" variant="outline" disabled>
                开始元优化（MVP 未启用）
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>

      {versionsError && (
        <div className="text-sm text-destructive">加载版本列表失败：{versionsError.message}</div>
      )}
    </section>
  )
}
