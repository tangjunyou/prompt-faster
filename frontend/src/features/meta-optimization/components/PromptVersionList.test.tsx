import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

import { PromptVersionList } from './PromptVersionList'
import type { TeacherPromptVersion } from '@/types/generated/models/TeacherPromptVersion'

const versions: TeacherPromptVersion[] = [
  {
    id: 'v1',
    version: 1,
    description: '初始版本',
    isActive: true,
    successRate: 0.5,
    taskCount: 2,
    createdAt: '2024-01-01T00:00:00Z',
  },
  {
    id: 'v2',
    version: 2,
    description: '改进版本',
    isActive: false,
    successRate: null,
    taskCount: 0,
    createdAt: '2024-02-01T00:00:00Z',
  },
]

describe('PromptVersionList', () => {
  it('renders versions and highlights active', () => {
    render(<PromptVersionList versions={versions} selectedId="v1" />)

    expect(screen.getByText('v1')).toBeInTheDocument()
    expect(screen.getByText('v2')).toBeInTheDocument()
    expect(screen.getByText('当前使用')).toBeInTheDocument()
    expect(screen.getByText('初始版本')).toBeInTheDocument()
  })

  it('renders empty state', () => {
    render(<PromptVersionList versions={[]} selectedId={null} />)
    expect(
      screen.getByText('暂无版本，请先创建一个 Prompt 版本。')
    ).toBeInTheDocument()
  })

  it('calls activate handler', () => {
    const onActivate = vi.fn()
    render(
      <PromptVersionList
        versions={versions}
        selectedId={null}
        onActivate={onActivate}
      />
    )

    const buttons = screen.getAllByRole('button', { name: '设为活跃' })
    fireEvent.click(buttons[0])

    expect(onActivate).toHaveBeenCalledWith('v2')
  })
})
