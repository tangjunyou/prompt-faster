/**
 * 迭代控制 Hooks
 *
 * 使用 TanStack Query 管理迭代控制相关的状态和操作
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { getSessionToken } from '@/stores/useAuthStore'
import { generateCorrelationId } from '@/stores/useTaskStore'
import { isApiError, isApiSuccess } from '@/lib/api'
import { OPTIMIZATION_TASKS_QUERY_KEY } from '@/features/task-config/hooks/useOptimizationTasks'
import {
  addRounds,
  getCandidates,
  terminateTask,
  type AddRoundsResponse,
  type TerminateTaskResponse,
} from '../services/iterationControlService'
import type { CandidatePromptListResponse } from '@/types/generated/models/CandidatePromptListResponse'

/** 候选 Prompt 列表 Query Key */
export const candidatesQueryKey = (
  taskId: string,
  options?: { limit?: number; offset?: number },
) => ['candidates', taskId, options?.limit ?? null, options?.offset ?? null] as const

/** 任务配置 Query Key（用于 invalidate） */
export const taskConfigQueryKey = (workspaceId: string, taskId: string) =>
  [...OPTIMIZATION_TASKS_QUERY_KEY, workspaceId, taskId] as const

/**
 * 获取候选 Prompt 列表 Hook
 */
export function useCandidates(
  taskId: string,
  enabled = true,
  options?: {
    limit?: number
    offset?: number
    onSuccess?: (data: CandidatePromptListResponse) => void
  },
) {
  const token = getSessionToken()

  return useQuery({
    queryKey: candidatesQueryKey(taskId, options),
    queryFn: async () => {
      const currentToken = getSessionToken()
      if (!currentToken) {
        throw new Error('未登录')
      }
      const correlationId = generateCorrelationId()
      const response = await getCandidates(taskId, currentToken, correlationId, options)

      if (isApiError(response)) {
        throw new Error(response.error.message)
      }

      return response.data
    },
    enabled: enabled && !!token,
    staleTime: 30000,
    onSuccess: options?.onSuccess,
  })
}

/**
 * 增加轮数 Mutation Hook
 */
export function useAddRounds(taskId: string, workspaceId?: string) {
  const queryClient = useQueryClient()

  return useMutation<AddRoundsResponse, Error, number>({
    mutationFn: async (additionalRounds: number) => {
      const token = getSessionToken()
      if (!token) {
        throw new Error('未登录')
      }
      const correlationId = generateCorrelationId()
      const response = await addRounds(taskId, additionalRounds, token, correlationId)

      if (isApiError(response)) {
        throw new Error(response.error.message)
      }

      if (isApiSuccess(response)) {
        return response.data
      }

      throw new Error('未知错误')
    },
    onSuccess: () => {
      if (workspaceId) {
        queryClient.invalidateQueries({ queryKey: taskConfigQueryKey(workspaceId, taskId) })
      }
      queryClient.invalidateQueries({ queryKey: OPTIMIZATION_TASKS_QUERY_KEY })
    },
  })
}

/**
 * 终止任务 Mutation Hook
 */
export function useTerminateTask(taskId: string, workspaceId?: string) {
  const queryClient = useQueryClient()

  return useMutation<TerminateTaskResponse, Error, string | undefined>({
    mutationFn: async (selectedIterationId?: string) => {
      const token = getSessionToken()
      if (!token) {
        throw new Error('未登录')
      }
      const correlationId = generateCorrelationId()
      const response = await terminateTask(taskId, token, selectedIterationId, correlationId)

      if (isApiError(response)) {
        throw new Error(response.error.message)
      }

      if (isApiSuccess(response)) {
        return response.data
      }

      throw new Error('未知错误')
    },
    onSuccess: () => {
      if (workspaceId) {
        queryClient.invalidateQueries({ queryKey: taskConfigQueryKey(workspaceId, taskId) })
      }
      queryClient.invalidateQueries({ queryKey: ['candidates', taskId] })
      queryClient.invalidateQueries({ queryKey: OPTIMIZATION_TASKS_QUERY_KEY })
    },
  })
}
