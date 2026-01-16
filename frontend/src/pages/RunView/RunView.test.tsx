import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest'

describe('RunView', () => {
  beforeEach(() => {
    vi.spyOn(window, 'requestAnimationFrame').mockImplementation((cb) => {
      return window.setTimeout(() => cb(performance.now()), 0)
    })
    vi.spyOn(window, 'cancelAnimationFrame').mockImplementation((id) => {
      window.clearTimeout(id)
    })
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

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

  it('回放后 Thinking Panel 内容应追加', async () => {
    vi.useFakeTimers()
    const { RunView } = await import('./RunView')
    render(<RunView />)

    const replayButton = screen.getByTestId('runview-demo-replay')
    fireEvent.click(replayButton)

    await vi.advanceTimersByTimeAsync(600)
    await vi.runAllTimersAsync()

    expect(screen.getByTestId('streaming-text-content')).toHaveTextContent('iter=1 token=0')
  })
})
