import { render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { IterationGraph } from './IterationGraph'
import type { IterationGraphEdgeFlowStates, IterationGraphEdgeId } from './types'
import { iterationGraphEdgeIds } from './types'
import { createInitialIterationGraphEdgeFlowStates } from '@/features/visualization/iterationGraphDemoReducer'

function makeEdgeFlowStates(overrides: Partial<IterationGraphEdgeFlowStates>): IterationGraphEdgeFlowStates {
  const base = createInitialIterationGraphEdgeFlowStates()
  for (const [key, value] of Object.entries(overrides)) {
    if (!value) continue
    base[key as IterationGraphEdgeId] = value
  }
  return base
}

describe('IterationGraph', () => {
  it('renders 4 nodes with stable labels', () => {
    render(
      <IterationGraph
        nodeStates={{
          pattern_extractor: 'idle',
          prompt_engineer: 'idle',
          quality_assessor: 'idle',
          reflection_agent: 'idle',
        }}
      />,
    )

    expect(screen.getByTestId('iteration-graph')).toBeInTheDocument()
    expect(screen.getByText('Pattern Extractor')).toBeInTheDocument()
    expect(screen.getByText('Prompt Engineer')).toBeInTheDocument()
    expect(screen.getByText('Quality Assessor')).toBeInTheDocument()
    expect(screen.getByText('Reflection Agent')).toBeInTheDocument()
  })

  it('updates node semantic classes based on input status', () => {
    const { rerender } = render(
      <IterationGraph
        nodeStates={{
          pattern_extractor: 'idle',
          prompt_engineer: 'idle',
          quality_assessor: 'idle',
          reflection_agent: 'idle',
        }}
      />,
    )

    const patternNode = screen.getByText('Pattern Extractor')
    expect(patternNode.className).toContain('bg-slate')

    rerender(
      <IterationGraph
        nodeStates={{
          pattern_extractor: 'running',
          prompt_engineer: 'idle',
          quality_assessor: 'idle',
          reflection_agent: 'idle',
        }}
      />,
    )

    expect(screen.getByText('Pattern Extractor').className).toContain('bg-blue')
  })

  it('renders edge animation state via ReactFlow edge props', async () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    render(
      <IterationGraph
        nodeStates={{
          pattern_extractor: 'idle',
          prompt_engineer: 'idle',
          quality_assessor: 'idle',
          reflection_agent: 'idle',
        }}
        edgeFlowStates={makeEdgeFlowStates({
          [patternToPrompt]: { state: 'flowing', lastActivatedSeq: 10 },
        })}
      />,
    )

    await waitFor(() => {
      const edge = screen
        .getByTestId('xyflow-mock')
        .querySelector(`[data-edgeid="${patternToPrompt}"]`)
      expect(edge).toBeTruthy()
      expect(edge?.getAttribute('data-animated')).toBe('true')
      expect(edge?.className).toContain('iteration-graph__edge--strong')
    })
  })

  it('applies parallel denoise: only top 2 active edges can be animated', async () => {
    const patternToPrompt = iterationGraphEdgeIds[0] as IterationGraphEdgeId
    const promptToQuality = iterationGraphEdgeIds[1] as IterationGraphEdgeId
    const qualityToReflection = iterationGraphEdgeIds[2] as IterationGraphEdgeId
    render(
      <IterationGraph
        nodeStates={{
          pattern_extractor: 'idle',
          prompt_engineer: 'idle',
          quality_assessor: 'idle',
          reflection_agent: 'idle',
        }}
        edgeFlowStates={makeEdgeFlowStates({
          [patternToPrompt]: { state: 'flowing', lastActivatedSeq: 1 },
          [promptToQuality]: { state: 'flowing', lastActivatedSeq: 2 },
          [qualityToReflection]: { state: 'flowing', lastActivatedSeq: 3 },
        })}
      />,
    )

    await waitFor(() => {
      const root = screen.getByTestId('xyflow-mock')
      const strong1 = root.querySelector(`[data-edgeid="${qualityToReflection}"]`)
      const strong2 = root.querySelector(`[data-edgeid="${promptToQuality}"]`)
      const weak = root.querySelector(`[data-edgeid="${patternToPrompt}"]`)

      expect(strong1?.getAttribute('data-animated')).toBe('true')
      expect(strong2?.getAttribute('data-animated')).toBe('true')
      expect(weak?.getAttribute('data-animated')).toBe('false')
      expect(weak?.className).toContain('iteration-graph__edge--weak')
    })
  })
})
