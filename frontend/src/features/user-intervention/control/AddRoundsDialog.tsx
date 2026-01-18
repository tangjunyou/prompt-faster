/**
 * 增加轮数对话框
 *
 * 允许用户在任务运行中/暂停状态下增加迭代轮数
 */

import { useState, useCallback } from 'react'
import { Plus, Minus } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useAddRounds } from './hooks/useIterationControl'

/** 最小增加轮数 */
const MIN_ROUNDS = 1
/** 最大增加轮数 */
const MAX_ROUNDS = 100
/** 默认增加轮数 */
const DEFAULT_ROUNDS = 5

export interface AddRoundsDialogProps {
  /** 任务 ID */
  taskId: string
  /** 工作区 ID（用于刷新任务数据） */
  workspaceId?: string
  /** 是否打开 */
  open: boolean
  /** 关闭回调 */
  onOpenChange: (open: boolean) => void
  /** 当前最大轮数 */
  currentMaxIterations?: number
  /** 当前轮次 */
  currentRound?: number
  /** 成功回调 */
  onSuccess?: () => void
}

export function AddRoundsDialog({
  taskId,
  workspaceId,
  open,
  onOpenChange,
  currentMaxIterations = 0,
  currentRound = 0,
  onSuccess,
}: AddRoundsDialogProps) {
  const [additionalRounds, setAdditionalRounds] = useState(DEFAULT_ROUNDS)
  const { mutate, isPending, error } = useAddRounds(taskId, workspaceId)

  const handleIncrement = useCallback(() => {
    setAdditionalRounds((prev) => Math.min(prev + 1, MAX_ROUNDS))
  }, [])

  const handleDecrement = useCallback(() => {
    setAdditionalRounds((prev) => Math.max(prev - 1, MIN_ROUNDS))
  }, [])

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value, 10)
    if (!isNaN(value)) {
      setAdditionalRounds(Math.max(MIN_ROUNDS, Math.min(value, MAX_ROUNDS)))
    }
  }, [])

  const handleSubmit = useCallback(() => {
    mutate(additionalRounds, {
      onSuccess: () => {
        onOpenChange(false)
        setAdditionalRounds(DEFAULT_ROUNDS)
        onSuccess?.()
      },
    })
  }, [additionalRounds, mutate, onOpenChange, onSuccess])

  const newMaxIterations = currentMaxIterations + additionalRounds

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>增加迭代轮数</DialogTitle>
          <DialogDescription>
            当前已执行 {currentRound} 轮，最大轮数 {currentMaxIterations}。
            增加轮数后将继续优化。
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="rounds" className="text-right">
              增加轮数
            </Label>
            <div className="col-span-3 flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleDecrement}
                disabled={additionalRounds <= MIN_ROUNDS || isPending}
                className="h-10 w-10"
                aria-label="减少轮数"
              >
                <Minus className="h-4 w-4" />
              </Button>
              <Input
                id="rounds"
                type="number"
                min={MIN_ROUNDS}
                max={MAX_ROUNDS}
                value={additionalRounds}
                onChange={handleInputChange}
                disabled={isPending}
                className="w-20 text-center"
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleIncrement}
                disabled={additionalRounds >= MAX_ROUNDS || isPending}
                className="h-10 w-10"
                aria-label="增加轮数"
              >
                <Plus className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="text-sm text-muted-foreground text-center">
            新的最大轮数: <span className="font-medium">{newMaxIterations}</span>
          </div>

          {error && (
            <div className="text-sm text-destructive text-center space-y-2">
              <div>{error.message}</div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={handleSubmit}
                disabled={isPending || additionalRounds < MIN_ROUNDS}
              >
                重试
              </Button>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isPending}
          >
            取消
          </Button>
          <Button
            type="button"
            onClick={handleSubmit}
            disabled={isPending || additionalRounds < MIN_ROUNDS}
          >
            {isPending ? '提交中...' : '确认增加'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default AddRoundsDialog
