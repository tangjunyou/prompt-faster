/**
 * WebSocket hook - 暂停/继续最小实现
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import { useAuthStore } from '@/stores/useAuthStore'
import type {
  IterationPausedPayload,
  IterationResumedPayload,
  TaskControlAckPayload,
  TaskControlPayload,
} from '@/types/generated/ws'

export type WsMessage<T> = {
  type: string
  payload: T
  timestamp: string
  correlationId: string
}

export type UseWebSocketOptions = {
  onPaused?: (payload: IterationPausedPayload, correlationId: string) => void
  onResumed?: (payload: IterationResumedPayload, correlationId: string) => void
  onAck?: (payload: TaskControlAckPayload, correlationId: string) => void
  onMessage?: (message: WsMessage<unknown>) => void
}

function buildWsUrl(token: string): string {
  const explicit = import.meta.env.VITE_WS_URL
  const base = explicit || import.meta.env.VITE_API_URL || 'http://localhost:3000/api/v1'
  const url = new URL(base)
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'

  if (!url.pathname.endsWith('/ws')) {
    url.pathname = `${url.pathname.replace(/\/$/, '')}/ws`
  }

  url.searchParams.set('token', token)
  return url.toString()
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const { onPaused, onResumed, onAck, onMessage } = options
  const [isConnected, setIsConnected] = useState(false)
  const socketRef = useRef<WebSocket | null>(null)
  const token = useAuthStore((state) => state.sessionToken)

  useEffect(() => {
    if (!token) return
    if (typeof WebSocket === 'undefined') return

    const ws = new WebSocket(buildWsUrl(token))
    socketRef.current = ws

    ws.onopen = () => setIsConnected(true)
    ws.onclose = () => setIsConnected(false)
    ws.onerror = () => setIsConnected(false)

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WsMessage<unknown>
        if (!message || typeof message !== 'object') return
        if (!message.type || typeof message.type !== 'string') return
        if (!message.correlationId || typeof message.correlationId !== 'string') return
        onMessage?.(message)

        if (message.type === 'iteration:paused') {
          onPaused?.(message.payload as IterationPausedPayload, message.correlationId)
        }
        if (message.type === 'iteration:resumed') {
          onResumed?.(message.payload as IterationResumedPayload, message.correlationId)
        }
        if (message.type === 'task:pause:ack' || message.type === 'task:resume:ack') {
          onAck?.(message.payload as TaskControlAckPayload, message.correlationId)
        }
      } catch (err) {
        console.warn('[WebSocket] failed to parse message', err)
      }
    }

    return () => {
      ws.close()
      socketRef.current = null
    }
  }, [token, onPaused, onResumed, onAck, onMessage])

  const sendCommand = useCallback(
    <T = TaskControlPayload>(type: string, payload: T, correlationId: string) => {
      const socket = socketRef.current
      if (!socket || socket.readyState !== WebSocket.OPEN) return false
      const message = { type, payload, correlationId }
      socket.send(JSON.stringify(message))
      return true
    },
    [],
  )

  return { isConnected, sendCommand }
}

export default useWebSocket
