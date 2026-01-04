import { describe, it, expect, vi } from 'vitest'
import { parseTestCasesJsonl } from './parseTestCasesJsonl'

describe('parseTestCasesJsonl', () => {
  it('应跳过空行并解析合法 JSONL', async () => {
    const text = [
      '',
      '{"id":"case-1","input":{"question":"你好"},"reference":{"Exact":{"expected":"ok"}}}',
      '   ',
      '{"id":"case-2","input":{"question":"x"},"reference":{"Hybrid":{"exact_parts":{"a":"b"},"constraints":[]}}}',
      '',
    ].join('\n')

    const res = await parseTestCasesJsonl(text)
    expect(res.errors).toHaveLength(0)
    expect(res.cases).toHaveLength(2)
    expect(res.cases[0]?.id).toBe('case-1')
    expect(res.cases[0]?.split).toBeNull()
    expect(res.cases[0]?.metadata).toBeNull()
    expect(res.stats.ok).toBe(2)
    expect(res.stats.failed).toBe(0)
  })

  it('任意行 JSON 无法解析时应返回行号错误', async () => {
    const text = [
      '{"id":"case-1","input":{},"reference":{"Exact":{"expected":"ok"}}}',
      '{ bad json }',
    ].join('\n')

    const res = await parseTestCasesJsonl(text)
    expect(res.cases).toHaveLength(1)
    expect(res.errors).toHaveLength(1)
    expect(res.errors[0]?.line).toBe(2)
  })

  it('缺少字段/字段类型不正确应报错', async () => {
    const text = [
      '{"input":{},"reference":{"Exact":{"expected":"ok"}}}', // missing id
      '{"id":"case-2","input":[],"reference":{"Exact":{"expected":"ok"}}}', // input should be object
      '{"id":"case-3","input":{},"reference":{"Exact":{"expected":123}}}', // expected should be string
      '{"id":"case-4","input":{},"reference":{"Bad":{"x":1}}}', // bad variant
    ].join('\n')

    const res = await parseTestCasesJsonl(text)
    expect(res.cases).toHaveLength(0)
    expect(res.errors.map((e) => e.line)).toEqual([1, 2, 3, 4])
  })

  it('id 重复应报错', async () => {
    const text = [
      '{"id":"dup","input":{},"reference":{"Exact":{"expected":"ok"}}}',
      '{"id":"dup","input":{},"reference":{"Exact":{"expected":"ok"}}}',
    ].join('\n')

    const res = await parseTestCasesJsonl(text)
    expect(res.cases).toHaveLength(1)
    expect(res.errors).toHaveLength(1)
    expect(res.errors[0]?.line).toBe(2)
  })

  it('split/metadata 结构应被校验', async () => {
    const text = [
      '{"id":"case-1","input":{},"reference":{"Exact":{"expected":"ok"}},"split":"train","metadata":{"a":1}}',
      '{"id":"case-2","input":{},"reference":{"Exact":{"expected":"ok"}},"split":"bad"}',
      '{"id":"case-3","input":{},"reference":{"Exact":{"expected":"ok"}},"metadata":[]}',
    ].join('\n')

    const res = await parseTestCasesJsonl(text)
    expect(res.cases).toHaveLength(1)
    expect(res.errors.map((e) => e.line)).toEqual([2, 3])
  })

  it('Constrained/Hybrid 内部结构应被最小校验（避免保存阶段才失败）', async () => {
    const text = [
      JSON.stringify({
        id: 'c-1',
        input: {},
        reference: {
          Constrained: {
            constraints: [{ name: 1, description: 'x', weight: 1 }],
            quality_dimensions: [{ name: 'q', description: 'd', weight: 1 }],
          },
        },
      }),
      JSON.stringify({
        id: 'c-2',
        input: {},
        reference: {
          Constrained: {
            constraints: [{ name: 'n', description: 'x', weight: null }],
            quality_dimensions: [{ name: 'q', description: 'd', weight: null }],
          },
        },
      }),
      JSON.stringify({
        id: 'h-1',
        input: {},
        reference: {
          Hybrid: {
            exact_parts: { a: 1 },
            constraints: [{ name: 'n', description: 'x', weight: 1 }],
          },
        },
      }),
      JSON.stringify({
        id: 'h-2',
        input: {},
        reference: {
          Hybrid: {
            exact_parts: { a: 'b' },
            constraints: [{ name: 'n', description: 'x' }],
          },
        },
      }),
    ].join('\n')

    const res = await parseTestCasesJsonl(text)
    expect(res.cases).toHaveLength(0)
    expect(res.errors.map((e) => e.line)).toEqual([1, 2, 3, 4])
  })

  it('应支持 100+ 行并触发进度回调与让出事件循环', async () => {
    const lines: string[] = []
    for (let i = 0; i < 120; i += 1) {
      lines.push(
        JSON.stringify({
          id: `case-${i}`,
          input: { i },
          reference: { Exact: { expected: 'ok' } },
        })
      )
    }
    const text = lines.join('\n')

    const onProgress = vi.fn()
    const shouldYield = vi.fn(async () => {})

    const res = await parseTestCasesJsonl(text, {
      progressEvery: 50,
      yieldEvery: 60,
      onProgress,
      shouldYield,
    })

    expect(res.errors).toHaveLength(0)
    expect(res.cases).toHaveLength(120)
    expect(onProgress).toHaveBeenCalled()
    expect(onProgress.mock.calls.length).toBeGreaterThanOrEqual(3)
    expect(shouldYield).toHaveBeenCalled()
  })

  it('错误列表应最多保留前 maxErrors 条并标记截断', async () => {
    const lines: string[] = []
    for (let i = 0; i < 55; i += 1) lines.push('{ bad json }')

    const res = await parseTestCasesJsonl(lines.join('\n'), { maxErrors: 50 })
    expect(res.errors).toHaveLength(50)
    expect(res.truncatedErrors).toBe(true)
    expect(res.stats.failed).toBe(55)
  })
})
