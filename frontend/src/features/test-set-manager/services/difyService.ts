import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api'
import type { DifyVariablesResponse } from '@/types/generated/api/DifyVariablesResponse'
import type { SaveDifyConfigRequest } from '@/types/generated/api/SaveDifyConfigRequest'
import type { SaveDifyConfigResponse } from '@/types/generated/api/SaveDifyConfigResponse'

export async function refreshDifyVariables(
  workspaceId: string,
  testSetId: string,
  token: string
): Promise<DifyVariablesResponse> {
  const response = await apiRequestWithAuth<DifyVariablesResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}/dify/variables/refresh`,
    { method: 'POST' },
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

export async function saveDifyConfig(
  workspaceId: string,
  testSetId: string,
  params: SaveDifyConfigRequest,
  token: string
): Promise<SaveDifyConfigResponse> {
  const response = await apiRequestWithAuth<SaveDifyConfigResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}/dify/config`,
    {
      method: 'PUT',
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

