/**
 * Checkpoint Hooks
 * 使用 TanStack Query 管理 Checkpoint 数据
 */

import { useQuery } from '@tanstack/react-query'
import { getCheckpointDetail, getCheckpoints } from '../services/checkpointService'
import type { CheckpointListResponse } from '@/types/generated/models/CheckpointListResponse'
import type { CheckpointResponse } from '@/types/generated/models/CheckpointResponse'

const CHECKPOINTS_KEY = 'checkpoints'

export function useCheckpoints(
  taskId: string,
  options?: {
    limit?: number
    enabled?: boolean
  }
) {
  const { limit, enabled = true } = options ?? {}

  return useQuery<CheckpointListResponse, Error>({
    queryKey: [CHECKPOINTS_KEY, taskId, { limit }],
    queryFn: () => getCheckpoints(taskId, limit),
    enabled: enabled && !!taskId,
    staleTime: 30 * 1000,
  })
}

export function useCheckpointDetail(
  checkpointId: string | null,
  options?: {
    enabled?: boolean
  }
) {
  const { enabled = true } = options ?? {}

  return useQuery<CheckpointResponse, Error>({
    queryKey: [CHECKPOINTS_KEY, checkpointId],
    queryFn: () => getCheckpointDetail(checkpointId!),
    enabled: enabled && !!checkpointId,
    staleTime: 30 * 1000,
  })
}
