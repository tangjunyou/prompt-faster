import { UnauthorizedError, apiRequestWithAuth, delWithAuth, isApiError } from '@/lib/api'

export interface WorkspaceResponse {
  id: string
  name: string
  description: string | null
  created_at: number
  updated_at: number
}

export interface CreateWorkspaceParams {
  name: string
  description?: string
}

interface DeleteWorkspaceResponse {
  message: string
}

export async function listWorkspaces(token: string): Promise<WorkspaceResponse[]> {
  const response = await apiRequestWithAuth<WorkspaceResponse[]>(
    '/workspaces',
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

export async function createWorkspace(
  params: CreateWorkspaceParams,
  token: string
): Promise<WorkspaceResponse> {
  const response = await apiRequestWithAuth<WorkspaceResponse>(
    '/workspaces',
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

export async function getWorkspace(id: string, token: string): Promise<WorkspaceResponse> {
  const response = await apiRequestWithAuth<WorkspaceResponse>(
    `/workspaces/${id}`,
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

export async function deleteWorkspace(id: string, token: string): Promise<DeleteWorkspaceResponse> {
  const response = await delWithAuth<DeleteWorkspaceResponse>(`/workspaces/${id}`, token)

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message)
    }
    throw new Error(response.error.message)
  }

  return response.data
}
