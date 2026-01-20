/**
 * 失败用例对比对话框
 */

import { lazy, Suspense } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import type { FailedCaseDetail } from '@/types/generated/models/FailedCaseDetail'

const MonacoDiffEditor = lazy(async () => {
  const mod = await import('@monaco-editor/react')
  return { default: mod.DiffEditor }
})

export interface CaseComparisonDialogProps {
  open: boolean
  caseDetail?: FailedCaseDetail | null
  isLoading?: boolean
  error?: Error | null
  onClose: () => void
}

export function CaseComparisonDialog({ open, caseDetail, isLoading, error, onClose }: CaseComparisonDialogProps) {
  const editorFallback = (
    <div className="rounded-lg border p-4 text-sm text-muted-foreground">正在加载对比视图...</div>
  )

  return (
    <Dialog open={open} onOpenChange={(value) => (value ? undefined : onClose())}>
      <DialogContent className="sm:max-w-[820px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>失败用例对比</DialogTitle>
          <DialogDescription>查看输入、期望与实际输出的差异</DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="text-sm text-muted-foreground">正在加载用例详情...</div>
        ) : error ? (
          <div className="text-sm text-destructive">加载失败：{error.message}</div>
        ) : caseDetail ? (
          <div className="space-y-4">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="secondary">第 {caseDetail.iterationRound} 轮</Badge>
              <span className="text-xs text-muted-foreground">{caseDetail.failureReason}</span>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">输入</div>
              <pre className="whitespace-pre-wrap rounded-lg border bg-muted/30 p-3 text-xs">
                {caseDetail.input}
              </pre>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">期望输出</div>
              <pre className="whitespace-pre-wrap rounded-lg border bg-muted/30 p-3 text-xs">
                {caseDetail.expectedOutput ?? '暂无期望输出'}
              </pre>
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">实际输出</div>
              {caseDetail.actualOutput ? (
                <pre className="whitespace-pre-wrap rounded-lg border bg-muted/30 p-3 text-xs">
                  {caseDetail.actualOutput}
                </pre>
              ) : (
                <div className="text-xs text-muted-foreground">暂无实际输出（未持久化）</div>
              )}
            </div>

            <div className="space-y-2">
              <div className="text-sm font-medium">差异对比</div>
              {caseDetail.expectedOutput && caseDetail.actualOutput ? (
                <Suspense fallback={editorFallback}>
                  <MonacoDiffEditor
                    height="260px"
                    original={caseDetail.expectedOutput}
                    modified={caseDetail.actualOutput}
                    language="text"
                    options={{
                      readOnly: true,
                      renderSideBySide: true,
                      minimap: { enabled: false },
                      wordWrap: 'on',
                    }}
                  />
                </Suspense>
              ) : (
                <div className="text-xs text-muted-foreground">
                  缺少期望或实际输出，无法生成差异对比
                </div>
              )}
            </div>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
