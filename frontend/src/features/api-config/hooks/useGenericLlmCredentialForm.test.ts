import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useGenericLlmCredentialForm } from './useGenericLlmCredentialForm';
import { validateCredential } from './form-utils';
import { useCredentialStore } from '@/stores/useCredentialStore';

describe('validateCredential (通用校验)', () => {
  it('应通过有效的凭证验证', () => {
    const result = validateCredential('https://api.siliconflow.cn', 'sk-xxxx');
    expect(result.isValid).toBe(true);
    expect(result.errors).toEqual({});
  });

  it('应拒绝空的 baseUrl', () => {
    const result = validateCredential('', 'sk-xxxx');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBe('API 地址不能为空');
  });

  it('应拒绝无效的 baseUrl 格式', () => {
    const result = validateCredential('not-a-url', 'sk-xxxx');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
  });

  it('应拒绝空的 apiKey', () => {
    const result = validateCredential('https://api.siliconflow.cn', '');
    expect(result.isValid).toBe(false);
    expect(result.errors.apiKey).toBe('API Key 不能为空');
  });

  it('应拒绝只有空格的输入', () => {
    const result = validateCredential('   ', '   ');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBe('API 地址不能为空');
    expect(result.errors.apiKey).toBe('API Key 不能为空');
  });

  it('应同时返回多个错误', () => {
    const result = validateCredential('', '');
    expect(result.isValid).toBe(false);
    expect(result.errors.baseUrl).toBeDefined();
    expect(result.errors.apiKey).toBeDefined();
  });
});

// 注：url-utils 的完整测试已移至 url-utils.test.ts，此处不再重复

