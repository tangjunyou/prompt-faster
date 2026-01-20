import { useQuery } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { getOverview } from '../services/metaOptimizationService'

export const META_OPTIMIZATION_OVERVIEW_QUERY_KEY = ['metaOptimization', 'overview'] as const

export function useMetaOptimizationOverview() {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: META_OPTIMIZATION_OVERVIEW_QUERY_KEY,
    queryFn: () => getOverview(sessionToken!),
    enabled: isAuthenticated,
  })
}
