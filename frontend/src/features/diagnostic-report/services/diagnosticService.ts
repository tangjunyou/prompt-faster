/**
 * 诊断报告服务层
 */

import { getWithAuth, isApiError } from '@/lib/api'
import { getSessionToken } from '@/stores/useAuthStore'
import type { DiagnosticReport } from '@/types/generated/models/DiagnosticReport'
import type { FailedCaseDetail } from '@/types/generated/models/FailedCaseDetail'

function getToken(): string {
  const token = getSessionToken()
  if (!token) {
    throw new Error('未登录，请先登录')
  }
  return token
}

export async function getDiagnosticReport(
  taskId: string,
  failedCasesLimit?: number
): Promise<DiagnosticReport> {
  const query = failedCasesLimit ? `?failed_cases_limit=${failedCasesLimit}` : ''
  const response = await getWithAuth<DiagnosticReport>(
    `/tasks/${taskId}/diagnostic${query}`,
    getToken()
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}

export async function getFailedCaseDetail(
  taskId: string,
  caseId: string
): Promise<FailedCaseDetail> {
  const response = await getWithAuth<FailedCaseDetail>(
    `/tasks/${taskId}/diagnostic/cases/${caseId}`,
    getToken()
  )
  if (isApiError(response)) {
    throw new Error(response.error.message)
  }
  return response.data
}
