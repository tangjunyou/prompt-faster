/**
 * 结果查看 Hook
 */

import { useQuery } from '@tanstack/react-query'
import { getResult } from '../services/resultService'
import type { TaskResultView } from '@/types/generated/models/TaskResultView'

const RESULT_KEY = 'taskResult'

export function useResult(
  taskId: string,
  options?: {
    enabled?: boolean
    staleTime?: number
  }
) {
  const { enabled = true, staleTime = 10 * 1000 } = options ?? {}

  return useQuery<TaskResultView, Error>({
    queryKey: [RESULT_KEY, taskId],
    queryFn: () => getResult(taskId),
    enabled: enabled && !!taskId,
    staleTime,
  })
}
