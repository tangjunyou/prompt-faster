/**
 * 迭代控制服务层
 *
 * 提供增加轮数、获取候选 Prompt 列表和终止任务的 API 调用
 */

import { apiRequestWithAuth, type ApiResponse } from '@/lib/api'
import type { AddRoundsResponse } from '@/types/generated/models/AddRoundsResponse'
import type { CandidatePromptListResponse } from '@/types/generated/models/CandidatePromptListResponse'
import type { TerminateTaskResponse } from '@/types/generated/models/TerminateTaskResponse'

/**
 * 增加迭代轮数
 * @param taskId 任务 ID
 * @param additionalRounds 要增加的轮数（1-100）
 * @param token 会话令牌
 * @param correlationId 关联 ID
 */
export async function addRounds(
  taskId: string,
  additionalRounds: number,
  token: string,
  correlationId?: string
): Promise<ApiResponse<AddRoundsResponse>> {
  return apiRequestWithAuth<AddRoundsResponse>(
    `/tasks/${taskId}/config`,
    {
      method: 'PATCH',
      body: JSON.stringify({ additionalRounds }),
    },
    token,
    30000,
    correlationId
  )
}

/**
 * 获取候选 Prompt 列表
 * @param taskId 任务 ID
 * @param token 会话令牌
 * @param correlationId 关联 ID
 */
export async function getCandidates(
  taskId: string,
  token: string,
  correlationId?: string,
  options?: { limit?: number; offset?: number }
): Promise<ApiResponse<CandidatePromptListResponse>> {
  const params = new URLSearchParams()
  if (options?.limit !== undefined) {
    params.set('limit', String(options.limit))
  }
  if (options?.offset !== undefined) {
    params.set('offset', String(options.offset))
  }
  const query = params.toString()
  return apiRequestWithAuth<CandidatePromptListResponse>(
    `/tasks/${taskId}/candidates${query ? `?${query}` : ''}`,
    { method: 'GET' },
    token,
    30000,
    correlationId
  )
}

/**
 * 终止任务
 * @param taskId 任务 ID
 * @param token 会话令牌
 * @param selectedIterationId 选定的迭代 ID（可选）
 * @param correlationId 关联 ID
 */
export async function terminateTask(
  taskId: string,
  token: string,
  selectedIterationId?: string,
  correlationId?: string
): Promise<ApiResponse<TerminateTaskResponse>> {
  return apiRequestWithAuth<TerminateTaskResponse>(
    `/tasks/${taskId}/terminate`,
    {
      method: 'POST',
      body: JSON.stringify({ selectedIterationId: selectedIterationId ?? null }),
    },
    token,
    30000,
    correlationId
  )
}

export type { AddRoundsRequest } from '@/types/generated/models/AddRoundsRequest'
export type { AddRoundsResponse } from '@/types/generated/models/AddRoundsResponse'
export type { CandidatePromptSummary } from '@/types/generated/models/CandidatePromptSummary'
export type { CandidatePromptListResponse } from '@/types/generated/models/CandidatePromptListResponse'
export type { TerminateTaskRequest } from '@/types/generated/models/TerminateTaskRequest'
export type { TerminateTaskResponse } from '@/types/generated/models/TerminateTaskResponse'
