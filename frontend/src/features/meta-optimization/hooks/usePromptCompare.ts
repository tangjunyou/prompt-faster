import { useEffect, useRef } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import { comparePrompts } from '../services/metaOptimizationService'
import type { PromptCompareRequest } from '@/types/generated/models/PromptCompareRequest'
import type { PromptCompareResponse } from '@/types/generated/models/PromptCompareResponse'

export function usePromptCompare() {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken
  const abortRef = useRef<AbortController | null>(null)

  useEffect(() => {
    return () => {
      abortRef.current?.abort()
    }
  }, [])

  return useMutation<PromptCompareResponse, Error, PromptCompareRequest>({
    mutationFn: async (request) => {
      if (!isAuthenticated || !sessionToken) {
        throw new Error('未登录')
      }
      abortRef.current?.abort()
      const controller = new AbortController()
      abortRef.current = controller
      return comparePrompts(request, sessionToken, controller.signal)
    },
    onSettled: () => {
      abortRef.current = null
    },
  })
}
