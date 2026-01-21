import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PromptComparePanel } from './PromptComparePanel'

const defaultVersions = [
  { id: 'v1', version: 1, description: 'v1', isActive: true, successRate: 0.5, taskCount: 1, createdAt: '2026-01-01T00:00:00Z' },
  { id: 'v2', version: 2, description: 'v2', isActive: false, successRate: 0.6, taskCount: 1, createdAt: '2026-01-02T00:00:00Z' },
]

const defaultTasks = [
  { id: 'task-1', workspaceId: 'ws-1', name: '任务 1', status: 'completed', passRate: null, createdAt: '2026-01-01T00:00:00Z' },
]

const mutate = vi.fn()
const reset = vi.fn()

let versionsState = {
  data: defaultVersions,
  isLoading: false,
  error: null as Error | null,
}

let tasksState = {
  data: defaultTasks,
  isLoading: false,
  error: null as Error | null,
}

let compareState = {
  mutate,
  reset,
  isPending: false,
  data: null as unknown,
  error: null as Error | null,
}

vi.mock('../hooks/usePromptVersions', () => ({
  usePromptVersions: () => versionsState,
}))

vi.mock('../hooks/useMetaOptimizationTasks', () => ({
  useMetaOptimizationTasks: () => tasksState,
}))

vi.mock('../hooks/usePromptCompare', () => ({
  usePromptCompare: () => compareState,
}))

describe('PromptComparePanel', () => {
  beforeEach(() => {
    mutate.mockClear()
    reset.mockClear()
    versionsState = {
      data: defaultVersions,
      isLoading: false,
      error: null,
    }
    tasksState = {
      data: defaultTasks,
      isLoading: false,
      error: null,
    }
    compareState = {
      mutate,
      reset,
      isPending: false,
      data: null,
      error: null,
    }
  })

  it('selects versions and triggers compare', () => {
    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <PromptComparePanel />
      </QueryClientProvider>
    )

    const selects = screen.getAllByRole('combobox')
    fireEvent.change(selects[0], { target: { value: 'v1' } })
    fireEvent.change(selects[1], { target: { value: 'v2' } })

    const taskLabel = screen.getByText('任务 1').closest('label')
    expect(taskLabel).toBeTruthy()
    const checkbox = taskLabel?.querySelector('input[type="checkbox"]') as HTMLInputElement
    fireEvent.click(checkbox)
    fireEvent.click(screen.getByRole('button', { name: '开始对比' }))

    expect(mutate).toHaveBeenCalledWith({
      versionIdA: 'v1',
      versionIdB: 'v2',
      taskIds: ['task-1'],
      testCaseIds: [],
    })
  })

  it('shows workspace error when selecting tasks across workspaces', () => {
    tasksState = {
      ...tasksState,
      data: [
        { id: 'task-1', workspaceId: 'ws-1', name: '任务 1', status: 'completed', passRate: null, createdAt: '2026-01-01T00:00:00Z' },
        { id: 'task-2', workspaceId: 'ws-2', name: '任务 2', status: 'completed', passRate: null, createdAt: '2026-01-02T00:00:00Z' },
      ],
    }

    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <PromptComparePanel />
      </QueryClientProvider>
    )

    const taskLabelA = screen.getByText('任务 1').closest('label')
    const taskLabelB = screen.getByText('任务 2').closest('label')
    const checkboxA = taskLabelA?.querySelector('input[type="checkbox"]') as HTMLInputElement
    const checkboxB = taskLabelB?.querySelector('input[type="checkbox"]') as HTMLInputElement
    fireEvent.click(checkboxA)
    fireEvent.click(checkboxB)

    expect(screen.getByText('请只选择同一工作区内的历史任务')).toBeInTheDocument()
  })

  it('shows versions error message', () => {
    versionsState = {
      ...versionsState,
      data: [],
      error: new Error('boom'),
    }

    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <PromptComparePanel />
      </QueryClientProvider>
    )

    expect(screen.getByText('加载版本失败：boom')).toBeInTheDocument()
  })

  it('shows compare error message', () => {
    compareState = {
      ...compareState,
      error: new Error('boom'),
    }

    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <PromptComparePanel />
      </QueryClientProvider>
    )

    expect(screen.getByText('对比失败：boom')).toBeInTheDocument()
  })
})
