import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api'
import type { GenericLlmModelsResponse } from '@/types/generated/api/GenericLlmModelsResponse'

export async function listTeacherModels(token: string): Promise<string[]> {
  const response = await apiRequestWithAuth<GenericLlmModelsResponse>(
    '/auth/generic-llm/models',
    { method: 'GET' },
    token
  )

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message)
    }
    throw new Error(response.error.message)
  }

  return response.data.models
}

