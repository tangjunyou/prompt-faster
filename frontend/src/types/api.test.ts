import { describe, it, expect } from 'vitest'
import { isApiError, isApiSuccess, type ApiResponse } from '@/lib/api'
import type { ApiError, ApiSuccess } from '@/types/generated/api'

describe('ApiResponse 类型守卫', () => {
  it('isApiSuccess 能识别成功响应', () => {
    const success: ApiSuccess<{ ok: boolean }> = { data: { ok: true } }
    const response: ApiResponse<{ ok: boolean }> = success

    expect(isApiSuccess(response)).toBe(true)
    expect(isApiError(response)).toBe(false)
  })

  it('isApiError 能识别错误响应', () => {
    const error: ApiError = {
      error: {
        code: 'TEST_ERROR',
        message: '测试错误',
      },
    }
    const response: ApiResponse<string> = error

    expect(isApiError(response)).toBe(true)
    expect(isApiSuccess(response)).toBe(false)
  })
})
