/**
 * 任务状态管理 Store
 * 管理优化任务的运行控制状态（暂停/继续）
 */

import { create } from 'zustand'
import type { RunControlState } from '@/types/generated/models/RunControlState'

/** 任务运行状态 */
export interface TaskRunState {
  /** 任务 ID */
  taskId: string
  /** 运行控制状态 */
  runControlState: RunControlState
  /** 暂停时间（ISO 8601） */
  pausedAt?: string
  /** 暂停时所处的阶段 */
  pausedStage?: string
  /** 当前迭代轮次 */
  iteration?: number
}

/** 任务 Store 状态 */
interface TaskState {
  /** 当前活跃任务的运行状态（按 taskId 索引） */
  taskStates: Record<string, TaskRunState>
  /** 最后一次操作的 correlationId */
  lastCorrelationId: string | null
}

/** 任务 Store Actions */
interface TaskActions {
  /** 设置任务运行控制状态 */
  setRunControlState: (taskId: string, state: RunControlState) => void
  /** 处理暂停事件 */
  handlePaused: (
    taskId: string,
    pausedAt: string,
    stage: string,
    iteration: number
  ) => void
  /** 处理继续事件 */
  handleResumed: (taskId: string) => void
  /** 设置 correlationId */
  setCorrelationId: (correlationId: string) => void
  /** 获取任务状态 */
  getTaskState: (taskId: string) => TaskRunState | undefined
  /** 检查是否可以暂停 */
  canPause: (taskId: string) => boolean
  /** 检查是否可以继续 */
  canResume: (taskId: string) => boolean
  /** 清除任务状态 */
  clearTaskState: (taskId: string) => void
  /** 重置所有状态 */
  reset: () => void
}

/** 初始状态 */
const initialState: TaskState = {
  taskStates: {},
  lastCorrelationId: null,
}

/**
 * 任务状态 Store
 *
 * 使用示例:
 * ```tsx
 * const { taskStates, setRunControlState, canPause, canResume } = useTaskStore()
 * ```
 */
export const useTaskStore = create<TaskState & TaskActions>((set, get) => ({
  ...initialState,

  setRunControlState: (taskId, runControlState) =>
    set((state) => ({
      taskStates: {
        ...state.taskStates,
        [taskId]: {
          ...state.taskStates[taskId],
          taskId,
          runControlState,
        },
      },
    })),

  handlePaused: (taskId, pausedAt, stage, iteration) =>
    set((state) => ({
      taskStates: {
        ...state.taskStates,
        [taskId]: {
          taskId,
          runControlState: 'paused',
          pausedAt,
          pausedStage: stage,
          iteration,
        },
      },
    })),

  handleResumed: (taskId) =>
    set((state) => ({
      taskStates: {
        ...state.taskStates,
        [taskId]: {
          taskId,
          runControlState: 'running',
          pausedAt: undefined,
          pausedStage: undefined,
        },
      },
    })),

  setCorrelationId: (correlationId) =>
    set({ lastCorrelationId: correlationId }),

  getTaskState: (taskId) => get().taskStates[taskId],

  canPause: (taskId) => {
    const taskState = get().taskStates[taskId]
    return taskState?.runControlState === 'running'
  },

  canResume: (taskId) => {
    const taskState = get().taskStates[taskId]
    return taskState?.runControlState === 'paused'
  },

  clearTaskState: (taskId) =>
    set((state) => {
      const { [taskId]: _, ...rest } = state.taskStates
      return { taskStates: rest }
    }),

  reset: () => set(initialState),
}))

/**
 * 生成唯一的 correlationId
 */
export function generateCorrelationId(): string {
  return `cid-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
}
