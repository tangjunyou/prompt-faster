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

// React Flow (xyflow) relies on DOM/layout features that jsdom doesn't fully implement.
// For unit/component tests we provide a lightweight deterministic mock.
vi.mock('@xyflow/react', async () => {
  const React = await import('react')

  type MockNode = { id: string; className?: string; data?: { label?: string } }
  type ReactFlowMockProps = { nodes?: MockNode[]; children?: ReactNode }

  const ReactFlow = (props: ReactFlowMockProps) => {
    const nodeEls = (props.nodes ?? []).map((n) =>
      React.createElement(
        'div',
        { key: n.id, 'data-nodeid': n.id, className: n.className },
        n.data?.label,
      ),
    )

    return React.createElement('div', { 'data-testid': 'xyflow-mock' }, ...nodeEls, props.children)
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
