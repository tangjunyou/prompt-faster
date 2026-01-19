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
import type { TaskHistoryResponse } from '@/types/generated/models/TaskHistoryResponse'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'
import type { RollbackRequest } from '@/types/generated/models/RollbackRequest'
import type { RollbackResponse } from '@/types/generated/models/RollbackResponse'

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

export async function getCheckpointList(
  taskId: string,
): Promise<CheckpointSummary[]> {
  const all: CheckpointSummary[] = []
  let offset = 0
  let total = 0

  do {
    const params = new URLSearchParams()
    params.set('include_archived', 'true')
    params.set('checkpoints_limit', '100')
    params.set('checkpoints_offset', String(offset))
    const queryString = params.toString()
    const url = `/tasks/${taskId}/history${queryString ? `?${queryString}` : ''}`

    const response = await getWithAuth<TaskHistoryResponse>(url, getToken())
    if (isApiError(response)) {
      throw new Error(response.error.message)
    }
    const page = response.data.checkpoints.checkpoints ?? []
    total = response.data.checkpoints.total ?? 0
    all.push(...page)
    offset = all.length
    if (page.length === 0) {
      break
    }
  } while (all.length < total)

  return all
}

export async function rollbackToCheckpoint(
  taskId: string,
  checkpointId: string,
): Promise<RollbackResponse> {
  const payload: RollbackRequest = {
    checkpointId,
    confirm: true,
  }
  const response = await postWithAuth<RollbackResponse>(
    `/tasks/${taskId}/rollback`,
    payload,
    getToken(),
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
