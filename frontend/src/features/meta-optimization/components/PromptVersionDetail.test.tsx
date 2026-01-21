import { describe, it, expect, vi, beforeEach } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PromptVersionDetail } from './PromptVersionDetail'
import { useAuthStore } from '@/stores/useAuthStore'

import type { TeacherPrompt } from '@/types/generated/models/TeacherPrompt'
import type { TeacherPromptVersion } from '@/types/generated/models/TeacherPromptVersion'

const validatePrompt = vi.fn()
const createPromptVersion = vi.fn()

vi.mock('../services/metaOptimizationService', () => ({
  validatePrompt: (...args: unknown[]) => validatePrompt(...args),
  createPromptVersion: (...args: unknown[]) => createPromptVersion(...args),
  activatePromptVersion: vi.fn(),
  getPromptVersion: vi.fn(),
}))

vi.mock('./PromptEditor', () => ({
  PromptEditor: ({ value, onChange }: { value: string; onChange?: (v: string) => void }) => (
    <textarea
      data-testid="prompt-editor"
      value={value}
      onChange={(event) => onChange?.(event.target.value)}
    />
  ),
}))

vi.mock('./PromptPreviewPanel', () => ({
  PromptPreviewPanel: () => <div data-testid="preview-panel" />,
}))

describe('PromptVersionDetail', () => {
  const prompt: TeacherPrompt = {
    id: 'v1',
    userId: 'u1',
    version: 1,
    content: 'prompt',
    description: null,
    isActive: true,
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  }

  const versions: TeacherPromptVersion[] = [
    {
      id: 'v1',
      version: 1,
      description: null,
      isActive: true,
      successRate: null,
      taskCount: 0,
      createdAt: '2026-01-01T00:00:00Z',
    },
    {
      id: 'v2',
      version: 2,
      description: null,
      isActive: false,
      successRate: null,
      taskCount: 0,
      createdAt: '2026-01-02T00:00:00Z',
    },
  ]

  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'token',
      currentUser: { id: 'u1', username: 'user1' },
      requiresRegistration: null,
    })
    validatePrompt.mockReset()
    createPromptVersion.mockReset()
    validatePrompt.mockResolvedValue({ isValid: true, errors: [], warnings: [] })
    createPromptVersion.mockResolvedValue({
      id: 'v2',
      version: 2,
      description: 'note',
      isActive: true,
      successRate: null,
      taskCount: 0,
      createdAt: '2026-01-02T00:00:00Z',
    })
  })

  it('calls validatePrompt before createPromptVersion', async () => {
    const queryClient = new QueryClient()
    render(
      <QueryClientProvider client={queryClient}>
        <PromptVersionDetail prompt={prompt} versions={versions} />
      </QueryClientProvider>
    )

    fireEvent.click(screen.getByRole('button', { name: '编辑' }))
    fireEvent.click(screen.getByRole('button', { name: '保存为新版本' }))

    const noteInput = screen.getByLabelText(/变更说明/gi)
    fireEvent.change(noteInput, { target: { value: 'update' } })

    fireEvent.click(screen.getByRole('button', { name: '确认保存' }))

    await waitFor(() => {
      expect(createPromptVersion).toHaveBeenCalled()
    })

    expect(validatePrompt).toHaveBeenCalled()
    expect(validatePrompt.mock.invocationCallOrder[0]).toBeLessThan(
      createPromptVersion.mock.invocationCallOrder[0]
    )
  })
})
