/**
 * Health 服务 - 仅负责 API 调用
 */

import { get, isApiError } from '@/lib/api'
import type { HealthResponse } from '@/types/generated/api/HealthResponse'

export async function fetchHealth(): Promise<HealthResponse> {
  const response = await get<HealthResponse>('/health')
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
