import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MemoryRouter, Routes, Route } from 'react-router'
import { ViewSwitcher } from './ViewSwitcher'

const renderWithRouter = (initialEntry: string) => {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <ViewSwitcher />
      <input aria-label="dummy-input" />
      <Routes>
        <Route path="/run" element={<div>Run Page</div>} />
        <Route path="/focus" element={<div>Focus Page</div>} />
        <Route path="/workspace" element={<div>Workspace Page</div>} />
      </Routes>
    </MemoryRouter>
  )
}

describe('ViewSwitcher', () => {
  it('高亮当前视图按钮', () => {
    renderWithRouter('/focus')
    expect(screen.getByTestId('view-switcher-focus')).toHaveAttribute('data-active', 'true')
    expect(screen.getByTestId('view-switcher-run')).toHaveAttribute('data-active', 'false')
  })

  it('支持快捷键切换视图', () => {
    renderWithRouter('/run')
    fireEvent.keyDown(window, { key: '2', code: 'Digit2', metaKey: true })
    expect(screen.getByText('Focus Page')).toBeInTheDocument()
  })

  it('输入框聚焦时不触发快捷键', () => {
    renderWithRouter('/run')
    const input = screen.getByLabelText('dummy-input')
    input.focus()

    fireEvent.keyDown(input, { key: '3', code: 'Digit3', metaKey: true })
    expect(screen.getByText('Run Page')).toBeInTheDocument()
  })
})
