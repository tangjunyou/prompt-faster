/**
 * configService 测试
 * Story 1.5 Task 10.1: configService 的 API 调用测试（msw）
 */

import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { getConfig, saveConfig } from './configService';
import type { ApiConfigResponse } from '@/types/credentials';
import type { SaveConfigRequest } from '@/types/generated/api/SaveConfigRequest';

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
  // GET /api/v1/auth/config - 成功
  http.get('http://localhost:3000/api/v1/auth/config', () => {
    return HttpResponse.json({
      data: mockConfigResponse,
    });
  }),

  // POST /api/v1/auth/config - 成功
  http.post('http://localhost:3000/api/v1/auth/config', async ({ request }) => {
    const body = await request.json() as SaveConfigRequest;
    
    // 验证请求体包含必需字段
    if (!body.dify || !body.generic_llm) {
      return HttpResponse.json(
        {
          error: {
            code: 'VALIDATION_ERROR',
            message: '请求体必须包含 Dify 和通用大模型凭证配置',
          },
        },
        { status: 400 }
      );
    }

    return HttpResponse.json({
      data: { message: '配置保存成功' },
    });
  }),
];

const server = setupServer(...handlers);

describe('configService', () => {
  beforeAll(() => server.listen());
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());

  describe('getConfig', () => {
    it('应该成功获取配置', async () => {
      const config = await getConfig('test-token');

      expect(config).toEqual(mockConfigResponse);
      expect(config.has_dify_key).toBe(true);
      expect(config.has_generic_llm_key).toBe(true);
      expect(config.teacher_settings.temperature).toBe(0.7);
    });

    it('应该在 API 返回错误时抛出异常', async () => {
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

      await expect(getConfig('test-token')).rejects.toThrow('数据库连接失败');
    });
  });

  describe('saveConfig', () => {
    it('应该成功保存完整配置', async () => {
      const request: SaveConfigRequest = {
        dify: {
          base_url: 'https://api.dify.ai',
          api_key: 'sk-test-dify-key',
        },
        generic_llm: {
          provider: 'siliconflow',
          base_url: 'https://api.siliconflow.cn',
          api_key: 'sk-test-llm-key',
        },
        teacher_settings: {
          temperature: 0.8,
          top_p: 0.95,
          max_tokens: 4096,
        },
      };

      const result = await saveConfig(request, 'test-token');

      expect(result.message).toBe('配置保存成功');
    });

    it('应该在缺少 Dify 凭证时抛出验证错误', async () => {
      const request: SaveConfigRequest = {
        dify: null,
        generic_llm: {
          provider: 'siliconflow',
          base_url: 'https://api.siliconflow.cn',
          api_key: 'sk-test-llm-key',
        },
        teacher_settings: {
          temperature: 0.7,
          top_p: 0.9,
          max_tokens: 2048,
        },
      };

      await expect(saveConfig(request, 'test-token')).rejects.toThrow();
    });

    it('应该在缺少通用大模型凭证时抛出验证错误', async () => {
      const request: SaveConfigRequest = {
        dify: {
          base_url: 'https://api.dify.ai',
          api_key: 'sk-test-dify-key',
        },
        generic_llm: null,
        teacher_settings: {
          temperature: 0.7,
          top_p: 0.9,
          max_tokens: 2048,
        },
      };

      await expect(saveConfig(request, 'test-token')).rejects.toThrow();
    });

    it('应该在 teacher_settings 参数超出范围时抛出验证错误', async () => {
      server.use(
        http.post('http://localhost:3000/api/v1/auth/config', () => {
          return HttpResponse.json(
            {
              error: {
                code: 'VALIDATION_ERROR',
                message: 'temperature 必须在 0.0 ~ 2.0 之间，当前值: 3.0',
              },
            },
            { status: 400 }
          );
        })
      );

      const request: SaveConfigRequest = {
        dify: {
          base_url: 'https://api.dify.ai',
          api_key: 'sk-test-dify-key',
        },
        generic_llm: {
          provider: 'siliconflow',
          base_url: 'https://api.siliconflow.cn',
          api_key: 'sk-test-llm-key',
        },
        teacher_settings: {
          temperature: 3.0, // 超出范围
          top_p: 0.9,
          max_tokens: 2048,
        },
      };

      await expect(saveConfig(request, 'test-token')).rejects.toThrow(
        'temperature 必须在 0.0 ~ 2.0 之间'
      );
    });
  });
});
