/**
 * 暂停/继续控制组件
 *
 * 功能：
 * - 暂停/继续按钮（点击区域 ≥ 44px × 44px，符合 WCAG 2.1 AA）
 * - Space 快捷键支持
 * - 根据任务状态自动切换按钮显示
 */

import { useCallback, useEffect } from 'react'
import { Pause, Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTaskStore, generateCorrelationId } from '@/stores/useTaskStore'
import type { RunControlState } from '@/types/generated/models/RunControlState'

export interface PauseResumeControlProps {
  /** 任务 ID */
  taskId: string
  /** 发送暂停命令的回调 */
  onPause?: (taskId: string, correlationId: string) => void
  /** 发送继续命令的回调 */
  onResume?: (taskId: string, correlationId: string) => void
  /** 是否禁用（如任务未启动） */
  disabled?: boolean
  /** 自定义类名 */
  className?: string
}

/**
 * 暂停/继续控制按钮
 */
export function PauseResumeControl({
  taskId,
  onPause,
  onResume,
  disabled = false,
  className,
}: PauseResumeControlProps) {
  const { taskStates, canPause, canResume, setCorrelationId } = useTaskStore()
  const taskState = taskStates[taskId]
  const runControlState: RunControlState = taskState?.runControlState ?? 'idle'

  const isPaused = runControlState === 'paused'
  const isRunning = runControlState === 'running'
  const canClickPause = canPause(taskId) && !disabled
  const canClickResume = canResume(taskId) && !disabled

  // 处理暂停点击
  const handlePause = useCallback(() => {
    if (!canClickPause) return
    const correlationId = generateCorrelationId()
    setCorrelationId(correlationId)
    onPause?.(taskId, correlationId)
  }, [canClickPause, taskId, onPause, setCorrelationId])

  // 处理继续点击
  const handleResume = useCallback(() => {
    if (!canClickResume) return
    const correlationId = generateCorrelationId()
    setCorrelationId(correlationId)
    onResume?.(taskId, correlationId)
  }, [canClickResume, taskId, onResume, setCorrelationId])

  // 处理点击（根据当前状态切换）
  const handleClick = useCallback(() => {
    if (isPaused) {
      handleResume()
    } else if (isRunning) {
      handlePause()
    }
  }, [isPaused, isRunning, handlePause, handleResume])

  // Space 快捷键支持
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null
      const isEditable =
        target?.isContentEditable ||
        !!target?.closest?.('input, textarea, [role=\"textbox\"], [contenteditable=\"true\"]') ||
        !!target?.closest?.('.monaco-editor, .cm-editor')

      if (isEditable) return

      if (event.code === 'Space' && !disabled) {
        event.preventDefault()
        handleClick()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [disabled, handleClick])

  // 根据状态决定按钮外观
  const buttonVariant = isPaused ? 'default' : 'outline'
  const Icon = isPaused ? Play : Pause
  const label = isPaused ? '继续' : '暂停'
  const shortcutHint = '(Space)'
  const isButtonDisabled = disabled || (!canClickPause && !canClickResume)

  // 构建 title 提示文本
  const titleText = isPaused && taskState?.pausedStage
    ? `${label}优化任务 ${shortcutHint} - 暂停于: ${taskState.pausedStage}`
    : `${label}优化任务 ${shortcutHint}`

  return (
    <Button
      variant={buttonVariant}
      size="lg"
      onClick={handleClick}
      disabled={isButtonDisabled}
      className={`min-w-[44px] min-h-[44px] ${className ?? ''}`}
      aria-label={titleText}
      title={titleText}
    >
      <Icon className="h-5 w-5 mr-2" />
      {label}
    </Button>
  )
}

export default PauseResumeControl