describe('useGenericLlmCredentialForm Hook', () => {
  // 每个测试前重置 Store
  beforeEach(() => {
    const store = useCredentialStore.getState();
    store.clearGenericLlmCredential();
  });

  describe('Provider 切换清空行为', () => {
    it('setGenericLlmProvider 后 baseUrl/apiKey 应为空', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      // 选择 provider 并填写凭证
      act(() => {
        result.current.handleProviderChange('siliconflow');
        result.current.handleBaseUrlChange('https://api.siliconflow.cn');
        result.current.handleApiKeyChange('sk-test-key');
      });

      // 验证已填写
      expect(result.current.baseUrl).toBe('https://api.siliconflow.cn');
      expect(result.current.apiKey).toBe('sk-test-key');

      // 切换 provider
      act(() => {
        result.current.handleProviderChange('modelscope');
      });

      // 验证已清空
      expect(result.current.provider).toBe('modelscope');
      expect(result.current.baseUrl).toBe('');
      expect(result.current.apiKey).toBe('');
      expect(result.current.status).toBe('empty');

      // 验证 Store 状态
      const store = useCredentialStore.getState();
      expect(store.genericLlm.baseUrl).toBe('');
      expect(store.genericLlm.apiKey).toBe('');
    });
  });

  describe('blur 校验', () => {
    it('失焦时应触发字段级错误提示', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      // 选择 provider
      act(() => {
        result.current.handleProviderChange('siliconflow');
      });

      // 输入无效 URL
      act(() => {
        result.current.handleBaseUrlChange('invalid-url');
      });

      // 触发 blur
      act(() => {
        result.current.handleBlur('baseUrl');
      });

      // 验证错误消息
      expect(result.current.errors.baseUrl).toBe('请输入有效的 HTTP/HTTPS 地址');
    });

    it('空字段失焦时应显示非空错误', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      act(() => {
        result.current.handleProviderChange('siliconflow');
      });

      // 触发空字段 blur
      act(() => {
        result.current.handleBlur('apiKey');
      });

      expect(result.current.errors.apiKey).toBe('API Key 不能为空');
    });
  });

  describe('submit 校验', () => {
    it('提交时应验证所有字段，阻止无效输入', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      // 选择 provider 并填写无效凭证
      act(() => {
        result.current.handleProviderChange('siliconflow');
        result.current.handleBaseUrlChange('not-a-valid-url');
        result.current.handleApiKeyChange('sk-test-key');
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

    it('有效凭证应成功保存', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      act(() => {
        result.current.handleProviderChange('siliconflow');
        result.current.handleBaseUrlChange('https://api.siliconflow.cn');
        result.current.handleApiKeyChange('sk-test-key');
      });

      act(() => {
        const submitResult = result.current.handleSubmit();
        expect(submitResult.success).toBe(true);
        expect(submitResult.action).toBe('saved');
      });

      // 验证 Store 状态
      const store = useCredentialStore.getState();
      expect(store.genericLlm.status).toBe('filled');
      expect(store.genericLlm.baseUrl).toBe('https://api.siliconflow.cn');
    });
  });

  describe('baseUrl normalize', () => {
    it('保存时应规范化 baseUrl（提取 origin + 去尾斜杠）', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      act(() => {
        result.current.handleProviderChange('siliconflow');
        result.current.handleBaseUrlChange('https://api.siliconflow.cn/v1/chat/');
        result.current.handleApiKeyChange('sk-test-key');
      });

      act(() => {
        result.current.handleSubmit();
      });

      // 验证 Store 中的 baseUrl 已规范化
      const store = useCredentialStore.getState();
      expect(store.genericLlm.baseUrl).toBe('https://api.siliconflow.cn');
    });
  });

  describe('双字段都空时清空凭证字段（保留 provider）', () => {
    it('两字段都空时应清空字段但保留 provider 选择', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      // 先填写凭证
      act(() => {
        result.current.handleProviderChange('siliconflow');
        result.current.handleBaseUrlChange('https://api.siliconflow.cn');
        result.current.handleApiKeyChange('sk-test-key');
      });

      // 提交保存
      act(() => {
        result.current.handleSubmit();
      });

      // 验证已保存
      expect(useCredentialStore.getState().genericLlm.status).toBe('filled');

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

      // 验证字段已清空但 provider 保留
      const store = useCredentialStore.getState();
      expect(store.genericLlm.status).toBe('empty');
      expect(store.genericLlm.baseUrl).toBe('');
      expect(store.genericLlm.apiKey).toBe('');
      expect(store.genericLlm.provider).toBe('siliconflow'); // provider 保留！
    });
  });

  describe('单一事实来源', () => {
    it('Store 更新后 Hook 返回值应同步', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      // 通过 Hook 更新
      act(() => {
        result.current.handleProviderChange('modelscope');
        result.current.handleBaseUrlChange('https://dashscope.aliyuncs.com');
      });

      // Hook 返回值应与 Store 同步
      expect(result.current.provider).toBe('modelscope');
      expect(result.current.baseUrl).toBe('https://dashscope.aliyuncs.com');
      expect(useCredentialStore.getState().genericLlm.baseUrl).toBe('https://dashscope.aliyuncs.com');
    });
  });

  describe('Store 隔离性', () => {
    beforeEach(() => {
      // 重置两个凭证状态
      useCredentialStore.getState().clearDifyCredential();
      useCredentialStore.getState().clearGenericLlmCredential();
    });

    it('修改 genericLlm 不应影响 dify 状态', () => {
      const store = useCredentialStore.getState();
      
      // 先设置 dify 状态
      store.setDifyFormField('baseUrl', 'https://dify.example.com');
      store.setDifyFormField('apiKey', 'app-dify-key');
      
      // 修改 genericLlm
      store.setGenericLlmProvider('siliconflow');
      store.setGenericLlmFormField('baseUrl', 'https://api.siliconflow.cn');
      store.setGenericLlmFormField('apiKey', 'sk-generic-key');
      
      // 验证 dify 未受影响
      const state = useCredentialStore.getState();
      expect(state.dify.baseUrl).toBe('https://dify.example.com');
      expect(state.dify.apiKey).toBe('app-dify-key');
      expect(state.dify.status).toBe('filled');
    });

    it('修改 dify 不应影响 genericLlm 状态', () => {
      const store = useCredentialStore.getState();
      
      // 先设置 genericLlm 状态
      store.setGenericLlmProvider('modelscope');
      store.setGenericLlmFormField('baseUrl', 'https://dashscope.aliyuncs.com');
      store.setGenericLlmFormField('apiKey', 'sk-generic-key');
      
      // 修改 dify
      store.setDifyFormField('baseUrl', 'https://dify.example.com');
      store.setDifyFormField('apiKey', 'app-dify-key');
      
      // 验证 genericLlm 未受影响
      const state = useCredentialStore.getState();
      expect(state.genericLlm.provider).toBe('modelscope');
      expect(state.genericLlm.baseUrl).toBe('https://dashscope.aliyuncs.com');
      expect(state.genericLlm.apiKey).toBe('sk-generic-key');
      expect(state.genericLlm.status).toBe('filled');
    });

    it('clearGenericLlmCredential 不应影响 dify 状态', () => {
      const store = useCredentialStore.getState();
      
      // 设置两个凭证
      store.setDifyFormField('baseUrl', 'https://dify.example.com');
      store.setDifyFormField('apiKey', 'app-dify-key');
      store.setGenericLlmProvider('siliconflow');
      store.setGenericLlmFormField('baseUrl', 'https://api.siliconflow.cn');
      
      // 清空 genericLlm
      store.clearGenericLlmCredential();
      
      // 验证 dify 未受影响
      const state = useCredentialStore.getState();
      expect(state.dify.baseUrl).toBe('https://dify.example.com');
      expect(state.dify.apiKey).toBe('app-dify-key');
      expect(state.genericLlm.provider).toBe(null);
      expect(state.genericLlm.baseUrl).toBe('');
    });
  });

  describe('provider 防御性校验', () => {
    it('provider 未选择时提交应失败', () => {
      const { result } = renderHook(() => useGenericLlmCredentialForm());

      // 直接通过 Store 设置字段（绕过 UI 限制）
      act(() => {
        useCredentialStore.getState().setGenericLlmFormField('baseUrl', 'https://api.siliconflow.cn');
        useCredentialStore.getState().setGenericLlmFormField('apiKey', 'sk-test-key');
      });

      // 提交时应因 provider 未选择而失败
      act(() => {
        const submitResult = result.current.handleSubmit();
        expect(submitResult.success).toBe(false);
        expect(submitResult.action).toBe('validation_failed');
      });

      // 注意：Store status 仍为 filled（因为 setGenericLlmFormField 会根据字段值推断 status）
      // 但 handleSubmit 返回 validation_failed，阻止了提交流程
      expect(useCredentialStore.getState().genericLlm.status).toBe('filled');
    });

    it('Store 防御：provider=null 时 updateGenericLlmCredentialFromForm 应保持 status=empty', () => {
      const store = useCredentialStore.getState();
      
      // 不设置 provider，直接调用 updateGenericLlmCredentialFromForm
      store.updateGenericLlmCredentialFromForm('https://api.siliconflow.cn', 'sk-test-key');
      
      // 验证 status 被强制为 empty（防御性检查生效）
      const state = useCredentialStore.getState();
      expect(state.genericLlm.provider).toBe(null);
      expect(state.genericLlm.status).toBe('empty');
    });
  });
});
