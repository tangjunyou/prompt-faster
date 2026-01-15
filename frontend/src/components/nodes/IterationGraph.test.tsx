import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { IterationGraph } from './IterationGraph'

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
})
