import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { getPromptVersions } from '../services/metaOptimizationService'

export const PROMPT_VERSIONS_QUERY_KEY = ['metaOptimization', 'promptVersions'] as const

export function usePromptVersions(params?: { limit?: number; offset?: number }) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...PROMPT_VERSIONS_QUERY_KEY, params?.limit ?? null, params?.offset ?? null],
    queryFn: () => getPromptVersions(sessionToken!, params),
    enabled: isAuthenticated,
    placeholderData: keepPreviousData,
  })
}
