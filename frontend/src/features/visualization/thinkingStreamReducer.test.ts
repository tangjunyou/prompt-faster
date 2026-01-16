import { describe, it, expect, vi, beforeEach } from 'vitest'

import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'

import {
  createInitialThinkingStreamState,
  reduceThinkingStreamState,
  resetThinkingStreamState,
  forceCompleteThinkingStreamState,
  setAutoScrollLocked,
  type ThinkingStreamState,
} from './thinkingStreamReducer'

function createStreamMessage(
  seq: number,
  content: string,
  correlationId = 'test-correlation',
): DemoWsMessage {
  return {
    type: 'thinking:stream',
    correlationId,
    timestamp: new Date().toISOString(),
    payload: {
      kind: 'stream',
      seq,
      content,
    },
  }
}

function createProgressMessage(
  seq: number,
  state: 'running_tests' | 'evaluating' | 'waiting_user' | 'completed' | 'failed',
  correlationId = 'test-correlation',
  stage?: 'pattern' | 'prompt' | 'quality' | 'reflection',
): DemoWsMessage {
  return {
    type: 'iteration:progress',
    correlationId,
    timestamp: new Date().toISOString(),
    payload: {
      kind: 'progress',
      seq,
      iteration: 1,
      state,
      step: state,
      stage,
    },
  }
}

