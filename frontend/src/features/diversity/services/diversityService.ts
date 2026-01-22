/**
 * 多样性分析服务层
 */

import { getWithAuth, isApiError, postWithAuth } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { DiversityAnalysisResult } from '@/types/generated/models/DiversityAnalysisResult'
import type { DiversityBaseline } from '@/types/generated/models/DiversityBaseline'

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

export async function getDiversityAnalysis(
  taskId: string
): Promise<DiversityAnalysisResult | null> {
  const response = await getWithAuth<DiversityAnalysisResult>(
    `/tasks/${taskId}/diversity`,
    getToken()
  )
  if (isApiError(response)) {
    if (response.error.code === 'NOT_FOUND') {
      return null
    }
    throw new Error(response.error.message)
  }
  return response.data
}

export async function recordBaseline(taskId: string): Promise<DiversityBaseline> {
  const response = await postWithAuth<DiversityBaseline>(
    `/tasks/${taskId}/diversity/baseline`,
    {},
    getToken()
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
