import { useQuery } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { listTeacherModels } from '../services/teacherModelService'

export const TEACHER_MODELS_QUERY_KEY = ['teacherModels'] as const

export function useTeacherModels() {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: TEACHER_MODELS_QUERY_KEY,
    queryFn: () => listTeacherModels(sessionToken!),
    enabled: isAuthenticated,
    staleTime: 60_000,
  })
}

