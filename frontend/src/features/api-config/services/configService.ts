/**
 * 配置 API 服务
 * 负责与后端 /api/v1/auth/config 端点通信
 */

import { apiRequest, isApiError } from '@/lib/api';
import type { ApiConfigResponse, SaveConfigRequest } from '@/types/credentials';

/**
 * 获取已保存的配置
 */
export async function getConfig(): Promise<ApiConfigResponse> {
  const response = await apiRequest<ApiConfigResponse>('/auth/config', {
    method: 'GET',
  });

  if (isApiError(response)) {
    throw new Error(response.error.message);
  }

  return response.data;
}

/**
 * 保存配置响应
 */
interface SaveConfigResponseData {
  message: string;
}

/**
 * 保存配置到后端
 */
export async function saveConfig(config: SaveConfigRequest): Promise<SaveConfigResponseData> {
  const response = await apiRequest<SaveConfigResponseData>('/auth/config', {
    method: 'POST',
    body: JSON.stringify(config),
  });

  if (isApiError(response)) {
    throw new Error(response.error.message);
  }

  return response.data;
}
