import { describe, it, expect } from 'vitest';
import { isValidUrl, normalizeBaseUrl } from './url-utils';

/**
 * url-utils 公共模块测试
 * 独立测试文件，避免测试分散在多个 Hook 测试中
 */
describe('url-utils', () => {
  describe('isValidUrl', () => {
    it('应接受有效的 http URL', () => {
      expect(isValidUrl('http://example.com')).toBe(true);
      expect(isValidUrl('http://localhost:8080')).toBe(true);
      expect(isValidUrl('http://192.168.1.1:3000')).toBe(true);
    });

    it('应接受有效的 https URL', () => {
      expect(isValidUrl('https://api.dify.ai')).toBe(true);
      expect(isValidUrl('https://api.siliconflow.cn')).toBe(true);
      expect(isValidUrl('https://dashscope.aliyuncs.com/api')).toBe(true);
      expect(isValidUrl('https://example.com/path/to/api')).toBe(true);
    });

    it('应拒绝空字符串', () => {
      expect(isValidUrl('')).toBe(false);
    });

    it('应拒绝非 URL 字符串', () => {
      expect(isValidUrl('not-a-url')).toBe(false);
      expect(isValidUrl('example.com')).toBe(false);
      expect(isValidUrl('just some text')).toBe(false);
    });

    it('应拒绝非 http/https 协议', () => {
      expect(isValidUrl('ftp://example.com')).toBe(false);
      expect(isValidUrl('file:///path/to/file')).toBe(false);
      expect(isValidUrl('ws://example.com')).toBe(false);
    });
  });

  describe('normalizeBaseUrl', () => {
    it('应去除末尾斜杠', () => {
      expect(normalizeBaseUrl('https://api.dify.ai/')).toBe('https://api.dify.ai');
      expect(normalizeBaseUrl('https://api.siliconflow.cn///')).toBe('https://api.siliconflow.cn');
    });

    it('应只保留 origin（去除路径）', () => {
      expect(normalizeBaseUrl('https://api.dify.ai/v1/chat')).toBe('https://api.dify.ai');
      expect(normalizeBaseUrl('http://localhost:8080/api')).toBe('http://localhost:8080');
      expect(normalizeBaseUrl('https://example.com/path/to/api?query=1')).toBe('https://example.com');
    });

    it('应去除前后空格', () => {
      expect(normalizeBaseUrl('  https://api.dify.ai  ')).toBe('https://api.dify.ai');
      expect(normalizeBaseUrl('\thttps://example.com\n')).toBe('https://example.com');
    });

    it('应保留端口号', () => {
      expect(normalizeBaseUrl('http://localhost:8080/api')).toBe('http://localhost:8080');
      expect(normalizeBaseUrl('https://example.com:3000/')).toBe('https://example.com:3000');
    });

    it('对无效 URL 应返回 trim 后的原值而非抛异常（防御性编程）', () => {
      expect(normalizeBaseUrl('not-a-url')).toBe('not-a-url');
      expect(normalizeBaseUrl('   invalid   ')).toBe('invalid');
      expect(normalizeBaseUrl('')).toBe('');
    });
  });
});
