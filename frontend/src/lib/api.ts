/**
 * API 客户端配置
 * 统一的 API 调用入口
 * 
 * Code Review Fix (Story 1.6):
 * - 添加 UnauthorizedError 用于 401 识别
 * - 添加 apiRequestWithAuth 统一鉴权注入点
 * - 支持 onUnauthorized 回调处理会话过期
 */

import type { ApiError as GeneratedApiError } from '@/types/generated/api/ApiError'
import type { ApiSuccess as GeneratedApiSuccess } from '@/types/generated/api/ApiSuccess'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api/v1'
const IS_TEST_ENV = import.meta.env.MODE === 'test'

/** 默认请求超时时间（毫秒）- Code Review Fix: 添加超时配置 */
const DEFAULT_TIMEOUT_MS = 30000

/**
 * 生成 correlationId
 * 避免依赖 store，保持 API 层可独立复用
 */
function createCorrelationId(): string {
  return `cid-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
}

/**
 * 未授权错误（401）
 * 用于 TanStack Query retry 判断，避免对 401 进行重试
 */
export class UnauthorizedError extends Error {
  constructor(message: string = '未授权，请重新登录') {
    super(message)
    this.name = 'UnauthorizedError'
  }
}

/** 全局 401 处理回调（由 useAuthStore 注册） */
let globalUnauthorizedHandler: (() => void) | null = null

/**
 * 注册全局 401 处理器
 * 应在应用启动时由 useAuthStore 调用
 */
export function registerUnauthorizedHandler(handler: () => void): void {
  globalUnauthorizedHandler = handler
}

/** API 成功响应（由 ts-rs 生成） */
export type ApiSuccess<T> = GeneratedApiSuccess<T>

/** API 错误响应（由 ts-rs 生成） */
export type ApiError = GeneratedApiError

/**
 * API 响应类型 - data 与 error 互斥
 * 调用方必须使用 isApiError/isApiSuccess 进行分支判断
 */
export type ApiResponse<T> = ApiSuccess<T> | ApiError

/**
 * 类型守卫：检查是否为错误响应
 */
export function isApiError<T>(response: ApiResponse<T>): response is ApiError {
  return 'error' in response
}

/**
 * 类型守卫：检查是否为成功响应
 */
export function isApiSuccess<T>(response: ApiResponse<T>): response is ApiSuccess<T> {
  return 'data' in response
}

/**
 * 发送 API 请求
 * 包含完整的错误处理：网络错误、非 JSON 响应、HTTP 错误、请求超时
 * 
 * @param endpoint - API 端点路径
 * @param options - fetch 请求选项
 * @param timeoutMs - 请求超时时间（毫秒），默认 30 秒
 */
export async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {},
  timeoutMs: number = DEFAULT_TIMEOUT_MS
): Promise<ApiResponse<T>> {
  const url = `${API_BASE_URL}${endpoint}`

  // Code Review Fix: 添加请求超时控制
  const shouldUseAbort = !IS_TEST_ENV && typeof AbortController !== 'undefined'
  const externalSignal = options.signal
  const controller = shouldUseAbort ? new AbortController() : null
  const timeoutId = controller ? setTimeout(() => controller.abort(), timeoutMs) : null

  if (controller && externalSignal) {
    if (externalSignal.aborted) {
      controller.abort()
    } else {
      externalSignal.addEventListener('abort', () => controller.abort(), { once: true })
    }
  }

  try {
    const response = await fetch(url, {
      ...options,
      signal: controller?.signal ?? externalSignal,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    })

    // 检查 Content-Type 是否为 JSON
    const contentType = response.headers.get('content-type')
    if (!contentType || !contentType.includes('application/json')) {
      // 非 JSON 响应，构造符合 ApiError 的兜底结构
      return {
        error: {
          code: 'INVALID_RESPONSE',
          message: `服务器返回非 JSON 响应: ${response.status} ${response.statusText}`,
        },
      }
    }

    const json = await response.json() as ApiResponse<T>

    // 严格执行 ApiResponse<T> 契约，不做 fallback
    return json
  } catch (err) {
    // 清理超时定时器
    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    // 处理超时错误
    if (err instanceof Error && err.name === 'AbortError') {
      return {
        error: {
          code: 'TIMEOUT_ERROR',
          message: '请求超时，请检查网络连接后重试',
        },
      }
    }

    // 网络错误或 JSON 解析失败，构造符合 ApiError 的兜底结构
    const message = err instanceof Error ? err.message : '网络请求失败'
    return {
      error: {
        code: 'NETWORK_ERROR',
        message,
      },
    }
  } finally {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
  }
}

/**
 * GET 请求
 */
export function get<T>(endpoint: string): Promise<ApiResponse<T>> {
  return apiRequest<T>(endpoint, { method: 'GET' })
}

/**
 * POST 请求
 */
export function post<T>(endpoint: string, body: unknown): Promise<ApiResponse<T>> {
  return apiRequest<T>(endpoint, {
    method: 'POST',
    body: JSON.stringify(body),
  })
}

/**
 * PUT 请求
 */
export function put<T>(endpoint: string, body: unknown): Promise<ApiResponse<T>> {
  return apiRequest<T>(endpoint, {
    method: 'PUT',
    body: JSON.stringify(body),
  })
}

/**
 * DELETE 请求
 */
export function del<T>(endpoint: string): Promise<ApiResponse<T>> {
  return apiRequest<T>(endpoint, { method: 'DELETE' })
}

/**
 * 带鉴权的 API 请求（统一鉴权注入点）
 * 
 * Story 1.6 Code Review Fix:
 * - 所有需要鉴权的请求必须通过此函数发送
 * - 统一注入 Authorization: Bearer <token> header
 * - 处理 401 响应，触发全局 unauthorized 处理器
 * 
 * @param endpoint - API 端点路径
 * @param options - fetch 请求选项
 * @param token - 会话令牌
 * @param timeoutMs - 请求超时时间（毫秒）
 */
export async function apiRequestWithAuth<T>(
  endpoint: string,
  options: RequestInit = {},
  token: string,
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
  correlationId?: string
): Promise<ApiResponse<T>> {
  const headerEntries = new Headers(options.headers ?? {})
  headerEntries.set('Authorization', `Bearer ${token}`)
  if (!headerEntries.has('x-correlation-id')) {
    headerEntries.set('x-correlation-id', correlationId ?? createCorrelationId())
  }

  const response = await apiRequest<T>(
    endpoint,
    {
      ...options,
      headers: Object.fromEntries(headerEntries.entries()),
    },
    timeoutMs
  )

  // 检查 401 未授权响应
  if (isApiError(response) && response.error.code === 'UNAUTHORIZED') {
    // 触发全局 401 处理器（清空 auth 状态 + 跳转登录页）
    if (globalUnauthorizedHandler) {
      globalUnauthorizedHandler()
    }
  }

  return response
}

/**
 * 带鉴权的 GET 请求
 */
export function getWithAuth<T>(
  endpoint: string,
  token: string,
  correlationId?: string
): Promise<ApiResponse<T>> {
  return apiRequestWithAuth<T>(endpoint, { method: 'GET' }, token, DEFAULT_TIMEOUT_MS, correlationId)
}

/**
 * 带鉴权的 POST 请求
 */
export function postWithAuth<T>(
  endpoint: string,
  body: unknown,
  token: string,
  correlationId?: string
): Promise<ApiResponse<T>> {
  return apiRequestWithAuth<T>(
    endpoint,
    {
      method: 'POST',
      body: JSON.stringify(body),
    },
    token,
    DEFAULT_TIMEOUT_MS,
    correlationId
  )
}

/**
 * 带鉴权的 DELETE 请求
 *
 * @param endpoint - API 端点路径（如 '/workspaces/123'）
 * @param token - 会话令牌
 * @returns Promise<ApiResponse<T>>
 */
export function delWithAuth<T>(
  endpoint: string,
  token: string,
  correlationId?: string
): Promise<ApiResponse<T>> {
  return apiRequestWithAuth<T>(endpoint, { method: 'DELETE' }, token, DEFAULT_TIMEOUT_MS, correlationId)
}
