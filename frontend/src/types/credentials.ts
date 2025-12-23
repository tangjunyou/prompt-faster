/**
 * 凭证状态类型
 * - empty: 未配置
 * - filled: 已填写，待测试
 * - testing: 测试中
 * - valid: 连接成功
 * - invalid: 连接失败
 */
export type CredentialStatus = 'empty' | 'filled' | 'testing' | 'valid' | 'invalid';

/**
 * 凭证状态徽章配置映射
 */
export const statusBadgeMap: Record<CredentialStatus, { color: string; variant: 'secondary' | 'warning' | 'default' | 'destructive'; text: string }> = {
  empty: { color: 'gray', variant: 'secondary', text: '未配置' },
  filled: { color: 'yellow', variant: 'warning', text: '已填写，待测试' },
  testing: { color: 'blue', variant: 'default', text: '测试中...' },
  valid: { color: 'green', variant: 'default', text: '连接成功' },
  invalid: { color: 'red', variant: 'destructive', text: '连接失败' },
};

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
