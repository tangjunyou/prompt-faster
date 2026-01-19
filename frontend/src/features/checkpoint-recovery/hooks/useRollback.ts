/**
 * 回滚 Hook
 */

import { useMutation, useQueryClient } from '@tanstack/react-query'
import { rollbackToCheckpoint } from '../services/recoveryService'
import type { RollbackResponse } from '@/types/generated/models/RollbackResponse'

type RollbackPayload = {
  taskId: string
  checkpointId: string
}

export function useRollback() {
  const queryClient = useQueryClient()
  return useMutation<RollbackResponse, Error, RollbackPayload>({
    mutationFn: ({ taskId, checkpointId }) =>
      rollbackToCheckpoint(taskId, checkpointId),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['task-history', variables.taskId] })
      queryClient.invalidateQueries({ queryKey: ['checkpoint-list', variables.taskId] })
    },
  })
}
