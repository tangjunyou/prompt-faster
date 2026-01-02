/**
 * 配置 API 服务
 * 负责与后端 /api/v1/auth/config 端点通信
 */

import { UnauthorizedError, apiRequestWithAuth, isApiError } from '@/lib/api';
import type { ConfigResponse } from '@/types/generated/api/ConfigResponse';
import type { SaveConfigRequest } from '@/types/generated/api/SaveConfigRequest';
import type { SaveConfigResponse } from '@/types/generated/api/SaveConfigResponse';

/**
 * 获取已保存的配置
 */
export async function getConfig(token: string): Promise<ConfigResponse> {
  const response = await apiRequestWithAuth<ConfigResponse>(
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
 * 保存配置到后端
 */
export async function saveConfig(
  config: SaveConfigRequest,
  token: string
): Promise<SaveConfigResponse> {
  const response = await apiRequestWithAuth<SaveConfigResponse>(
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
