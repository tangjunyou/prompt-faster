import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'

/**
 * Thinking Panel 流式状态
 * 用于展示老师模型的思考过程（流式输出）
 */
export type ThinkingStreamStatus = 'idle' | 'streaming' | 'complete' | 'error'

export type ThinkingStreamState = {
  /** 当前关联的 correlationId（AR2 隔离边界） */
  correlationId: string | null
  /** 累积的流式文本内容 */
  text: string
  /** 当前文本是否已被截断 */
  isTruncated: boolean
  /** 当前截断阈值（最大字符数） */
  maxChars: number
  /** 当前截断阈值（最大行数） */
  maxLines: number
  /** 当前状态：idle | streaming | complete | error */
  status: ThinkingStreamStatus
  /** 最后处理的 seq（用于保序/幂等） */
  lastSeq: number
  /** 用户是否锁定了自动滚动（手动滚动上移后置为 true） */
  isAutoScrollLocked: boolean
}

/**
 * 创建初始 ThinkingStreamState
 */
export function createInitialThinkingStreamState(): ThinkingStreamState {
  return {
    correlationId: null,
    text: '',
    isTruncated: false,
    maxChars: DEFAULT_MAX_CHARS,
    maxLines: DEFAULT_MAX_LINES,
    status: 'idle',
    lastSeq: -1,
    isAutoScrollLocked: false,
  }
}

/**
 * ThinkingStream reducer 配置选项
 */
export type ThinkingStreamReducerOptions = {
  /** 最大字符数（兜底策略，默认 10000） */
  maxChars?: number
  /** 最大行数（兜底策略，默认 500） */
  maxLines?: number
}

const DEFAULT_MAX_CHARS = 10000
const DEFAULT_MAX_LINES = 500

/**
 * 应用长文本兜底策略（截断）
 * 优先级：先按 maxLines 再按 maxChars 截断（保留末尾）
 */
function applyTextTruncation(
  text: string,
  maxLines: number,
  maxChars: number,
): { text: string; isTruncated: boolean } {
  let result = text
  let isTruncated = false

  // 先按 maxLines 截断（保留末尾）
  const lines = result.split('\n')
  if (lines.length > maxLines) {
    result = lines.slice(-maxLines).join('\n')
    isTruncated = true
  }

  // 再按 maxChars 截断（保留末尾）
  if (result.length > maxChars) {
    result = result.slice(-maxChars)
    isTruncated = true
  }

  return { text: result, isTruncated }
}

/**
 * 纯函数 reducer：将 DemoWsMessage 归约到 ThinkingStreamState
 *
 * 状态机规则（写死，避免实现漂移）：
 * - idle -> streaming：收到第一条 thinking:stream
 * - streaming -> complete：收到 terminal iteration:progress（state in {completed, failed}）
 * - * -> error：收到明确错误信号（未来真实 WS 接入时补齐；本 demo 若无 error 事件则保持 N/A）
 *
 * seq 规则（最小可用且可测）：
 * - 忽略 seq <= lastSeq（重复/乱序）
 * - 若出现 seq > lastSeq + 1 视为"跳跃"，记录 warning（不中断追加）
 *
 * correlationId 隔离规则：
 * - 首条消息设定 correlationId
 * - 后续消息若 correlationId 不匹配则丢弃
 */
export function reduceThinkingStreamState(
  prev: ThinkingStreamState,
  message: DemoWsMessage,
  options: ThinkingStreamReducerOptions = {},
): ThinkingStreamState {
  const maxChars = options.maxChars ?? DEFAULT_MAX_CHARS
  const maxLines = options.maxLines ?? DEFAULT_MAX_LINES
  const payload = message.payload
  const msgCorrelationId = message.correlationId

  // correlationId 隔离检查
  if (prev.correlationId !== null && msgCorrelationId !== prev.correlationId) {
    // 不匹配当前 correlationId，丢弃消息
    return prev
  }

  // 处理 thinking:stream 消息
  if (message.type === 'thinking:stream' && payload.kind === 'stream') {
    const seq = payload.seq

    // seq 保序检查：忽略重复/乱序
    if (seq <= prev.lastSeq) {
      return prev
    }

    // seq 跳跃检查（仅 warning，不中断）
    if (seq > prev.lastSeq + 1 && prev.lastSeq >= 0) {
      console.warn(
        `[ThinkingStream] seq 跳跃: expected ${prev.lastSeq + 1}, got ${seq}`,
      )
    }

    // 更新状态：idle -> streaming 或保持 streaming
    const truncation = applyTextTruncation(
      prev.text + payload.content,
      maxLines,
      maxChars,
    )

    return {
      ...prev,
      correlationId: msgCorrelationId ?? prev.correlationId,
      text: truncation.text,
      isTruncated: truncation.isTruncated,
      maxChars,
      maxLines,
      status: 'streaming',
      lastSeq: seq,
    }
  }

  // 处理 iteration:progress 消息（terminal 状态）
  if (message.type === 'iteration:progress' && payload.kind === 'progress') {
    const seq = payload.seq

    // seq 保序检查
    if (seq <= prev.lastSeq) {
      return prev
    }

    // 设置 correlationId（若为首条消息）
    const newCorrelationId = prev.correlationId ?? msgCorrelationId ?? null

    // 检查 terminal 状态
    if (payload.state === 'completed' || payload.state === 'failed') {
      return {
        ...prev,
        correlationId: newCorrelationId,
        status: 'complete',
        lastSeq: seq,
      }
    }

    // 非 terminal progress 消息：仅更新 lastSeq 和 correlationId
    return {
      ...prev,
      correlationId: newCorrelationId,
      maxChars,
      maxLines,
      lastSeq: seq,
    }
  }

  // 其他消息类型：保持原状态
  return prev
}

/**
 * 重置 ThinkingStreamState（用于 RunView 回放开始时）
 */
export function resetThinkingStreamState(): ThinkingStreamState {
  return createInitialThinkingStreamState()
}

/**
 * 强制完成 ThinkingStreamState（兜底：回放结束时若 reducer 未置位）
 */
export function forceCompleteThinkingStreamState(
  prev: ThinkingStreamState,
): ThinkingStreamState {
  if (prev.status === 'streaming') {
    return {
      ...prev,
      status: 'complete',
    }
  }
  return prev
}

/**
 * 更新自动滚动锁定状态
 */
export function setAutoScrollLocked(
  prev: ThinkingStreamState,
  isLocked: boolean,
): ThinkingStreamState {
  if (prev.isAutoScrollLocked === isLocked) {
    return prev
  }
  return {
    ...prev,
    isAutoScrollLocked: isLocked,
  }
}
