/**
 * Recovery 服务层
 * 封装断点恢复与连接状态 API 调用
 */

import { get, getWithAuth, isApiError, postWithAuth } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { RecoveryRequest } from '@/types/generated/models/RecoveryRequest'
import type { RecoveryResponse } from '@/types/generated/models/RecoveryResponse'
import type { UnfinishedTasksResponse } from '@/types/generated/models/UnfinishedTasksResponse'
import type { ConnectivityResponse } from '@/types/generated/models/ConnectivityResponse'

type AbortRecoveryResponse = { success: boolean; message: string }

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

export async function getUnfinishedTasks(): Promise<UnfinishedTasksResponse> {
  const response = await getWithAuth<UnfinishedTasksResponse>(
    '/recovery/unfinished-tasks',
    getToken(),
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function recoverTask(
  taskId: string,
  checkpointId?: string,
): Promise<RecoveryResponse> {
  const payload: RecoveryRequest = { checkpointId: checkpointId ?? null }
  const response = await postWithAuth<RecoveryResponse>(
    `/recovery/tasks/${taskId}/recover`,
    payload,
    getToken(),
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function abortRecovery(taskId: string): Promise<AbortRecoveryResponse> {
  const response = await postWithAuth<AbortRecoveryResponse>(
    `/recovery/tasks/${taskId}/abort`,
    {},
    getToken(),
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function getConnectivity(): Promise<ConnectivityResponse> {
  const response = await get<ConnectivityResponse>('/connectivity')
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
