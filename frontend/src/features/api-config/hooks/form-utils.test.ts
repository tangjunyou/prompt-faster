import { describe, it, expect } from 'vitest';
import { validateCredential } from './form-utils';

/**
 * form-utils 公共模块测试
 * 独立测试文件，与 url-utils.test.ts 模式一致
 */
describe('form-utils', () => {
  describe('validateCredential', () => {
    it('应通过有效的凭证验证', () => {
      const result = validateCredential('https://api.example.com', 'sk-test-key');
      expect(result.isValid).toBe(true);
      expect(result.errors).toEqual({});
    });

    it('应拒绝空的 baseUrl', () => {
      const result = validateCredential('', 'sk-test-key');
      expect(result.isValid).toBe(false);
      expect(result.errors.baseUrl).toBe('API 地址不能为空');
      expect(result.errors.apiKey).toBeUndefined();
    });

    it('应拒绝只有空格的 baseUrl', () => {
      const result = validateCredential('   ', 'sk-test-key');
      expect(result.isValid).toBe(false);
      expect(result.errors.baseUrl).toBe('API 地址不能为空');
    });

    it('应拒绝无效的 baseUrl 格式', () => {
      const result = validateCredential('not-a-url', 'sk-test-key');
      expect(result.isValid).toBe(false);
      expect(result.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
    });

    it('应拒绝非 http/https 协议', () => {
      const result = validateCredential('ftp://example.com', 'sk-test-key');
      expect(result.isValid).toBe(false);
      expect(result.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
    });

    it('应拒绝空的 apiKey', () => {
      const result = validateCredential('https://api.example.com', '');
      expect(result.isValid).toBe(false);
      expect(result.errors.apiKey).toBe('API Key 不能为空');
      expect(result.errors.baseUrl).toBeUndefined();
    });

    it('应拒绝只有空格的 apiKey', () => {
      const result = validateCredential('https://api.example.com', '   ');
      expect(result.isValid).toBe(false);
      expect(result.errors.apiKey).toBe('API Key 不能为空');
    });

    it('应同时返回多个错误', () => {
      const result = validateCredential('', '');
      expect(result.isValid).toBe(false);
      expect(result.errors.baseUrl).toBe('API 地址不能为空');
      expect(result.errors.apiKey).toBe('API Key 不能为空');
    });

    it('应同时返回格式错误和空值错误', () => {
      const result = validateCredential('invalid', '');
      expect(result.isValid).toBe(false);
      expect(result.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
      expect(result.errors.apiKey).toBe('API Key 不能为空');
    });
  });
});
