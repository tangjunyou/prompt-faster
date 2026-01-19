/**
 * 任务历史聚合 Hook
 * 使用 TanStack Query 获取迭代历史 + Checkpoint 列表
 */

import { useQuery } from '@tanstack/react-query'
import { getTaskHistory } from '../services/taskHistoryService'
import type { TaskHistoryResponse } from '@/types/generated/models/TaskHistoryResponse'

const HISTORY_KEY = 'task-history'

export function useTaskHistory(
  taskId: string,
  options?: {
    includeArchived?: boolean
    iterationsLimit?: number
    checkpointsLimit?: number
    checkpointsOffset?: number
    enabled?: boolean
  }
) {
  const { enabled = true, ...rest } = options ?? {}

  return useQuery<TaskHistoryResponse, Error>({
    queryKey: [HISTORY_KEY, taskId, rest],
    queryFn: () => getTaskHistory(taskId, rest),
    enabled: enabled && !!taskId,
    staleTime: 30 * 1000,
  })
}
