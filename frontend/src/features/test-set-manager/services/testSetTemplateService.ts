import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api'
import type { SaveAsTemplateRequest } from '@/types/generated/api/SaveAsTemplateRequest'
import type { TestSetTemplateListItemResponse } from '@/types/generated/api/TestSetTemplateListItemResponse'
import type { TestSetTemplateResponse } from '@/types/generated/api/TestSetTemplateResponse'

export async function listTestSetTemplates(
  workspaceId: string,
  token: string
): Promise<TestSetTemplateListItemResponse[]> {
  const response = await apiRequestWithAuth<TestSetTemplateListItemResponse[]>(
    `/workspaces/${workspaceId}/test-set-templates`,
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

export async function getTestSetTemplate(
  workspaceId: string,
  templateId: string,
  token: string
): Promise<TestSetTemplateResponse> {
  const response = await apiRequestWithAuth<TestSetTemplateResponse>(
    `/workspaces/${workspaceId}/test-set-templates/${templateId}`,
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

export async function saveAsTemplate(
  workspaceId: string,
  testSetId: string,
  params: SaveAsTemplateRequest,
  token: string
): Promise<TestSetTemplateResponse> {
  const response = await apiRequestWithAuth<TestSetTemplateResponse>(
    `/workspaces/${workspaceId}/test-sets/${testSetId}/save-as-template`,
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

