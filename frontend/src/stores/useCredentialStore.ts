import { create } from 'zustand';
import type { DifyCredential, CredentialStatus } from '@/types/credentials';

/**
 * 凭证状态管理 Store
 * 
 * API 设计原则：只暴露明确的领域操作，不允许外部直接设置 status
 */
interface CredentialState {
  dify: DifyCredential;
  /** 从表单更新 Dify 凭证（内部自动推断 status） */
  updateDifyCredentialFromForm: (baseUrl: string, apiKey: string) => void;
  /** 清空 Dify 凭证 */
  clearDifyCredential: () => void;
  /** 直接更新表单字段（用于受控输入，不改变 status） */
  setDifyFormField: (field: 'baseUrl' | 'apiKey', value: string) => void;
}

const initialDifyCredential: DifyCredential = {
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
}));
