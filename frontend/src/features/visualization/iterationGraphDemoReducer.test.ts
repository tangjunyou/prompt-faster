import { describe, expect, it } from 'vitest'

import type { IterationState } from '@/types/generated/models'
import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'

import {
  createInitialIterationGraphNodeStates,
  reduceDemoWsMessageToIterationGraphNodeStates,
} from './iterationGraphDemoReducer'

function progressMessage(state: IterationState): DemoWsMessage {
  return {
    type: 'iteration:progress',
    correlationId: 'cid-test',
    timestamp: '2026-01-15T00:00:00.000Z',
    payload: {
      kind: 'progress',
      seq: 0,
      iteration: 1,
      state,
      step: 'step',
    },
  }
}

function streamMessage(): DemoWsMessage {
  return {
    type: 'thinking:stream',
    correlationId: 'cid-test',
    timestamp: '2026-01-15T00:00:00.000Z',
    payload: {
      kind: 'stream',
      seq: 1,
      content: 'token',
    },
  }
}

describe('iterationGraphDemoReducer', () => {
  it('initializes all nodes as idle', () => {
    expect(createInitialIterationGraphNodeStates()).toEqual({
      pattern_extractor: 'idle',
      prompt_engineer: 'idle',
      quality_assessor: 'idle',
      reflection_agent: 'idle',
    })
  })

  it('maps running_tests -> pattern_extractor running', () => {
    const next = reduceDemoWsMessageToIterationGraphNodeStates(
      createInitialIterationGraphNodeStates(),
      progressMessage('running_tests'),
    )
    expect(next).toEqual({
      pattern_extractor: 'running',
      prompt_engineer: 'idle',
      quality_assessor: 'idle',
      reflection_agent: 'idle',
    })
  })

  it('maps evaluating -> prompt_engineer/quality_assessor running', () => {
    const next = reduceDemoWsMessageToIterationGraphNodeStates(
      createInitialIterationGraphNodeStates(),
      progressMessage('evaluating'),
    )
    expect(next.pattern_extractor).toBe('success')
    expect(next.prompt_engineer).toBe('running')
    expect(next.quality_assessor).toBe('running')
    expect(next.reflection_agent).toBe('idle')
  })

  it('maps waiting_user -> reflection_agent paused', () => {
    const next = reduceDemoWsMessageToIterationGraphNodeStates(
      createInitialIterationGraphNodeStates(),
      progressMessage('waiting_user'),
    )
    expect(next.reflection_agent).toBe('paused')
  })

  it('maps failed -> reflection_agent error', () => {
    const next = reduceDemoWsMessageToIterationGraphNodeStates(
      createInitialIterationGraphNodeStates(),
      progressMessage('failed'),
    )
    expect(next.reflection_agent).toBe('error')
  })

  it('maps completed -> all success', () => {
    const next = reduceDemoWsMessageToIterationGraphNodeStates(
      createInitialIterationGraphNodeStates(),
      progressMessage('completed'),
    )
    expect(next).toEqual({
      pattern_extractor: 'success',
      prompt_engineer: 'success',
      quality_assessor: 'success',
      reflection_agent: 'success',
    })
  })

  it('maps stream -> reflection_agent running', () => {
    const next = reduceDemoWsMessageToIterationGraphNodeStates(
      createInitialIterationGraphNodeStates(),
      streamMessage(),
    )
    expect(next.reflection_agent).toBe('running')
  })
})

