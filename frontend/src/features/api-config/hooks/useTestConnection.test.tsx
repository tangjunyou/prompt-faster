/**
 * useTestConnection Hook 测试
 * 测试连接测试 mutation 的状态管理和错误处理
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTestDifyConnection, useTestGenericLlmConnection } from './useTestConnection';
import { useCredentialStore } from '@/stores/useCredentialStore';
import * as credentialService from '../services/credentialService';

// Mock credentialService
vi.mock('../services/credentialService', () => ({
  testDifyConnection: vi.fn(),
  testGenericLlmConnection: vi.fn(),
}));

// 创建测试用的 QueryClient
const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

// 渲染 hook 的 wrapper
const createWrapper = () => {
  const queryClient = createTestQueryClient();
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('useTestDifyConnection', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useCredentialStore.getState().clearDifyCredential();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('成功测试连接时应更新状态为 valid', async () => {
    const mockResponse = {
      data: { message: '连接成功', models: null },
    };
    vi.mocked(credentialService.testDifyConnection).mockResolvedValue(mockResponse);

    const { result } = renderHook(() => useTestDifyConnection(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        baseUrl: 'https://api.dify.ai',
        apiKey: 'app-test-key',
      });
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const store = useCredentialStore.getState();
    expect(store.dify.status).toBe('valid');
  });

  it('测试开始时应设置状态为 testing', async () => {
    // 创建一个 pending promise 来控制时序
    let resolvePromise: (value: unknown) => void;
    const pendingPromise = new Promise((resolve) => {
      resolvePromise = resolve;
    });
    vi.mocked(credentialService.testDifyConnection).mockReturnValue(pendingPromise as Promise<never>);

    const { result } = renderHook(() => useTestDifyConnection(), {
      wrapper: createWrapper(),
    });

    // 触发 mutation（不等待完成）
    await act(async () => {
      result.current.mutate({
        baseUrl: 'https://api.dify.ai',
        apiKey: 'app-test-key',
      });
      // 给 React 时间处理状态更新
      await Promise.resolve();
    });

    // 检查 pending 状态
    await waitFor(() => {
      expect(result.current.isPending).toBe(true);
    });

    const store = useCredentialStore.getState();
    expect(store.dify.status).toBe('testing');

    // 清理：解决 promise 并等待完成
    await act(async () => {
      resolvePromise!({ data: { message: '连接成功' } });
    });
  });

  it('测试失败时应设置状态为 invalid', async () => {
    const mockErrorResponse = {
      error: { code: 'AUTH_INVALID_CREDENTIALS', message: '无效的 API Key' },
    };
    vi.mocked(credentialService.testDifyConnection).mockResolvedValue(mockErrorResponse);

    const { result } = renderHook(() => useTestDifyConnection(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          baseUrl: 'https://api.dify.ai',
          apiKey: 'invalid-key',
        });
      } catch {
        // 预期会抛出错误
      }
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const store = useCredentialStore.getState();
    expect(store.dify.status).toBe('invalid');
  });

  it('网络错误时应设置状态为 invalid', async () => {
    vi.mocked(credentialService.testDifyConnection).mockRejectedValue(new Error('Network Error'));

    const { result } = renderHook(() => useTestDifyConnection(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          baseUrl: 'https://api.dify.ai',
          apiKey: 'test-key',
        });
      } catch {
        // 预期会抛出错误
      }
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const store = useCredentialStore.getState();
    expect(store.dify.status).toBe('invalid');
  });
});

describe('useTestGenericLlmConnection', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useCredentialStore.getState().clearGenericLlmCredential();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('成功测试连接时应更新状态为 valid 并返回模型列表', async () => {
    const mockResponse = {
      data: {
        message: '连接成功，可用模型: 3',
        models: ['gpt-4', 'gpt-3.5-turbo', 'claude-3'],
      },
    };
    vi.mocked(credentialService.testGenericLlmConnection).mockResolvedValue(mockResponse);

    const { result } = renderHook(() => useTestGenericLlmConnection(), {
      wrapper: createWrapper(),
    });

    let data: { message: string; models: string[] | null } | undefined;
    await act(async () => {
      data = await result.current.mutateAsync({
        baseUrl: 'https://api.siliconflow.cn',
        apiKey: 'sk-test-key',
        provider: 'siliconflow',
      });
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(data).toBeDefined();
    expect(data?.models).toHaveLength(3);
    expect(data?.models).toContain('gpt-4');

    const store = useCredentialStore.getState();
    expect(store.genericLlm.status).toBe('valid');
  });

  it('测试失败时应设置状态为 invalid', async () => {
    const mockErrorResponse = {
      error: { code: 'AUTH_INVALID_CREDENTIALS', message: '无效的 API Key' },
    };
    vi.mocked(credentialService.testGenericLlmConnection).mockResolvedValue(mockErrorResponse);

    const { result } = renderHook(() => useTestGenericLlmConnection(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          baseUrl: 'https://api.siliconflow.cn',
          apiKey: 'invalid-key',
          provider: 'siliconflow',
        });
      } catch {
        // 预期会抛出错误
      }
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const store = useCredentialStore.getState();
    expect(store.genericLlm.status).toBe('invalid');
  });

  it('验证错误时应设置状态为 invalid', async () => {
    const mockErrorResponse = {
      error: { code: 'AUTH_VALIDATION_ERROR', message: '不支持的 Provider' },
    };
    vi.mocked(credentialService.testGenericLlmConnection).mockResolvedValue(mockErrorResponse);

    const { result } = renderHook(() => useTestGenericLlmConnection(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          baseUrl: 'https://api.example.com',
          apiKey: 'test-key',
          provider: 'invalid-provider',
        });
      } catch {
        // 预期会抛出错误
      }
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    const store = useCredentialStore.getState();
    expect(store.genericLlm.status).toBe('invalid');
  });
});
