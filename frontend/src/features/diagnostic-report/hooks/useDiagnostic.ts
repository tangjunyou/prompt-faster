/**
 * 诊断报告 Hook
 */

import { useQuery } from '@tanstack/react-query'
import { getDiagnosticReport } from '../services/diagnosticService'
import type { DiagnosticReport } from '@/types/generated/models/DiagnosticReport'

const DIAGNOSTIC_KEY = 'diagnosticReport'

export function useDiagnostic(
  taskId: string,
  options?: {
    enabled?: boolean
    staleTime?: number
    failedCasesLimit?: number
  }
) {
  const { enabled = true, staleTime = 10 * 1000, failedCasesLimit } = options ?? {}

  return useQuery<DiagnosticReport, Error>({
    queryKey: [DIAGNOSTIC_KEY, taskId, failedCasesLimit ?? null],
    queryFn: () => getDiagnosticReport(taskId, failedCasesLimit),
    enabled: enabled && !!taskId,
    staleTime,
  })
}
