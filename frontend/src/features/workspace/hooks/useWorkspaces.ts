/**
 * 工作区管理 Hooks
 * 提供工作区的 CRUD 操作接口，使用 TanStack Query 管理状态
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import {
  createWorkspace,
  deleteWorkspace,
  getWorkspace,
  listWorkspaces,
} from '../services/workspaceService'
import type { CreateWorkspaceRequest } from '@/types/generated/api/CreateWorkspaceRequest'
import type { WorkspaceResponse } from '@/types/generated/api/WorkspaceResponse'

export const WORKSPACES_QUERY_KEY = ['workspaces'] as const

export function getWorkspacesQueryOptions(sessionToken: string) {
  return {
    queryKey: WORKSPACES_QUERY_KEY,
    queryFn: () => listWorkspaces(sessionToken),
  }
}

/**
 * 获取当前用户的工作区列表
 *
 * @returns TanStack Query 返回对象，包含 data, isLoading, error 等状态
 * @example
 * ```tsx
 * const { data: workspaces, isLoading } = useWorkspaces()
 * ```
 */
export function useWorkspaces(options?: { enabled?: boolean }) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: WORKSPACES_QUERY_KEY,
    queryFn: () => listWorkspaces(sessionToken!),
    enabled: isAuthenticated && (options?.enabled ?? true),
  })
}

/**
 * 获取单个工作区详情
 *
 * @param id - 工作区 ID
 * @returns TanStack Query 返回对象
 * @example
 * ```tsx
 * const { data: workspace } = useWorkspace('workspace-id')
 * ```
 */
export function useWorkspace(id: string, options?: { enabled?: boolean }) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...WORKSPACES_QUERY_KEY, id],
    queryFn: () => getWorkspace(id, sessionToken!),
    enabled: isAuthenticated && !!id && (options?.enabled ?? true),
  })
}

/**
 * 创建新工作区
 *
 * 成功后自动刷新工作区列表缓存
 *
 * @returns TanStack Mutation 返回对象
 * @example
 * ```tsx
 * const { mutate: create, isPending } = useCreateWorkspace()
 * create({ name: '新工作区', description: '描述' })
 * ```
 */
export function useCreateWorkspace() {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (params: CreateWorkspaceRequest) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return createWorkspace(params, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: WORKSPACES_QUERY_KEY })
    },
  })
}

/**
 * 删除工作区
 *
 * 成功后自动刷新工作区列表缓存
 *
 * @returns TanStack Mutation 返回对象
 * @example
 * ```tsx
 * const { mutate: remove } = useDeleteWorkspace()
 * remove('workspace-id')
 * ```
 */
export function useDeleteWorkspace() {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (workspaceId: string) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return deleteWorkspace(workspaceId, sessionToken)
    },
    onSuccess: (_data, workspaceId) => {
      // 先就地更新列表缓存，避免 UI 等待 invalidate/refetch 的窗口期渲染旧列表（CI 下尤其容易触发）。
      queryClient.setQueryData<WorkspaceResponse[]>(WORKSPACES_QUERY_KEY, (prev) => {
        const current = prev ?? []
        return current.filter((ws) => ws.id !== workspaceId)
      })
      queryClient.invalidateQueries({ queryKey: WORKSPACES_QUERY_KEY })
    },
  })
}
