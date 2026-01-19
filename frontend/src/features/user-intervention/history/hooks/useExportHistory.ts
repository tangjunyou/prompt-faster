import { useMutation } from '@tanstack/react-query'

import { exportHistory } from '../services/taskHistoryService'

export function useExportHistory() {
  return useMutation({
    mutationFn: ({ taskId }: { taskId: string }) => exportHistory(taskId),
  })
}
