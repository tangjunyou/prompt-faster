/**
 * 连接测试 TanStack Query Hooks
 * 
 * 使用 useMutation 处理连接测试请求
 */

import { useMutation } from '@tanstack/react-query';
import { useCredentialStore } from '@/stores/useCredentialStore';
import { isApiError } from '@/lib/api';
import {
  testDifyConnection,
  testGenericLlmConnection,
} from '../services/credentialService';
import type { TestConnectionResult } from '@/types/generated/api/TestConnectionResult';

/**
 * Dify 连接测试 Mutation Hook
 * 
 * @returns TanStack Query mutation 对象
 * 
 * @example
 * ```tsx
 * const { mutate, isPending, isSuccess, isError, data, error } = useTestDifyConnection();
 * mutate({ baseUrl: 'https://api.dify.ai', apiKey: 'app-xxx' });
 * ```
 */
export function useTestDifyConnection() {
  const { setDifyStatus } = useCredentialStore();

  return useMutation({
    mutationFn: async ({ baseUrl, apiKey }: { baseUrl: string; apiKey: string }) => {
      const response = await testDifyConnection({
        base_url: baseUrl,
        api_key: apiKey,
      });
      if (isApiError(response)) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onMutate: () => {
      setDifyStatus('testing');
    },
    onSuccess: () => {
      setDifyStatus('valid');
    },
    onError: () => {
      setDifyStatus('invalid');
    },
  });
}

/**
 * 通用大模型连接测试 Mutation Hook
 * 
 * @returns TanStack Query mutation 对象
 * 
 * @example
 * ```tsx
 * const { mutate, isPending, isSuccess, isError, data, error } = useTestGenericLlmConnection();
 * mutate({ baseUrl: 'https://api.siliconflow.cn', apiKey: 'sk-xxx', provider: 'siliconflow' });
 * ```
 */
export function useTestGenericLlmConnection() {
  const { setGenericLlmStatus } = useCredentialStore();

  return useMutation({
    mutationFn: async ({
      baseUrl,
      apiKey,
      provider,
    }: {
      baseUrl: string;
      apiKey: string;
      provider: string;
    }) => {
      const response = await testGenericLlmConnection({
        base_url: baseUrl,
        api_key: apiKey,
        provider,
      });
      if (isApiError(response)) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onMutate: () => {
      setGenericLlmStatus('testing');
    },
    onSuccess: () => {
      setGenericLlmStatus('valid');
    },
    onError: () => {
      setGenericLlmStatus('invalid');
    },
  });
}

export type { TestConnectionResult };
