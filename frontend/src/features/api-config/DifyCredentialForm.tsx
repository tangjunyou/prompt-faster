import { useState } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { useDifyCredentialForm, type SubmitResult } from './hooks/useDifyCredentialForm';

/**
 * Dify 凭证表单组件
 */
export function DifyCredentialForm() {
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

  // 反馈消息状态
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const onSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const result: SubmitResult = handleSubmit();
    
    // 根据结果显示反馈
    if (result.success) {
      if (result.action === 'saved') {
        setFeedback({ type: 'success', message: '已保存，待测试连接' });
      } else if (result.action === 'cleared') {
        setFeedback({ type: 'success', message: '凭证已清空' });
      }
      // 3秒后清除反馈
      setTimeout(() => setFeedback(null), 3000);
    } else {
      setFeedback({ type: 'error', message: '请修正下方标红字段后重试' });
      setTimeout(() => setFeedback(null), 5000);
    }
  };

  return (
    <form onSubmit={onSubmit} className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">Dify 工作流凭证</h3>
        {/* M1: 使用 warning variant 实现黄色徽章，符合 Story 规范 */}
        <Badge variant={status === 'empty' ? 'secondary' : 'warning'}>
          {status === 'empty' ? '未配置' : '已填写，待测试'}
        </Badge>
      </div>

      {/* L1: 反馈消息 */}
      {feedback && (
        <div
          className={`p-3 rounded-md text-sm ${
            feedback.type === 'success'
              ? 'bg-green-50 text-green-700 border border-green-200'
              : 'bg-red-50 text-red-700 border border-red-200'
          }`}
        >
          {feedback.message}
        </div>
      )}

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
        />
        {errors.baseUrl && (
          <p className="text-sm text-red-500">{errors.baseUrl}</p>
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
        />
        {errors.apiKey && (
          <p className="text-sm text-red-500">{errors.apiKey}</p>
        )}
      </div>

      <Button type="submit" className="w-full">
        保存
      </Button>
    </form>
  );
}
