import { useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { DifyCredentialForm } from './DifyCredentialForm';
import { GenericLlmCredentialForm } from './GenericLlmCredentialForm';
import { TeacherModelParamsForm } from './TeacherModelParamsForm';
import { FeedbackAlert } from './FeedbackAlert';
import { useFeedback } from './hooks/useFeedback';
import { useLoadApiConfig, useSaveApiConfig, buildSaveConfigRequest } from './hooks/useApiConfig';
import { useCredentialStore } from '@/stores/useCredentialStore';

/**
 * API 配置面板组件
 * 作为 api-config feature 的对外入口
 */
export function ApiConfigPanel() {
  const { feedback, showFeedback, clearFeedback } = useFeedback();
  
  // 加载已保存的配置
  const { isLoading: isLoadingConfig } = useLoadApiConfig();
  
  // 保存配置 mutation
  const saveConfigMutation = useSaveApiConfig();
  
  // 从 Store 读取状态
  const difyStatus = useCredentialStore((state) => state.dify.status);
  const difyApiKey = useCredentialStore((state) => state.dify.apiKey);
  const genericLlmStatus = useCredentialStore((state) => state.genericLlm.status);
  const genericLlmApiKey = useCredentialStore((state) => state.genericLlm.apiKey);
  const isDirty = useCredentialStore((state) => state.isDirty);
  const hasTeacherSettingsErrors = useCredentialStore((state) => state.hasTeacherSettingsErrors);
  
  // 判断是否可以保存 (Story 1.5 Task 9.1):
  // - 两组凭证都必须是 valid 状态（本次会话已测试通过）
  // - 且两组凭证的 apiKey 都必须非空
  // - 老师模型参数必须在有效范围内 (Code Review Fix: Issue #4/#6)
  // 注意：'saved' 状态表示已保存但未测试，不允许直接再次保存
  const canSave = 
    difyStatus === 'valid' && 
    genericLlmStatus === 'valid' &&
    !!difyApiKey && 
    !!genericLlmApiKey &&
    !hasTeacherSettingsErrors;
  
  // 保存按钮禁用提示
  const getSaveButtonTooltip = () => {
    // 检查 'saved' 状态（已保存但需重新测试）
    if (difyStatus === 'saved' || genericLlmStatus === 'saved') {
      return '检测到已保存的配置，请重新输入 API Key 并测试连接';
    }
    if (difyStatus !== 'valid' && genericLlmStatus !== 'valid') {
      return '请先测试 Dify 和通用大模型连接';
    }
    if (difyStatus !== 'valid') {
      return '请先测试 Dify 连接';
    }
    if (genericLlmStatus !== 'valid') {
      return '请先测试通用大模型连接';
    }
    // 检查 apiKey 是否为空（虽然 status 是 valid）
    if (!difyApiKey || !genericLlmApiKey) {
      return '请确保已输入 API Key';
    }
    // 检查老师模型参数验证错误 (Code Review Fix: Issue #4/#6)
    if (hasTeacherSettingsErrors) {
      return '老师模型参数超出有效范围，请修正';
    }
    return '';
  };
  
  const handleSave = async () => {
    clearFeedback();
    
    try {
      const request = buildSaveConfigRequest();
      await saveConfigMutation.mutateAsync(request);
      showFeedback('success', '配置保存成功', 3000);
    } catch (error) {
      showFeedback('error', error instanceof Error ? error.message : '保存失败', 5000);
    }
  };
  
  // 加载时显示加载状态
  useEffect(() => {
    if (isLoadingConfig) {
      // 可以添加加载指示器
    }
  }, [isLoadingConfig]);

  return (
    <div className="space-y-6 w-full max-w-2xl">
      {/* Dify 凭证配置 */}
      <Card>
        <CardHeader>
          <CardTitle>API 配置</CardTitle>
          <CardDescription>
            配置用于优化测试的 API 凭证
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-8">
          <DifyCredentialForm />
          <div className="border-t pt-6">
            <GenericLlmCredentialForm />
          </div>
        </CardContent>
      </Card>
      
      {/* 老师模型参数 */}
      <TeacherModelParamsForm />
      
      {/* 保存配置区域 */}
      <Card>
        <CardContent className="pt-6">
          <div className="space-y-4">
            {/* 反馈消息 */}
            <FeedbackAlert feedback={feedback} />
            
            {/* 保存按钮 */}
            <div className="flex items-center justify-between">
              <div className="text-sm text-muted-foreground">
                {!canSave && (
                  <span className="text-amber-600">{getSaveButtonTooltip()}</span>
                )}
                {isDirty && canSave && (
                  <span className="text-amber-600">有未保存的修改</span>
                )}
              </div>
              <Button
                onClick={handleSave}
                disabled={!canSave || saveConfigMutation.isPending}
                data-testid="save-config-button"
              >
                {saveConfigMutation.isPending ? '保存中...' : '保存配置'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
