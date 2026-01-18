/**
 * 断点恢复相关 Hooks
 */

import { useMutation, useQueryClient } from '@tanstack/react-query'
import { abortRecovery, recoverTask } from '../services/recoveryService'
import { unfinishedTasksQueryKey } from './useUnfinishedTasks'
import type { RecoveryResponse } from '@/types/generated/models/RecoveryResponse'

export function useRecoverTask() {
  const queryClient = useQueryClient()
  return useMutation<
    RecoveryResponse,
    Error,
    { taskId: string; checkpointId?: string }
  >({
    mutationFn: ({ taskId, checkpointId }) => recoverTask(taskId, checkpointId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: unfinishedTasksQueryKey() })
    },
  })
}

export function useAbortRecovery() {
  const queryClient = useQueryClient()
  return useMutation<
    { success: boolean; message: string },
    Error,
    { taskId: string }
  >({
    mutationFn: ({ taskId }) => abortRecovery(taskId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: unfinishedTasksQueryKey() })
    },
  })
}
