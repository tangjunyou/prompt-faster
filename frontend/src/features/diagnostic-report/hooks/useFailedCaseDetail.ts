/**
 * 失败用例详情 Hook
 */

import { useQuery } from '@tanstack/react-query'
import { getFailedCaseDetail } from '../services/diagnosticService'
import type { FailedCaseDetail } from '@/types/generated/models/FailedCaseDetail'

const FAILED_CASE_DETAIL_KEY = 'failedCaseDetail'

export function useFailedCaseDetail(
  taskId: string,
  caseId?: string | null,
  options?: {
    enabled?: boolean
    staleTime?: number
  }
) {
  const { enabled = true, staleTime = 10 * 1000 } = options ?? {}

  return useQuery<FailedCaseDetail, Error>({
    queryKey: [FAILED_CASE_DETAIL_KEY, taskId, caseId ?? null],
    queryFn: () => getFailedCaseDetail(taskId, caseId ?? ''),
    enabled: enabled && !!taskId && !!caseId,
    staleTime,
  })
}
