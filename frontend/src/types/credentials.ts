import type { ConfigResponse } from '@/types/generated/api/ConfigResponse';
import type { SaveConfigRequest as GeneratedSaveConfigRequest } from '@/types/generated/api/SaveConfigRequest';

/**
 * 凭证状态类型
 * - empty: 未配置
 * - filled: 已填写，待测试
 * - testing: 测试中
 * - valid: 连接成功（本次会话已测试通过）
 * - invalid: 连接失败
 * - saved: 已保存到后端（但本次会话未测试，需重新测试才能更新）
 */
export type CredentialStatus = 'empty' | 'filled' | 'testing' | 'valid' | 'invalid' | 'saved';

/**
 * 凭证状态徽章配置映射
 */
export const statusBadgeMap: Record<CredentialStatus, { color: string; variant: 'secondary' | 'warning' | 'default' | 'destructive'; text: string }> = {
  empty: { color: 'gray', variant: 'secondary', text: '未配置' },
  filled: { color: 'yellow', variant: 'warning', text: '已填写，待测试' },
  testing: { color: 'blue', variant: 'default', text: '测试中...' },
  valid: { color: 'green', variant: 'default', text: '连接成功' },
  invalid: { color: 'red', variant: 'destructive', text: '连接失败' },
  saved: { color: 'blue', variant: 'secondary', text: '已保存，需重新测试' },
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

/**
 * 老师模型参数配置
 */
export interface TeacherModelSettings {
  temperature: number;  // 0.0 ~ 2.0，默认 0.7
  topP: number;         // 0.0 ~ 1.0，默认 0.9
  maxTokens: number;    // 1 ~ 8192，默认 2048
}

/**
 * 老师模型参数默认值
 */
export const defaultTeacherSettings: TeacherModelSettings = {
  temperature: 0.7,
  topP: 0.9,
  maxTokens: 2048,
};

/**
 * 老师模型参数范围约束
 */
export const teacherSettingsConstraints = {
  temperature: { min: 0.0, max: 2.0, step: 0.1 },  // 与后端 validate_teacher_settings 范围一致
  topP: { min: 0.0, max: 1.0, step: 0.1 },          // 与后端 validate_teacher_settings 范围一致
  maxTokens: { min: 1, max: 8192, step: 64 },       // Code Review Fix: step 从 1 改为 64 提升用户体验
} as const;

/**
 * API 配置响应类型（由 ts-rs 生成）
 */
export type ApiConfigResponse = ConfigResponse;

/**
 * 保存配置请求类型（由 ts-rs 生成）
 */
export type SaveConfigRequest = GeneratedSaveConfigRequest;
