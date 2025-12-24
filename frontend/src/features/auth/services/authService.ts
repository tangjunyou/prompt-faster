/**
 * 认证 API 服务
 * 提供注册、登录、登出、获取当前用户等纯函数
 * 
 * Code Review Fix (Story 1.6):
 * - 使用 apiRequestWithAuth 统一鉴权注入点
 * - 禁止在 service 中自行拼接 Authorization header
 */

import { apiRequest, apiRequestWithAuth, type ApiResponse } from '@/lib/api'

/** 用户信息 */
export interface UserInfo {
  id: string
  username: string
}

/** 认证响应 */
export interface AuthResponse {
  session_token: string
  user: UserInfo
}

/** 登出响应 */
export interface LogoutResponse {
  message: string
}

/** 系统状态响应 */
export interface SystemStatusResponse {
  has_users: boolean
  requires_registration: boolean
}

/** 注册请求参数 */
export interface RegisterParams {
  username: string
  password: string
}

/** 登录请求参数 */
export interface LoginParams {
  username: string
  password: string
}

/**
 * 获取系统状态
 * 用于判断是否需要显示注册页面
 */
export async function getSystemStatus(): Promise<ApiResponse<SystemStatusResponse>> {
  return apiRequest<SystemStatusResponse>('/auth/status', { method: 'GET' })
}

/**
 * 用户注册
 */
export async function register(params: RegisterParams): Promise<ApiResponse<AuthResponse>> {
  return apiRequest<AuthResponse>('/auth/register', {
    method: 'POST',
    body: JSON.stringify(params),
  })
}

/**
 * 用户登录
 */
export async function login(params: LoginParams): Promise<ApiResponse<AuthResponse>> {
  return apiRequest<AuthResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify(params),
  })
}

/**
 * 用户登出
 * Code Review Fix: 使用 apiRequestWithAuth 统一鉴权注入点
 */
export async function logout(token: string): Promise<ApiResponse<LogoutResponse>> {
  return apiRequestWithAuth<LogoutResponse>('/auth/logout', { method: 'POST' }, token)
}

/**
 * 获取当前用户信息
 * Code Review Fix: 使用 apiRequestWithAuth 统一鉴权注入点
 */
export async function getMe(token: string): Promise<ApiResponse<UserInfo>> {
  return apiRequestWithAuth<UserInfo>('/auth/me', { method: 'GET' }, token)
}
