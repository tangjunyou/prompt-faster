import { useQuery } from '@tanstack/react-query'

import { getTimeline } from '../services/taskHistoryService'
import type { HistoryEventFilter } from '../services/taskHistoryService'
import type { TimelineResponse } from '@/types/generated/models/TimelineResponse'

const TIMELINE_KEY = 'task-history-timeline'

export function useTimeline(
  taskId: string,
  filter?: HistoryEventFilter,
  options?: { limit?: number; offset?: number; enabled?: boolean }
) {
  const { enabled = true, ...rest } = options ?? {}

  return useQuery<TimelineResponse, Error>({
    queryKey: [TIMELINE_KEY, taskId, filter, rest],
    queryFn: () => getTimeline(taskId, filter, rest),
    enabled: enabled && !!taskId,
    staleTime: 30 * 1000,
  })
}
