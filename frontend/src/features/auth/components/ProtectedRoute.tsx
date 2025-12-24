/**
 * 路由守卫组件
 * 未登录用户访问受保护页面时重定向到登录页
 */

import type { ReactNode } from 'react'
import { Navigate } from 'react-router'
import { useAuthStore } from '@/stores/useAuthStore'

interface ProtectedRouteProps {
  children: ReactNode
}

export function ProtectedRoute({ children }: ProtectedRouteProps) {
  const { authStatus } = useAuthStore()

  // 加载中时显示空白
  if (authStatus === 'loading') {
    return null
  }

  // 未登录时重定向到登录页
  if (authStatus !== 'authenticated') {
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}