describe('thinkingStreamReducer', () => {
  describe('createInitialThinkingStreamState', () => {
    it('应返回正确的初始状态', () => {
      const state = createInitialThinkingStreamState()

      expect(state).toEqual({
        correlationId: null,
        currentStage: null,
        currentStageStartSeq: null,
        stageHistory: [],
        text: '',
        isTruncated: false,
        maxChars: 10000,
        maxLines: 500,
        status: 'idle',
        lastSeq: -1,
        isAutoScrollLocked: false,
      })
    })
  })

  describe('reduceThinkingStreamState', () => {
    let initialState: ThinkingStreamState

    beforeEach(() => {
      initialState = createInitialThinkingStreamState()
    })

    describe('thinking:stream 消息处理', () => {
      it('首条 stream 消息应将状态从 idle 转换为 streaming', () => {
        const msg = createStreamMessage(0, 'Hello')
        const newState = reduceThinkingStreamState(initialState, msg)

        expect(newState.status).toBe('streaming')
        expect(newState.text).toBe('Hello')
        expect(newState.correlationId).toBe('test-correlation')
        expect(newState.lastSeq).toBe(0)
      })

      it('后续 stream 消息应追加文本', () => {
        let state = reduceThinkingStreamState(initialState, createStreamMessage(0, 'Hello'))
        state = reduceThinkingStreamState(state, createStreamMessage(1, ' World'))

        expect(state.text).toBe('Hello World')
        expect(state.lastSeq).toBe(1)
      })

      it('应忽略重复的 seq（幂等）', () => {
        let state = reduceThinkingStreamState(initialState, createStreamMessage(0, 'Hello'))
        state = reduceThinkingStreamState(state, createStreamMessage(0, ' Duplicate'))

        expect(state.text).toBe('Hello')
        expect(state.lastSeq).toBe(0)
      })

      it('应忽略乱序的 seq（seq < lastSeq）', () => {
        let state = reduceThinkingStreamState(initialState, createStreamMessage(5, 'First'))
        state = reduceThinkingStreamState(state, createStreamMessage(3, 'OldMsg'))

        expect(state.text).toBe('First')
        expect(state.lastSeq).toBe(5)
      })

      it('seq 跳跃时应记录 warning 但继续追加', () => {
        const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

        let state = reduceThinkingStreamState(initialState, createStreamMessage(0, 'First'))
        state = reduceThinkingStreamState(state, createStreamMessage(5, 'Jumped'))

        expect(state.text).toBe('FirstJumped')
        expect(state.lastSeq).toBe(5)
        expect(warnSpy).toHaveBeenCalledWith(
          expect.stringContaining('seq 跳跃'),
        )

        warnSpy.mockRestore()
      })
    })

    describe('correlationId 隔离', () => {
      it('应丢弃不匹配 correlationId 的消息', () => {
        let state = reduceThinkingStreamState(
          initialState,
          createStreamMessage(0, 'Hello', 'correlation-A'),
        )
        state = reduceThinkingStreamState(
          state,
          createStreamMessage(1, ' World', 'correlation-B'),
        )

        expect(state.text).toBe('Hello')
        expect(state.correlationId).toBe('correlation-A')
        expect(state.lastSeq).toBe(0)
      })

      it('首条消息应设定 correlationId', () => {
        const state = reduceThinkingStreamState(
          initialState,
          createStreamMessage(0, 'Hello', 'my-correlation'),
        )

        expect(state.correlationId).toBe('my-correlation')
      })
    })

    describe('iteration:progress 消息处理', () => {
      it('completed 状态应将 status 设为 complete', () => {
        let state = reduceThinkingStreamState(initialState, createStreamMessage(0, 'Text'))
        state = reduceThinkingStreamState(state, createProgressMessage(1, 'completed'))

        expect(state.status).toBe('complete')
        expect(state.lastSeq).toBe(1)
      })

      it('failed 状态应将 status 设为 complete', () => {
        let state = reduceThinkingStreamState(initialState, createStreamMessage(0, 'Text'))
        state = reduceThinkingStreamState(state, createProgressMessage(1, 'failed'))

        expect(state.status).toBe('complete')
      })

      it('非 terminal 状态不应改变 status', () => {
        let state = reduceThinkingStreamState(initialState, createStreamMessage(0, 'Text'))
        state = reduceThinkingStreamState(state, createProgressMessage(1, 'running_tests'))

        expect(state.status).toBe('streaming')
        expect(state.lastSeq).toBe(1)
      })
    })

    describe('环节标识与历史归档', () => {
      it('应更新 currentStage 并在切换时归档历史', () => {
        let state = reduceThinkingStreamState(
          initialState,
          createProgressMessage(0, 'running_tests', 'test-correlation', 'pattern'),
        )
        state = reduceThinkingStreamState(state, createStreamMessage(1, 'Pattern output'))
        state = reduceThinkingStreamState(
          state,
          createProgressMessage(2, 'evaluating', 'test-correlation', 'prompt'),
        )

        expect(state.currentStage).toBe('prompt')
        expect(state.text).toBe('')
        expect(state.stageHistory).toHaveLength(1)
        expect(state.stageHistory[0]).toMatchObject({
          stage: 'pattern',
          summary: 'Pattern output',
          text: 'Pattern output',
          startSeq: 0,
          endSeq: 1,
        })
      })

      it('终结时应归档最后一个环节输出', () => {
        let state = reduceThinkingStreamState(
          initialState,
          createProgressMessage(0, 'running_tests', 'test-correlation', 'reflection'),
        )
        state = reduceThinkingStreamState(state, createStreamMessage(1, 'Final output'))
        state = reduceThinkingStreamState(
          state,
          createProgressMessage(2, 'completed', 'test-correlation', 'reflection'),
        )

        expect(state.stageHistory).toHaveLength(1)
        expect(state.stageHistory[0]).toMatchObject({
          stage: 'reflection',
          summary: 'Final output',
          text: 'Final output',
        })
      })

      it('环节历史应限制在 20 条以内', () => {
        const stages = ['pattern', 'prompt', 'quality', 'reflection'] as const
        let state = initialState

        for (let i = 0; i < 24; i++) {
          const stage = stages[i % stages.length]
          state = reduceThinkingStreamState(
            state,
            createProgressMessage(i * 2, 'running_tests', 'test-correlation', stage),
          )
          state = reduceThinkingStreamState(state, createStreamMessage(i * 2 + 1, `Stage ${i}`))
        }

        state = reduceThinkingStreamState(
          state,
          createProgressMessage(50, 'evaluating', 'test-correlation', 'pattern'),
        )

        expect(state.stageHistory.length).toBeLessThanOrEqual(20)
      })
    })

    describe('长文本兜底策略（截断）', () => {
      it('应按 maxChars 截断（保留末尾）', () => {
        const longText = 'A'.repeat(50)
        const state = reduceThinkingStreamState(initialState, createStreamMessage(0, longText), {
          maxChars: 30,
          maxLines: 1000,
        })

        expect(state.text.length).toBe(30)
        expect(state.text).toBe('A'.repeat(30))
        expect(state.isTruncated).toBe(true)
        expect(state.maxChars).toBe(30)
        expect(state.maxLines).toBe(1000)
      })

      it('应按 maxLines 截断（保留末尾）', () => {
        const multiLineText = Array.from({ length: 10 }, (_, i) => `Line ${i}`).join('\n')
        const state = reduceThinkingStreamState(
          initialState,
          createStreamMessage(0, multiLineText),
          { maxLines: 5, maxChars: 10000 },
        )

        const lines = state.text.split('\n')
        expect(lines.length).toBe(5)
        expect(lines[0]).toBe('Line 5')
        expect(lines[4]).toBe('Line 9')
        expect(state.isTruncated).toBe(true)
        expect(state.maxChars).toBe(10000)
        expect(state.maxLines).toBe(5)
      })

      it('优先按 maxLines 再按 maxChars 截断', () => {
        const multiLineText = Array.from({ length: 10 }, () => 'A'.repeat(20)).join('\n')
        const state = reduceThinkingStreamState(
          initialState,
          createStreamMessage(0, multiLineText),
          { maxLines: 5, maxChars: 50 },
        )

        expect(state.text.length).toBeLessThanOrEqual(50)
        expect(state.isTruncated).toBe(true)
        expect(state.maxChars).toBe(50)
        expect(state.maxLines).toBe(5)
      })
    })
  })

  describe('resetThinkingStreamState', () => {
    it('应返回初始状态', () => {
      const state = resetThinkingStreamState()
      expect(state).toEqual(createInitialThinkingStreamState())
    })
  })

  describe('forceCompleteThinkingStreamState', () => {
    it('streaming 状态应变为 complete', () => {
      const streamingState: ThinkingStreamState = {
        ...createInitialThinkingStreamState(),
        status: 'streaming',
        text: 'Some text',
      }

      const newState = forceCompleteThinkingStreamState(streamingState)
      expect(newState.status).toBe('complete')
      expect(newState.text).toBe('Some text')
    })

    it('非 streaming 状态应保持不变', () => {
      const idleState = createInitialThinkingStreamState()
      const newState = forceCompleteThinkingStreamState(idleState)

      expect(newState).toBe(idleState)
    })
  })

  describe('setAutoScrollLocked', () => {
    it('应更新 isAutoScrollLocked 状态', () => {
      const state = createInitialThinkingStreamState()

      const lockedState = setAutoScrollLocked(state, true)
      expect(lockedState.isAutoScrollLocked).toBe(true)

      const unlockedState = setAutoScrollLocked(lockedState, false)
      expect(unlockedState.isAutoScrollLocked).toBe(false)
    })

    it('相同值时应返回原对象（引用相等）', () => {
      const state = createInitialThinkingStreamState()
      const sameState = setAutoScrollLocked(state, false)

      expect(sameState).toBe(state)
    })
  })
})
