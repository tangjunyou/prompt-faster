import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'

import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'
import type { IterationGraphEdgeFlowStates, IterationGraphEdgeId } from '@/components/nodes/types'
import type { IterationState } from '@/types/generated/models'
import { iterationGraphEdgeIds } from '@/components/nodes/types'

import { createIterationGraphEdgeFlowMachine } from './iterationGraphEdgeFlowMachine'

function progressMessage(seq: number, state: IterationState): DemoWsMessage {
  return {
    type: 'iteration:progress',
    correlationId: 'cid-test',
    timestamp: '2026-01-15T00:00:00.000Z',
    payload: {
      kind: 'progress',
      seq,
      iteration: 1,
      state,
      step: 'step',
    },
  }
}

function streamMessage(seq: number): DemoWsMessage {
  return {
    type: 'thinking:stream',
    correlationId: 'cid-test',
    timestamp: '2026-01-15T00:00:00.000Z',
    payload: {
      kind: 'stream',
      seq,
      content: 'token',
    },
  }
}

describe('IterationGraphEdgeFlowMachine', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('transitions flowing -> cooldown -> idle for pulse activations', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    let last: IterationGraphEdgeFlowStates | null = null
    const machine = createIterationGraphEdgeFlowMachine(
      (next) => {
        last = next
      },
      { flowingMs: 10, cooldownMs: 10 },
    )

    machine.applyDemoWsMessage(progressMessage(1, 'running_tests'))
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('flowing')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('cooldown')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('idle')

    machine.dispose()
  })

  it('endAll moves flowing edges into cooldown immediately', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    let last: IterationGraphEdgeFlowStates | null = null
    const machine = createIterationGraphEdgeFlowMachine(
      (next) => {
        last = next
      },
      { flowingMs: 100, cooldownMs: 10 },
    )

    machine.applyDemoWsMessage(progressMessage(1, 'running_tests'))
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('flowing')

    machine.applyDemoWsMessage(progressMessage(2, 'waiting_user'))
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('cooldown')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('idle')

    machine.dispose()
  })

  it('keeps parallel edge flows independent', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    const promptToQuality = iterationGraphEdgeIds[1] as IterationGraphEdgeId
    let last: IterationGraphEdgeFlowStates | null = null
    const machine = createIterationGraphEdgeFlowMachine(
      (next) => {
        last = next
      },
      { flowingMs: 10, cooldownMs: 10 },
    )

    machine.applyDemoWsMessage(progressMessage(1, 'evaluating'))
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('flowing')
    expect(last![promptToQuality].state).toBe('flowing')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('cooldown')
    expect(last![promptToQuality].state).toBe('cooldown')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('idle')
    expect(last![promptToQuality].state).toBe('idle')

    machine.dispose()
  })

  it('extends stream flow until streamGraceMs elapses after last stream message', () => {
    const qualityToReflection = iterationGraphEdgeIds[2] as IterationGraphEdgeId
    let last: IterationGraphEdgeFlowStates | null = null
    const machine = createIterationGraphEdgeFlowMachine(
      (next) => {
        last = next
      },
      { streamGraceMs: 10, cooldownMs: 10 },
    )

    machine.applyDemoWsMessage(streamMessage(1))
    expect(last).not.toBeNull()
    expect(last![qualityToReflection].state).toBe('flowing')
    expect(last![qualityToReflection].lastActivatedSeq).toBe(1)

    vi.advanceTimersByTime(5)
    machine.applyDemoWsMessage(streamMessage(2))
    expect(last).not.toBeNull()
    expect(last![qualityToReflection].state).toBe('flowing')
    expect(last![qualityToReflection].lastActivatedSeq).toBe(2)

    vi.advanceTimersByTime(9)
    expect(last).not.toBeNull()
    expect(last![qualityToReflection].state).toBe('flowing')

    vi.advanceTimersByTime(1)
    expect(last).not.toBeNull()
    expect(last![qualityToReflection].state).toBe('cooldown')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![qualityToReflection].state).toBe('idle')

    machine.dispose()
  })

  it('uses pulse mode when prefersReducedMotion is true', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    let last: IterationGraphEdgeFlowStates | null = null
    const machine = createIterationGraphEdgeFlowMachine(
      (next) => {
        last = next
      },
      { pulseMs: 10 },
    )

    machine.applyDemoWsMessage(progressMessage(1, 'running_tests'), { prefersReducedMotion: true })
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('cooldown')

    vi.advanceTimersByTime(10)
    expect(last).not.toBeNull()
    expect(last![patternToPrompt].state).toBe('idle')

    machine.dispose()
  })
})
