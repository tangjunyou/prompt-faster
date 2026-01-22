/**
 * ArtifactEditor 组件测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ArtifactEditor } from './ArtifactEditor'

vi.mock('@monaco-editor/react', () => ({
  default: ({ value, onChange }: { value: string; onChange?: (v: string) => void }) => (
    <textarea
      data-testid="monaco"
      value={value}
      onChange={(event) => onChange?.(event.target.value)}
    />
  ),
}))

describe('ArtifactEditor', () => {
  const baseArtifacts = {
    patterns: [
      {
        id: 'p1',
        pattern: 'Pattern 1',
        source: 'system' as const,
        confidence: 0.5,
      },
    ],
    candidatePrompts: [
      {
        id: 'c1',
        content: 'Prompt 1',
        source: 'system' as const,
        score: null,
        isBest: true,
      },
    ],
    userGuidance: null,
    failureArchive: null,
    diversityAnalysis: null,
    updatedAt: '2026-01-17T00:00:00Z',
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows disabled hint when artifacts are empty and disabled', () => {
    render(
      <ArtifactEditor
        taskId="task-1"
        artifacts={undefined}
        onSave={vi.fn()}
        disabled
      />
    )

    expect(screen.getByText('暂无可编辑的产物')).toBeInTheDocument()
    expect(screen.getByText('⚠️ 请先暂停任务再编辑')).toBeInTheDocument()
  })

  it('keeps editing until save succeeds and shows success message', async () => {
    const onSave = vi.fn()
    const { rerender } = render(
      <ArtifactEditor key="initial" taskId="task-1" artifacts={baseArtifacts} onSave={onSave} />
    )

    fireEvent.click(screen.getByText('编辑'))

    const editor = await screen.findByTestId('monaco') as HTMLTextAreaElement
    fireEvent.change(editor, { target: { value: 'Pattern 1 updated' } })

    fireEvent.click(screen.getByText('保存'))

    expect(onSave).toHaveBeenCalled()
    expect(screen.getByText('取消')).toBeInTheDocument()

    rerender(
      <ArtifactEditor
        key="success"
        taskId="task-1"
        artifacts={baseArtifacts}
        onSave={onSave}
        showSuccess
      />
    )

    expect(screen.getByText('编辑')).toBeInTheDocument()
    expect(await screen.findByText('✅ 保存成功')).toBeInTheDocument()
  })
})
