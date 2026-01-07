/**
 * 认证状态管理 Store
 * 使用 Zustand 管理登录状态、sessionToken、currentUser
 * 
 * 安全约束:
 * - sessionToken 仅存在内存，不落 localStorage/sessionStorage
 * - 页面刷新会导致内存态丢失，用户需要重新登录（MVP 行为）
 */

import { create } from 'zustand'
import { isApiError, isApiSuccess, type ApiResponse } from '@/lib/api'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import {
  getMe as getMeRequest,
  login as loginRequest,
  register as registerRequest,
  type AuthResponse,
  type LoginParams,
  type RegisterParams,
} from '@/features/auth/services/authService'
import type { UserInfo } from '@/types/generated/api/UserInfo'

/** 认证状态 */
export type AuthStatus = 'unauthenticated' | 'authenticated' | 'loading'

/** 认证 Store 状态 */
interface AuthState {
  /** 认证状态 */
  authStatus: AuthStatus
  /** 会话令牌（仅存内存） */
  sessionToken: string | null
  /** 当前用户信息 */
  currentUser: UserInfo | null
  /** 系统是否需要注册（首次启动） */
  requiresRegistration: boolean | null
}

/** 认证 Store Actions */
interface AuthActions {
  /** 设置认证状态 */
  setAuthStatus: (status: AuthStatus) => void
  /** 登录成功 */
  loginSuccess: (token: string, user: UserInfo) => void
  /** 登录 */
  login: (params: LoginParams) => Promise<ApiResponse<AuthResponse>>
  /** 注册 */
  register: (params: RegisterParams) => Promise<ApiResponse<AuthResponse>>
  /** 登出 */
  logout: () => void
  /** 加载当前用户信息 */
  loadMe: () => Promise<ApiResponse<UserInfo>>
  /** 设置是否需要注册 */
  setRequiresRegistration: (requires: boolean) => void
  /** 重置状态 */
  reset: () => void
}

/** 初始状态 */
const initialState: AuthState = {
  authStatus: 'unauthenticated',
  sessionToken: null,
  currentUser: null,
  requiresRegistration: null,
}

/**
 * 认证 Store
 * 
 * 使用示例:
 * ```tsx
 * const { authStatus, currentUser, loginSuccess, logout } = useAuthStore()
 * ```
 */
export const useAuthStore = create<AuthState & AuthActions>((set, get) => ({
  ...initialState,

  setAuthStatus: (status) => set({ authStatus: status }),

  loginSuccess: (token, user) =>
    set({
      authStatus: 'authenticated',
      sessionToken: token,
      currentUser: user,
    }),

  login: async (params) => {
    set({ authStatus: 'loading' })

    const response = await loginRequest(params)
    if (isApiError(response)) {
      set({ authStatus: 'unauthenticated' })
      return response
    }

    set({
      authStatus: 'authenticated',
      sessionToken: response.data.session_token,
      currentUser: response.data.user,
    })

    return response
  },

  register: async (params) => {
    set({ authStatus: 'loading' })

    const response = await registerRequest(params)
    if (isApiError(response)) {
      set({ authStatus: 'unauthenticated' })
      return response
    }

    set({
      authStatus: 'authenticated',
      sessionToken: response.data.session_token,
      currentUser: response.data.user,
    })

    return response
  },

  logout: () => {
    useWorkspaceStore.getState().reset()
    set({
      authStatus: 'unauthenticated',
      sessionToken: null,
      currentUser: null,
    })
  },

  loadMe: async () => {
    const token = get().sessionToken
    if (!token) {
      return {
        error: {
          code: 'UNAUTHORIZED',
          message: '请先登录',
        },
      }
    }

    const response = await getMeRequest(token)
    if (isApiSuccess(response)) {
      set({ currentUser: response.data })
    }

    return response
  },

  setRequiresRegistration: (requires) =>
    set({ requiresRegistration: requires }),

  reset: () => set(initialState),
}))

/**
 * 获取当前 session token
 * 用于 API 请求注入 Authorization header
 */
export function getSessionToken(): string | null {
  return useAuthStore.getState().sessionToken
}
