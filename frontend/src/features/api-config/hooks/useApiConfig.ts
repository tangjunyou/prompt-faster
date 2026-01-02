/**
 * API 配置 Hooks
 * 封装 TanStack Query 的加载和保存配置逻辑
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getConfig, saveConfig } from '../services/configService';
import { useAuthStore } from '@/stores/useAuthStore';
import { useCredentialStore } from '@/stores/useCredentialStore';
import type { SaveConfigRequest } from '@/types/generated/api/SaveConfigRequest';

/** 配置查询 key */
const CONFIG_QUERY_KEY = ['api-config'] as const;

/**
 * 加载 API 配置 Hook
 * 
 * 自动从后端加载配置并填充 Store
 */
export function useLoadApiConfig() {
  const hydrateFromServer = useCredentialStore((state) => state.hydrateFromServer);
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const authStatus = useAuthStore((state) => state.authStatus);

  return useQuery({
    queryKey: CONFIG_QUERY_KEY,
    queryFn: async () => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录');
      }

      const config = await getConfig(sessionToken);
      hydrateFromServer(config);
      return config;
    },
    staleTime: 5 * 60 * 1000, // 5 分钟
  });
}

/**
 * 保存 API 配置 Hook
 * 
 * 将 Store 中的配置保存到后端
 */
export function useSaveApiConfig() {
  const queryClient = useQueryClient();
  const markClean = useCredentialStore((state) => state.markClean);
  const sessionToken = useAuthStore((state) => state.sessionToken);
  const authStatus = useAuthStore((state) => state.authStatus);

  return useMutation({
    mutationFn: async (config: SaveConfigRequest) => {
      if (authStatus !== 'authenticated' || !sessionToken) {
        throw new Error('未登录');
      }

      return saveConfig(config, sessionToken);
    },
    onSuccess: () => {
      // 保存成功后标记为干净状态
      markClean();
      // 刷新配置缓存
      queryClient.invalidateQueries({ queryKey: CONFIG_QUERY_KEY });
    },
  });
}

/**
 * 构建保存配置请求
 * 
 * 从 Store 中读取当前状态并构建请求体
 * 
 * @throws Error 如果凭证不完整（防御性检查，正常情况下不会触发）
 */
export function buildSaveConfigRequest(): SaveConfigRequest {
  const state = useCredentialStore.getState();
  
  // 防御性检查：后端要求必须同时包含两组凭证
  // 正常情况下 canSave 会阻止不完整请求，此处为兜底
  const isDifyValid = state.dify.status === 'valid' && state.dify.apiKey && state.dify.baseUrl;
  const isGenericLlmValid = state.genericLlm.status === 'valid' && 
    state.genericLlm.apiKey && 
    state.genericLlm.baseUrl && 
    state.genericLlm.provider;

  if (!isDifyValid || !isGenericLlmValid) {
    throw new Error('凭证配置不完整，请确保 Dify 和通用大模型凭证都已测试通过');
  }

  // Code Review Fix: 类型现在是 required，直接构建完整对象
  const request: SaveConfigRequest = {
    dify: {
      base_url: state.dify.baseUrl,
      api_key: state.dify.apiKey,
    },
    generic_llm: {
      provider: state.genericLlm.provider!,
      base_url: state.genericLlm.baseUrl,
      api_key: state.genericLlm.apiKey,
    },
    teacher_settings: {
      temperature: state.teacherSettings.temperature,
      top_p: state.teacherSettings.topP,
      max_tokens: state.teacherSettings.maxTokens,
    },
  };

  return request;
}
