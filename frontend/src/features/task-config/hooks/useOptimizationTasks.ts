/**
 * 优化任务 Hooks
 * - 以 workspace 为第一边界
 * - 使用 TanStack Query 管理缓存
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import type { CreateOptimizationTaskRequest } from '@/types/generated/api/CreateOptimizationTaskRequest'
import {
  createOptimizationTask,
  getOptimizationTask,
  listOptimizationTasks,
  updateOptimizationTaskConfig,
} from '../services/optimizationTaskService'
import type { UpdateOptimizationTaskConfigRequest } from '@/types/generated/api/UpdateOptimizationTaskConfigRequest'

const OPTIMIZATION_TASKS_QUERY_KEY = ['optimizationTasks'] as const

export function useOptimizationTasks(workspaceId: string) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...OPTIMIZATION_TASKS_QUERY_KEY, workspaceId],
    queryFn: () => listOptimizationTasks(workspaceId, sessionToken!),
    enabled: isAuthenticated && !!workspaceId,
  })
}

export function useOptimizationTask(workspaceId: string, taskId: string) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...OPTIMIZATION_TASKS_QUERY_KEY, workspaceId, taskId],
    queryFn: () => getOptimizationTask(workspaceId, taskId, sessionToken!),
    enabled: isAuthenticated && !!workspaceId && !!taskId,
  })
}

export function useCreateOptimizationTask(workspaceId: string) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (params: CreateOptimizationTaskRequest) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return createOptimizationTask(workspaceId, params, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [...OPTIMIZATION_TASKS_QUERY_KEY, workspaceId] })
    },
  })
}

export function useUpdateOptimizationTaskConfig(workspaceId: string, taskId: string) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (params: UpdateOptimizationTaskConfigRequest) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return updateOptimizationTaskConfig(workspaceId, taskId, params, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [...OPTIMIZATION_TASKS_QUERY_KEY, workspaceId] })
      queryClient.invalidateQueries({ queryKey: [...OPTIMIZATION_TASKS_QUERY_KEY, workspaceId, taskId] })
    },
  })
}
