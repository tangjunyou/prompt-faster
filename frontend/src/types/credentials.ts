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
