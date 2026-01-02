import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import App from './App'
import { useAuthStore } from '@/stores/useAuthStore'

const renderWithProviders = (initialEntry: string) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialEntry]}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>
  )
}

describe('App routes', () => {
  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'unauthenticated',
      sessionToken: null,
      currentUser: null,
      requiresRegistration: null,
    })
  })

  it('渲染 /run 路由', () => {
    renderWithProviders('/run')
    expect(screen.getByTestId('run-view')).toBeInTheDocument()
  })

  it('渲染 /focus 路由', () => {
    renderWithProviders('/focus')
    expect(screen.getByTestId('focus-view')).toBeInTheDocument()
  })

  it('渲染 /workspace 路由', () => {
    renderWithProviders('/workspace')
    expect(screen.getByTestId('workspace-view')).toBeInTheDocument()
  })

  it('默认路由应跳转到 /run', () => {
    renderWithProviders('/')
    expect(screen.getByTestId('run-view')).toBeInTheDocument()
  })
})
