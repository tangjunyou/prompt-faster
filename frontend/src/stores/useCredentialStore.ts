import { create } from 'zustand';
import type { 
  DifyCredential, 
  CredentialStatus, 
  GenericLlmCredential, 
  GenericLlmProvider,
  TeacherModelSettings,
  ApiConfigResponse,
} from '@/types/credentials';
import { teacherSettingsConstraints } from '@/types/credentials';
import { defaultTeacherSettings } from '@/types/credentials';

/**
 * 凭证状态管理 Store
 * 
 * API 设计原则：
 * - 通过 setDifyFormField/setGenericLlmFormField 实时更新表单并推断 status
 * - setDifyStatus/setGenericLlmStatus 供 useTestConnection hook 内部使用，用于测试状态切换
 * - 修改凭证字段后 status 会自动重置为 filled/empty（修改即失效，需重新测试）
 * - isPersisted 标记当前配置是否已持久化到后端
 * - isDirty 标记是否有未保存的修改
 */
interface CredentialState {
  dify: DifyCredential;
  /** 从表单更新 Dify 凭证（内部自动推断 status） */
  updateDifyCredentialFromForm: (baseUrl: string, apiKey: string) => void;
  /** 清空 Dify 凭证 */
  clearDifyCredential: () => void;
  /** 直接更新表单字段（用于受控输入，实时推断 status 以同步状态徽章） */
  setDifyFormField: (field: 'baseUrl' | 'apiKey', value: string) => void;
  /** 直接设置 Dify 凭证状态（用于连接测试） */
  setDifyStatus: (status: CredentialStatus) => void;
  
  // 通用大模型凭证相关
  genericLlm: GenericLlmCredential;
  /** 设置通用大模型 Provider（切换时清空 baseUrl/apiKey） */
  setGenericLlmProvider: (provider: GenericLlmProvider) => void;
  /** 直接更新表单字段（用于受控输入，实时推断 status 以同步状态徽章） */
  setGenericLlmFormField: (field: 'baseUrl' | 'apiKey', value: string) => void;
  /** 从表单更新通用大模型凭证（内部自动推断 status） */
  updateGenericLlmCredentialFromForm: (baseUrl: string, apiKey: string) => void;
  /** 清空通用大模型凭证字段（保留 provider 选择） */
  clearGenericLlmFields: () => void;
  /** 完全重置通用大模型凭证（包括 provider） */
  clearGenericLlmCredential: () => void;
  /** 直接设置通用大模型凭证状态（用于连接测试） */
  setGenericLlmStatus: (status: CredentialStatus) => void;
  
  // 老师模型参数
  teacherSettings: TeacherModelSettings;
  /** 老师模型参数是否有验证错误 (Code Review Fix: Issue #4/#6) */
  hasTeacherSettingsErrors: boolean;
  /** 更新老师模型参数 */
  setTeacherSettings: (settings: Partial<TeacherModelSettings>) => void;
  /** 重置老师模型参数为默认值 */
  resetTeacherSettings: () => void;
  
  // 持久化状态
  /** 是否已从服务器加载配置 */
  isHydrated: boolean;
  /** 是否有未保存的修改 */
  isDirty: boolean;
  /** 标记为有未保存的修改 */
  markDirty: () => void;
  /** 标记为已保存 */
  markClean: () => void;
  /** 从服务器配置填充 Store */
  hydrateFromServer: (config: ApiConfigResponse) => void;
}

const initialDifyCredential: DifyCredential = {
  baseUrl: '',
  apiKey: '',
  status: 'empty',
};

const initialGenericLlmCredential: GenericLlmCredential = {
  provider: null,
  baseUrl: '',
  apiKey: '',
  status: 'empty',
};

/**
 * 根据 baseUrl 和 apiKey 推断凭证状态
 */
const inferStatus = (baseUrl: string, apiKey: string): CredentialStatus => {
  return baseUrl.trim() && apiKey.trim() ? 'filled' : 'empty';
};

/**
 * 验证老师模型参数是否在有效范围内 (Code Review Fix: Issue #4/#6)
 */
const validateTeacherSettings = (settings: TeacherModelSettings): boolean => {
  const { temperature, topP, maxTokens } = settings;
  const constraints = teacherSettingsConstraints;
  
  if (temperature < constraints.temperature.min || temperature > constraints.temperature.max) {
    return false;
  }
  if (topP < constraints.topP.min || topP > constraints.topP.max) {
    return false;
  }
  if (maxTokens < constraints.maxTokens.min || maxTokens > constraints.maxTokens.max) {
    return false;
  }
  return true;
};

