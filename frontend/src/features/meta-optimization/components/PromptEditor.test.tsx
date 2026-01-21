import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { PromptEditor } from './PromptEditor'

vi.mock('@monaco-editor/react', () => ({
  default: ({ value, onChange }: { value: string; onChange?: (v: string) => void }) => (
    <textarea
      data-testid="monaco"
      value={value}
      onChange={(event) => onChange?.(event.target.value)}
    />
  ),
}))

describe('PromptEditor', () => {
  it('renders editor and emits changes', async () => {
    const onChange = vi.fn()
    render(<PromptEditor value="initial" onChange={onChange} />)

    const editor = (await screen.findByTestId('monaco')) as HTMLTextAreaElement
    fireEvent.change(editor, { target: { value: 'updated' } })

    expect(onChange).toHaveBeenCalledWith('updated')
  })
})
