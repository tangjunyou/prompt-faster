import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'

import { ExportDialog } from './ExportDialog'

const mutateAsync = vi.fn().mockResolvedValue({
  blob: new Blob(['data'], { type: 'application/json' }),
  filename: 'result.json',
})

vi.mock('../hooks/useExportResult', () => ({
  useExportResult: () => ({
    mutateAsync,
    isPending: false,
  }),
}))

describe('ExportDialog', () => {
  it('应渲染格式选项并触发导出', async () => {
    const onOpenChange = vi.fn()
    if (!URL.createObjectURL) {
      Object.defineProperty(URL, 'createObjectURL', {
        value: vi.fn(),
        writable: true,
      })
    }
    if (!URL.revokeObjectURL) {
      Object.defineProperty(URL, 'revokeObjectURL', {
        value: vi.fn(),
        writable: true,
      })
    }
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {})
    const urlSpy = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:result')

    render(<ExportDialog taskId="task-1" open onOpenChange={onOpenChange} />)

    expect(screen.getByText('Markdown')).toBeInTheDocument()
    expect(screen.getByText('JSON')).toBeInTheDocument()
    expect(screen.getByText('XML')).toBeInTheDocument()

    fireEvent.click(screen.getByText('JSON'))
    fireEvent.click(screen.getByRole('button', { name: '确认导出' }))

    await waitFor(() => {
      expect(mutateAsync).toHaveBeenCalledWith({ taskId: 'task-1', format: 'json' })
      expect(clickSpy).toHaveBeenCalled()
      expect(onOpenChange).toHaveBeenCalledWith(false)
    })

    clickSpy.mockRestore()
    urlSpy.mockRestore()
  })
})
