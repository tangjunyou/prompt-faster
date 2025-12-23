/**
 * 表单工具函数和共享类型
 * 由 Dify 和 Generic LLM 表单共用
 */

import { isValidUrl } from './url-utils';

/**
 * 表单错误接口
 */
export interface FormErrors {
  baseUrl?: string;
  apiKey?: string;
}

/**
 * 提交结果接口
 */
export interface SubmitResult {
  success: boolean;
  action: 'saved' | 'cleared' | 'validation_failed';
}

/**
 * 校验结果接口
 */
export interface ValidationResult {
  isValid: boolean;
  errors: FormErrors;
}

/**
 * 验证凭证（baseUrl + apiKey）
 * Dify 和 Generic LLM 表单共用
 */
export const validateCredential = (baseUrl: string, apiKey: string): ValidationResult => {
  const errors: FormErrors = {};

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
