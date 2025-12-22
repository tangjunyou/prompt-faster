import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { isValidUrl, normalizeBaseUrl, validateDifyCredential, useDifyCredentialForm } from './useDifyCredentialForm';
import { useCredentialStore } from '@/stores/useCredentialStore';

describe('isValidUrl', () => {
  it('应接受有效的 http URL', () => {
    expect(isValidUrl('http://example.com')).toBe(true);
    expect(isValidUrl('http://localhost:8080')).toBe(true);
  });

  it('应接受有效的 https URL', () => {
    expect(isValidUrl('https://api.dify.ai')).toBe(true);
    expect(isValidUrl('https://example.com/api')).toBe(true);
  });

  it('应拒绝无效的 URL', () => {
    expect(isValidUrl('')).toBe(false);
    expect(isValidUrl('not-a-url')).toBe(false);
    expect(isValidUrl('ftp://example.com')).toBe(false);
    expect(isValidUrl('example.com')).toBe(false);
  });
});

describe('normalizeBaseUrl', () => {
  it('应去除末尾斜杠', () => {
    expect(normalizeBaseUrl('https://api.dify.ai/')).toBe('https://api.dify.ai');
    expect(normalizeBaseUrl('https://api.dify.ai///')).toBe('https://api.dify.ai');
  });

  it('应只保留 origin（去除路径）', () => {
    expect(normalizeBaseUrl('https://api.dify.ai/v1/chat')).toBe('https://api.dify.ai');
    expect(normalizeBaseUrl('http://localhost:8080/api')).toBe('http://localhost:8080');
  });

  it('应去除前后空格', () => {
    expect(normalizeBaseUrl('  https://api.dify.ai  ')).toBe('https://api.dify.ai');
  });
});

describe('validateDifyCredential', () => {
  it('应通过有效的凭证验证', () => {
    const result = validateDifyCredential('https://api.dify.ai', 'app-xxxx');
    expect(result.isValid).toBe(true);
    expect(result.errors).toEqual({});
  });

  it('应拒绝空的 baseUrl', () => {
    const result = validateDifyCredential('', 'app-xxxx');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBe('API 地址不能为空');
  });

  it('应拒绝无效的 baseUrl 格式', () => {
    const result = validateDifyCredential('not-a-url', 'app-xxxx');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
  });

  it('应拒绝空的 apiKey', () => {
    const result = validateDifyCredential('https://api.dify.ai', '');
    expect(result.isValid).toBe(false);
    expect(result.errors.apiKey).toBe('API Key 不能为空');
  });

  it('应拒绝只有空格的输入', () => {
    const result = validateDifyCredential('   ', '   ');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBe('API 地址不能为空');
    expect(result.errors.apiKey).toBe('API Key 不能为空');
  });

  it('应同时返回多个错误', () => {
    const result = validateDifyCredential('', '');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBeDefined();
    expect(result.errors.apiKey).toBeDefined();
  });
});

describe('normalizeBaseUrl 防御性编程', () => {
  it('对无效 URL 应返回 trim 后的原值而非抛异常', () => {
    // 这个测试验证 M5 修复：normalizeBaseUrl 不会抛出异常
    expect(normalizeBaseUrl('not-a-url')).toBe('not-a-url');
    expect(normalizeBaseUrl('   invalid   ')).toBe('invalid');
  });
});

describe('useDifyCredentialForm Hook', () => {
  // 每个测试前重置 Store
  beforeEach(() => {
    const store = useCredentialStore.getState();
    store.clearDifyCredential();
  });

  it('AC#1: 填写合法凭证后 status 应变为 filled', () => {
    const { result } = renderHook(() => useDifyCredentialForm());

    // 初始状态
    expect(result.current.status).toBe('empty');

    // 填写凭证
    act(() => {
      result.current.handleBaseUrlChange('https://api.dify.ai');
      result.current.handleApiKeyChange('app-test-key');
    });

    // 提交
    act(() => {
      const submitResult = result.current.handleSubmit();
      expect(submitResult.success).toBe(true);
      expect(submitResult.action).toBe('saved');
    });

    // 验证 Store 状态
    const store = useCredentialStore.getState();
    expect(store.dify.status).toBe('filled');
    expect(store.dify.baseUrl).toBe('https://api.dify.ai');
  });

  it('AC#2: 无效 URL 应返回验证错误', () => {
    const { result } = renderHook(() => useDifyCredentialForm());

    // 填写无效凭证
    act(() => {
      result.current.handleBaseUrlChange('not-a-valid-url');
      result.current.handleApiKeyChange('app-test-key');
    });

    // 提交
    act(() => {
      const submitResult = result.current.handleSubmit();
      expect(submitResult.success).toBe(false);
      expect(submitResult.action).toBe('validation_failed');
    });

    // 验证错误消息
    expect(result.current.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
  });

  it('AC#3: 清空两字段后应清空 Store 并回到 empty 状态', () => {
    const { result } = renderHook(() => useDifyCredentialForm());

    // 先填写凭证
    act(() => {
      result.current.handleBaseUrlChange('https://api.dify.ai');
      result.current.handleApiKeyChange('app-test-key');
    });

    // 提交保存
    act(() => {
      result.current.handleSubmit();
    });

    // 验证已保存（由于单一事实来源，填写时 status 已实时更新）
    expect(useCredentialStore.getState().dify.status).toBe('filled');

    // 清空两字段
    act(() => {
      result.current.handleBaseUrlChange('');
      result.current.handleApiKeyChange('');
    });

    // 提交清空操作
    act(() => {
      const submitResult = result.current.handleSubmit();
      expect(submitResult.success).toBe(true);
      expect(submitResult.action).toBe('cleared');
    });

    // 验证 Store 已清空
    const store = useCredentialStore.getState();
    expect(store.dify.status).toBe('empty');
    expect(store.dify.baseUrl).toBe('');
    expect(store.dify.apiKey).toBe('');
  });

  it('单一事实来源: Store 更新后 Hook 返回值应同步', () => {
    const { result } = renderHook(() => useDifyCredentialForm());

    // 通过 Hook 更新
    act(() => {
      result.current.handleBaseUrlChange('https://test.com');
    });

    // Hook 返回值应与 Store 同步
    expect(result.current.baseUrl).toBe('https://test.com');
    expect(useCredentialStore.getState().dify.baseUrl).toBe('https://test.com');
  });
});
