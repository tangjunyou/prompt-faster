import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { IterationGraph } from '@/components/nodes/IterationGraph'
import type { IterationGraphEdgeFlowStates, IterationGraphNodeStates } from '@/components/nodes/types'
import { usePrefersReducedMotion } from '@/hooks'
import { createDeterministicDemoWsMessages } from '@/features/ws-demo/demoWsMessages'
import {
  createInitialIterationGraphEdgeFlowStates,
  createInitialIterationGraphNodeStates,
  reduceDemoWsMessageToIterationGraphNodeStates,
} from '@/features/visualization/iterationGraphDemoReducer'
import {
  createIterationGraphEdgeFlowMachine,
  type IterationGraphEdgeFlowMachine,
} from '@/features/visualization/iterationGraphEdgeFlowMachine'

type Result = {
  frames: number
  durationMs: number
  fps: number
}

export function PerfNfr3View() {
  const [result, setResult] = useState<Result | null>(null)
  const [isRunning, setIsRunning] = useState(false)
  const prefersReducedMotion = usePrefersReducedMotion()
  const [nodeStates, setNodeStates] = useState<IterationGraphNodeStates>(() =>
    createInitialIterationGraphNodeStates(),
  )
  const [edgeFlowStates, setEdgeFlowStates] = useState<IterationGraphEdgeFlowStates>(() =>
    createInitialIterationGraphEdgeFlowStates(),
  )
  const rafId = useRef<number | null>(null)
  const abortControllerRef = useRef<AbortController | null>(null)
  const isRunningRef = useRef(false)
  const edgeFlowMachineRef = useRef<IterationGraphEdgeFlowMachine | null>(null)

  const demoMessages = useMemo(
    () =>
      createDeterministicDemoWsMessages({
        correlationId: 'nfr3-demo',
        iterations: 2,
        tokensPerIteration: 80,
      }),
    [],
  )

  const stop = useCallback(() => {
    abortControllerRef.current?.abort()
    abortControllerRef.current = null

    if (rafId.current != null) {
      cancelAnimationFrame(rafId.current)
      rafId.current = null
    }

    isRunningRef.current = false
    setIsRunning(false)
    edgeFlowMachineRef.current?.reset()
  }, [])

  useEffect(() => {
    edgeFlowMachineRef.current = createIterationGraphEdgeFlowMachine(setEdgeFlowStates)
    return () => {
      edgeFlowMachineRef.current?.dispose()
      stop()
    }
  }, [stop])

  const run = async () => {
    if (isRunningRef.current) return
    isRunningRef.current = true
    setIsRunning(true)
    setResult(null)
    setNodeStates(createInitialIterationGraphNodeStates())
    edgeFlowMachineRef.current?.reset()

    abortControllerRef.current?.abort()
    const abortController = new AbortController()
    abortControllerRef.current = abortController
    const { signal } = abortController

    const start = performance.now()
    const durationMs = 1000
    let frames = 0
    let idx = 0

    await new Promise<void>((resolve) => {
      signal.addEventListener('abort', () => resolve(), { once: true })

      const tick = () => {
        if (signal.aborted) {
          resolve()
          return
        }

        frames += 1
        const msg = demoMessages[idx]
        if (msg) {
          idx += 1
          setNodeStates((prev) => reduceDemoWsMessageToIterationGraphNodeStates(prev, msg))
          edgeFlowMachineRef.current?.applyDemoWsMessage(msg, { prefersReducedMotion })
        } else {
          idx = 0
          setNodeStates(createInitialIterationGraphNodeStates())
          edgeFlowMachineRef.current?.reset()
        }
        const now = performance.now()
        if (now - start >= durationMs) {
          resolve()
          return
        }
        rafId.current = requestAnimationFrame(tick)
      }
      rafId.current = requestAnimationFrame(tick)
    })

    if (abortControllerRef.current === abortController) {
      abortControllerRef.current = null
    }

    if (signal.aborted) {
      isRunningRef.current = false
      setIsRunning(false)
      return
    }

    const end = performance.now()
    const d = end - start
    setResult({
      frames,
      durationMs: d,
      fps: frames / (d / 1000),
    })
    isRunningRef.current = false
    setIsRunning(false)
  }

  return (
    <div className="mx-auto max-w-3xl p-6">
      <h1 className="text-xl font-semibold">NFR3：节点图渲染性能（口径预置）</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        该页面用于 NFR3 的 FPS 测量口径与回归入口（纯本地、确定性、不出网）：真实节点图渲染 + 确定性状态更新。
      </p>

      <div className="mt-6 flex items-center gap-3">
        <button
          className="rounded border px-3 py-2 text-sm"
          onClick={run}
          disabled={isRunning}
          data-testid="nfr3-run"
        >
          {isRunning ? '测量中...' : '运行测量'}
        </button>
        <button
          className="rounded border px-3 py-2 text-sm"
          onClick={stop}
          disabled={!isRunning}
          data-testid="nfr3-stop"
        >
          停止
        </button>
      </div>

      <div className="mt-6 rounded border p-4 text-sm" data-testid="nfr3-result">
        <div>fps: {result ? result.fps.toFixed(2) : '-'}</div>
        <div>frames: {result ? result.frames : '-'}</div>
        <div>durationMs: {result ? result.durationMs.toFixed(2) : '-'}</div>
      </div>

      <IterationGraph
        nodeStates={nodeStates}
        edgeFlowStates={edgeFlowStates}
        prefersReducedMotion={prefersReducedMotion}
        className="mt-6 h-[560px] w-full rounded-lg border bg-white"
      />
    </div>
  )
}
