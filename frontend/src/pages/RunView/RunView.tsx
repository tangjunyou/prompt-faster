import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { IterationGraph } from '@/components/nodes/IterationGraph'
import { StageHistoryPanel, StageIndicator, StreamingText } from '@/components/streaming'
import type { IterationGraphEdgeFlowStates, IterationGraphNodeStates } from '@/components/nodes/types'
import { usePrefersReducedMotion, useWebSocket } from '@/hooks'
import { PauseResumeControl } from '@/features/user-intervention'
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
import {
  createInitialThinkingStreamState,
  reduceThinkingStreamState,
  forceCompleteThinkingStreamState,
  setAutoScrollLocked,
  type ThinkingStreamState,
} from '@/features/visualization/thinkingStreamReducer'
import { useTaskStore } from '@/stores/useTaskStore'
import type { IterationPausedPayload, IterationResumedPayload } from '@/types/generated/ws'

export function RunView() {
  const taskId = useMemo(() => {
    const params = new URLSearchParams(window.location.search)
    return params.get('taskId') ?? 'demo-task'
  }, [])

  const demoMessages = useMemo(
    () =>
      createDeterministicDemoWsMessages({
        correlationId: 'runview-demo',
        iterations: 1,
        tokensPerIteration: 30,
      }),
    [],
  )

  const prefersReducedMotion = usePrefersReducedMotion()

  const [nodeStates, setNodeStates] = useState<IterationGraphNodeStates>(() =>
    createInitialIterationGraphNodeStates(),
  )
  const [edgeFlowStates, setEdgeFlowStates] = useState<IterationGraphEdgeFlowStates>(() =>
    createInitialIterationGraphEdgeFlowStates(),
  )
  const [thinkingState, setThinkingState] = useState<ThinkingStreamState>(() =>
    createInitialThinkingStreamState(),
  )
  const [isReplaying, setIsReplaying] = useState(false)
  const replayTimerRef = useRef<number | null>(null)
  const edgeFlowMachineRef = useRef<IterationGraphEdgeFlowMachine | null>(null)
  const pendingThinkingMessagesRef = useRef(demoMessages.slice(0, 0))
  const thinkingFlushRafRef = useRef<number | null>(null)
  const pausedNodeStatesRef = useRef<IterationGraphNodeStates | null>(null)
  const isPausedRef = useRef(false)

  const { taskStates, setRunControlState, handlePaused, handleResumed } = useTaskStore()
  const runControlState = taskStates[taskId]?.runControlState ?? 'idle'
  const isPaused = runControlState === 'paused'

  const handlePausedEvent = useCallback(
    (payload: IterationPausedPayload) => {
      if (payload.taskId !== taskId) return
      handlePaused(payload.taskId, payload.pausedAt, payload.stage, payload.iteration)
      setNodeStates((prev) => {
        if (!isPausedRef.current) {
          pausedNodeStatesRef.current = prev
        }
        isPausedRef.current = true
        return {
          pattern_extractor: 'paused',
          prompt_engineer: 'paused',
          quality_assessor: 'paused',
          reflection_agent: 'paused',
        }
      })
    },
    [taskId, handlePaused],
  )

  const handleResumedEvent = useCallback(
    (payload: IterationResumedPayload) => {
      if (payload.taskId !== taskId) return
      handleResumed(payload.taskId)
      setNodeStates((prev) => {
        const restored = pausedNodeStatesRef.current ?? prev
        pausedNodeStatesRef.current = null
        isPausedRef.current = false
        return restored
      })
    },
    [taskId, handleResumed],
  )

  const { isConnected, sendCommand } = useWebSocket({
    onPaused: handlePausedEvent,
    onResumed: handleResumedEvent,
    onMessage: (message) => {
      if (
        message.type === 'iteration:started' ||
        message.type === 'iteration:progress' ||
        message.type === 'iteration:resumed'
      ) {
        setRunControlState(taskId, 'running')
      }
    },
  })

  useEffect(() => {
    edgeFlowMachineRef.current = createIterationGraphEdgeFlowMachine(setEdgeFlowStates)
    return () => {
      edgeFlowMachineRef.current?.dispose()
      if (replayTimerRef.current !== null) {
        window.clearInterval(replayTimerRef.current)
      }
      if (thinkingFlushRafRef.current !== null) {
        cancelAnimationFrame(thinkingFlushRafRef.current)
        thinkingFlushRafRef.current = null
      }
    }
  }, [])

  function scheduleThinkingFlush() {
    if (thinkingFlushRafRef.current !== null) return
    thinkingFlushRafRef.current = requestAnimationFrame(() => {
      thinkingFlushRafRef.current = null
      const pending = pendingThinkingMessagesRef.current
      if (pending.length === 0) return
      pendingThinkingMessagesRef.current = []
      setThinkingState((prev) =>
        pending.reduce((state, msg) => reduceThinkingStreamState(state, msg), prev),
      )
    })
  }

  function handleReplay() {
    if (replayTimerRef.current !== null) {
      window.clearInterval(replayTimerRef.current)
      replayTimerRef.current = null
    }

    setIsReplaying(true)
    setNodeStates(createInitialIterationGraphNodeStates())
    setThinkingState(createInitialThinkingStreamState())
    edgeFlowMachineRef.current?.reset()
    pendingThinkingMessagesRef.current = []

    let idx = 0
    replayTimerRef.current = window.setInterval(() => {
      const msg = demoMessages[idx]
      if (!msg) {
        if (replayTimerRef.current !== null) {
          window.clearInterval(replayTimerRef.current)
          replayTimerRef.current = null
        }
        // 兜底：回放结束时若 reducer 未置位 complete，强制完成
        setThinkingState((prev) => forceCompleteThinkingStreamState(prev))
        setIsReplaying(false)
        return
      }
      idx += 1
      setNodeStates((prev) => reduceDemoWsMessageToIterationGraphNodeStates(prev, msg))
      pendingThinkingMessagesRef.current = [...pendingThinkingMessagesRef.current, msg]
      scheduleThinkingFlush()
      edgeFlowMachineRef.current?.applyDemoWsMessage(msg, { prefersReducedMotion })
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
        <div className="flex items-center gap-2">
          <PauseResumeControl
            taskId={taskId}
            onPause={(id, correlationId) =>
              sendCommand('task:pause', { taskId: id }, correlationId)
            }
            onResume={(id, correlationId) =>
              sendCommand('task:resume', { taskId: id }, correlationId)
            }
            disabled={!isConnected}
          />
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
      </div>

      <div className="mt-3 grid grid-cols-1 gap-4 lg:grid-cols-3">
        <IterationGraph
          nodeStates={nodeStates}
          edgeFlowStates={edgeFlowStates}
          prefersReducedMotion={prefersReducedMotion}
          className="h-[560px] w-full rounded-lg border bg-white lg:col-span-2"
        />

        <aside
          className="flex flex-col rounded-lg border bg-white lg:col-span-1"
          data-testid="thinking-panel"
        >
          <div className="border-b px-4 py-2">
            <div className="text-sm font-medium text-muted-foreground">思考过程</div>
            {isPaused ? (
              <div
                className="mt-2 inline-flex items-center rounded-full bg-yellow-50 px-3 py-1 text-xs font-medium text-yellow-900"
                data-testid="thinking-paused-indicator"
              >
                已暂停
              </div>
            ) : null}
            <StageIndicator
              stage={thinkingState.currentStage}
              prefersReducedMotion={prefersReducedMotion}
              className="mt-2"
            />
          </div>
          <StreamingText
            text={thinkingState.text}
            status={thinkingState.status}
            isTruncated={thinkingState.isTruncated}
            maxChars={thinkingState.maxChars}
            maxLines={thinkingState.maxLines}
            isAutoScrollLocked={thinkingState.isAutoScrollLocked}
            onAutoScrollLockedChange={(isLocked) =>
              setThinkingState((prev) => setAutoScrollLocked(prev, isLocked))
            }
            prefersReducedMotion={prefersReducedMotion}
            className="m-2 h-[500px] flex-1"
          />
          <StageHistoryPanel
            history={thinkingState.stageHistory}
            prefersReducedMotion={prefersReducedMotion}
            className="border-t"
          />
        </aside>
      </div>
    </section>
  )
}
