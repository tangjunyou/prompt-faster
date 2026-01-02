import { QueryClient } from '@tanstack/react-query'
import { UnauthorizedError } from './api'

/**
 * TanStack Query 客户端配置
 */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // 统一缓存与重试策略：避免对 401 重试，减少无效请求
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
