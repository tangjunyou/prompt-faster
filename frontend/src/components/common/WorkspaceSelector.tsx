import { useEffect, useMemo, useState } from 'react'
import { useLocation, useNavigate } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useAuthStore } from '@/stores/useAuthStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { useCreateWorkspace, useWorkspaces } from '@/features/workspace/hooks/useWorkspaces'
import { getOptimizationTasksQueryOptions } from '@/features/task-config/hooks/useOptimizationTasks'
import { getTestSetsQueryOptions } from '@/features/test-set-manager/hooks/useTestSets'
import type { WorkspaceResponse } from '@/types/generated/api/WorkspaceResponse'
import { getWorkspaceIdFromPathname, getWorkspaceSwitchTargetPath } from '@/features/workspace/utils/workspaceRouting'

const EMPTY_WORKSPACES: WorkspaceResponse[] = []

export function WorkspaceSelector() {
  const navigate = useNavigate()
  const location = useLocation()
  const queryClient = useQueryClient()

  const authStatus = useAuthStore((state) => state.authStatus)
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const currentUser = useAuthStore((state) => state.currentUser)

  const lastWorkspaceIdByUser = useWorkspaceStore((state) => state.lastWorkspaceIdByUser)
  const setLastWorkspaceId = useWorkspaceStore((state) => state.setLastWorkspaceId)

  const [isOpen, setIsOpen] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [draftName, setDraftName] = useState('')
  const [draftDescription, setDraftDescription] = useState('')
  const [localCreateError, setLocalCreateError] = useState<string | null>(null)
  const [localSelectedWorkspace, setLocalSelectedWorkspace] = useState<WorkspaceResponse | null>(null)

  const { data, isLoading, isFetching, error, refetch } = useWorkspaces()
  const workspaces = data ?? EMPTY_WORKSPACES

  const {
    mutateAsync: createWorkspace,
    isPending: isCreating,
    error: createError,
  } = useCreateWorkspace()

  const userId = currentUser?.id ?? null
  const workspaceIdFromPath = getWorkspaceIdFromPathname(location.pathname)

  useEffect(() => {
    if (!userId || !workspaceIdFromPath) return
    setLastWorkspaceId(userId, workspaceIdFromPath)
  }, [setLastWorkspaceId, userId, workspaceIdFromPath])

  const resolvedWorkspaceId = useMemo(() => {
    // Priority order for resolved workspace ID:
    // 1) URL param: /workspaces/:id/* (single source of truth)
    // 2) lastWorkspaceId for current user (only if still exists)
    // 3) first workspace from list (backend order)
    if (workspaceIdFromPath) return workspaceIdFromPath

    const lastWorkspaceId = userId ? lastWorkspaceIdByUser[userId] : undefined
    if (lastWorkspaceId && workspaces.some((ws) => ws.id === lastWorkspaceId)) {
      return lastWorkspaceId
    }

    if (workspaces.length > 0) return workspaces[0]!.id
    return null
  }, [lastWorkspaceIdByUser, userId, workspaceIdFromPath, workspaces])

  const currentWorkspace = useMemo(() => {
    if (!resolvedWorkspaceId) return null
    return workspaces.find((ws) => ws.id === resolvedWorkspaceId) ?? null
  }, [resolvedWorkspaceId, workspaces])

  const triggerLabel = useMemo(() => {
    if (currentWorkspace) return currentWorkspace.name
    if (localSelectedWorkspace && localSelectedWorkspace.id === resolvedWorkspaceId) {
      return localSelectedWorkspace.name
    }
    if (isLoading) return '加载中...'
    return '选择工作区'
  }, [currentWorkspace, isLoading, localSelectedWorkspace, resolvedWorkspaceId])

  if (authStatus !== 'authenticated') return null

  const listErrorMessage = error instanceof Error ? error.message : '加载失败'
  const createErrorMessage = createError instanceof Error ? createError.message : null

  const prefetchWorkspaceData = (workspaceId: string) => {
    if (!sessionToken) return

    void queryClient
      .prefetchQuery(getOptimizationTasksQueryOptions(workspaceId, sessionToken))
      .catch(() => {})

    void queryClient
      .prefetchQuery(getTestSetsQueryOptions(workspaceId, sessionToken))
      .catch(() => {})
  }

  const handleSwitch = (workspace: WorkspaceResponse) => {
    if (userId) setLastWorkspaceId(userId, workspace.id)
    prefetchWorkspaceData(workspace.id)
    setIsOpen(false)
    navigate(getWorkspaceSwitchTargetPath(location.pathname, workspace.id))
  }

  const handleToggleOpen = () => {
    setIsOpen((prev) => {
      const next = !prev
      if (next) {
        void refetch()
      }
      return next
    })
  }

  const openCreateDialog = () => {
    setLocalCreateError(null)
    setDraftName('')
    setDraftDescription('')
    setIsCreateDialogOpen(true)
  }

  const handleCreate = async () => {
    setLocalCreateError(null)

    const name = draftName.trim()
    if (!name) {
      setLocalCreateError('名称不能为空')
      return
    }
    if (name.length > 128) {
      setLocalCreateError('名称长度不能超过 128')
      return
    }

    try {
      const created = await createWorkspace({
        name,
        description: draftDescription.trim() ? draftDescription.trim() : null,
      })
      if (userId) setLastWorkspaceId(userId, created.id)
      setLocalSelectedWorkspace(created)
      prefetchWorkspaceData(created.id)
      setIsCreateDialogOpen(false)
      setIsOpen(false)
      navigate(getWorkspaceSwitchTargetPath(location.pathname, created.id))
    } catch {
      // 错误由 createError 渲染（只展示 message）
    }
  }

  return (
    <div className="relative" data-testid="workspace-selector">
      <Button
        variant="outline"
        size="sm"
        onClick={handleToggleOpen}
        data-testid="workspace-selector-trigger"
      >
        {triggerLabel}
      </Button>

      {isOpen && (
        <div
          className="absolute left-0 top-full z-50 mt-2 w-64 rounded-md border bg-background p-2 shadow"
          data-testid="workspace-selector-menu"
        >
          <div className="flex items-center justify-between gap-2 px-1 pb-2">
            <div className="text-xs text-muted-foreground">
              {isFetching ? '刷新中...' : '工作区'}
            </div>
            <Button variant="outline" size="sm" onClick={openCreateDialog}>
              新建工作区
            </Button>
          </div>

          {error && (
            <div className="px-1 py-2 text-sm text-red-500">
              加载失败：{listErrorMessage}
            </div>
          )}

          {!error && !isLoading && workspaces.length === 0 && (
            <div className="px-1 py-2 text-sm text-muted-foreground">暂无工作区</div>
          )}

          {!error && workspaces.length > 0 && (
            <ul className="space-y-1">
              {workspaces.map((ws) => {
                const isActive = ws.id === resolvedWorkspaceId
                return (
                  <li key={ws.id}>
                    <button
                      type="button"
                      className="flex w-full items-center justify-between rounded-md px-2 py-2 text-left text-sm hover:bg-accent"
                      onClick={() => handleSwitch(ws)}
                      data-testid={`workspace-option-${ws.id}`}
                    >
                      <span className="min-w-0 truncate">{ws.name}</span>
                      {isActive && <span className="text-xs text-muted-foreground">当前</span>}
                    </button>
                  </li>
                )
              })}
            </ul>
          )}
        </div>
      )}

      {isCreateDialogOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div role="dialog" aria-modal="true" className="w-full max-w-md">
            <Card>
              <CardHeader>
                <CardTitle>新建工作区</CardTitle>
              </CardHeader>
              <CardContent className="flex flex-col gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="workspace-create-name">名称</Label>
                  <Input
                    id="workspace-create-name"
                    value={draftName}
                    onChange={(e) => setDraftName(e.target.value)}
                    placeholder="例如：默认工作区"
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="workspace-create-description">描述</Label>
                  <Input
                    id="workspace-create-description"
                    value={draftDescription}
                    onChange={(e) => setDraftDescription(e.target.value)}
                    placeholder="可选"
                  />
                </div>

                {localCreateError && (
                  <div className="text-sm text-red-500">{localCreateError}</div>
                )}
                {createErrorMessage && (
                  <div className="text-sm text-red-500">创建失败：{createErrorMessage}</div>
                )}

                <div className="flex items-center justify-end gap-2">
                  <Button
                    variant="outline"
                    onClick={() => setIsCreateDialogOpen(false)}
                    disabled={isCreating}
                  >
                    取消
                  </Button>
                  <Button onClick={() => void handleCreate()} disabled={isCreating}>
                    {isCreating ? '创建中...' : '创建'}
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}
    </div>
  )
}
