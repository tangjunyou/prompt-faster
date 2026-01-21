import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { CaseComparisonList } from './CaseComparisonList'
import type { CaseComparisonResult } from '@/types/generated/models/CaseComparisonResult'

const comparisons: CaseComparisonResult[] = [
  {
    testCaseId: 'tc-1',
    input: { foo: 'bar' },
    reference: { Exact: { expected: 'ok' } },
    versionAOutput: 'bad',
    versionAPassed: false,
    versionBOutput: 'ok',
    versionBPassed: true,
    isDifferent: true,
    differenceNote: '版本 B 在此用例改进',
    versionAError: 'A failed',
    versionBError: null,
  },
  {
    testCaseId: 'tc-2',
    input: { foo: 'baz' },
    reference: { Exact: { expected: 'ok' } },
    versionAOutput: 'ok',
    versionAPassed: true,
    versionBOutput: 'ok',
    versionBPassed: true,
    isDifferent: false,
    differenceNote: null,
    versionAError: null,
    versionBError: null,
  },
  {
    testCaseId: 'tc-3',
    input: { foo: 'diff' },
    reference: { Exact: { expected: 'ok' } },
    versionAOutput: 'hello',
    versionAPassed: true,
    versionBOutput: 'world',
    versionBPassed: true,
    isDifferent: true,
    differenceNote: '两版本均通过，但输出内容存在差异',
    versionAError: null,
    versionBError: null,
  },
]

describe('CaseComparisonList', () => {
  it('renders comparisons and supports filtering', () => {
    render(<CaseComparisonList comparisons={comparisons} />)

    expect(screen.getByText('用例 ID: tc-1')).toBeInTheDocument()
    expect(screen.getByText('用例 ID: tc-2')).toBeInTheDocument()
    expect(screen.getByText('用例 ID: tc-3')).toBeInTheDocument()

    fireEvent.click(screen.getByLabelText('只看差异'))

    expect(screen.getByText('用例 ID: tc-1')).toBeInTheDocument()
    expect(screen.getByText('用例 ID: tc-3')).toBeInTheDocument()
    expect(screen.queryByText('用例 ID: tc-2')).toBeNull()
  })

  it('highlights output-diff cases', () => {
    render(<CaseComparisonList comparisons={comparisons} />)
    const label = screen.getByText('用例 ID: tc-3')
    let node: HTMLElement | null = label.parentElement
    while (node && !node.className.includes('border-amber-300')) {
      node = node.parentElement
    }
    expect(node?.className).toContain('border-amber-300')
  })
})
