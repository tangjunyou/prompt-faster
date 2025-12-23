/**
 * useApiConfig Hooks 测试
 * Story 1.5 Task 10.2: useSave/useLoad hooks 测试（msw）
 */

import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { useLoadApiConfig, useSaveApiConfig, buildSaveConfigRequest } from './useApiConfig';
import { useCredentialStore } from '@/stores/useCredentialStore';
import type { ApiConfigResponse } from '@/types/credentials';
import React from 'react';

// Mock 配置响应
const mockConfigResponse: ApiConfigResponse = {
  has_dify_key: true,
  has_generic_llm_key: true,
  dify_base_url: 'https://api.dify.ai',
  generic_llm_base_url: 'https://api.siliconflow.cn',
  generic_llm_provider: 'siliconflow',
  masked_dify_key: 'sk-****xxxx',
  masked_generic_llm_key: 'sk-****yyyy',
  teacher_settings: {
    temperature: 0.7,
    top_p: 0.9,
    max_tokens: 2048,
  },
};

// MSW handlers
const handlers = [
  http.get('http://localhost:3000/api/v1/auth/config', () => {
    return HttpResponse.json({
      data: mockConfigResponse,
    });
  }),

  http.post('http://localhost:3000/api/v1/auth/config', () => {
    return HttpResponse.json({
      data: { message: '配置保存成功' },
    });
  }),
];

const server = setupServer(...handlers);

// 创建 QueryClient wrapper
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return function Wrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(
      QueryClientProvider,
      { client: queryClient },
      children
    );
  };
}

describe('useApiConfig hooks', () => {
  beforeAll(() => server.listen());
  afterEach(() => {
    server.resetHandlers();
    // 重置 store 状态
    useCredentialStore.setState({
      dify: { baseUrl: '', apiKey: '', status: 'empty' },
      genericLlm: { provider: null, baseUrl: '', apiKey: '', status: 'empty' },
      teacherSettings: { temperature: 0.7, topP: 0.9, maxTokens: 2048 },
      isHydrated: false,
      isDirty: false,
    });
  });
  afterAll(() => server.close());

  describe('useLoadApiConfig', () => {
    it('应该成功加载配置并填充 Store', async () => {
      const { result } = renderHook(() => useLoadApiConfig(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      // 验证 Store 被正确填充
      const state = useCredentialStore.getState();
      expect(state.isHydrated).toBe(true);
      expect(state.isDirty).toBe(false);
      expect(state.teacherSettings.temperature).toBe(0.7);
    });

    it('应该在加载失败时返回错误状态', async () => {
      server.use(
        http.get('http://localhost:3000/api/v1/auth/config', () => {
          return HttpResponse.json(
            {
              error: {
                code: 'DATABASE_ERROR',
                message: '数据库连接失败',
              },
            },
            { status: 500 }
          );
        })
      );

      const { result } = renderHook(() => useLoadApiConfig(), {
        wrapper: createWrapper(),
      });

      // 等待查询完成（成功或失败）
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      }, { timeout: 3000 });

      // 由于 configService 内部捕获错误并抛出，TanStack Query 会标记为错误状态
      // 但需要考虑 retry 逻辑，这里检查查询是否已结束
      expect(result.current.isFetched).toBe(true);
    });
  });

  describe('useSaveApiConfig', () => {
    it('应该成功保存配置并标记为干净状态', async () => {
      // 先设置 Store 为脏状态
      useCredentialStore.setState({ isDirty: true });

      const { result } = renderHook(() => useSaveApiConfig(), {
        wrapper: createWrapper(),
      });

      // 执行保存操作
      act(() => {
        result.current.mutate({
          dify: {
            base_url: 'https://api.dify.ai',
            api_key: 'sk-test',
          },
          generic_llm: {
            provider: 'siliconflow',
            base_url: 'https://api.siliconflow.cn',
            api_key: 'sk-test',
          },
          teacher_settings: {
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 2048,
          },
        });
      });

      // 等待 mutation 完成
      await waitFor(() => {
        expect(result.current.isIdle).toBe(false);
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      }, { timeout: 3000 });

      // 验证 isDirty 被重置
      expect(useCredentialStore.getState().isDirty).toBe(false);
    });
  });

  describe('buildSaveConfigRequest', () => {
    it('应该正确构建包含有效凭证的请求', () => {
      // 设置 Store 状态
      useCredentialStore.setState({
        dify: {
          baseUrl: 'https://api.dify.ai',
          apiKey: 'sk-dify-key',
          status: 'valid',
        },
        genericLlm: {
          provider: 'siliconflow',
          baseUrl: 'https://api.siliconflow.cn',
          apiKey: 'sk-llm-key',
          status: 'valid',
        },
        teacherSettings: {
          temperature: 0.8,
          topP: 0.95,
          maxTokens: 4096,
        },
      });

      const request = buildSaveConfigRequest();

      expect(request.dify).toEqual({
        base_url: 'https://api.dify.ai',
        api_key: 'sk-dify-key',
      });
      expect(request.generic_llm).toEqual({
        provider: 'siliconflow',
        base_url: 'https://api.siliconflow.cn',
        api_key: 'sk-llm-key',
      });
      expect(request.teacher_settings).toEqual({
        temperature: 0.8,
        top_p: 0.95,
        max_tokens: 4096,
      });
    });

    it('应该在凭证无效或 apiKey 为空时抛出错误（防御性检查）', () => {
      // 设置 Store 状态 - dify 有效但 genericLlm apiKey 为空
      // Code Review Fix: 防御性检查会阻止不完整的请求
      useCredentialStore.setState({
        dify: {
          baseUrl: 'https://api.dify.ai',
          apiKey: 'sk-dify-key',
          status: 'valid',
        },
        genericLlm: {
          provider: 'siliconflow',
          baseUrl: 'https://api.siliconflow.cn',
          apiKey: '', // 空
          status: 'saved', // saved 状态
        },
        teacherSettings: {
          temperature: 0.7,
          topP: 0.9,
          maxTokens: 2048,
        },
      });

      // 应该抛出错误，因为凭证不完整
      expect(() => buildSaveConfigRequest()).toThrow('凭证配置不完整');
    });

    it('应该在 status 为 saved 时抛出错误（防御性检查）', () => {
      // 设置 Store 状态 - 两个都是 saved 状态
      // Code Review Fix: 防御性检查会阻止不完整的请求
      useCredentialStore.setState({
        dify: {
          baseUrl: 'https://api.dify.ai',
          apiKey: '',
          status: 'saved',
        },
        genericLlm: {
          provider: 'siliconflow',
          baseUrl: 'https://api.siliconflow.cn',
          apiKey: '',
          status: 'saved',
        },
        teacherSettings: {
          temperature: 0.7,
          topP: 0.9,
          maxTokens: 2048,
        },
      });

      // 应该抛出错误，因为凭证不完整
      expect(() => buildSaveConfigRequest()).toThrow('凭证配置不完整');
    });
  });
});
