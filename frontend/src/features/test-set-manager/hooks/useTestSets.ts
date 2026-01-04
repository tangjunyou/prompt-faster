/**
 * 测试集管理 Hooks
 * - 以 workspace 为第一边界
 * - 使用 TanStack Query 管理缓存
 */

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useAuthStore } from '@/stores/useAuthStore'
import type { CreateTestSetRequest } from '@/types/generated/api/CreateTestSetRequest'
import type { UpdateTestSetRequest } from '@/types/generated/api/UpdateTestSetRequest'
import {
  createTestSet,
  deleteTestSet,
  getTestSet,
  listTestSets,
  updateTestSet,
} from '../services/testSetService'

const TEST_SETS_QUERY_KEY = ['testSets'] as const

export function useTestSets(workspaceId: string) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...TEST_SETS_QUERY_KEY, workspaceId],
    queryFn: () => listTestSets(workspaceId, sessionToken!),
    enabled: isAuthenticated && !!workspaceId,
  })
}

export function useTestSet(workspaceId: string, testSetId: string) {
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)
  const isAuthenticated = authStatus === 'authenticated' && !!sessionToken

  return useQuery({
    queryKey: [...TEST_SETS_QUERY_KEY, workspaceId, testSetId],
    queryFn: () => getTestSet(workspaceId, testSetId, sessionToken!),
    enabled: isAuthenticated && !!workspaceId && !!testSetId,
  })
}

export function useCreateTestSet(workspaceId: string) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (params: CreateTestSetRequest) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return createTestSet(workspaceId, params, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [...TEST_SETS_QUERY_KEY, workspaceId] })
    },
  })
}

export function useUpdateTestSet(workspaceId: string, testSetId: string) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (params: UpdateTestSetRequest) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return updateTestSet(workspaceId, testSetId, params, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [...TEST_SETS_QUERY_KEY, workspaceId] })
      queryClient.invalidateQueries({ queryKey: [...TEST_SETS_QUERY_KEY, workspaceId, testSetId] })
    },
  })
}

export function useDeleteTestSet(workspaceId: string) {
  const queryClient = useQueryClient()
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const authStatus = useAuthStore((state) => state.authStatus)

  return useMutation({
    mutationFn: async (testSetId: string) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录')
      }
      return deleteTestSet(workspaceId, testSetId, sessionToken)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [...TEST_SETS_QUERY_KEY, workspaceId] })
    },
  })
}

