import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { OfflineBanner } from './OfflineBanner'
import type { ConnectivityResponse } from '@/types/generated/models/ConnectivityResponse'

const API_BASE = 'http://localhost:3000/api/v1'

let connectivityResponse: ConnectivityResponse = {
  status: 'online',
  lastCheckedAt: '2026-01-01T00:00:00Z',
  message: null,
  availableFeatures: [],
  restrictedFeatures: [],
}

const server = setupServer(
  http.get(`${API_BASE}/connectivity`, () => {
    return HttpResponse.json({ data: connectivityResponse })
  })
)

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
  )
}

describe('OfflineBanner', () => {
  beforeAll(() => server.listen())
  afterEach(() => server.resetHandlers())
  afterAll(() => server.close())

  it('显示离线状态提示', async () => {
    Object.defineProperty(window.navigator, 'onLine', {
      configurable: true,
      value: true,
    })

    connectivityResponse = {
      status: 'offline',
      lastCheckedAt: '2026-01-01T00:00:00Z',
      message: '网络断开',
      availableFeatures: ['view_history'],
      restrictedFeatures: ['run_optimization'],
    }

    renderWithProviders(<OfflineBanner />)

    expect(await screen.findByText('当前离线')).toBeInTheDocument()
    expect(screen.getByText('网络断开')).toBeInTheDocument()
    expect(screen.getByText('受限功能：run_optimization')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '重试检测' })).toBeInTheDocument()
  })

  it('显示网络不稳定状态提示', async () => {
    Object.defineProperty(window.navigator, 'onLine', {
      configurable: true,
      value: true,
    })

    connectivityResponse = {
      status: 'limited',
      lastCheckedAt: '2026-01-01T00:00:00Z',
      message: '部分服务不可用',
      availableFeatures: ['view_history'],
      restrictedFeatures: ['api_connection_test'],
    }

    renderWithProviders(<OfflineBanner />)

    expect(await screen.findByText('网络不稳定')).toBeInTheDocument()
    expect(screen.getByText('部分服务不可用')).toBeInTheDocument()
    expect(screen.getByText('受限功能：api_connection_test')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '重试检测' })).toBeInTheDocument()
  })
})
