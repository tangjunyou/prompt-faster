import type { IterationState } from '@/types/generated/models'

export type WsMessage<T> = {
  type: string
  payload: T
  timestamp: string
  correlationId?: string
}

export type DemoProgressPayload = {
  seq: number
  iteration: number
  state: IterationState
  step: string
}

export type DemoStreamPayload = {
  seq: number
  content: string
}

export type DemoServerMessagePayload =
  | ({ kind: 'progress' } & DemoProgressPayload)
  | ({ kind: 'stream' } & DemoStreamPayload)

export type DemoWsMessage = WsMessage<DemoServerMessagePayload>

type DemoWsMessagesOptions = {
  correlationId: string
  iterations: number
  tokensPerIteration: number
}

function rfc3339At(ms: number): string {
  return new Date(ms).toISOString()
}

export function createDeterministicDemoWsMessages(options: DemoWsMessagesOptions): DemoWsMessage[] {
  const correlationId = options.correlationId.trim()
  if (!correlationId) {
    throw new Error('correlationId is required')
  }
  const iterations = Math.max(1, Math.floor(options.iterations))
  const tokensPerIteration = Math.max(1, Math.floor(options.tokensPerIteration))

  const baseMs = Date.UTC(2026, 0, 15, 0, 0, 0, 0) // 2026-01-15T00:00:00.000Z
  const out: DemoWsMessage[] = []
  let seq = 0

  for (let iteration = 1; iteration <= iterations; iteration++) {
    out.push({
      type: 'iteration:progress',
      correlationId,
      timestamp: rfc3339At(baseMs + seq * 10),
      payload: {
        kind: 'progress',
        seq,
        iteration,
        state: 'running_tests',
        step: '运行测试',
      },
    })
    seq += 1

    out.push({
      type: 'iteration:progress',
      correlationId,
      timestamp: rfc3339At(baseMs + seq * 10),
      payload: {
        kind: 'progress',
        seq,
        iteration,
        state: 'evaluating',
        step: '评估',
      },
    })
    seq += 1

    out.push({
      type: 'iteration:progress',
      correlationId,
      timestamp: rfc3339At(baseMs + seq * 10),
      payload: {
        kind: 'progress',
        seq,
        iteration,
        state: 'waiting_user',
        step: '等待用户',
      },
    })
    seq += 1

    for (let t = 0; t < tokensPerIteration; t++) {
      out.push({
        type: 'thinking:stream',
        correlationId,
        timestamp: rfc3339At(baseMs + seq * 10),
        payload: {
          kind: 'stream',
          seq,
          content: `iter=${iteration} token=${t}`,
        },
      })
      seq += 1
    }

    const finalState = iteration % 2 === 0 ? 'failed' : 'completed'
    out.push({
      type: 'iteration:progress',
      correlationId,
      timestamp: rfc3339At(baseMs + seq * 10),
      payload: {
        kind: 'progress',
        seq,
        iteration,
        state: finalState,
        step: finalState === 'failed' ? '失败' : '完成',
      },
    })
    seq += 1
  }

  return out
}
