/**
 * PauseResumeControl 组件测试
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { PauseResumeControl } from './PauseResumeControl'
import { useTaskStore } from '@/stores/useTaskStore'

describe('PauseResumeControl', () => {
  const taskId = 'test-task-1'

  beforeEach(() => {
    // 重置 store 状态
    useTaskStore.getState().reset()
  })

  it('renders pause button when task is running', () => {
    // 设置任务为运行状态
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(<PauseResumeControl taskId={taskId} />)

    expect(screen.getByText('暂停')).toBeInTheDocument()
  })

  it('renders resume button when task is paused', () => {
    // 设置任务为暂停状态
    useTaskStore.getState().handlePaused(taskId, new Date().toISOString(), 'reflecting', 2)

    render(<PauseResumeControl taskId={taskId} />)

    expect(screen.getByText('继续')).toBeInTheDocument()
  })

  it('calls onPause when pause button is clicked', () => {
    const onPause = vi.fn()
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(<PauseResumeControl taskId={taskId} onPause={onPause} />)

    fireEvent.click(screen.getByText('暂停'))

    expect(onPause).toHaveBeenCalledWith(taskId, expect.stringContaining('cid-'))
  })

  it('calls onResume when resume button is clicked', () => {
    const onResume = vi.fn()
    useTaskStore.getState().handlePaused(taskId, new Date().toISOString(), 'reflecting', 2)

    render(<PauseResumeControl taskId={taskId} onResume={onResume} />)

    fireEvent.click(screen.getByText('继续'))

    expect(onResume).toHaveBeenCalledWith(taskId, expect.stringContaining('cid-'))
  })

  it('is disabled when disabled prop is true', () => {
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(<PauseResumeControl taskId={taskId} disabled />)

    expect(screen.getByRole('button')).toBeDisabled()
  })

  it('is disabled when task is idle', () => {
    // 默认状态是 idle
    render(<PauseResumeControl taskId={taskId} />)

    expect(screen.getByRole('button')).toBeDisabled()
  })

  it('button has minimum click area of 44px x 44px', () => {
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(<PauseResumeControl taskId={taskId} />)

    const button = screen.getByRole('button')
    expect(button).toHaveClass('min-w-[44px]')
    expect(button).toHaveClass('min-h-[44px]')
  })

  it('responds to Space key when running', () => {
    const onPause = vi.fn()
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(<PauseResumeControl taskId={taskId} onPause={onPause} />)

    fireEvent.keyDown(window, { code: 'Space' })

    expect(onPause).toHaveBeenCalled()
  })

  it('responds to Space key when paused', () => {
    const onResume = vi.fn()
    useTaskStore.getState().handlePaused(taskId, new Date().toISOString(), 'reflecting', 2)

    render(<PauseResumeControl taskId={taskId} onResume={onResume} />)

    fireEvent.keyDown(window, { code: 'Space' })

    expect(onResume).toHaveBeenCalled()
  })

  it('does not respond to Space key when disabled', () => {
    const onPause = vi.fn()
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(<PauseResumeControl taskId={taskId} onPause={onPause} disabled />)

    fireEvent.keyDown(window, { code: 'Space' })

    expect(onPause).not.toHaveBeenCalled()
  })

  it('does not respond to Space key in input fields', () => {
    const onPause = vi.fn()
    useTaskStore.getState().setRunControlState(taskId, 'running')

    render(
      <>
        <input data-testid="test-input" />
        <PauseResumeControl taskId={taskId} onPause={onPause} />
      </>
    )

    const input = screen.getByTestId('test-input')
    fireEvent.keyDown(input, { code: 'Space' })

    expect(onPause).not.toHaveBeenCalled()
  })
})
