import { useState } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Loader2 } from 'lucide-react';
import { useGenericLlmCredentialForm, type SubmitResult } from './hooks/useGenericLlmCredentialForm';
import { useFeedback } from './hooks/useFeedback';
import { useTestGenericLlmConnection } from './hooks/useTestConnection';
import { FeedbackAlert } from './FeedbackAlert';
import { API_KEY_MAX_LENGTH, BASE_URL_MAX_LENGTH } from './constants';
import { statusBadgeMap, type GenericLlmProvider } from '@/types/credentials';


/**
 * Provider 配置信息
 */
const PROVIDER_CONFIG: Record<GenericLlmProvider, { label: string; urlPlaceholder: string; keyPlaceholder: string }> = {
  siliconflow: {
    label: '硅基流动',
    urlPlaceholder: 'https://api.siliconflow.cn',
    keyPlaceholder: 'sk-xxxxxxxxxxxxxxxx',
  },
  modelscope: {
    label: '魔搭社区',
    urlPlaceholder: 'https://dashscope.aliyuncs.com/compatible-mode',
    keyPlaceholder: 'sk-xxxxxxxxxxxxxxxx',
  },
};

/**
 * 通用大模型凭证表单组件
 * 
 * @description 提供硅基流动和魔搭社区的 API 凭证配置界面
 * @example
 * ```tsx
 * <GenericLlmCredentialForm />
 * ```
 */
export function GenericLlmCredentialForm() {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const {
    provider,
    baseUrl,
    apiKey,
    errors,
    status,
    handleProviderChange,
    handleBaseUrlChange,
    handleApiKeyChange,
    handleBlur,
    handleSubmit,
  } = useGenericLlmCredentialForm();

  // 使用共享的 useFeedback Hook
  const { feedback, showFeedback } = useFeedback();

  // 连接测试 mutation
  const testConnection = useTestGenericLlmConnection();

  // 表单是否填写完整（需要 provider 已选择）
  const isFormComplete = provider !== null && baseUrl.trim() !== '' && apiKey.trim() !== '';

  // 处理测试连接
  const handleTestConnection = async () => {
    if (!isFormComplete || !provider) return;
    
    try {
      const result = await testConnection.mutateAsync({ baseUrl, apiKey, provider });
      // 展示后端返回的 message（包含模型数量）
      // 如果有模型列表，显示前 5 个作为预览
      let feedbackMessage = result?.message || '连接成功';
      if (result?.models && result.models.length > 0) {
        const preview = result.models.slice(0, 5).join(', ');
        const suffix = result.models.length > 5 ? ` 等 ${result.models.length} 个模型` : '';
        feedbackMessage = `${feedbackMessage}：${preview}${suffix}`;
      }
      showFeedback('success', feedbackMessage, 5000);
    } catch (error) {
      const message = error instanceof Error ? error.message : '连接失败';
      showFeedback('error', message, 5000);
    }
  };

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    
    // 添加最小延迟以显示 loading 状态，同时为未来后端对接预留
    await new Promise((resolve) => setTimeout(resolve, 150));
    
    const result: SubmitResult = handleSubmit();
    
    // 根据结果显示反馈
    if (result.success) {
      if (result.action === 'saved') {
        showFeedback('success', '已保存，待测试连接', 3000);
      } else if (result.action === 'cleared') {
        showFeedback('success', '凭证已清空', 3000);
      }
    } else {
      showFeedback('error', '请修正下方标红字段后重试', 5000);
    }
    
    setIsSubmitting(false);
  };

  /**
   * 获取状态徽章显示内容
   */
  const getStatusBadge = () => {
    // 特殊处理：未选择 provider 时显示"未配置"
    if (status === 'empty' && provider === null) {
      return {
        variant: 'secondary' as const,
        text: '未配置',
      };
    }
    // 使用 statusBadgeMap 获取配置
    const config = statusBadgeMap[status];
    return {
      variant: config.variant,
      text: status === 'empty' && provider !== null ? '待填写' : config.text,
    };
  };

  const statusBadge = getStatusBadge();

  return (
    <form onSubmit={onSubmit} className="space-y-4" data-testid="generic-llm-credential-form">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">通用大模型凭证</h3>
        <Badge variant={statusBadge.variant}>
          {statusBadge.text}
        </Badge>
      </div>

      <FeedbackAlert feedback={feedback} />

      {/* Provider 选择器 */}
      <div className="space-y-2">
        <Label>Provider 类型</Label>
        <div className="flex gap-2">
          <Button
            type="button"
            variant={provider === 'siliconflow' ? 'default' : 'outline'}
            onClick={() => handleProviderChange('siliconflow')}
            data-testid="provider-siliconflow"
          >
            {PROVIDER_CONFIG.siliconflow.label}
          </Button>
          <Button
            type="button"
            variant={provider === 'modelscope' ? 'default' : 'outline'}
            onClick={() => handleProviderChange('modelscope')}
            data-testid="provider-modelscope"
          >
            {PROVIDER_CONFIG.modelscope.label}
          </Button>
        </div>
      </div>

      {/* 表单字段（仅在选择 Provider 后显示） */}
      {provider && (
        <>
          <div className="space-y-2">
            <Label htmlFor="generic-llm-base-url">API 地址</Label>
            <Input
              id="generic-llm-base-url"
              type="text"
              placeholder={PROVIDER_CONFIG[provider].urlPlaceholder}
              value={baseUrl}
              onChange={(e) => handleBaseUrlChange(e.target.value)}
              onBlur={() => handleBlur('baseUrl')}
              className={errors.baseUrl ? 'border-red-500' : ''}
              maxLength={BASE_URL_MAX_LENGTH}
              aria-invalid={!!errors.baseUrl}
              aria-describedby={errors.baseUrl ? 'generic-llm-base-url-error' : undefined}
              data-testid="generic-llm-base-url-input"
            />
            {errors.baseUrl && (
              <p id="generic-llm-base-url-error" className="text-sm text-red-500" role="alert">{errors.baseUrl}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="generic-llm-api-key">API Key</Label>
            <Input
              id="generic-llm-api-key"
              type="password"
              placeholder={PROVIDER_CONFIG[provider].keyPlaceholder}
              value={apiKey}
              onChange={(e) => handleApiKeyChange(e.target.value)}
              onBlur={() => handleBlur('apiKey')}
              className={errors.apiKey ? 'border-red-500' : ''}
              maxLength={API_KEY_MAX_LENGTH}
              autoComplete="new-password"
              spellCheck={false}
              aria-invalid={!!errors.apiKey}
              aria-describedby={errors.apiKey ? 'generic-llm-api-key-error' : undefined}
              data-testid="generic-llm-api-key-input"
            />
            {errors.apiKey && (
              <p id="generic-llm-api-key-error" className="text-sm text-red-500" role="alert">{errors.apiKey}</p>
            )}
          </div>

          <div className="flex gap-2">
            <Button 
              type="submit" 
              className="flex-1" 
              disabled={isSubmitting}
              data-testid="generic-llm-submit-btn"
            >
              {isSubmitting ? '保存中...' : '保存'}
            </Button>
            <Button
              type="button"
              variant="outline"
              className="flex-1"
              disabled={!isFormComplete || testConnection.isPending}
              onClick={handleTestConnection}
              data-testid="generic-llm-test-connection-btn"
            >
              {testConnection.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  测试中...
                </>
              ) : (
                '测试连接'
              )}
            </Button>
          </div>
        </>
      )}
    </form>
  );
}
