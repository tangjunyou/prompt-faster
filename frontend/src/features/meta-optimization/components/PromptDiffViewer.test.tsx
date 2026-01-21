import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { PromptDiffViewer } from './PromptDiffViewer'

vi.mock('@monaco-editor/react', () => ({
  DiffEditor: (props: { original: string; modified: string }) => (
    <div data-testid="diff-editor" data-original={props.original} data-modified={props.modified} />
  ),
}))

describe('PromptDiffViewer', () => {
  it('renders diff editor with version labels', async () => {
    render(
      <PromptDiffViewer
        versionA={{ version: 1, content: 'hello' }}
        versionB={{ version: 2, content: 'world' }}
      />
    )

    expect(screen.getByText('版本 1（基准）')).toBeInTheDocument()
    expect(screen.getByText('版本 2（对比）')).toBeInTheDocument()

    const diff = await screen.findByTestId('diff-editor')
    expect(diff.getAttribute('data-original')).toBe('hello')
    expect(diff.getAttribute('data-modified')).toBe('world')
  })
})
