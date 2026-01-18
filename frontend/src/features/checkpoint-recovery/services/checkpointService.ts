/**
 * Checkpoint 服务层
 * 封装 Checkpoint 相关 API 调用
 */

import { getWithAuth, isApiError } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { CheckpointListResponse } from '@/types/generated/models/CheckpointListResponse'
import type { CheckpointResponse } from '@/types/generated/models/CheckpointResponse'

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

/**
 * 获取任务的 Checkpoint 列表
 */
export async function getCheckpoints(
  taskId: string,
  limit?: number
): Promise<CheckpointListResponse> {
  const params = new URLSearchParams()
  if (limit !== undefined) {
    params.set('limit', String(limit))
  }
  const queryString = params.toString()
  const url = `/tasks/${taskId}/checkpoints${queryString ? `?${queryString}` : ''}`

  const response = await getWithAuth<CheckpointListResponse>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

/**
 * 获取单个 Checkpoint 详情
 */
export async function getCheckpointDetail(
  checkpointId: string
): Promise<CheckpointResponse> {
  const url = `/checkpoints/${checkpointId}`
  const response = await getWithAuth<CheckpointResponse>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
