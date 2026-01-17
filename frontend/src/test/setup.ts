import '@testing-library/jest-dom'
import { vi } from 'vitest'
import type { ReactNode } from 'react'

// Avoid Node's experimental WebStorage warning in MSW by providing a stable in-memory Storage.
class MemoryStorage implements Storage {
  private store = new Map<string, string>()

  get length(): number {
    return this.store.size
  }

  clear(): void {
    this.store.clear()
  }

  getItem(key: string): string | null {
    return this.store.has(key) ? this.store.get(key)! : null
  }

  key(index: number): string | null {
    return Array.from(this.store.keys())[index] ?? null
  }

  removeItem(key: string): void {
    this.store.delete(key)
  }

  setItem(key: string, value: string): void {
    this.store.set(key, String(value))
  }
}

globalThis.localStorage = new MemoryStorage()

// Mock WebSocket to avoid real network connections in tests.
class MockWebSocket {
  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3

  url: string
  readyState = MockWebSocket.CONNECTING
  onopen: ((event: Event) => void) | null = null
  onclose: ((event: Event) => void) | null = null
  onmessage: ((event: MessageEvent) => void) | null = null
  onerror: ((event: Event) => void) | null = null
  private listeners = new Map<string, Set<EventListener>>()

  constructor(url: string) {
    this.url = url
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN
      this.onopen?.(new Event('open'))
      this.dispatchEvent('open', new Event('open'))
    }, 0)
  }

  send(_data: string) {
    if (this.readyState !== MockWebSocket.OPEN) {
      this.onerror?.(new Event('error'))
      this.dispatchEvent('error', new Event('error'))
    }
  }

  close() {
    if (this.readyState === MockWebSocket.CLOSED) return
    this.readyState = MockWebSocket.CLOSED
    this.onclose?.(new Event('close'))
    this.dispatchEvent('close', new Event('close'))
  }

  addEventListener(type: string, listener: EventListener) {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set())
    }
    this.listeners.get(type)!.add(listener)
  }

  removeEventListener(type: string, listener: EventListener) {
    this.listeners.get(type)?.delete(listener)
  }

  dispatchEvent(type: string, event: Event) {
    this.listeners.get(type)?.forEach((listener) => listener(event))
  }
}

// @ts-expect-error - override test env WebSocket
globalThis.WebSocket = MockWebSocket

// React Flow (xyflow) relies on DOM/layout features that jsdom doesn't fully implement.
// For unit/component tests we provide a lightweight deterministic mock.
vi.mock('@xyflow/react', async () => {
  const React = await import('react')

  type MockNode = { id: string; className?: string; data?: { label?: string } }
  type MockEdge = { id: string; animated?: boolean; className?: string }
  type ReactFlowMockProps = { nodes?: MockNode[]; edges?: MockEdge[]; children?: ReactNode }

  const ReactFlow = (props: ReactFlowMockProps) => {
    const nodeEls = (props.nodes ?? []).map((n) =>
      React.createElement(
        'div',
        { key: n.id, 'data-nodeid': n.id, className: n.className },
        n.data?.label,
      ),
    )

    const edgeEls = (props.edges ?? []).map((e) =>
      React.createElement('div', {
        key: e.id,
        'data-edgeid': e.id,
        'data-animated': String(!!e.animated),
        className: e.className,
      }),
    )

    return React.createElement('div', { 'data-testid': 'xyflow-mock' }, ...nodeEls, ...edgeEls, props.children)
  }

  return {
    __esModule: true,
    ReactFlow,
    default: ReactFlow,
    Background: () => null,
    Controls: () => null,
    MarkerType: { ArrowClosed: 'ArrowClosed' },
    useNodesState: <T,>(initialNodes: T) => {
      const [nodes, setNodes] = React.useState<T>(initialNodes)
      return [nodes, setNodes, () => {}] as const
    },
    useEdgesState: <T,>(initialEdges: T) => {
      const [edges, setEdges] = React.useState<T>(initialEdges)
      return [edges, setEdges, () => {}] as const
    },
  }
})
