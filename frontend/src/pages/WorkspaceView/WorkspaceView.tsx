import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react'
import { Link, useLocation, useNavigate } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import {
  useCreateWorkspace,
  useDeleteWorkspace,
  useWorkspaces,
} from '@/features/workspace/hooks/useWorkspaces'
import type { WorkspaceResponse } from '@/types/generated/api/WorkspaceResponse'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { getWorkspaceIdFromPathname, getWorkspaceSwitchTargetPath } from '@/features/workspace/utils/workspaceRouting'

/**
 * 标准页面示例（路由 + TanStack Query + ts-rs 类型）
 *
 * 可复制模式示例：
 * ```tsx
 * const { data, isLoading, error } = useWorkspaces()
 * const { mutateAsync, isPending } = useCreateWorkspace()
 * const workspaces: WorkspaceResponse[] = data ?? []
 * ```
 */
export function WorkspaceView() {
  const navigate = useNavigate()
  const location = useLocation()
  const queryClient = useQueryClient()

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [deleteTarget, setDeleteTarget] = useState<WorkspaceResponse | null>(null)
  const [deleteSuccessMessage, setDeleteSuccessMessage] = useState<string | null>(null)
  const deleteRequestInFlightRef = useRef(false)

  // 1) 使用 hooks 统一数据获取与缓存
  const { data, isLoading, error } = useWorkspaces()
  // 2) 使用 mutation 处理写操作，并复用 hooks 内部的缓存刷新策略
  const {
    mutateAsync: createWorkspace,
    isPending: isCreating,
    error: createError,
  } = useCreateWorkspace()

  const {
    mutateAsync: deleteWorkspace,
    isPending: isDeleting,
    error: deleteError,
    reset: resetDeleteWorkspace,
  } = useDeleteWorkspace()

  // 3) ts-rs 生成类型，避免重复定义请求/响应 DTO
  const workspaces: WorkspaceResponse[] = data ?? []
  const listErrorMessage = error instanceof Error ? error.message : '加载失败'
  const createErrorMessage = createError instanceof Error ? createError.message : null
  const deleteErrorMessage = deleteError instanceof Error ? deleteError.message : null

  const authStatus = useAuthStore((state) => state.authStatus)
  const currentUser = useAuthStore((state) => state.currentUser)

  const lastWorkspaceIdByUser = useWorkspaceStore((state) => state.lastWorkspaceIdByUser)
  const setLastWorkspaceId = useWorkspaceStore((state) => state.setLastWorkspaceId)
  const clearLastWorkspaceId = useWorkspaceStore((state) => state.clearLastWorkspaceId)

  const currentWorkspaceId = useMemo(() => {
    const fromPath = getWorkspaceIdFromPathname(location.pathname)
    if (fromPath) return fromPath
    const userId = currentUser?.id
    if (!userId) return null
    return lastWorkspaceIdByUser[userId] ?? null
  }, [currentUser?.id, lastWorkspaceIdByUser, location.pathname])

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!name.trim()) return

    try {
      await createWorkspace({
        name: name.trim(),
        description: description.trim() ? description.trim() : null,
      })
      setName('')
      setDescription('')
    } catch {
      // 错误状态由 useMutation 负责渲染
    }
  }

  const removeWorkspaceScopedCache = (workspaceId: string) => {
    queryClient.removeQueries({ queryKey: ['testSets', workspaceId] })
    queryClient.removeQueries({ queryKey: ['optimizationTasks', workspaceId] })
    queryClient.removeQueries({ queryKey: ['testSetTemplates', workspaceId] })
    queryClient.removeQueries({ queryKey: ['workspaces', workspaceId] })
  }

  useEffect(() => {
    if (!deleteTarget) return

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return
      if (isDeleting) return
      setDeleteTarget(null)
      resetDeleteWorkspace()
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [deleteTarget, isDeleting, resetDeleteWorkspace])

  const openDeleteDialog = (workspace: WorkspaceResponse) => {
    if (authStatus !== 'authenticated') return
    setDeleteSuccessMessage(null)
    resetDeleteWorkspace()
    setDeleteTarget(workspace)
  }

  const handleConfirmDelete = async () => {
    if (authStatus !== 'authenticated') return
    if (!deleteTarget) return
    if (deleteRequestInFlightRef.current) return

    const deletedId = deleteTarget.id
    const deletedName = deleteTarget.name
    const remaining = workspaces.filter((ws) => ws.id !== deletedId)
    const nextId = remaining[0]?.id ?? null

    deleteRequestInFlightRef.current = true
    try {
      await deleteWorkspace(deletedId)

      const userId = currentUser?.id ?? null

      // 必须在导航前清理缓存（避免短时间窗口渲染旧缓存）
      removeWorkspaceScopedCache(deletedId)

      // 同步 lastWorkspaceIdByUser，避免默认落入 404
      if (userId) {
        const lastWorkspaceId = lastWorkspaceIdByUser[userId]
        if (lastWorkspaceId === deletedId) {
          if (nextId) setLastWorkspaceId(userId, nextId)
          else clearLastWorkspaceId(userId)
        }
      }

      setDeleteTarget(null)
      setDeleteSuccessMessage(`已删除工作区：${deletedName}`)

      if (currentWorkspaceId === deletedId) {
        if (nextId) {
          navigate(getWorkspaceSwitchTargetPath(location.pathname, nextId), {
            state: { flashMessage: `已删除工作区：${deletedName}` },
          })
        } else {
          navigate('/workspace', { replace: true })
        }
      }
    } catch {
      // 错误状态由 useMutation 负责渲染（只展示 message）
    } finally {
      deleteRequestInFlightRef.current = false
    }
  }

  return (
    <section className="mx-auto flex max-w-5xl flex-col gap-6 px-4 py-6" data-testid="workspace-view">
      <div>
        <h1 className="text-2xl font-semibold">Workspace View</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          标准页面示例：展示路由、TanStack Query 数据获取与 ts-rs 类型结合使用。
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>工作区列表</CardTitle>
          <CardDescription>使用 useQuery 获取数据，并在页面层处理 loading/error 状态。</CardDescription>
        </CardHeader>
        <CardContent>
          {deleteSuccessMessage && (
            <div className="mb-3 text-sm text-green-600">{deleteSuccessMessage}</div>
          )}

          {isLoading && <div className="text-sm text-muted-foreground">加载中...</div>}

          {error && (
            <div className="text-sm text-red-500">
              加载失败：{listErrorMessage}
            </div>
          )}

          {!isLoading && !error && workspaces.length === 0 && (
            <div className="text-sm text-muted-foreground">暂无工作区，请先创建一个。</div>
          )}

          {!isLoading && !error && workspaces.length > 0 && (
            <ul className="space-y-2 text-sm">
              {workspaces.map((workspace) => (
                <li key={workspace.id} className="rounded-md border px-3 py-2">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <div className="font-medium">{workspace.name}</div>
                      <div className="text-muted-foreground">
                        {workspace.description || '暂无描述'}
                      </div>
                    </div>
                    <div className="shrink-0">
                      <div className="flex items-center gap-2">
                        <Button asChild size="sm" variant="outline">
                          <Link to={`/workspaces/${workspace.id}/test-sets`}>管理测试集</Link>
                        </Button>
                        <Button asChild size="sm" variant="outline">
                          <Link to={`/workspaces/${workspace.id}/tasks`}>管理任务</Link>
                        </Button>
                        {authStatus === 'authenticated' && (
                          <Button
                            size="sm"
                            variant="destructive"
                            onClick={() => openDeleteDialog(workspace)}
                            data-testid={`workspace-delete-${workspace.id}`}
                          >
                            删除
                          </Button>
                        )}
                      </div>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>创建工作区</CardTitle>
          <CardDescription>使用 useMutation 提交写操作，并处理提交中与错误状态。</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-2">
              <Label htmlFor="workspace-name">名称</Label>
              <Input
                id="workspace-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="例如：默认工作区"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="workspace-description">描述</Label>
              <Input
                id="workspace-description"
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                placeholder="可选"
              />
            </div>

            {createErrorMessage && (
              <div className="text-sm text-red-500">
                创建失败：{createErrorMessage}
              </div>
            )}

            <div>
              <Button type="submit" disabled={isCreating}>
                {isCreating ? '创建中...' : '创建工作区'}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      {deleteTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="workspace-delete-dialog-title"
            aria-describedby="workspace-delete-dialog-description"
            className="w-full max-w-md"
            data-testid="workspace-delete-dialog"
          >
            <Card>
              <CardHeader>
                <CardTitle id="workspace-delete-dialog-title">删除工作区</CardTitle>
              </CardHeader>
              <CardContent className="flex flex-col gap-4">
                <div
                  id="workspace-delete-dialog-description"
                  className="text-sm text-muted-foreground"
                >
                  此操作将删除该工作区及其所有数据（包括测试集、优化任务等），且无法撤销。确定要删除“{deleteTarget.name}”吗？
                </div>

                {deleteErrorMessage && (
                  <div className="text-sm text-red-500">{deleteErrorMessage}</div>
                )}

                <div className="flex items-center justify-end gap-2">
                  <Button
                    variant="outline"
                    onClick={() => {
                      setDeleteTarget(null)
                      resetDeleteWorkspace()
                    }}
                    disabled={isDeleting}
                    data-testid="workspace-delete-cancel"
                  >
                    取消
                  </Button>
                  <Button
                    variant="destructive"
                    onClick={() => void handleConfirmDelete()}
                    disabled={isDeleting}
                    data-testid="workspace-delete-confirm"
                  >
                    {isDeleting ? '删除中...' : '确认删除'}
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}
    </section>
  )
}
