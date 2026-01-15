import { describe, expect, it } from 'vitest'

import { createDeterministicDemoWsMessages } from './demoWsMessages'

describe('createDeterministicDemoWsMessages', () => {
  it('is deterministic for same inputs', () => {
    const a = createDeterministicDemoWsMessages({ correlationId: 'cid-1', iterations: 2, tokensPerIteration: 3 })
    const b = createDeterministicDemoWsMessages({ correlationId: 'cid-1', iterations: 2, tokensPerIteration: 3 })
    expect(a).toEqual(b)
  })

  it('preserves strict seq order (no out-of-order)', () => {
    const msgs = createDeterministicDemoWsMessages({ correlationId: 'cid-order', iterations: 3, tokensPerIteration: 2 })
    const seqs = msgs.map((m) => m.payload.seq)
    expect(seqs.length).toBeGreaterThan(0)
    for (let i = 1; i < seqs.length; i++) {
      expect(seqs[i]).toBe(seqs[i - 1] + 1)
    }
  })

  it('does not cross-talk between correlationIds', () => {
    const a = createDeterministicDemoWsMessages({ correlationId: 'cid-a', iterations: 1, tokensPerIteration: 1 })
    const b = createDeterministicDemoWsMessages({ correlationId: 'cid-b', iterations: 1, tokensPerIteration: 1 })

    expect(a.every((m) => m.correlationId === 'cid-a')).toBe(true)
    expect(b.every((m) => m.correlationId === 'cid-b')).toBe(true)
  })
})

