import { useCallback, useEffect, useRef, useState } from 'react'

import type { ThinkingStreamStatus } from '@/features/visualization/thinkingStreamReducer'

export type StreamingTextProps = {
  /** 流式文本内容 */
  text: string
  /** 当前状态 */
  status: ThinkingStreamStatus
  /** 复制回调（可选） */
  onCopy?: (text: string) => void
  /** 当前文本是否已被截断（由外部处理时传入） */
  isTruncated?: boolean
  /** 自动滚动锁定状态（受控） */
  isAutoScrollLocked?: boolean
  /** 自动滚动锁定状态变化回调（受控） */
  onAutoScrollLockedChange?: (isLocked: boolean) => void
  /** 是否偏好减弱动画（系统级 prefers-reduced-motion） */
  prefersReducedMotion?: boolean
  /** 最大字符数（兜底截断，默认 10000） */
  maxChars?: number
  /** 最大行数（兜底截断，默认 500） */
  maxLines?: number
  /** 自定义 className */
  className?: string
}

const DEFAULT_MAX_CHARS = 10000
const DEFAULT_MAX_LINES = 500
const BOTTOM_THRESHOLD_PX = 100

/**
 * 应用文本截断（保留末尾）
 * 优先级：先按 maxLines 再按 maxChars 截断
 */
function truncateText(
  text: string,
  maxLines: number,
  maxChars: number,
): { text: string; isTruncated: boolean } {
  let result = text
  let isTruncated = false

  // 先按 maxLines 截断
  const lines = result.split('\n')
  if (lines.length > maxLines) {
    result = lines.slice(-maxLines).join('\n')
    isTruncated = true
  }

  // 再按 maxChars 截断
  if (result.length > maxChars) {
    result = result.slice(-maxChars)
    isTruncated = true
  }

  return { text: result, isTruncated }
}

/**
 * StreamingText 组件
 * 用于展示流式文本输出，支持自动滚动、复制、可访问性
 */