export const useCredentialStore = create<CredentialState>((set) => ({
  dify: initialDifyCredential,
  
  updateDifyCredentialFromForm: (baseUrl: string, apiKey: string) =>
    set(() => ({
      dify: {
        baseUrl,
        apiKey,
        status: inferStatus(baseUrl, apiKey),
      },
    })),
  
  clearDifyCredential: () => set({ dify: initialDifyCredential }),
  
  setDifyStatus: (status) =>
    set((state) => ({
      dify: {
        ...state.dify,
        status,
      },
    })),
  
  setDifyFormField: (field, value) =>
    set((state) => ({
      dify: {
        ...state.dify,
        [field]: value,
        // 实时更新 status（用于表单输入时的状态徽章同步）
        status: inferStatus(
          field === 'baseUrl' ? value : state.dify.baseUrl,
          field === 'apiKey' ? value : state.dify.apiKey
        ),
      },
      isDirty: true, // Story 1.5 AC 6.1: 任意字段变更后标记为未保存
    })),
  
  // 通用大模型凭证相关实现
  genericLlm: initialGenericLlmCredential,
  
  setGenericLlmProvider: (provider) =>
    set(() => ({
      genericLlm: {
        ...initialGenericLlmCredential,
        provider,
        status: 'empty',
      },
      isDirty: true, // Story 1.5 AC 6.1: 任意字段变更后标记为未保存
    })),
  
  setGenericLlmFormField: (field, value) =>
    set((state) => ({
      genericLlm: {
        ...state.genericLlm,
        [field]: value,
        // 实时更新 status（用于表单输入时的状态徽章同步）
        status: inferStatus(
          field === 'baseUrl' ? value : state.genericLlm.baseUrl,
          field === 'apiKey' ? value : state.genericLlm.apiKey
        ),
      },
      isDirty: true, // Story 1.5 AC 6.1: 任意字段变更后标记为未保存
    })),
  
  updateGenericLlmCredentialFromForm: (baseUrl, apiKey) =>
    set((state) => ({
      genericLlm: {
        ...state.genericLlm,
        baseUrl,
        apiKey,
        // 防御性检查：provider 未选择时强制 status 为 empty
        status: state.genericLlm.provider ? inferStatus(baseUrl, apiKey) : 'empty',
      },
    })),
  
  clearGenericLlmFields: () =>
    set((state) => ({
      genericLlm: {
        ...state.genericLlm,
        baseUrl: '',
        apiKey: '',
        status: 'empty',
      },
    })),
  
  clearGenericLlmCredential: () => set({ genericLlm: initialGenericLlmCredential }),
  
  setGenericLlmStatus: (status) =>
    set((state) => ({
      genericLlm: {
        ...state.genericLlm,
        status,
      },
    })),
  
  // 老师模型参数
  teacherSettings: defaultTeacherSettings,
  hasTeacherSettingsErrors: false,
  
  setTeacherSettings: (settings) =>
    set((state) => {
      const newSettings = {
        ...state.teacherSettings,
        ...settings,
      };
      return {
        teacherSettings: newSettings,
        hasTeacherSettingsErrors: !validateTeacherSettings(newSettings),
        isDirty: true,
      };
    }),
  
  resetTeacherSettings: () =>
    set({
      teacherSettings: defaultTeacherSettings,
      hasTeacherSettingsErrors: false,
      isDirty: true,
    }),
  
  // 持久化状态
  isHydrated: false,
  isDirty: false,
  
  markDirty: () => set({ isDirty: true }),
  
  markClean: () => set({ isDirty: false }),
  
  hydrateFromServer: (config) =>
    set(() => {
      const loadedTeacherSettings: TeacherModelSettings = {
        temperature: config.teacher_settings.temperature,
        topP: config.teacher_settings.top_p,
        maxTokens: config.teacher_settings.max_tokens,
      };
      
      const newState: Partial<CredentialState> = {
        isHydrated: true,
        isDirty: false,
        teacherSettings: loadedTeacherSettings,
        // Code Review Fix: 验证从服务器加载的老师模型参数是否在有效范围内
        hasTeacherSettingsErrors: !validateTeacherSettings(loadedTeacherSettings),
      };
      
      // 如果后端有 Dify 配置，更新 dify 状态
      // Story 1.5 修复: 使用 'saved' 状态而非 'valid'，因为本次会话未测试
      // 用户需要重新输入 API Key 并测试通过后才能保存更新
      if (config.has_dify_key && config.dify_base_url) {
        newState.dify = {
          baseUrl: config.dify_base_url,
          apiKey: '', // 不存储明文，用户需要重新输入才能更新
          status: 'saved', // 已保存但需重新测试
        };
      }
      
      // 如果后端有通用大模型配置，更新 genericLlm 状态
      if (config.has_generic_llm_key && config.generic_llm_base_url) {
        newState.genericLlm = {
          provider: config.generic_llm_provider as GenericLlmProvider | null,
          baseUrl: config.generic_llm_base_url,
          apiKey: '', // 不存储明文
          status: 'saved', // 已保存但需重新测试
        };
      }
      
      return newState;
    }),
}));
