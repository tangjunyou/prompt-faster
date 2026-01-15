import type { DemoWsMessage } from '@/features/ws-demo/demoWsMessages'
import type { IterationGraphNodeStates } from '@/components/nodes/types'

export function createInitialIterationGraphNodeStates(): IterationGraphNodeStates {
  return {
    pattern_extractor: 'idle',
    prompt_engineer: 'idle',
    quality_assessor: 'idle',
    reflection_agent: 'idle',
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
