import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router'

import { IterationGraph } from '@/components/nodes/IterationGraph'
import { StageHistoryPanel, StageIndicator, StreamingText } from '@/components/streaming'
import type { IterationGraphEdgeFlowStates, IterationGraphNodeStates } from '@/components/nodes/types'
import { usePrefersReducedMotion, useWebSocket } from '@/hooks'
import { ArtifactEditor, GuidanceInput, PauseResumeControl, HistoryPanel } from '@/features/user-intervention'
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
import type { IterationArtifacts } from '@/types/generated/models/IterationArtifacts'
import type { IterationPausedPayload, IterationResumedPayload } from '@/types/generated/ws'
import type { ArtifactGetAckPayload, ArtifactUpdateAckPayload, ArtifactUpdatedPayload } from '@/types/generated/ws'
import type { GuidanceSendAckPayload, GuidanceSentPayload, GuidanceAppliedPayload } from '@/types/generated/ws'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

export function RunView() {
  const navigate = useNavigate()
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

  const {
    taskStates,
    setRunControlState,
    handlePaused,
    handleResumed,
    setArtifacts,
    canEditArtifacts,
    canSendGuidance,
    setSendingGuidance,
    setGuidanceError,
    handleGuidanceSent,
    handleGuidanceApplied,
  } = useTaskStore()
  const runControlState = taskStates[taskId]?.runControlState ?? 'idle'
  const isPaused = runControlState === 'paused'
  const artifacts = taskStates[taskId]?.artifacts
  const userGuidance = taskStates[taskId]?.userGuidance
  const isSendingGuidance = taskStates[taskId]?.isSendingGuidance ?? false
  const guidanceError = taskStates[taskId]?.guidanceError ?? null
  const [isSavingArtifacts, setIsSavingArtifacts] = useState(false)
  const [artifactSaveError, setArtifactSaveError] = useState<string | null>(null)
  const [artifactSaveSuccessVisible, setArtifactSaveSuccessVisible] = useState(false)
  const [artifactSaveResetVersion, setArtifactSaveResetVersion] = useState(0)
  const artifactSaveTimerRef = useRef<number | null>(null)
  const [rightPanelTab, setRightPanelTab] = useState<'current' | 'history'>('current')

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

  // 处理产物获取 ACK
  const handleArtifactGetAck = useCallback(
    (payload: ArtifactGetAckPayload) => {
      if (payload.taskId !== taskId) return
      if (payload.ok && payload.artifacts) {
        setArtifacts(taskId, payload.artifacts)
      }
    },
    [taskId, setArtifacts],
  )

  // 处理产物更新 ACK
  const handleArtifactUpdateAck = useCallback(
    (payload: ArtifactUpdateAckPayload) => {
      if (payload.taskId !== taskId) return
      setIsSavingArtifacts(false)
      if (payload.ok && payload.artifacts) {
        setArtifacts(taskId, payload.artifacts)
        setArtifactSaveError(null)
        setArtifactSaveResetVersion((prev) => prev + 1)
        setArtifactSaveSuccessVisible(true)
        if (artifactSaveTimerRef.current) {
          window.clearTimeout(artifactSaveTimerRef.current)
        }
        artifactSaveTimerRef.current = window.setTimeout(
          () => setArtifactSaveSuccessVisible(false),
          3000,
        )
      } else {
        setArtifactSaveError(payload.reason || '保存失败，请稍后重试')
        setArtifactSaveSuccessVisible(false)
      }
    },
    [taskId, setArtifacts],
  )

  // 处理产物已更新事件（广播）
  const handleArtifactUpdated = useCallback(
    (payload: ArtifactUpdatedPayload) => {
      if (payload.taskId !== taskId) return
      setArtifacts(taskId, payload.artifacts)
    },
    [taskId, setArtifacts],
  )

  // 处理引导发送 ACK
  const handleGuidanceSendAck = useCallback(
    (payload: GuidanceSendAckPayload) => {
      if (payload.taskId !== taskId) return
      setSendingGuidance(taskId, false)
      if (!payload.ok) {
        const reason = payload.reason ?? undefined
        const message = reason === 'task_not_paused'
          ? '任务未暂停，无法发送引导。请先暂停任务再发送。'
          : reason === 'task_not_found_or_forbidden'
            ? '无法找到任务或权限不足。请刷新后重试，或确认任务归属。'
            : reason === 'missing_task_id'
              ? '未识别到任务，请刷新页面后重试。'
              : '发送失败，请稍后重试。'
        setGuidanceError(taskId, message)
      }
    },
    [taskId, setSendingGuidance, setGuidanceError],
  )

  // 处理引导已发送事件（广播）
  const handleGuidanceSentEvent = useCallback(
    (payload: GuidanceSentPayload) => {
      if (payload.taskId !== taskId) return
      const status = payload.status === 'applied' ? 'applied' : 'pending'
      handleGuidanceSent(
        taskId,
        payload.guidanceId,
        payload.contentPreview,
        status,
        payload.createdAt,
      )
    },
    [taskId, handleGuidanceSent],
  )

  // 处理引导已应用事件（广播）
  const handleGuidanceAppliedEvent = useCallback(
    (payload: GuidanceAppliedPayload) => {
      if (payload.taskId !== taskId) return
      handleGuidanceApplied(taskId, payload.guidanceId, payload.appliedAt)
    },
    [taskId, handleGuidanceApplied],
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
      // 处理产物相关事件
      if (message.type === 'artifact:get:ack') {
        handleArtifactGetAck(message.payload as ArtifactGetAckPayload)
      }
      if (message.type === 'artifact:update:ack') {
        handleArtifactUpdateAck(message.payload as ArtifactUpdateAckPayload)
      }
      if (message.type === 'artifact:updated') {
        handleArtifactUpdated(message.payload as ArtifactUpdatedPayload)
      }
      // 处理引导相关事件
      if (message.type === 'guidance:send:ack') {
        handleGuidanceSendAck(message.payload as GuidanceSendAckPayload)
      }
      if (message.type === 'guidance:sent') {
        handleGuidanceSentEvent(message.payload as GuidanceSentPayload)
      }
      if (message.type === 'guidance:applied') {
        handleGuidanceAppliedEvent(message.payload as GuidanceAppliedPayload)
      }
    },
  })

  // 暂停时自动获取产物
  useEffect(() => {
    if (isPaused && isConnected && !artifacts) {
      const correlationId = `cid-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
      sendCommand('artifact:get', { taskId }, correlationId)
    }
  }, [isPaused, isConnected, artifacts, taskId, sendCommand])

  // 保存产物
  const handleSaveArtifacts = useCallback(
    (updatedArtifacts: IterationArtifacts, correlationId: string) => {
      setIsSavingArtifacts(true)
      setArtifactSaveError(null)
      sendCommand('artifact:update', { taskId, artifacts: updatedArtifacts }, correlationId)
    },
    [taskId, sendCommand],
  )

  // 发送引导
  const handleSendGuidance = useCallback(
    (content: string, correlationId: string) => {
      setSendingGuidance(taskId, true)
      setGuidanceError(taskId, null)
      sendCommand('guidance:send', { taskId, content }, correlationId)
    },
    [taskId, sendCommand, setSendingGuidance, setGuidanceError],
  )

  const handleStartOptimization = useCallback(() => {
    navigate('/workspace')
  }, [navigate])

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
          className="flex h-[560px] flex-col rounded-lg border bg-white lg:col-span-1"
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
            className="m-2 flex-1"
          />
          <StageHistoryPanel
            history={thinkingState.stageHistory}
            prefersReducedMotion={prefersReducedMotion}
            className="border-t"
          />
          <div className="border-t p-3">
            <Tabs
              value={rightPanelTab}
              onValueChange={(value) => setRightPanelTab(value as 'current' | 'history')}
            >
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="current">当前产物</TabsTrigger>
                <TabsTrigger value="history">历史记录</TabsTrigger>
              </TabsList>
              <TabsContent value="current" className="mt-3 max-h-[280px] overflow-y-auto">
                <ArtifactEditor
                  key={`${taskId}-${artifactSaveResetVersion}`}
                  taskId={taskId}
                  artifacts={artifacts}
                  onSave={handleSaveArtifacts}
                  disabled={!canEditArtifacts(taskId)}
                  isSaving={isSavingArtifacts}
                  saveError={artifactSaveError}
                  showSuccess={artifactSaveSuccessVisible}
                />
              </TabsContent>
              <TabsContent value="history" className="mt-3 max-h-[280px] overflow-y-auto">
                <HistoryPanel taskId={taskId} onStartOptimization={handleStartOptimization} />
              </TabsContent>
            </Tabs>
          </div>
        </aside>
      </div>

      {/* 对话引导输入 */}
      <div className="mt-6">
        <GuidanceInput
          taskId={taskId}
          onSend={handleSendGuidance}
          guidance={userGuidance}
          disabled={!canSendGuidance(taskId)}
          isSending={isSendingGuidance}
          sendError={guidanceError}
        />
      </div>

    </section>
  )
}
