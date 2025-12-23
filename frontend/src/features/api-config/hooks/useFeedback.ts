import { useState, useRef, useEffect, useCallback } from 'react';

/**
 * 反馈消息类型
 */
export interface FeedbackMessage {
  type: 'success' | 'error';
  message: string;
}

/**
 * useFeedback Hook 返回值
 */
export interface UseFeedbackReturn {
  feedback: FeedbackMessage | null;
  showFeedback: (type: 'success' | 'error', message: string, duration: number) => void;
  clearFeedback: () => void;
}

/**
 * 反馈消息 Hook
 * 用于显示临时反馈消息，自动在指定时间后清除
 * 
 * Dify 和 Generic LLM 表单共用
 */
export function useFeedback(): UseFeedbackReturn {
  const [feedback, setFeedback] = useState<FeedbackMessage | null>(null);
  const feedbackTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 清理定时器（组件卸载时）
  useEffect(() => {
    return () => {
      if (feedbackTimeoutRef.current) {
        clearTimeout(feedbackTimeoutRef.current);
      }
    };
  }, []);

  const clearFeedback = useCallback(() => {
    if (feedbackTimeoutRef.current) {
      clearTimeout(feedbackTimeoutRef.current);
      feedbackTimeoutRef.current = null;
    }
    setFeedback(null);
  }, []);

  const showFeedback = useCallback((type: 'success' | 'error', message: string, duration: number) => {
    // 清除之前的定时器，避免竞态
    if (feedbackTimeoutRef.current) {
      clearTimeout(feedbackTimeoutRef.current);
    }
    setFeedback({ type, message });
    feedbackTimeoutRef.current = setTimeout(() => {
      setFeedback(null);
      feedbackTimeoutRef.current = null;
    }, duration);
  }, []);

  return {
    feedback,
    showFeedback,
    clearFeedback,
  };
}
