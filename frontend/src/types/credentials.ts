/**
 * 凭证状态类型
 * - empty: 未配置
 * - filled: 已填写，待测试
 */
export type CredentialStatus = 'empty' | 'filled';

/**
 * Dify API 凭证接口
 */
export interface DifyCredential {
  baseUrl: string;
  apiKey: string;
  status: CredentialStatus;
}

/**
 * 通用大模型 Provider 类型
 */
export type GenericLlmProvider = 'siliconflow' | 'modelscope';

/**
 * 通用大模型 API 凭证接口
 */
export interface GenericLlmCredential {
  provider: GenericLlmProvider | null;  // null 表示未选择
  baseUrl: string;
  apiKey: string;
  status: CredentialStatus;  // 由 Store 方法推断，不允许外部直接设置
}
