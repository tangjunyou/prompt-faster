/**
 * 回滚确认对话框
 */

import { AlertTriangle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import type { CheckpointSummary } from '@/types/generated/models/CheckpointSummary'
import { formatCheckpointTime, formatPassRateSummary } from '@/lib/formatters'

export interface RollbackConfirmDialogProps {
  open: boolean
  checkpoint: CheckpointSummary | null
  isSubmitting?: boolean
  onConfirm: () => void
  onCancel: () => void
}

export function RollbackConfirmDialog({
  open,
  checkpoint,
  isSubmitting,
  onConfirm,
  onCancel,
}: RollbackConfirmDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(value) => (value ? undefined : onCancel())}>
      <DialogContent className="sm:max-w-[520px]">
        <DialogHeader>
          <DialogTitle>确认回滚</DialogTitle>
          <DialogDescription>
            请确认要回滚到所选的历史 Checkpoint。
          </DialogDescription>
        </DialogHeader>

        {checkpoint ? (
          <div className="rounded-md border p-3 text-xs space-y-1">
            <div className="flex items-center justify-between">
              <span className="font-mono text-[11px] text-muted-foreground">
                迭代 #{checkpoint.iteration}
              </span>
              <span className="text-[11px] text-muted-foreground">
                {formatCheckpointTime(checkpoint.createdAt)}
              </span>
            </div>
            <div className="text-[11px] text-muted-foreground">
              {formatPassRateSummary(checkpoint.passRateSummary)}
            </div>
          </div>
        ) : (
          <p className="text-xs text-muted-foreground">未选择 Checkpoint</p>
        )}

        <div className="flex items-start gap-2 rounded-md border border-amber-200/80 bg-amber-50 p-3 text-xs text-amber-700">
          <AlertTriangle className="h-4 w-4 mt-0.5" />
          <div>
            <p className="font-medium">回滚后，该 Checkpoint 之后的状态将被归档</p>
            <p className="mt-1">归档的 Checkpoint 仅可查看，无法再次回滚。</p>
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button variant="outline" onClick={onCancel}>
            取消
          </Button>
          <Button
            variant="destructive"
            onClick={onConfirm}
            disabled={!checkpoint || isSubmitting}
          >
            {isSubmitting ? '回滚中...' : '确认回滚'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default RollbackConfirmDialog
