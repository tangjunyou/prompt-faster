/**
 * 任务历史聚合服务层
 * 封装任务历史聚合 API 调用
 */

import { getWithAuth, isApiError } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { TaskHistoryResponse } from '@/types/generated/models/TaskHistoryResponse'

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

export async function getTaskHistory(
  taskId: string,
  options?: {
    includeArchived?: boolean
    iterationsLimit?: number
    checkpointsLimit?: number
    checkpointsOffset?: number
  }
): Promise<TaskHistoryResponse> {
  const params = new URLSearchParams()
  if (options?.includeArchived !== undefined) {
    params.set('include_archived', String(options.includeArchived))
  }
  if (options?.iterationsLimit !== undefined) {
    params.set('iterations_limit', String(options.iterationsLimit))
  }
  if (options?.checkpointsLimit !== undefined) {
    params.set('checkpoints_limit', String(options.checkpointsLimit))
  }
  if (options?.checkpointsOffset !== undefined) {
    params.set('checkpoints_offset', String(options.checkpointsOffset))
  }
  const queryString = params.toString()
  const url = `/tasks/${taskId}/history${queryString ? `?${queryString}` : ''}`

  const response = await getWithAuth<TaskHistoryResponse>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
