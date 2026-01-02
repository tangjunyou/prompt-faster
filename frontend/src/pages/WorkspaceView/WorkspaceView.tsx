import { useState, type FormEvent } from 'react'
import { useCreateWorkspace, useWorkspaces } from '@/features/workspace/hooks/useWorkspaces'
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
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')

  // 1) 使用 hooks 统一数据获取与缓存
  const { data, isLoading, error } = useWorkspaces()
  // 2) 使用 mutation 处理写操作，并复用 hooks 内部的缓存刷新策略
  const {
    mutateAsync: createWorkspace,
    isPending: isCreating,
    error: createError,
  } = useCreateWorkspace()

  // 3) ts-rs 生成类型，避免重复定义请求/响应 DTO
  const workspaces: WorkspaceResponse[] = data ?? []
  const listErrorMessage = error instanceof Error ? error.message : '加载失败'
  const createErrorMessage = createError instanceof Error ? createError.message : null

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
                  <div className="font-medium">{workspace.name}</div>
                  <div className="text-muted-foreground">
                    {workspace.description || '暂无描述'}
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
    </section>
  )
}
