/**
 * credentialService 测试
 * 测试 API 服务函数的请求格式和响应处理
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { testDifyConnection, testGenericLlmConnection } from './credentialService';
import * as api from '@/lib/api';

// Mock api 模块
vi.mock('@/lib/api', () => ({
  post: vi.fn(),
  isApiError: vi.fn((response) => 'error' in response),
  isApiSuccess: vi.fn((response) => 'data' in response),
}));

describe('credentialService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('testDifyConnection', () => {
    it('应发送正确的请求格式', async () => {
      const mockResponse = { data: { message: '连接成功', models: null } };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      await testDifyConnection({
        base_url: 'https://api.dify.ai',
        api_key: 'app-test-key',
      });

      expect(api.post).toHaveBeenCalledWith('/auth/test-connection/dify', {
        base_url: 'https://api.dify.ai',
        api_key: 'app-test-key',
      });
    });

    it('成功时应返回包含 data 的响应', async () => {
      const mockResponse = { data: { message: '连接成功', models: null } };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testDifyConnection({
        base_url: 'https://api.dify.ai',
        api_key: 'app-test-key',
      });

      expect(result).toEqual(mockResponse);
      expect(api.isApiSuccess(result)).toBe(true);
    });

    it('失败时应返回包含 error 的响应', async () => {
      const mockResponse = {
        error: { code: 'AUTH_INVALID_CREDENTIALS', message: '无效的 API Key' },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testDifyConnection({
        base_url: 'https://api.dify.ai',
        api_key: 'invalid-key',
      });

      expect(result).toEqual(mockResponse);
      expect(api.isApiError(result)).toBe(true);
    });

    it('验证错误时应返回 VALIDATION_ERROR', async () => {
      const mockResponse = {
        error: { code: 'AUTH_VALIDATION_ERROR', message: 'URL 不能为空' },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testDifyConnection({ base_url: '', api_key: 'test-key' });

      expect(result).toEqual(mockResponse);
    });
  });

  describe('testGenericLlmConnection', () => {
    it('应发送正确的请求格式（包含 provider）', async () => {
      const mockResponse = { data: { message: '连接成功', models: ['gpt-4'] } };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      await testGenericLlmConnection({
        base_url: 'https://api.siliconflow.cn',
        api_key: 'sk-test-key',
        provider: 'siliconflow',
      });

      expect(api.post).toHaveBeenCalledWith('/auth/test-connection/generic-llm', {
        base_url: 'https://api.siliconflow.cn',
        api_key: 'sk-test-key',
        provider: 'siliconflow',
      });
    });

    it('成功时应返回包含模型列表的响应', async () => {
      const mockResponse = {
        data: {
          message: '连接成功，可用模型: 3',
          models: ['gpt-4', 'gpt-3.5-turbo', 'claude-3'],
        },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testGenericLlmConnection({
        base_url: 'https://api.siliconflow.cn',
        api_key: 'sk-test-key',
        provider: 'siliconflow',
      });

      expect(result).toEqual(mockResponse);
      if ('data' in result) {
        expect(result.data.models).toHaveLength(3);
        expect(result.data.models).toContain('gpt-4');
      }
    });

    it('无效 provider 应返回验证错误', async () => {
      const mockResponse = {
        error: {
          code: 'AUTH_VALIDATION_ERROR',
          message: '不支持的 Provider: invalid',
        },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testGenericLlmConnection({
        base_url: 'https://api.example.com',
        api_key: 'test-key',
        provider: 'invalid',
      });

      expect(result).toEqual(mockResponse);
      expect(api.isApiError(result)).toBe(true);
    });

    it('SSRF 攻击尝试应返回验证错误', async () => {
      const mockResponse = {
        error: {
          code: 'AUTH_VALIDATION_ERROR',
          message: '禁止访问本地地址',
        },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testGenericLlmConnection({
        base_url: 'http://localhost:8080',
        api_key: 'test-key',
        provider: 'siliconflow',
      });

      expect(result).toEqual(mockResponse);
      expect(api.isApiError(result)).toBe(true);
    });

    it('401 错误应返回 AUTH_INVALID_CREDENTIALS', async () => {
      const mockResponse = {
        error: { code: 'AUTH_INVALID_CREDENTIALS', message: '无效的 API Key' },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testGenericLlmConnection({
        base_url: 'https://api.siliconflow.cn',
        api_key: 'invalid-key',
        provider: 'siliconflow',
      });

      expect(result).toEqual(mockResponse);
      if ('error' in result) {
        expect(result.error.code).toBe('AUTH_INVALID_CREDENTIALS');
      }
    });

    it('超时错误应返回 AUTH_CONNECTION_TIMEOUT', async () => {
      const mockResponse = {
        error: { code: 'AUTH_CONNECTION_TIMEOUT', message: '连接超时' },
      };
      vi.mocked(api.post).mockResolvedValue(mockResponse);

      const result = await testGenericLlmConnection({
        base_url: 'https://slow-api.example.com',
        api_key: 'test-key',
        provider: 'siliconflow',
      });

      expect(result).toEqual(mockResponse);
      if ('error' in result) {
        expect(result.error.code).toBe('AUTH_CONNECTION_TIMEOUT');
      }
    });
  });
});
