import { describe, expect, it } from 'vitest'

import type { IterationGraphEdgeFlowStates, IterationGraphEdgeId } from './types'
import { iterationGraphEdgeIds } from './types'
import { computeIterationGraphEdgeDenoiseLevels } from './edgeDenoise'
import { createInitialIterationGraphEdgeFlowStates } from '@/features/visualization/iterationGraphDemoReducer'

function makeStates(overrides: Partial<IterationGraphEdgeFlowStates>): IterationGraphEdgeFlowStates {
  const base = createInitialIterationGraphEdgeFlowStates()
  for (const [key, value] of Object.entries(overrides)) {
    if (!value) continue
    base[key as IterationGraphEdgeId] = value
  }
  return base
}

describe('computeIterationGraphEdgeDenoiseLevels', () => {
  it('marks a single active edge as strong', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    const promptToQuality = iterationGraphEdgeIds[1] as IterationGraphEdgeId
    const qualityToReflection = iterationGraphEdgeIds[2] as IterationGraphEdgeId
    const levels = computeIterationGraphEdgeDenoiseLevels(
      makeStates({
        [patternToPrompt]: { state: 'flowing', lastActivatedSeq: 10 },
      }),
    )

    expect(levels[patternToPrompt]).toBe('strong')
    expect(levels[promptToQuality]).toBe('off')
    expect(levels[qualityToReflection]).toBe('off')
  })

  it('limits strong edges to 2 and weakens the rest', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    const promptToQuality = iterationGraphEdgeIds[1] as IterationGraphEdgeId
    const qualityToReflection = iterationGraphEdgeIds[2] as IterationGraphEdgeId
    const levels = computeIterationGraphEdgeDenoiseLevels(
      makeStates({
        [patternToPrompt]: { state: 'flowing', lastActivatedSeq: 1 },
        [promptToQuality]: { state: 'flowing', lastActivatedSeq: 2 },
        [qualityToReflection]: { state: 'flowing', lastActivatedSeq: 3 },
      }),
    )

    expect(levels[qualityToReflection]).toBe('strong')
    expect(levels[promptToQuality]).toBe('strong')
    expect(levels[patternToPrompt]).toBe('weak')
  })

  it('breaks ties deterministically when activation sequence is equal', () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    const promptToQuality = iterationGraphEdgeIds[1] as IterationGraphEdgeId
    const qualityToReflection = iterationGraphEdgeIds[2] as IterationGraphEdgeId
    const levels = computeIterationGraphEdgeDenoiseLevels(
      makeStates({
        [patternToPrompt]: { state: 'flowing', lastActivatedSeq: 1 },
        [promptToQuality]: { state: 'flowing', lastActivatedSeq: 1 },
        [qualityToReflection]: { state: 'flowing', lastActivatedSeq: 1 },
      }),
    )

    expect(levels[patternToPrompt]).toBe('strong')
    expect(levels[promptToQuality]).toBe('strong')
    expect(levels[qualityToReflection]).toBe('weak')
  })
})
