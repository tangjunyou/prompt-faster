import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

describe('RunView', () => {
  it('renders node graph container and 4 base nodes', async () => {
    const { RunView } = await import('./RunView')
    render(<RunView />)

    expect(screen.getByTestId('run-view')).toBeInTheDocument()
    expect(screen.getByTestId('iteration-graph')).toBeInTheDocument()

    expect(screen.getByText('Pattern Extractor')).toBeInTheDocument()
    expect(screen.getByText('Prompt Engineer')).toBeInTheDocument()
    expect(screen.getByText('Quality Assessor')).toBeInTheDocument()
    expect(screen.getByText('Reflection Agent')).toBeInTheDocument()
  })
})
