/**
 * 配置 API 服务
 * 负责与后端 /api/v1/auth/config 端点通信
 */

import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api';
import type { ApiConfigResponse, SaveConfigRequest } from '@/types/credentials';

/**
 * 获取已保存的配置
 */
export async function getConfig(token: string): Promise<ApiConfigResponse> {
  const response = await apiRequestWithAuth<ApiConfigResponse>(
    '/auth/config',
    {
      method: 'GET',
    },
    token
  );

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message);
    }
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
export async function saveConfig(
  config: SaveConfigRequest,
  token: string
): Promise<SaveConfigResponseData> {
  const response = await apiRequestWithAuth<SaveConfigResponseData>(
    '/auth/config',
    {
      method: 'POST',
      body: JSON.stringify(config),
    },
    token
  );

  if (isApiError(response)) {
    if (response.error.code === 'UNAUTHORIZED') {
      throw new UnauthorizedError(response.error.message);
    }
    throw new Error(response.error.message);
  }

  return response.data;
}
