import { useState, useCallback } from 'react';
import { useCredentialStore } from '@/stores/useCredentialStore';

/**
 * 验证 URL 是否为有效的 http/https 地址
 */
export const isValidUrl = (url: string): boolean => {
  try {
    const parsed = new URL(url);
    return ['http:', 'https:'].includes(parsed.protocol);
  } catch {
    return false;
  }
};

/**
 * 规范化 Base URL：提取 origin 并去除末尾斜杠
 * 注意：调用前应先用 isValidUrl 验证，否则可能抛出异常
 */
export const normalizeBaseUrl = (baseUrl: string): string => {
  try {
    const trimmed = baseUrl.trim().replace(/\/+$/, '');
    const parsed = new URL(trimmed);
    return parsed.origin;
  } catch {
    // 防御性编程：如果解析失败，返回 trim 后的原值
    return baseUrl.trim().replace(/\/+$/, '');
  }
};

/**
 * 验证 Dify 凭证
 */
export const validateDifyCredential = (baseUrl: string, apiKey: string) => {
  const errors: { baseUrl?: string; apiKey?: string } = {};

  if (!baseUrl.trim()) {
    errors.baseUrl = 'API 地址不能为空';
  } else if (!isValidUrl(baseUrl)) {
    errors.baseUrl = '请输入有效的 HTTP/HTTPS 地址';
  }

  if (!apiKey.trim()) {
    errors.apiKey = 'API Key 不能为空';
  }

  return {
    isValid: Object.keys(errors).length === 0,
    errors,
  };
};

export interface FormErrors {
  baseUrl?: string;
  apiKey?: string;
}

export interface SubmitResult {
  success: boolean;
  action: 'saved' | 'cleared' | 'validation_failed';
}

/**
 * Dify 凭证表单 Hook
 * 
 * 设计原则：Store 作为单一事实来源，表单直接读写 Store
 */
export function useDifyCredentialForm() {
  const { dify, updateDifyCredentialFromForm, clearDifyCredential, setDifyFormField } = useCredentialStore();
  const [errors, setErrors] = useState<FormErrors>({});
  const [touched, setTouched] = useState<{ baseUrl: boolean; apiKey: boolean }>({
    baseUrl: false,
    apiKey: false,
  });

  const validateField = useCallback((field: 'baseUrl' | 'apiKey', value: string) => {
    if (field === 'baseUrl') {
      if (!value.trim()) {
        return 'API 地址不能为空';
      }
      if (!isValidUrl(value)) {
        return '请输入有效的 HTTP/HTTPS 地址';
      }
    }
    if (field === 'apiKey' && !value.trim()) {
      return 'API Key 不能为空';
    }
    return undefined;
  }, []);

  // 直接更新 Store，实现单一事实来源
  const handleBaseUrlChange = useCallback((value: string) => {
    setDifyFormField('baseUrl', value);
    if (touched.baseUrl) {
      setErrors((prev) => ({ ...prev, baseUrl: validateField('baseUrl', value) }));
    }
  }, [touched.baseUrl, validateField, setDifyFormField]);

  const handleApiKeyChange = useCallback((value: string) => {
    setDifyFormField('apiKey', value);
    if (touched.apiKey) {
      setErrors((prev) => ({ ...prev, apiKey: validateField('apiKey', value) }));
    }
  }, [touched.apiKey, validateField, setDifyFormField]);

  const handleBlur = useCallback((field: 'baseUrl' | 'apiKey') => {
    setTouched((prev) => ({ ...prev, [field]: true }));
    const value = field === 'baseUrl' ? dify.baseUrl : dify.apiKey;
    setErrors((prev) => ({ ...prev, [field]: validateField(field, value) }));
  }, [dify.baseUrl, dify.apiKey, validateField]);

  const handleSubmit = useCallback((): SubmitResult => {
    const trimmedBaseUrl = dify.baseUrl.trim();
    const trimmedApiKey = dify.apiKey.trim();

    // 两字段都为空时，执行清空操作
    if (!trimmedBaseUrl && !trimmedApiKey) {
      clearDifyCredential();
      setErrors({});
      setTouched({ baseUrl: false, apiKey: false });
      return { success: true, action: 'cleared' };
    }

    // 验证
    const validation = validateDifyCredential(trimmedBaseUrl, trimmedApiKey);
    if (!validation.isValid) {
      setErrors(validation.errors);
      setTouched({ baseUrl: true, apiKey: true });
      return { success: false, action: 'validation_failed' };
    }

    // 规范化并保存到 Store
    const normalizedUrl = normalizeBaseUrl(trimmedBaseUrl);
    updateDifyCredentialFromForm(normalizedUrl, trimmedApiKey);
    setErrors({});
    return { success: true, action: 'saved' };
  }, [dify.baseUrl, dify.apiKey, updateDifyCredentialFromForm, clearDifyCredential]);

  return {
    // 直接从 Store 读取，实现单一事实来源
    baseUrl: dify.baseUrl,
    apiKey: dify.apiKey,
    errors,
    status: dify.status,
    handleBaseUrlChange,
    handleApiKeyChange,
    handleBlur,
    handleSubmit,
  };
}
