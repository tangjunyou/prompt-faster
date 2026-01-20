/**
 * 结果查看与导出服务层
 */

import { getWithAuth, isApiError } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { TaskResultView } from '@/types/generated/models/TaskResultView'
import type { ResultExportFormat } from '@/types/generated/models/ResultExportFormat'
import type { ExportResultResponse } from '@/types/generated/models/ExportResultResponse'

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

function sanitizeFilename(input: string): string {
  const sanitized = input
    .replace(/[\\/:*?"<>|]+/g, '_')
    .replace(/\s+/g, '_')
    .replace(/_+/g, '_')
    .replace(/^_+|_+$/g, '')
  return sanitized
}

function contentTypeFor(format: ResultExportFormat): string {
  switch (format) {
    case 'json':
      return 'application/json'
    case 'xml':
      return 'application/xml'
    default:
      return 'text/markdown'
  }
}

function fallbackFilename(taskId: string, format: ResultExportFormat): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-')
  const ext = format === 'json' ? 'json' : format === 'xml' ? 'xml' : 'md'
  return `${taskId}_prompt_${timestamp}.${ext}`
}

export async function getResult(taskId: string): Promise<TaskResultView> {
  const response = await getWithAuth<TaskResultView>(`/tasks/${taskId}/result`, getToken())
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function exportResult(
  taskId: string,
  format: ResultExportFormat
): Promise<{ blob: Blob; filename: string }> {
  const response = await getWithAuth<ExportResultResponse>(
    `/tasks/${taskId}/result/export?format=${format}`,
    getToken()
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }

  const payload = response.data
  const rawFilename = payload.filename || fallbackFilename(taskId, format)
  const safeName = sanitizeFilename(rawFilename)
  const filename = safeName.length > 0 ? safeName : fallbackFilename(taskId, format)
  const blob = new Blob([payload.content], { type: contentTypeFor(format) })
  return { blob, filename }
}
