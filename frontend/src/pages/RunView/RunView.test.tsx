import { render, screen, fireEvent, act } from '@testing-library/react'
import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router'

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  })

function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{ui}</MemoryRouter>
    </QueryClientProvider>
  )
}

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
    renderWithQueryClient(<RunView />)

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
    renderWithQueryClient(<RunView />)

    const replayButton = screen.getByTestId('runview-demo-replay')
    fireEvent.click(replayButton)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(200)
    })

    expect(screen.getByTestId('stage-indicator')).toHaveTextContent('规律抽取中')

    await act(async () => {
      await vi.advanceTimersByTimeAsync(400)
      await vi.runAllTimersAsync()
    })

    expect(screen.getByTestId('streaming-text-content')).toHaveTextContent(
      'iter=1 stage=reflection token=0',
    )
    expect(screen.getByTestId('stage-indicator')).toHaveTextContent('反思迭代中')
  })
})
