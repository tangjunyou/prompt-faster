import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api'
import type { CreateOptimizationTaskRequest } from '@/types/generated/api/CreateOptimizationTaskRequest'
import type { OptimizationTaskListItemResponse } from '@/types/generated/api/OptimizationTaskListItemResponse'
import type { OptimizationTaskResponse } from '@/types/generated/api/OptimizationTaskResponse'

export async function listOptimizationTasks(
  workspaceId: string,
  token: string
): Promise<OptimizationTaskListItemResponse[]> {
  const response = await apiRequestWithAuth<OptimizationTaskListItemResponse[]>(
    `/workspaces/${workspaceId}/optimization-tasks`,
    { method: 'GET' },
    token
  )

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message)
    }
    throw new Error(response.error.message)
  }

  return response.data
}

export async function createOptimizationTask(
  workspaceId: string,
  params: CreateOptimizationTaskRequest,
  token: string
): Promise<OptimizationTaskResponse> {
  const response = await apiRequestWithAuth<OptimizationTaskResponse>(
    `/workspaces/${workspaceId}/optimization-tasks`,
    {
      method: 'POST',
      body: JSON.stringify(params),
    },
    token
  )

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message)
    }
    throw new Error(response.error.message)
  }

  return response.data
}

export async function getOptimizationTask(
  workspaceId: string,
  taskId: string,
  token: string
): Promise<OptimizationTaskResponse> {
  const response = await apiRequestWithAuth<OptimizationTaskResponse>(
    `/workspaces/${workspaceId}/optimization-tasks/${taskId}`,
    { method: 'GET' },
    token
  )

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message)
    }
    throw new Error(response.error.message)
  }

  return response.data
}

