/**
 * 未完成任务检测 Hook
 */

import { useQuery } from '@tanstack/react-query'
import { getUnfinishedTasks } from '../services/recoveryService'
import { useAuthStore } from '@/stores/useAuthStore'
import type { UnfinishedTasksResponse } from '@/types/generated/models/UnfinishedTasksResponse'

const UNFINISHED_TASKS_KEY = ['recovery', 'unfinished-tasks'] as const

export function useUnfinishedTasks(options?: { enabled?: boolean }) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const enabled = options?.enabled ?? true

  return useQuery<UnfinishedTasksResponse, Error>({
    queryKey: UNFINISHED_TASKS_KEY,
    queryFn: getUnfinishedTasks,
    enabled: enabled && !!sessionToken,
    staleTime: 30 * 1000,
  })
}

export function unfinishedTasksQueryKey() {
  return UNFINISHED_TASKS_KEY
}
