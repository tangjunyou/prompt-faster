import { useQuery } from '@tanstack/react-query'

import { getHistoryEvents } from '../services/taskHistoryService'
import type { HistoryEventFilter } from '../services/taskHistoryService'
import type { HistoryEventResponse } from '@/types/generated/models/HistoryEventResponse'

const HISTORY_EVENTS_KEY = 'task-history-events'

export function useHistoryEvents(
  taskId: string,
  filter?: HistoryEventFilter,
  options?: { limit?: number; offset?: number; enabled?: boolean }
) {
  const { enabled = true, ...rest } = options ?? {}

  return useQuery<HistoryEventResponse, Error>({
    queryKey: [HISTORY_EVENTS_KEY, taskId, filter, rest],
    queryFn: () => getHistoryEvents(taskId, filter, rest),
    enabled: enabled && !!taskId,
    staleTime: 30 * 1000,
  })
}
