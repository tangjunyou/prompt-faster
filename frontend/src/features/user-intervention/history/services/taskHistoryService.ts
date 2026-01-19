/**
 * 任务历史聚合服务层
 * 封装任务历史聚合 API 调用
 */

import { getWithAuth, isApiError } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { TaskHistoryResponse } from '@/types/generated/models/TaskHistoryResponse'
import type { HistoryEventResponse } from '@/types/generated/models/HistoryEventResponse'
import type { TimelineResponse } from '@/types/generated/models/TimelineResponse'
import type { HistoryExportData } from '@/types/generated/models/HistoryExportData'

export interface HistoryEventFilter {
  eventTypes?: string[]
  actor?: 'system' | 'user'
  iterationMin?: number
  iterationMax?: number
  timeStart?: number
  timeEnd?: number
}

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

export async function getTaskHistory(
  taskId: string,
  options?: {
    includeArchived?: boolean
    iterationsLimit?: number
    checkpointsLimit?: number
    checkpointsOffset?: number
  }
): Promise<TaskHistoryResponse> {
  const params = new URLSearchParams()
  if (options?.includeArchived !== undefined) {
    params.set('include_archived', String(options.includeArchived))
  }
  if (options?.iterationsLimit !== undefined) {
    params.set('iterations_limit', String(options.iterationsLimit))
  }
  if (options?.checkpointsLimit !== undefined) {
    params.set('checkpoints_limit', String(options.checkpointsLimit))
  }
  if (options?.checkpointsOffset !== undefined) {
    params.set('checkpoints_offset', String(options.checkpointsOffset))
  }
  const queryString = params.toString()
  const url = `/tasks/${taskId}/history${queryString ? `?${queryString}` : ''}`

  const response = await getWithAuth<TaskHistoryResponse>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

function buildFilterParams(filter?: HistoryEventFilter): URLSearchParams {
  const params = new URLSearchParams()
  if (!filter) {
    return params
  }
  if (filter.eventTypes && filter.eventTypes.length > 0) {
    params.set('event_types', filter.eventTypes.join(','))
  }
  if (filter.actor) {
    params.set('actor', filter.actor)
  }
  if (filter.iterationMin !== undefined) {
    params.set('iteration_min', String(filter.iterationMin))
  }
  if (filter.iterationMax !== undefined) {
    params.set('iteration_max', String(filter.iterationMax))
  }
  if (filter.timeStart !== undefined) {
    params.set('time_start', String(filter.timeStart))
  }
  if (filter.timeEnd !== undefined) {
    params.set('time_end', String(filter.timeEnd))
  }
  return params
}

export async function getHistoryEvents(
  taskId: string,
  filter?: HistoryEventFilter,
  options?: { limit?: number; offset?: number }
): Promise<HistoryEventResponse> {
  const params = buildFilterParams(filter)
  if (options?.limit !== undefined) {
    params.set('limit', String(options.limit))
  }
  if (options?.offset !== undefined) {
    params.set('offset', String(options.offset))
  }
  const queryString = params.toString()
  const url = `/tasks/${taskId}/history/events${queryString ? `?${queryString}` : ''}`

  const response = await getWithAuth<HistoryEventResponse>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function getTimeline(
  taskId: string,
  filter?: HistoryEventFilter,
  options?: { limit?: number; offset?: number }
): Promise<TimelineResponse> {
  const params = buildFilterParams(filter)
  if (options?.limit !== undefined) {
    params.set('limit', String(options.limit))
  }
  if (options?.offset !== undefined) {
    params.set('offset', String(options.offset))
  }
  const queryString = params.toString()
  const url = `/tasks/${taskId}/history/timeline${queryString ? `?${queryString}` : ''}`

  const response = await getWithAuth<TimelineResponse>(url, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function exportHistory(
  taskId: string
): Promise<{ blob: Blob; filename: string }> {
  const response = await getWithAuth<HistoryExportData>(
    `/tasks/${taskId}/history/export`,
    getToken()
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }

  const data = response.data
  const taskName = data.task?.name ? data.task.name : taskId
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
  const safeTaskName = taskName
    .replace(/[\\/:*?"<>|]+/g, '_')
    .replace(/\s+/g, '_')
    .replace(/_+/g, '_')
    .replace(/^_+|_+$/g, '')
  const name = safeTaskName.length > 0 ? safeTaskName : taskId
  const filename = `${name}_history_${timestamp}.json`
  const payload = JSON.stringify(response, null, 2)
  const blob = new Blob([payload], { type: 'application/json' })
  return { blob, filename }
}
