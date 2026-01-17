/**
 * 历史迭代服务层
 * 封装历史迭代相关的 API 调用
 */

import { getWithAuth, isApiError } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { IterationHistorySummary } from '@/types/generated/models/IterationHistorySummary'
import type { IterationHistoryDetail } from '@/types/generated/models/IterationHistoryDetail'

/**
 * 获取会话令牌
 */
function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

/**
 * 获取历史迭代列表
 * @param taskId 任务 ID
 * @param limit 最大返回条数（可选，默认 100）
 */
export async function getIterationHistory(
  taskId: string,
  limit?: number
): Promise<IterationHistorySummary[]> {
  const params = new URLSearchParams()
  if (limit !== undefined) {
    params.set('limit', String(limit))
  }
  const queryString = params.toString()
  const url = `/tasks/${taskId}/iterations${queryString ? `?${queryString}` : ''}`

  const response = await getWithAuth<IterationHistorySummary[]>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

/**
 * 获取单个迭代详情
 * @param taskId 任务 ID
 * @param iterationId 迭代 ID
 */
export async function getIterationDetail(
  taskId: string,
  iterationId: string
): Promise<IterationHistoryDetail> {
  const url = `/tasks/${taskId}/iterations/${iterationId}`
  const response = await getWithAuth<IterationHistoryDetail>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
