/**
 * 凭证服务 - API 连接测试
 * 
 * 使用 @/lib/api 的 post() 函数发送请求
 */

import { post, type ApiResponse } from '@/lib/api';

/**
 * 连接测试结果
 */
export interface TestConnectionResult {
  message: string;
  models?: string[];
}

/**
 * 测试 Dify API 连接
 * 
 * @param baseUrl - Dify API 基础 URL
 * @param apiKey - Dify API Key
 * @returns API 响应（成功时包含 TestConnectionResult）
 */
export function testDifyConnection(
  baseUrl: string,
  apiKey: string
): Promise<ApiResponse<TestConnectionResult>> {
  return post<TestConnectionResult>('/auth/test-connection/dify', {
    base_url: baseUrl,
    api_key: apiKey,
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
  baseUrl: string,
  apiKey: string,
  provider: string
): Promise<ApiResponse<TestConnectionResult>> {
  return post<TestConnectionResult>('/auth/test-connection/generic-llm', {
    base_url: baseUrl,
    api_key: apiKey,
    provider,
  });
}
