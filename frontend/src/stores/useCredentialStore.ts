import { create } from 'zustand';
import type { DifyCredential, CredentialStatus, GenericLlmCredential, GenericLlmProvider } from '@/types/credentials';

/**
 * 凭证状态管理 Store
 * 
 * API 设计原则：
 * - 通过 setDifyFormField/setGenericLlmFormField 实时更新表单并推断 status
 * - setDifyStatus/setGenericLlmStatus 供 useTestConnection hook 内部使用，用于测试状态切换
 * - 修改凭证字段后 status 会自动重置为 filled/empty（修改即失效，需重新测试）
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
}));
