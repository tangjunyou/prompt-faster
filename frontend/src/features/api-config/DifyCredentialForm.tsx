import { useState } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { useDifyCredentialForm, type SubmitResult } from './hooks/useDifyCredentialForm';
import { useFeedback } from './hooks/useFeedback';
import { FeedbackAlert } from './FeedbackAlert';
import { API_KEY_MAX_LENGTH, BASE_URL_MAX_LENGTH } from './constants';


/**
 * Dify 凭证表单组件
 * 
 * @description 提供 Dify 工作流的 API 凭证配置界面
 * @example
 * ```tsx
 * <DifyCredentialForm />
 * ```
 */
export function DifyCredentialForm() {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const {
    baseUrl,
    apiKey,
    errors,
    status,
    handleBaseUrlChange,
    handleApiKeyChange,
    handleBlur,
    handleSubmit,
  } = useDifyCredentialForm();

  // 使用共享的 useFeedback Hook
  const { feedback, showFeedback } = useFeedback();

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

  return (
    <form onSubmit={onSubmit} className="space-y-4" data-testid="dify-credential-form">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">Dify 工作流凭证</h3>
        {/* M1: 使用 warning variant 实现黄色徽章，符合 Story 规范 */}
        <Badge variant={status === 'empty' ? 'secondary' : 'warning'}>
          {status === 'empty' ? '未配置' : '已填写，待测试'}
        </Badge>
      </div>

      <FeedbackAlert feedback={feedback} />

      <div className="space-y-2">
        <Label htmlFor="dify-base-url">API 地址</Label>
        <Input
          id="dify-base-url"
          type="text"
          placeholder="https://api.dify.ai"
          value={baseUrl}
          onChange={(e) => handleBaseUrlChange(e.target.value)}
          onBlur={() => handleBlur('baseUrl')}
          className={errors.baseUrl ? 'border-red-500' : ''}
          maxLength={BASE_URL_MAX_LENGTH}
          aria-invalid={!!errors.baseUrl}
          aria-describedby={errors.baseUrl ? 'dify-base-url-error' : undefined}
          data-testid="dify-base-url-input"
        />
        {errors.baseUrl && (
          <p id="dify-base-url-error" className="text-sm text-red-500" role="alert">{errors.baseUrl}</p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="dify-api-key">API Key</Label>
        <Input
          id="dify-api-key"
          type="password"
          placeholder="app-xxxxxxxxxxxxxxxx"
          value={apiKey}
          onChange={(e) => handleApiKeyChange(e.target.value)}
          onBlur={() => handleBlur('apiKey')}
          className={errors.apiKey ? 'border-red-500' : ''}
          maxLength={API_KEY_MAX_LENGTH}
          autoComplete="new-password"
          spellCheck={false}
          aria-invalid={!!errors.apiKey}
          aria-describedby={errors.apiKey ? 'dify-api-key-error' : undefined}
          data-testid="dify-api-key-input"
        />
        {errors.apiKey && (
          <p id="dify-api-key-error" className="text-sm text-red-500" role="alert">{errors.apiKey}</p>
        )}
      </div>

      <Button 
        type="submit" 
        className="w-full" 
        disabled={isSubmitting}
        data-testid="dify-submit-btn"
      >
        {isSubmitting ? '保存中...' : '保存'}
      </Button>
    </form>
  );
}
