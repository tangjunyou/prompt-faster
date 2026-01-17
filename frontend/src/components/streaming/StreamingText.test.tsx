import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'

import { StreamingText } from './StreamingText'

describe('StreamingText', () => {
  afterEach(() => {
    vi.useRealTimers()
  })
  describe('基础渲染', () => {
    it('应渲染组件容器', () => {
      render(<StreamingText text="" status="idle" />)

      expect(screen.getByTestId('streaming-text')).toBeInTheDocument()
    })

    it('空文本时应显示占位提示', () => {
      render(<StreamingText text="" status="idle" />)

      expect(screen.getByText('等待思考内容...')).toBeInTheDocument()
    })

    it('应渲染传入的文本内容', () => {
      render(<StreamingText text="Hello World" status="streaming" />)

      expect(screen.getByTestId('streaming-text-content')).toHaveTextContent('Hello World')
    })
  })

  describe('状态指示器', () => {
    it('idle 状态应显示"等待中"', () => {
      render(<StreamingText text="" status="idle" />)

      expect(screen.getByText('等待中')).toBeInTheDocument()
    })

    it('streaming 状态应显示"生成中"', () => {
      render(<StreamingText text="text" status="streaming" />)

      expect(screen.getByText('生成中')).toBeInTheDocument()
    })

    it('complete 状态应显示"完成"', () => {
      render(<StreamingText text="text" status="complete" />)

      expect(screen.getByText('完成')).toBeInTheDocument()
    })

    it('error 状态应显示"错误"', () => {
      render(<StreamingText text="text" status="error" />)

      expect(screen.getByText('错误')).toBeInTheDocument()
    })
  })

  describe('复制功能', () => {
    it('应渲染复制按钮', () => {
      render(<StreamingText text="Hello" status="streaming" />)

      expect(screen.getByTestId('streaming-text-copy')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: '复制思考内容' })).toBeInTheDocument()
    })

    it('点击复制按钮应调用 clipboard API', async () => {
      const mockWriteText = vi.fn().mockResolvedValue(undefined)
      Object.assign(navigator, {
        clipboard: { writeText: mockWriteText },
      })

      render(<StreamingText text="Copy me" status="streaming" />)

      const copyButton = screen.getByTestId('streaming-text-copy')
      fireEvent.click(copyButton)

      await waitFor(() => {
        expect(mockWriteText).toHaveBeenCalledWith('Copy me')
      })
    })

    it('点击复制按钮应触发 onCopy 回调', async () => {
      const mockWriteText = vi.fn().mockResolvedValue(undefined)
      Object.assign(navigator, {
        clipboard: { writeText: mockWriteText },
      })

      const onCopy = vi.fn()
      render(<StreamingText text="Copy me" status="streaming" onCopy={onCopy} />)

      const copyButton = screen.getByTestId('streaming-text-copy')
      fireEvent.click(copyButton)

      await waitFor(() => {
        expect(onCopy).toHaveBeenCalledWith('Copy me')
      })
    })

    it('复制失败时应提示用户', async () => {
      const mockWriteText = vi.fn().mockRejectedValue(new Error('copy failed'))
      Object.assign(navigator, {
        clipboard: { writeText: mockWriteText },
      })

      render(<StreamingText text="Copy me" status="streaming" />)

      const copyButton = screen.getByTestId('streaming-text-copy')
      fireEvent.click(copyButton)

      await waitFor(() => {
        expect(screen.getByText('复制失败')).toBeInTheDocument()
      })

      await waitFor(() => {
        expect(screen.queryByText('复制失败')).not.toBeInTheDocument()
      }, { timeout: 2500 })
    })
  })

  describe('可访问性', () => {
    it('内容区域应有 aria-live="polite"', () => {
      render(<StreamingText text="Hello" status="streaming" />)

      const content = screen.getByTestId('streaming-text-content')
      expect(content).toHaveAttribute('aria-live', 'polite')
    })

    it('内容区域应有 role="log"', () => {
      render(<StreamingText text="Hello" status="streaming" />)

      const content = screen.getByTestId('streaming-text-content')
      expect(content).toHaveAttribute('role', 'log')
    })

    it('内容区域应可聚焦（tabIndex=0）', () => {
      render(<StreamingText text="Hello" status="streaming" />)

      const content = screen.getByTestId('streaming-text-content')
      expect(content).toHaveAttribute('tabIndex', '0')
    })

    it('复制按钮应有可读 aria-label', () => {
      render(<StreamingText text="Hello" status="streaming" />)

      const copyButton = screen.getByTestId('streaming-text-copy')
      expect(copyButton).toHaveAttribute('aria-label', '复制思考内容')
    })
  })

  describe('长文本截断', () => {
    it('应显示截断提示', () => {
      const longText = 'A'.repeat(20000)
      render(<StreamingText text={longText} status="streaming" maxChars={10000} />)

      expect(screen.getByText(/较早内容已省略/)).toBeInTheDocument()
    })

    it('未截断时不应显示截断提示', () => {
      render(<StreamingText text="Short text" status="streaming" />)

      expect(screen.queryByText(/较早内容已省略/)).not.toBeInTheDocument()
    })
  })

  describe('prefersReducedMotion', () => {
    it('streaming 状态下默认显示光标', () => {
      const { container } = render(
        <StreamingText text="Hello" status="streaming" prefersReducedMotion={false} />,
      )

      const cursor = container.querySelector('.animate-pulse.bg-slate-400')
      expect(cursor).toBeInTheDocument()
    })

    it('prefersReducedMotion=true 时不显示光标', () => {
      const { container } = render(
        <StreamingText text="Hello" status="streaming" prefersReducedMotion={true} />,
      )

      const cursor = container.querySelector('.animate-pulse.bg-slate-400')
      expect(cursor).not.toBeInTheDocument()
    })

    it('non-streaming 状态下不显示光标', () => {
      const { container } = render(
        <StreamingText text="Hello" status="complete" prefersReducedMotion={false} />,
      )

      const cursor = container.querySelector('.animate-pulse.bg-slate-400')
      expect(cursor).not.toBeInTheDocument()
    })
  })

  describe('自动滚动', () => {
    it('应在新内容时调用 scrollTo', async () => {
      const { rerender } = render(<StreamingText text="Initial" status="streaming" />)

      const content = screen.getByTestId('streaming-text-content')
      const scrollToSpy = vi.fn()
      content.scrollTo = scrollToSpy

      // 模拟容器尺寸（在底部）
      Object.defineProperty(content, 'scrollHeight', { value: 100, configurable: true })
      Object.defineProperty(content, 'scrollTop', { value: 0, configurable: true })
      Object.defineProperty(content, 'clientHeight', { value: 100, configurable: true })

      rerender(<StreamingText text="Initial + More" status="streaming" />)

      // 使用 rAF，需要等待
      await waitFor(
        () => {
          expect(scrollToSpy).toHaveBeenCalled()
        },
        { timeout: 100 },
      )
    })

    it('用户滚动离开底部应锁定并显示回到底部按钮', () => {
      render(<StreamingText text="Line" status="streaming" />)

      const content = screen.getByTestId('streaming-text-content')
      Object.defineProperty(content, 'scrollHeight', { value: 400, configurable: true })
      Object.defineProperty(content, 'scrollTop', { value: 0, configurable: true })
      Object.defineProperty(content, 'clientHeight', { value: 100, configurable: true })

      fireEvent.scroll(content)

      expect(screen.getByTestId('streaming-text-scroll-bottom')).toBeInTheDocument()
    })

    it('点击回到底部按钮应解锁并隐藏按钮', async () => {
      render(<StreamingText text="Line" status="streaming" />)

      const content = screen.getByTestId('streaming-text-content')
      const scrollToSpy = vi.fn()
      content.scrollTo = scrollToSpy

      Object.defineProperty(content, 'scrollHeight', { value: 400, configurable: true })
      Object.defineProperty(content, 'scrollTop', { value: 0, configurable: true })
      Object.defineProperty(content, 'clientHeight', { value: 100, configurable: true })

      fireEvent.scroll(content)

      const button = screen.getByTestId('streaming-text-scroll-bottom')
      fireEvent.click(button)

      await waitFor(() => {
        expect(scrollToSpy).toHaveBeenCalled()
      })
      expect(screen.queryByTestId('streaming-text-scroll-bottom')).not.toBeInTheDocument()
    })

    it('受控锁定状态变化应触发回调', () => {
      const onAutoScrollLockedChange = vi.fn()
      render(
        <StreamingText
          text="Line"
          status="streaming"
          isAutoScrollLocked={false}
          onAutoScrollLockedChange={onAutoScrollLockedChange}
        />,
      )

      const content = screen.getByTestId('streaming-text-content')
      Object.defineProperty(content, 'scrollHeight', { value: 400, configurable: true })
      Object.defineProperty(content, 'scrollTop', { value: 0, configurable: true })
      Object.defineProperty(content, 'clientHeight', { value: 100, configurable: true })

      fireEvent.scroll(content)

      expect(onAutoScrollLockedChange).toHaveBeenCalledWith(true)
    })
  })

  describe('自定义样式', () => {
    it('应应用自定义 className', () => {
      render(<StreamingText text="Hello" status="streaming" className="custom-class" />)

      const container = screen.getByTestId('streaming-text')
      expect(container).toHaveClass('custom-class')
    })
  })
})
