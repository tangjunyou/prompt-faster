import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api'
import type { DeleteGenericConfigResponse } from '@/types/generated/api/DeleteGenericConfigResponse'
import type { SaveGenericConfigRequest } from '@/types/generated/api/SaveGenericConfigRequest'
import type { SaveGenericConfigResponse } from '@/types/generated/api/SaveGenericConfigResponse'

export async function saveGenericConfig(
  workspaceId: string,
  testSetId: string,
  params: SaveGenericConfigRequest,
  token: string
): Promise<SaveGenericConfigResponse> {
  const response = await apiRequestWithAuth<SaveGenericConfigResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}/generic/config`,
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

export async function deleteGenericConfig(
  workspaceId: string,
  testSetId: string,
  token: string
): Promise<DeleteGenericConfigResponse> {
  const response = await apiRequestWithAuth<DeleteGenericConfigResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}/generic/config`,
    { method: 'DELETE' },
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
