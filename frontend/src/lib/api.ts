/**
 * API 客户端配置
 * 统一的 API 调用入口
 */

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api/v1'

/** 默认请求超时时间（毫秒）- Code Review Fix: 添加超时配置 */
const DEFAULT_TIMEOUT_MS = 30000

/**
 * API 成功响应
 */
export interface ApiSuccess<T> {
  data: T
  meta?: {
    page?: number
    pageSize?: number
    total?: number
  }
}

/**
 * API 错误响应
 */
export interface ApiError {
  error: {
    code: string
    message: string
    details?: Record<string, unknown>
  }
}

/**
 * API 响应类型 - data 与 error 互斥
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
  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs)

  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal,
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
    clearTimeout(timeoutId)
    
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
    clearTimeout(timeoutId)
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
