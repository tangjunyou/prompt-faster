import { useEffect, useMemo, useRef, useState } from 'react'

import { IterationGraph } from '@/components/nodes/IterationGraph'
import type { IterationGraphNodeStates } from '@/components/nodes/types'
import { createDeterministicDemoWsMessages } from '@/features/ws-demo/demoWsMessages'
import {
  createInitialIterationGraphNodeStates,
  reduceDemoWsMessageToIterationGraphNodeStates,
} from '@/features/visualization/iterationGraphDemoReducer'

export function RunView() {
  const demoMessages = useMemo(
    () =>
      createDeterministicDemoWsMessages({
        correlationId: 'runview-demo',
        iterations: 1,
        tokensPerIteration: 30,
      }),
    [],
  )

  const [nodeStates, setNodeStates] = useState<IterationGraphNodeStates>(() =>
    createInitialIterationGraphNodeStates(),
  )
  const [isReplaying, setIsReplaying] = useState(false)
  const replayTimerRef = useRef<number | null>(null)

  useEffect(() => {
    return () => {
      if (replayTimerRef.current !== null) {
        window.clearInterval(replayTimerRef.current)
      }
    }
  }, [])

  function handleReplay() {
    if (replayTimerRef.current !== null) {
      window.clearInterval(replayTimerRef.current)
      replayTimerRef.current = null
    }

    setIsReplaying(true)
    setNodeStates(createInitialIterationGraphNodeStates())

    let idx = 0
    replayTimerRef.current = window.setInterval(() => {
      const msg = demoMessages[idx]
      if (!msg) {
        if (replayTimerRef.current !== null) {
          window.clearInterval(replayTimerRef.current)
          replayTimerRef.current = null
        }
        setIsReplaying(false)
        return
      }
      idx += 1
      setNodeStates((prev) => reduceDemoWsMessageToIterationGraphNodeStates(prev, msg))
    }, 120)
  }

  return (
    <section className="mx-auto max-w-5xl px-4 py-6" data-testid="run-view">
      <h1 className="text-2xl font-semibold">Run View</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        默认执行视图：用于运行优化任务与查看实时执行状态。
      </p>

      <div className="mt-6 flex items-center justify-between gap-3">
        <div className="text-sm text-muted-foreground">节点图（基础渲染）</div>
        {import.meta.env.DEV ? (
          <button
            type="button"
            className="rounded-md border px-3 py-1.5 text-sm hover:bg-muted disabled:cursor-not-allowed disabled:opacity-60"
            onClick={handleReplay}
            disabled={isReplaying}
            data-testid="runview-demo-replay"
          >
            回放/模拟运行
          </button>
        ) : null}
      </div>

      <div className="mt-3 grid grid-cols-1 gap-4 lg:grid-cols-3">
        <IterationGraph
          nodeStates={nodeStates}
          className="h-[560px] w-full rounded-lg border bg-white lg:col-span-2"
        />

        <aside className="rounded-lg border bg-white p-4 text-sm text-muted-foreground">
          思考面板（占位）：后续 Story 会在此展示 streaming thinking / stage indicator 等内容。
        </aside>
      </div>
    </section>
  )
}
