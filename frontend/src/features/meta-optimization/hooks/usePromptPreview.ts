import { useMutation } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { previewPrompt } from '../services/metaOptimizationService'
import type { PromptPreviewRequest } from '@/types/generated/models/PromptPreviewRequest'
import type { PromptPreviewResponse } from '@/types/generated/models/PromptPreviewResponse'

export function usePromptPreview() {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useMutation<PromptPreviewResponse, Error, PromptPreviewRequest>({
    mutationFn: async (request) => {
      if (!isAuthenticated || !sessionToken) {
        throw new Error('未登录')
      }
      return previewPrompt(request, sessionToken)
    },
  })
}
