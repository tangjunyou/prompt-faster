/**
 * 凭证服务 - API 连接测试
 * 
 * 使用 @/lib/api 的 post() 函数发送请求
 */

import { post, type ApiResponse } from '@/lib/api';
import type { TestConnectionResult } from '@/types/generated/api/TestConnectionResult';
import type { TestDifyConnectionRequest } from '@/types/generated/api/TestDifyConnectionRequest';
import type { TestGenericLlmConnectionRequest } from '@/types/generated/api/TestGenericLlmConnectionRequest';

/**
 * 测试 Dify API 连接
 * 
 * @param baseUrl - Dify API 基础 URL
 * @param apiKey - Dify API Key
 * @returns API 响应（成功时包含 TestConnectionResult）
 */
export function testDifyConnection(
  payload: TestDifyConnectionRequest
): Promise<ApiResponse<TestConnectionResult>> {
  return post<TestConnectionResult>('/auth/test-connection/dify', {
    base_url: payload.base_url,
    api_key: payload.api_key,
  });
}

/**
 * 测试通用大模型 API 连接
 * 
 * @param baseUrl - API 基础 URL
 * @param apiKey - API Key
 * @param provider - Provider 标识 ("siliconflow" | "modelscope")
 * @returns API 响应（成功时包含 TestConnectionResult 和模型列表）
 */
export function testGenericLlmConnection(
  payload: TestGenericLlmConnectionRequest
): Promise<ApiResponse<TestConnectionResult>> {
  return post<TestConnectionResult>('/auth/test-connection/generic-llm', {
    base_url: payload.base_url,
    api_key: payload.api_key,
    provider: payload.provider,
  });
}
