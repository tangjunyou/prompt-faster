import { describe, expect, it } from 'vitest'

import { nodeStatusToClassName } from './nodeStyles'
import type { NodeStatus } from './types'

describe('nodeStatusToClassName', () => {
  it('maps all statuses to stable semantic classes', () => {
    const cases: Array<{ status: NodeStatus; expected: string }> = [
      { status: 'idle', expected: 'bg-slate' }, // 灰色：待执行
      { status: 'running', expected: 'bg-blue' }, // 蓝色：执行中
      { status: 'success', expected: 'bg-emerald' }, // 绿色：成功
      { status: 'error', expected: 'bg-red' }, // 红色：失败
      { status: 'paused', expected: 'bg-yellow' }, // 黄色：暂停/需介入
    ]

    for (const c of cases) {
      expect(nodeStatusToClassName(c.status)).toContain(c.expected)
      expect(nodeStatusToClassName(c.status)).toBe(nodeStatusToClassName(c.status))
    }
  })
})

