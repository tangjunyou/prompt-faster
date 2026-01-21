import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api'
import type { CreateTeacherPromptInput } from '@/types/generated/models/CreateTeacherPromptInput'
import type { MetaOptimizationOverview } from '@/types/generated/models/MetaOptimizationOverview'
import type { MetaOptimizationTaskSummary } from '@/types/generated/models/MetaOptimizationTaskSummary'
import type { PromptPreviewRequest } from '@/types/generated/models/PromptPreviewRequest'
import type { PromptPreviewResponse } from '@/types/generated/models/PromptPreviewResponse'
import type { PromptValidationRequest } from '@/types/generated/models/PromptValidationRequest'
import type { PromptValidationResult } from '@/types/generated/models/PromptValidationResult'
import type { TeacherPrompt } from '@/types/generated/models/TeacherPrompt'
import type { TeacherPromptVersion } from '@/types/generated/models/TeacherPromptVersion'

const BASE_PATH = '/meta-optimization'

export async function getPromptVersions(
  token: string,
  params?: { limit?: number; offset?: number }
): Promise<TeacherPromptVersion[]> {
  const query = new URLSearchParams()
  if (params?.limit) query.set('limit', String(params.limit))
  if (params?.offset) query.set('offset', String(params.offset))
  const suffix = query.toString() ? `?${query.toString()}` : ''

  const response = await apiRequestWithAuth<TeacherPromptVersion[]>(
    `${BASE_PATH}/prompts${suffix}`,
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

export async function getPromptVersion(id: string, token: string): Promise<TeacherPrompt> {
  const response = await apiRequestWithAuth<TeacherPrompt>(
    `${BASE_PATH}/prompts/${id}`,
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

export async function createPromptVersion(
  input: CreateTeacherPromptInput,
  token: string
): Promise<TeacherPromptVersion> {
  const response = await apiRequestWithAuth<TeacherPromptVersion>(
    `${BASE_PATH}/prompts`,
    {
      method: 'POST',
      body: JSON.stringify(input),
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

export async function activatePromptVersion(id: string, token: string): Promise<TeacherPrompt> {
  const response = await apiRequestWithAuth<TeacherPrompt>(
    `${BASE_PATH}/prompts/${id}/activate`,
    { method: 'PUT', body: JSON.stringify({}) },
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

export async function getOverview(token: string): Promise<MetaOptimizationOverview> {
  const response = await apiRequestWithAuth<MetaOptimizationOverview>(
    `${BASE_PATH}/stats`,
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

export async function getMetaOptimizationTasks(
  token: string,
  params?: { limit?: number; offset?: number }
): Promise<MetaOptimizationTaskSummary[]> {
  const query = new URLSearchParams()
  if (params?.limit) query.set('limit', String(params.limit))
  if (params?.offset) query.set('offset', String(params.offset))
  const suffix = query.toString() ? `?${query.toString()}` : ''

  const response = await apiRequestWithAuth<MetaOptimizationTaskSummary[]>(
    `${BASE_PATH}/tasks${suffix}`,
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

export async function previewPrompt(
  request: PromptPreviewRequest,
  token: string
): Promise<PromptPreviewResponse> {
  const response = await apiRequestWithAuth<PromptPreviewResponse>(
    `${BASE_PATH}/prompts/preview`,
    {
      method: 'POST',
      body: JSON.stringify(request),
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

export async function validatePrompt(
  request: PromptValidationRequest,
  token: string
): Promise<PromptValidationResult> {
  const response = await apiRequestWithAuth<PromptValidationResult>(
    `${BASE_PATH}/prompts/validate`,
    {
      method: 'POST',
      body: JSON.stringify(request),
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
