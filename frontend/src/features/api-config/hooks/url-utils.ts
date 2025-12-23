/**
 * URL 校验和规范化工具函数
 * 由 Dify 和 Generic LLM 表单共用
 */

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
