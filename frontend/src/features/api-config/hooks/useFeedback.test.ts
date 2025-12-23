import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useFeedback } from './useFeedback';

/**
 * useFeedback Hook 测试
 * 覆盖反馈消息显示、自动清除和定时器清理
 */
describe('useFeedback Hook', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('初始状态', () => {
    it('初始 feedback 应为 null', () => {
      const { result } = renderHook(() => useFeedback());
      expect(result.current.feedback).toBeNull();
    });
  });

  describe('showFeedback', () => {
    it('应正确显示 success 类型反馈', () => {
      const { result } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('success', '保存成功', 3000);
      });

      expect(result.current.feedback).toEqual({
        type: 'success',
        message: '保存成功',
      });
    });

    it('应正确显示 error 类型反馈', () => {
      const { result } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('error', '保存失败', 5000);
      });

      expect(result.current.feedback).toEqual({
        type: 'error',
        message: '保存失败',
      });
    });

    it('应在指定时间后自动清除反馈', () => {
      const { result } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('success', '测试消息', 3000);
      });

      expect(result.current.feedback).not.toBeNull();

      // 快进 3 秒
      act(() => {
        vi.advanceTimersByTime(3000);
      });

      expect(result.current.feedback).toBeNull();
    });

    it('连续调用 showFeedback 应取消前一个定时器', () => {
      const { result } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('success', '第一条消息', 3000);
      });

      // 1 秒后显示第二条消息
      act(() => {
        vi.advanceTimersByTime(1000);
        result.current.showFeedback('error', '第二条消息', 3000);
      });

      expect(result.current.feedback?.message).toBe('第二条消息');

      // 再过 2 秒（第一个定时器本应触发，但已被取消）
      act(() => {
        vi.advanceTimersByTime(2000);
      });

      // 第二条消息仍然存在
      expect(result.current.feedback?.message).toBe('第二条消息');

      // 再过 1 秒，第二个定时器触发
      act(() => {
        vi.advanceTimersByTime(1000);
      });

      expect(result.current.feedback).toBeNull();
    });
  });

  describe('clearFeedback', () => {
    it('应立即清除反馈', () => {
      const { result } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('success', '测试消息', 3000);
      });

      expect(result.current.feedback).not.toBeNull();

      act(() => {
        result.current.clearFeedback();
      });

      expect(result.current.feedback).toBeNull();
    });

    it('clearFeedback 应取消挂起的定时器', () => {
      const { result } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('success', '测试消息', 3000);
      });

      act(() => {
        result.current.clearFeedback();
      });

      // 快进 3 秒，不应有任何副作用
      act(() => {
        vi.advanceTimersByTime(3000);
      });

      expect(result.current.feedback).toBeNull();
    });
  });

  describe('组件卸载时清理', () => {
    it('卸载时应清除定时器（无内存泄漏）', () => {
      const { result, unmount } = renderHook(() => useFeedback());

      act(() => {
        result.current.showFeedback('success', '测试消息', 3000);
      });

      // 卸载组件
      unmount();

      // 快进时间，不应抛出错误
      expect(() => {
        vi.advanceTimersByTime(3000);
      }).not.toThrow();
    });
  });
});
