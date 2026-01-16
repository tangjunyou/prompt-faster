import type { IterationState } from '@/types/generated/models'
import type { StageType } from '@/features/visualization/thinkingStages'

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
  stage?: StageType
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

  const stageDescriptors: Array<{ stage: StageType; state: IterationState; step: string }> = [
    { stage: 'pattern', state: 'extracting_rules', step: '规律抽取' },
    { stage: 'prompt', state: 'generating_prompt', step: '候选生成' },
    { stage: 'quality', state: 'evaluating', step: '质量评估' },
    { stage: 'reflection', state: 'reflecting', step: '反思迭代' },
  ]

  for (let iteration = 1; iteration <= iterations; iteration++) {
    const baseTokens = Math.floor(tokensPerIteration / stageDescriptors.length)
    const remainder = tokensPerIteration % stageDescriptors.length
    const stageTokens = stageDescriptors.map((_, index) =>
      baseTokens + (index < remainder ? 1 : 0),
    )

    stageDescriptors.forEach((descriptor, stageIndex) => {
      out.push({
        type: 'iteration:progress',
        correlationId,
        timestamp: rfc3339At(baseMs + seq * 10),
        payload: {
          kind: 'progress',
          seq,
          iteration,
          state: descriptor.state,
          step: descriptor.step,
          stage: descriptor.stage,
        },
      })
      seq += 1

      for (let t = 0; t < stageTokens[stageIndex]; t++) {
        out.push({
          type: 'thinking:stream',
          correlationId,
          timestamp: rfc3339At(baseMs + seq * 10),
          payload: {
            kind: 'stream',
            seq,
            content: `iter=${iteration} stage=${descriptor.stage} token=${t}`,
          },
        })
        seq += 1
      }
    })

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
        stage: 'reflection',
      },
    })
    seq += 1
  }

  return out
}
