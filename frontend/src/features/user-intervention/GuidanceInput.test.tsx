/**
 * GuidanceInput 组件测试
 */

import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { GuidanceInput } from './GuidanceInput'

describe('GuidanceInput', () => {
  it('renders placeholder and disabled hint', () => {
    render(
      <GuidanceInput
        taskId="task-1"
        onSend={vi.fn()}
        disabled
      />
    )

    expect(screen.getByPlaceholderText('告诉老师模型你的想法...')).toBeInTheDocument()
    expect(screen.getByText('⚠️ 请先暂停任务再发送引导')).toBeInTheDocument()
  })

  it('sends guidance and clears input on success', () => {
    const onSend = vi.fn()
    render(
      <GuidanceInput
        taskId="task-1"
        onSend={onSend}
      />
    )

    const textarea = screen.getByPlaceholderText('告诉老师模型你的想法...') as HTMLTextAreaElement
    fireEvent.change(textarea, { target: { value: '  给模型加更多结构约束  ' } })

    fireEvent.click(screen.getByText('发送'))

    expect(onSend).toHaveBeenCalledTimes(1)
    const [content, correlationId] = onSend.mock.calls[0]
    expect(content).toBe('给模型加更多结构约束')
    expect(correlationId).toMatch(/^cid-/)
    expect(textarea.value).toBe('')
  })

  it('shows validation error when content too long', () => {
    render(
      <GuidanceInput
        taskId="task-1"
        onSend={vi.fn()}
      />
    )

    const textarea = screen.getByPlaceholderText('告诉老师模型你的想法...') as HTMLTextAreaElement
    fireEvent.change(textarea, { target: { value: 'a'.repeat(2001) } })
    fireEvent.click(screen.getByText('发送'))

    expect(screen.getByText(/引导内容超过最大长度限制/)).toBeInTheDocument()
  })

  it('shows saved message when guidance pending', () => {
    render(
      <GuidanceInput
        taskId="task-1"
        onSend={vi.fn()}
        guidance={{
          id: 'g1',
          content: 'preview',
          status: 'pending',
          createdAt: '2026-01-17T12:00:00Z',
        }}
      />
    )

    expect(screen.getByText('✅ 引导已保存，将在下一轮迭代生效')).toBeInTheDocument()
  })

  it('keeps guidance visible when disabled', () => {
    render(
      <GuidanceInput
        taskId="task-1"
        onSend={vi.fn()}
        disabled
        guidance={{
          id: 'g2',
          content: 'preview applied',
          status: 'applied',
          createdAt: '2026-01-17T12:00:00Z',
          appliedAt: '2026-01-17T12:01:00Z',
        }}
      />
    )

    expect(screen.getByText('已应用')).toBeInTheDocument()
    expect(screen.getByText('preview applied')).toBeInTheDocument()
  })
})