export function StreamingText({
  text,
  status,
  onCopy,
  isTruncated,
  isAutoScrollLocked: isAutoScrollLockedProp,
  onAutoScrollLockedChange,
  prefersReducedMotion = false,
  maxChars = DEFAULT_MAX_CHARS,
  maxLines = DEFAULT_MAX_LINES,
  className = '',
}: StreamingTextProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [internalAutoScrollLocked, setInternalAutoScrollLocked] = useState(false)
  const [showScrollToBottom, setShowScrollToBottom] = useState(false)
  const [copyStatus, setCopyStatus] = useState<'success' | 'error' | null>(null)
  const lastTextLengthRef = useRef(0)
  const rafIdRef = useRef<number | null>(null)

  const isAutoScrollLocked = isAutoScrollLockedProp ?? internalAutoScrollLocked
  const setAutoScrollLocked = useCallback(
    (value: boolean) => {
      if (onAutoScrollLockedChange) {
        onAutoScrollLockedChange(value)
      } else {
        setInternalAutoScrollLocked(value)
      }
    },
    [onAutoScrollLockedChange],
  )

  // 应用截断（若外部已截断则直接使用）
  const truncation = isTruncated == null ? truncateText(text, maxLines, maxChars) : null
  const displayText = truncation ? truncation.text : text
  const isTextTruncated = isTruncated ?? truncation?.isTruncated ?? false

  // 检查是否在底部
  const isAtBottom = useCallback(() => {
    const container = containerRef.current
    if (!container) return true

    const { scrollTop, scrollHeight, clientHeight } = container
    return scrollHeight - scrollTop - clientHeight <= BOTTOM_THRESHOLD_PX
  }, [])

  // 滚动到底部
  const scrollToBottom = useCallback(() => {
    const container = containerRef.current
    if (!container) return

    if (prefersReducedMotion || typeof container.scrollTo !== 'function') {
      // 减弱动画模式或 jsdom 环境：直接跳转
      container.scrollTop = container.scrollHeight
    } else {
      // 平滑滚动
      container.scrollTo({
        top: container.scrollHeight,
        behavior: 'smooth',
      })
    }
    setAutoScrollLocked(false)
    setShowScrollToBottom(false)
  }, [prefersReducedMotion, setAutoScrollLocked])

  // 处理用户滚动
  const handleScroll = useCallback(() => {
    const atBottom = isAtBottom()

    if (!atBottom && !isAutoScrollLocked) {
      // 用户滚动离开底部，锁定自动滚动
      setAutoScrollLocked(true)
      setShowScrollToBottom(true)
    } else if (atBottom && isAutoScrollLocked) {
      // 用户滚回底部，解锁
      setAutoScrollLocked(false)
      setShowScrollToBottom(false)
    }
  }, [isAtBottom, isAutoScrollLocked, setAutoScrollLocked])

  const isScrollButtonVisible = showScrollToBottom || isAutoScrollLocked

  // 新内容到达时自动滚动（使用 rAF 批量处理）
  useEffect(() => {
    if (text.length === lastTextLengthRef.current) return
    lastTextLengthRef.current = text.length

    // 如果锁定了自动滚动，只显示提示
    if (isAutoScrollLocked) {
      return
    }

    // 使用 requestAnimationFrame 批量处理，避免每次 text 变化都重排
    if (rafIdRef.current != null) {
      cancelAnimationFrame(rafIdRef.current)
    }
    rafIdRef.current = requestAnimationFrame(() => {
      scrollToBottom()
    })
    return () => {
      if (rafIdRef.current != null) {
        cancelAnimationFrame(rafIdRef.current)
        rafIdRef.current = null
      }
    }
  }, [text, isAutoScrollLocked, scrollToBottom])

  // 复制处理
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(displayText)
      onCopy?.(displayText)
      setCopyStatus('success')
    } catch (err) {
      console.error('复制失败:', err)
      setCopyStatus('error')
    }
    window.setTimeout(() => {
      setCopyStatus(null)
    }, 2000)
  }, [displayText, onCopy])

  // 状态指示器样式
  const statusIndicator = {
    idle: { text: '等待中', className: 'bg-gray-400' },
    streaming: { text: '生成中', className: 'bg-green-500 animate-pulse' },
    complete: { text: '完成', className: 'bg-blue-500' },
    error: { text: '错误', className: 'bg-red-500' },
  }[status]

  // 减弱动画模式下禁用 pulse 动画
  const indicatorClassName = prefersReducedMotion
    ? statusIndicator.className.replace('animate-pulse', '')
    : statusIndicator.className

  return (
    <div
      className={`relative flex flex-col rounded-lg border bg-slate-900 text-slate-100 ${className}`}
      data-testid="streaming-text"
    >
      {/* 头部：状态指示器 + 复制按钮 */}
      <div className="flex items-center justify-between border-b border-slate-700 px-3 py-2">
        <div className="flex items-center gap-2 text-xs">
          <span
            className={`h-2 w-2 rounded-full ${indicatorClassName}`}
            aria-hidden="true"
          />
          <span className="text-slate-400">{statusIndicator.text}</span>
        </div>

        {copyStatus && (
          <span
            className={`text-xs ${copyStatus === 'success' ? 'text-green-400' : 'text-red-400'}`}
            aria-live="polite"
          >
            {copyStatus === 'success' ? '已复制' : '复制失败'}
          </span>
        )}

        <button
          type="button"
          onClick={handleCopy}
          className="rounded px-2 py-1 text-xs text-slate-400 hover:bg-slate-700 hover:text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-slate-900"
          aria-label="复制思考内容"
          data-testid="streaming-text-copy"
        >
          复制
        </button>
      </div>

      {/* 截断提示 */}
      {isTextTruncated && (
        <div className="border-b border-slate-700 bg-slate-800 px-3 py-1 text-xs text-slate-500">
          ⚠️ 较早内容已省略（保留最近 {maxLines} 行 / {maxChars} 字符）
        </div>
      )}

      {/* 文本内容区域 */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 overflow-auto p-3 font-mono text-sm leading-relaxed whitespace-pre-wrap"
        aria-live="polite"
        aria-atomic="false"
        role="log"
        tabIndex={0}
        data-testid="streaming-text-content"
      >
        {displayText || (
          <span className="text-slate-500 italic">等待思考内容...</span>
        )}

        {/* 流式光标（仅在 streaming 状态且非减弱动画模式下显示） */}
        {status === 'streaming' && !prefersReducedMotion && (
          <span
            className="ml-0.5 inline-block h-4 w-1.5 animate-pulse bg-slate-400"
            aria-hidden="true"
          />
        )}
      </div>

      {/* 回到底部按钮 */}
      {isScrollButtonVisible && (
        <button
          type="button"
          onClick={scrollToBottom}
          className="absolute right-3 bottom-3 rounded-full bg-blue-600 px-3 py-1 text-xs text-white shadow-lg hover:bg-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
          aria-label="回到底部查看最新内容"
          data-testid="streaming-text-scroll-bottom"
        >
          ↓ 回到底部
        </button>
      )}
    </div>
  )
}
