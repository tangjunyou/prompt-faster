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

const WORKSPACES_QUERY_KEY = ['workspaces'] as const

/**
 * 获取当前用户的工作区列表
 *
 * @returns TanStack Query 返回对象，包含 data, isLoading, error 等状态
 * @example
 * ```tsx
 * const { data: workspaces, isLoading } = useWorkspaces()
 * ```
 */
export function useWorkspaces() {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: WORKSPACES_QUERY_KEY,
    queryFn: () => listWorkspaces(sessionToken!),
    enabled: isAuthenticated,
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
export function useWorkspace(id: string) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...WORKSPACES_QUERY_KEY, id],
    queryFn: () => getWorkspace(id, sessionToken!),
    enabled: isAuthenticated && !!id,
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
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: WORKSPACES_QUERY_KEY })
    },
  })
}
