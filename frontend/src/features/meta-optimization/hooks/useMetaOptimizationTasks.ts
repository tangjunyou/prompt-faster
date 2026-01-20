import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { getMetaOptimizationTasks } from '../services/metaOptimizationService'

export const META_OPTIMIZATION_TASKS_QUERY_KEY = ['metaOptimization', 'tasks'] as const

export function useMetaOptimizationTasks(params?: { limit?: number; offset?: number }) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...META_OPTIMIZATION_TASKS_QUERY_KEY, params?.limit ?? null, params?.offset ?? null],
    queryFn: () => getMetaOptimizationTasks(sessionToken!, params),
    enabled: isAuthenticated,
    placeholderData: keepPreviousData,
  })
}
