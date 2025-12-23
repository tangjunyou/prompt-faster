import { useState, useCallback } from 'react';
import { useCredentialStore } from '@/stores/useCredentialStore';
import { normalizeBaseUrl } from './url-utils';
import { validateCredential, type FormErrors, type SubmitResult } from './form-utils';
import type { GenericLlmProvider } from '@/types/credentials';

// 重新导出共享类型，保持向后兼容
export type { FormErrors, SubmitResult } from './form-utils';

/**
 * 验证通用大模型凭证（使用共享校验函数）
 */
export const validateGenericLlmCredential = validateCredential;

/**
 * 通用大模型凭证表单 Hook
 * 
 * 设计原则：Store 作为单一事实来源，表单直接读写 Store
 */
export function useGenericLlmCredentialForm() {
  const {
    genericLlm,
    setGenericLlmProvider,
    setGenericLlmFormField,
    updateGenericLlmCredentialFromForm,
    clearGenericLlmFields,
  } = useCredentialStore();
  
  const [errors, setErrors] = useState<FormErrors>({});
  const [touched, setTouched] = useState<{ baseUrl: boolean; apiKey: boolean }>({
    baseUrl: false,
    apiKey: false,
  });

  // 复用 validateCredential 的单字段校验，避免两套规则维护
  const validateField = useCallback((field: 'baseUrl' | 'apiKey', value: string) => {
    // 构造临时值进行校验，复用统一的校验逻辑
    const tempBaseUrl = field === 'baseUrl' ? value : 'https://placeholder.com';
    const tempApiKey = field === 'apiKey' ? value : 'placeholder-key';
    const result = validateCredential(tempBaseUrl, tempApiKey);
    return result.errors[field];
  }, []);

  // 切换 Provider（清空表单状态）
  const handleProviderChange = useCallback((provider: GenericLlmProvider) => {
    setGenericLlmProvider(provider);
    setErrors({});
    setTouched({ baseUrl: false, apiKey: false });
  }, [setGenericLlmProvider]);

  // 直接更新 Store，实现单一事实来源
  const handleBaseUrlChange = useCallback((value: string) => {
    setGenericLlmFormField('baseUrl', value);
    if (touched.baseUrl) {
      setErrors((prev) => ({ ...prev, baseUrl: validateField('baseUrl', value) }));
    }
  }, [touched.baseUrl, validateField, setGenericLlmFormField]);

  const handleApiKeyChange = useCallback((value: string) => {
    setGenericLlmFormField('apiKey', value);
    if (touched.apiKey) {
      setErrors((prev) => ({ ...prev, apiKey: validateField('apiKey', value) }));
    }
  }, [touched.apiKey, validateField, setGenericLlmFormField]);

  const handleBlur = useCallback((field: 'baseUrl' | 'apiKey') => {
    setTouched((prev) => ({ ...prev, [field]: true }));
    const value = field === 'baseUrl' ? genericLlm.baseUrl : genericLlm.apiKey;
    setErrors((prev) => ({ ...prev, [field]: validateField(field, value) }));
  }, [genericLlm.baseUrl, genericLlm.apiKey, validateField]);

  const handleSubmit = useCallback((): SubmitResult => {
    const trimmedBaseUrl = genericLlm.baseUrl.trim();
    const trimmedApiKey = genericLlm.apiKey.trim();

    // 两字段都为空时，清空字段但保留 provider 选择
    if (!trimmedBaseUrl && !trimmedApiKey) {
      clearGenericLlmFields();
      setErrors({});
      setTouched({ baseUrl: false, apiKey: false });
      return { success: true, action: 'cleared' };
    }

    // 防御性校验：provider 必须已选择才能保存
    if (!genericLlm.provider) {
      return { success: false, action: 'validation_failed' };
    }

    // 验证
    const validation = validateCredential(trimmedBaseUrl, trimmedApiKey);
    if (!validation.isValid) {
      setErrors(validation.errors);
      setTouched({ baseUrl: true, apiKey: true });
      return { success: false, action: 'validation_failed' };
    }

    // 规范化并保存到 Store
    const normalizedUrl = normalizeBaseUrl(trimmedBaseUrl);
    updateGenericLlmCredentialFromForm(normalizedUrl, trimmedApiKey);
    setErrors({});
    return { success: true, action: 'saved' };
  }, [genericLlm.baseUrl, genericLlm.apiKey, genericLlm.provider, updateGenericLlmCredentialFromForm, clearGenericLlmFields]);

  return {
    // 直接从 Store 读取，实现单一事实来源
    provider: genericLlm.provider,
    baseUrl: genericLlm.baseUrl,
    apiKey: genericLlm.apiKey,
    errors,
    status: genericLlm.status,
    handleProviderChange,
    handleBaseUrlChange,
    handleApiKeyChange,
    handleBlur,
    handleSubmit,
  };
}
