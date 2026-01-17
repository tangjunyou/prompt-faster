/**
 * 引导输入组件
 * 允许用户在暂停状态下向老师模型发送引导信息
 */

import { useState, useCallback } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { MessageSquare, Send, Clock, CheckCircle } from 'lucide-react'

/** 引导状态 */
export type GuidanceStatus = 'pending' | 'applied'

/** 用户引导信息 */
export interface UserGuidance {
  /** 引导 ID */
  id: string
  /** 引导内容 */
  content: string
  /** 引导状态 */
  status: GuidanceStatus
  /** 创建时间 */
  createdAt: string
  /** 应用时间 */
  appliedAt?: string
}

export interface GuidanceInputProps {
  /** 任务 ID */
  taskId: string
  /** 发送回调 */
  onSend: (content: string, correlationId: string) => void
  /** 当前引导信息 */
  guidance?: UserGuidance | null
  /** 是否禁用（非 Paused 状态） */
  disabled?: boolean
  /** 是否正在发送 */
  isSending?: boolean
  /** 发送失败错误信息 */
  sendError?: string | null
}

/** 引导内容最大长度 */
const MAX_CONTENT_LENGTH = 2000

/**
 * 引导输入组件
 * 支持在暂停状态下发送引导信息给老师模型
 */
export function GuidanceInput({
  taskId: _taskId,
  onSend,
  guidance,
  disabled = false,
  isSending = false,
  sendError = null,
}: GuidanceInputProps) {
  // taskId 保留用于未来扩展
  void _taskId

  // 输入内容
  const [content, setContent] = useState('')
  // 输入校验错误
  const [validationError, setValidationError] = useState<string | null>(null)

  // 校验内容
  const validateContent = useCallback((value: string): string | null => {
    const trimmed = value.trim()
    if (trimmed.length === 0) {
      return '引导内容不能为空'
    }
    if (trimmed.length > MAX_CONTENT_LENGTH) {
      return `引导内容超过最大长度限制（${MAX_CONTENT_LENGTH} 字符）`
    }
    return null
  }, [])

  // 处理内容变化
  const handleContentChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value
    setContent(value)
    // 清除校验错误（用户正在输入）
    if (validationError) {
      setValidationError(null)
    }
  }, [validationError])

  // 发送引导
  const handleSend = useCallback(() => {
    const error = validateContent(content)
    if (error) {
      setValidationError(error)
      return
    }

    const correlationId = `cid-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
    onSend(content.trim(), correlationId)
    // 发送后清空输入（成功与否由父组件通过 guidance 状态反馈）
    setContent('')
    setValidationError(null)
  }, [content, onSend, validateContent])

  // 处理按键（Enter 发送，Shift+Enter 换行）
  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && !disabled && !isSending) {
      e.preventDefault()
      handleSend()
    }
  }, [disabled, isSending, handleSend])

  // 格式化时间显示
  const formatTime = (isoString: string): string => {
    try {
      const date = new Date(isoString)
      return date.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      })
    } catch {
      return isoString
    }
  }

  // 当前显示的错误信息
  const displayError = sendError ?? validationError

  return (
    <Card className="w-full">
      <CardHeader className="pb-3">
        <CardTitle className="text-lg flex items-center gap-2">
          <MessageSquare className="h-5 w-5" />
          对话引导
        </CardTitle>
        <p className="text-sm text-muted-foreground mt-1">
          告诉老师模型你的想法，引导下一轮优化方向
        </p>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* 已发送的引导信息 */}
        {guidance && (
          <div className="p-3 rounded-lg border bg-muted/30">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">当前引导</span>
              <Badge
                variant={guidance.status === 'applied' ? 'default' : 'secondary'}
                className="flex items-center gap-1"
              >
                {guidance.status === 'applied' ? (
                  <>
                    <CheckCircle className="h-3 w-3" />
                    已应用
                  </>
                ) : (
                  <>
                    <Clock className="h-3 w-3" />
                    等待应用
                  </>
                )}
              </Badge>
            </div>
            <p className="text-sm whitespace-pre-wrap break-words">
              {guidance.content}
            </p>
            <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
              <span>发送于 {formatTime(guidance.createdAt)}</span>
              {guidance.appliedAt && (
                <span>应用于 {formatTime(guidance.appliedAt)}</span>
              )}
            </div>
          </div>
        )}

        {/* 输入区域 */}
        <div className="space-y-2">
          <Textarea
            placeholder="告诉老师模型你的想法..."
            value={content}
            onChange={handleContentChange}
            onKeyDown={handleKeyDown}
            disabled={disabled || isSending}
            className="min-h-[80px] resize-none"
            maxLength={MAX_CONTENT_LENGTH + 100} // 允许稍微超出以显示错误
          />
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              {content.length}/{MAX_CONTENT_LENGTH} 字符
              {!disabled && ' • Enter 发送，Shift+Enter 换行'}
            </span>
            <Button
              size="sm"
              onClick={handleSend}
              disabled={disabled || isSending || content.trim().length === 0}
              className="min-w-[44px] min-h-[44px]"
            >
              <Send className="h-4 w-4 mr-1" />
              {isSending ? '发送中...' : '发送'}
            </Button>
          </div>
        </div>

        {/* 错误提示 */}
        {displayError && (
          <p className="text-sm text-destructive">⚠️ {displayError}</p>
        )}

        {/* 成功提示 */}
        {guidance && guidance.status === 'pending' && !displayError && (
          <p className="text-sm text-emerald-600">
            ✅ 引导已保存，将在下一轮迭代生效
          </p>
        )}

        {/* 禁用状态提示 */}
        {disabled && (
          <p className="text-sm text-amber-600">
            ⚠️ 请先暂停任务再发送引导
          </p>
        )}
      </CardContent>
    </Card>
  )
}

export default GuidanceInput
