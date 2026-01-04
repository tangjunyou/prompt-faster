import { UnauthorizedError, apiRequestWithAuth, delWithAuth, isApiError } from '@/lib/api'
import type { CreateTestSetRequest } from '@/types/generated/api/CreateTestSetRequest'
import type { DeleteTestSetResponse } from '@/types/generated/api/DeleteTestSetResponse'
import type { TestSetListItemResponse } from '@/types/generated/api/TestSetListItemResponse'
import type { TestSetResponse } from '@/types/generated/api/TestSetResponse'
import type { UpdateTestSetRequest } from '@/types/generated/api/UpdateTestSetRequest'

export async function listTestSets(
  workspaceId: string,
  token: string
): Promise<TestSetListItemResponse[]> {
  const response = await apiRequestWithAuth<TestSetListItemResponse[]>(
    `/workspaces/${workspaceId}/test-sets`,
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

export async function createTestSet(
  workspaceId: string,
  params: CreateTestSetRequest,
  token: string
): Promise<TestSetResponse> {
  const response = await apiRequestWithAuth<TestSetResponse>(
    `/workspaces/${workspaceId}/test-sets`,
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

export async function getTestSet(
  workspaceId: string,
  testSetId: string,
  token: string
): Promise<TestSetResponse> {
  const response = await apiRequestWithAuth<TestSetResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}`,
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

export async function updateTestSet(
  workspaceId: string,
  testSetId: string,
  params: UpdateTestSetRequest,
  token: string
): Promise<TestSetResponse> {
  const response = await apiRequestWithAuth<TestSetResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}`,
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

export async function deleteTestSet(
  workspaceId: string,
  testSetId: string,
  token: string
): Promise<DeleteTestSetResponse> {
  const response = await delWithAuth<DeleteTestSetResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}`,
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
