import { useMutation } from '@tanstack/react-query'
import { exportResult } from '../services/resultService'
import type { ResultExportFormat } from '@/types/generated/models/ResultExportFormat'

export function useExportResult() {
  return useMutation({
    mutationFn: ({ taskId, format }: { taskId: string; format: ResultExportFormat }) =>
      exportResult(taskId, format),
  })
}
