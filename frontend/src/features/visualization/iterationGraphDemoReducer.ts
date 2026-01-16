import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'
import type { IterationGraphEdgeFlowStates, IterationGraphEdgeId, IterationGraphNodeStates } from '@/components/nodes/types'
import { iterationGraphEdgeIds } from '@/components/nodes/types'

export function createInitialIterationGraphNodeStates(): IterationGraphNodeStates {
  return {
    pattern_extractor: 'idle',
    prompt_engineer: 'idle',
    quality_assessor: 'idle',
    reflection_agent: 'idle',
  }
}

export function createInitialIterationGraphEdgeFlowStates(): IterationGraphEdgeFlowStates {
  const [patternToPrompt, promptToQuality, qualityToReflection] = iterationGraphEdgeIds
  return {
    [patternToPrompt]: { state: 'idle', lastActivatedSeq: -1 },
    [promptToQuality]: { state: 'idle', lastActivatedSeq: -1 },
    [qualityToReflection]: { state: 'idle', lastActivatedSeq: -1 },
  }
}

export function reduceDemoWsMessageToIterationGraphNodeStates(
  prev: IterationGraphNodeStates,
  message: DemoWsMessage,
): IterationGraphNodeStates {
  const payload = message.payload

  if (payload.kind === 'progress') {
    if (payload.state === 'running_tests') {
      return {
        ...prev,
        pattern_extractor: 'running',
        prompt_engineer: 'idle',
        quality_assessor: 'idle',
        reflection_agent: 'idle',
      }
    }

    if (payload.state === 'evaluating') {
      return {
        ...prev,
        pattern_extractor: 'success',
        prompt_engineer: 'running',
        quality_assessor: 'running',
        reflection_agent: 'idle',
      }
    }

    if (payload.state === 'waiting_user' || payload.state === 'human_intervention') {
      return {
        ...prev,
        pattern_extractor: 'success',
        prompt_engineer: 'success',
        quality_assessor: 'success',
        reflection_agent: 'paused',
      }
    }

    if (payload.state === 'failed') {
      return {
        ...prev,
        pattern_extractor: 'success',
        prompt_engineer: 'success',
        quality_assessor: 'success',
        reflection_agent: 'error',
      }
    }

    if (payload.state === 'completed') {
      return {
        ...prev,
        pattern_extractor: 'success',
        prompt_engineer: 'success',
        quality_assessor: 'success',
        reflection_agent: 'success',
      }
    }
  }

  if (payload.kind === 'stream') {
    return { ...prev, reflection_agent: 'running' }
  }

  return prev
}

export type IterationGraphEdgeFlowSignal =
  | { kind: 'activate'; edgeIds: IterationGraphEdgeId[]; activation: 'pulse' | 'stream'; seq: number }
  | { kind: 'endAll'; seq: number }
  | { kind: 'none'; seq: number }

export function mapDemoWsMessageToIterationGraphEdgeFlowSignal(
  message: DemoWsMessage,
): IterationGraphEdgeFlowSignal {
  const payload = message.payload
  const [patternToPrompt, promptToQuality, qualityToReflection] = iterationGraphEdgeIds

  if (payload.kind === 'progress') {
    if (payload.state === 'running_tests') {
      return { kind: 'activate', edgeIds: [patternToPrompt], activation: 'pulse', seq: payload.seq }
    }

    if (payload.state === 'evaluating') {
      return {
        kind: 'activate',
        edgeIds: [promptToQuality, patternToPrompt],
        activation: 'pulse',
        seq: payload.seq,
      }
    }

    if (
      payload.state === 'waiting_user' ||
      payload.state === 'human_intervention' ||
      payload.state === 'completed' ||
      payload.state === 'failed'
    ) {
      return { kind: 'endAll', seq: payload.seq }
    }
  }

  if (payload.kind === 'stream') {
    return { kind: 'activate', edgeIds: [qualityToReflection], activation: 'stream', seq: payload.seq }
  }

  return { kind: 'none', seq: payload.seq }
}
