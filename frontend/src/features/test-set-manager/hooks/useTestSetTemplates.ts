/**
 * 测试集模板 Hooks
 * - 以 workspace 为第一边界
 * - 使用 TanStack Query 管理缓存
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import type { SaveAsTemplateRequest } from '@/types/generated/api/SaveAsTemplateRequest'
import {
  getTestSetTemplate,
  listTestSetTemplates,
  saveAsTemplate,
} from '../services/testSetTemplateService'

const TEST_SET_TEMPLATES_QUERY_KEY = ['testSetTemplates'] as const

export function useTestSetTemplates(workspaceId: string, options?: { enabled?: boolean }) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...TEST_SET_TEMPLATES_QUERY_KEY, workspaceId],
    queryFn: () => listTestSetTemplates(workspaceId, sessionToken!),
    enabled: isAuthenticated && !!workspaceId && (options?.enabled ?? true),
  })
}

export function useTestSetTemplate(workspaceId: string, templateId: string) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...TEST_SET_TEMPLATES_QUERY_KEY, workspaceId, templateId],
    queryFn: () => getTestSetTemplate(workspaceId, templateId, sessionToken!),
    enabled: isAuthenticated && !!workspaceId && !!templateId,
  })
}

export function useSaveAsTemplate(workspaceId: string) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (args: { testSetId: string; params: SaveAsTemplateRequest }) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return saveAsTemplate(workspaceId, args.testSetId, args.params, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [...TEST_SET_TEMPLATES_QUERY_KEY, workspaceId] })
    },
  })
}
