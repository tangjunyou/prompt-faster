import { QueryClient } from '@tanstack/react-query'
import { UnauthorizedError } from './api'

/**
 * TanStack Query 客户端配置
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60, // 1 分钟
      gcTime: 1000 * 60 * 5, // 5 分钟
      retry: (failureCount, error) => {
        if (error instanceof UnauthorizedError) return false
        return failureCount < 1
      },
      refetchOnWindowFocus: false,
    },
  },
})
