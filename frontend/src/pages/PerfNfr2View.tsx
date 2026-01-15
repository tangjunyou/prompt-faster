import { useMemo, useState } from 'react'

import { createDeterministicDemoWsMessages } from '@/features/ws-demo/demoWsMessages'

export function PerfNfr2View() {
  const [latencyMs, setLatencyMs] = useState<number | null>(null)
  const [firstMessageType, setFirstMessageType] = useState<string | null>(null)
  const [messageCount, setMessageCount] = useState<number | null>(null)

  const messages = useMemo(
    () => createDeterministicDemoWsMessages({ correlationId: 'perf-nfr2', iterations: 1, tokensPerIteration: 5 }),
    [],
  )

  const run = async () => {
    setLatencyMs(null)
    setFirstMessageType(null)
    setMessageCount(null)

    const start = performance.now()

    // 用本地确定性消息流模拟“首条 WS 消息到达”口径（NFR2 的可回归测量入口）。
    await Promise.resolve()
    const first = messages[0]
    const end = performance.now()

    setLatencyMs(end - start)
    setFirstMessageType(first?.type ?? null)
    setMessageCount(messages.length)
  }

  return (
    <div className="mx-auto max-w-3xl p-6">
      <h1 className="text-xl font-semibold">NFR2：流式输出首字节延迟（口径预置）</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        该页面用于预置 NFR2 的测量口径与回归入口（纯本地、确定性、不出网）。
      </p>

      <div className="mt-6 flex items-center gap-3">
        <button
          className="rounded border px-3 py-2 text-sm"
          onClick={run}
          data-testid="nfr2-run"
        >
          运行测量
        </button>
      </div>

      <div className="mt-6 rounded border p-4 text-sm" data-testid="nfr2-result">
        <div>firstMessageLatencyMs: {latencyMs == null ? '-' : latencyMs.toFixed(2)}</div>
        <div>firstMessageType: {firstMessageType ?? '-'}</div>
        <div>messageCount: {messageCount ?? '-'}</div>
        <div className="mt-2 text-muted-foreground">
          说明：Epic 5 接入真实 WS 后，将“firstMessageLatencyMs”的起止点替换为“发起运行请求 → 收到第一条 WS 消息”。
        </div>
      </div>
    </div>
  )
}

