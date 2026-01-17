/**
 * 历史迭代 Hooks
 * 使用 TanStack Query 管理历史迭代数据
 */

import { useQuery } from '@tanstack/react-query'
import { getIterationHistory, getIterationDetail } from '../services/iterationHistoryService'
import type { IterationHistorySummary } from '@/types/generated/models/IterationHistorySummary'
import type { IterationHistoryDetail } from '@/types/generated/models/IterationHistoryDetail'

/** QueryKey 前缀 */
const ITERATIONS_KEY = 'iterations'

/**
 * 获取历史迭代列表 Hook
 * @param taskId 任务 ID
 * @param options 选项
 */
export function useIterationHistory(
  taskId: string,
  options?: {
    limit?: number
    enabled?: boolean
  }
) {
  const { limit, enabled = true } = options ?? {}

  return useQuery<IterationHistorySummary[], Error>({
    queryKey: [ITERATIONS_KEY, taskId, { limit }],
    queryFn: () => getIterationHistory(taskId, limit),
    enabled: enabled && !!taskId,
    staleTime: 30 * 1000, // 30 秒
  })
}

/**
 * 获取单个迭代详情 Hook
 * @param taskId 任务 ID
 * @param iterationId 迭代 ID
 * @param options 选项
 */
export function useIterationDetail(
  taskId: string,
  iterationId: string | null,
  options?: {
    enabled?: boolean
  }
) {
  const { enabled = true } = options ?? {}

  return useQuery<IterationHistoryDetail, Error>({
    queryKey: [ITERATIONS_KEY, taskId, iterationId],
    queryFn: () => getIterationDetail(taskId, iterationId!),
    enabled: enabled && !!taskId && !!iterationId,
    staleTime: 30 * 1000, // 30 秒
  })
}
