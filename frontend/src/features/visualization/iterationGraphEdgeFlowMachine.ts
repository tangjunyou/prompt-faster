import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'
import type { IterationGraphEdgeFlowStates, IterationGraphEdgeId } from '@/components/nodes/types'
import { iterationGraphEdgeIds } from '@/components/nodes/types'

import {
  createInitialIterationGraphEdgeFlowStates,
  mapDemoWsMessageToIterationGraphEdgeFlowSignal,
} from './iterationGraphDemoReducer'

export type IterationGraphEdgeFlowMachineOptions = {
  flowingMs?: number
  cooldownMs?: number
  pulseMs?: number
  streamGraceMs?: number
}

type EdgeTimers = Record<
  IterationGraphEdgeId,
  {
    flowTimeoutId: number | null
    cooldownTimeoutId: number | null
  }
>

export type IterationGraphEdgeFlowMachine = {
  getState: () => IterationGraphEdgeFlowStates
  reset: () => void
  dispose: () => void
  applyDemoWsMessage: (message: DemoWsMessage, opts?: { prefersReducedMotion?: boolean }) => void
}

export function createIterationGraphEdgeFlowMachine(
  onChange: (next: IterationGraphEdgeFlowStates) => void,
  options: IterationGraphEdgeFlowMachineOptions = {},
): IterationGraphEdgeFlowMachine {
  const [patternToPrompt, promptToQuality, qualityToReflection] = iterationGraphEdgeIds
  const flowingMs = options.flowingMs ?? 320
  const cooldownMs = options.cooldownMs ?? 280
  const pulseMs = options.pulseMs ?? 220
  const streamGraceMs = options.streamGraceMs ?? 220

  let state: IterationGraphEdgeFlowStates = createInitialIterationGraphEdgeFlowStates()

  const timers: EdgeTimers = {
    [patternToPrompt]: { flowTimeoutId: null, cooldownTimeoutId: null },
    [promptToQuality]: { flowTimeoutId: null, cooldownTimeoutId: null },
    [qualityToReflection]: { flowTimeoutId: null, cooldownTimeoutId: null },
  }

  function emitIfChanged(next: IterationGraphEdgeFlowStates) {
    // Record identity changes are fine here; avoid needless re-renders on no-op.
    const unchanged = iterationGraphEdgeIds.every(
      (edgeId) =>
        next[edgeId].state === state[edgeId].state &&
        next[edgeId].lastActivatedSeq === state[edgeId].lastActivatedSeq,
    )

    if (unchanged) return
    state = next
    onChange(next)
  }

  function clearFlowTimer(edgeId: IterationGraphEdgeId) {
    const t = timers[edgeId]
    if (t.flowTimeoutId != null) {
      window.clearTimeout(t.flowTimeoutId)
      t.flowTimeoutId = null
    }
  }

  function clearCooldownTimer(edgeId: IterationGraphEdgeId) {
    const t = timers[edgeId]
    if (t.cooldownTimeoutId != null) {
      window.clearTimeout(t.cooldownTimeoutId)
      t.cooldownTimeoutId = null
    }
  }

  function clearAllTimers(edgeId: IterationGraphEdgeId) {
    clearFlowTimer(edgeId)
    clearCooldownTimer(edgeId)
  }

  function scheduleIdle(edgeId: IterationGraphEdgeId, delayMs: number) {
    clearCooldownTimer(edgeId)
    timers[edgeId].cooldownTimeoutId = window.setTimeout(() => {
      emitIfChanged({
        ...state,
        [edgeId]: { ...state[edgeId], state: 'idle' },
      })
      timers[edgeId].cooldownTimeoutId = null
    }, delayMs)
  }

  function transitionToCooldown(edgeId: IterationGraphEdgeId) {
    clearFlowTimer(edgeId)
    emitIfChanged({
      ...state,
      [edgeId]: { ...state[edgeId], state: 'cooldown' },
    })
    scheduleIdle(edgeId, cooldownMs)
  }

  function activateEdge(edgeId: IterationGraphEdgeId, seq: number, activation: 'pulse' | 'stream', prefersReducedMotion: boolean) {
    clearCooldownTimer(edgeId)

    if (prefersReducedMotion) {
      clearFlowTimer(edgeId)
      emitIfChanged({
        ...state,
        [edgeId]: { state: 'cooldown', lastActivatedSeq: seq },
      })
      scheduleIdle(edgeId, pulseMs)
      return
    }

    const prevEdge = state[edgeId]
    const nextEdge =
      prevEdge.state === 'flowing'
        ? prevEdge.lastActivatedSeq === seq
          ? prevEdge
          : { ...prevEdge, lastActivatedSeq: seq }
        : { state: 'flowing', lastActivatedSeq: seq }

    const nextState: IterationGraphEdgeFlowStates = nextEdge === prevEdge ? state : { ...state, [edgeId]: nextEdge }

    emitIfChanged(nextState)

    clearFlowTimer(edgeId)
    const endDelayMs = activation === 'stream' ? streamGraceMs : flowingMs
    timers[edgeId].flowTimeoutId = window.setTimeout(() => {
      transitionToCooldown(edgeId)
      timers[edgeId].flowTimeoutId = null
    }, endDelayMs)
  }

  function endAll() {
    for (const edgeId of Object.keys(timers) as IterationGraphEdgeId[]) {
      clearFlowTimer(edgeId)
      if (state[edgeId].state === 'flowing') {
        transitionToCooldown(edgeId)
      } else if (state[edgeId].state === 'cooldown') {
        scheduleIdle(edgeId, cooldownMs)
      }
    }
  }

  function reset() {
    for (const edgeId of Object.keys(timers) as IterationGraphEdgeId[]) {
      clearAllTimers(edgeId)
    }
    state = createInitialIterationGraphEdgeFlowStates()
    onChange(state)
  }

  function dispose() {
    for (const edgeId of Object.keys(timers) as IterationGraphEdgeId[]) {
      clearAllTimers(edgeId)
    }
  }

  function applyDemoWsMessage(message: DemoWsMessage, opts?: { prefersReducedMotion?: boolean }) {
    const prefersReducedMotion = opts?.prefersReducedMotion ?? false
    const signal = mapDemoWsMessageToIterationGraphEdgeFlowSignal(message)

    if (signal.kind === 'none') return
    if (signal.kind === 'endAll') {
      endAll()
      return
    }
    for (const edgeId of signal.edgeIds) {
      activateEdge(edgeId, signal.seq, signal.activation, prefersReducedMotion)
    }
  }

  return {
    getState: () => state,
    reset,
    dispose,
    applyDemoWsMessage,
  }
}
