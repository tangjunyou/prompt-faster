/**
 * Checkpoint 列表 Hook（基于 History API）
 */

import { useQuery } from '@tanstack/react-query'
import { getCheckpointList } from '../services/recoveryService'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'

const CHECKPOINT_LIST_KEY = 'checkpoint-list'

export function useCheckpointList(
  taskId: string,
  options?: {
    enabled?: boolean
  }
) {
  const { enabled = true } = options ?? {}

  return useQuery<CheckpointSummary[], Error>({
    queryKey: [CHECKPOINT_LIST_KEY, taskId],
    queryFn: () => getCheckpointList(taskId),
    enabled: enabled && !!taskId,
    staleTime: 30 * 1000,
  })
}
