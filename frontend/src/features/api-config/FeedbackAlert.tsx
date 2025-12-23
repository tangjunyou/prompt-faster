import type { FeedbackMessage } from './hooks/useFeedback';

/**
 * FeedbackAlert 组件 Props
 */
interface FeedbackAlertProps {
  feedback: FeedbackMessage | null;
}

/**
 * 反馈消息提示组件
 * 
 * @description 显示成功/错误反馈消息，由 Dify 和 Generic LLM 表单共用
 * @example
 * ```tsx
 * <FeedbackAlert feedback={feedback} />
 * ```
 */
export function FeedbackAlert({ feedback }: FeedbackAlertProps) {
  if (!feedback) {
    return null;
  }

  return (
    <div
      role={feedback.type === 'error' ? 'alert' : 'status'}
      aria-live={feedback.type === 'error' ? 'assertive' : 'polite'}
      className={`p-3 rounded-md text-sm ${
        feedback.type === 'success'
          ? 'bg-green-50 text-green-700 border border-green-200'
          : 'bg-red-50 text-red-700 border border-red-200'
      }`}
    >
      {feedback.message}
    </div>
  );
}
