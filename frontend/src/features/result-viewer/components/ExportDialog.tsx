/**
 * 导出结果对话框
 */

import { useMemo, useState } from 'react'
import { Download, FileCode2, FileText, Braces } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import type { ResultExportFormat } from '@/types/generated/models/ResultExportFormat'
import { useExportResult } from '../hooks/useExportResult'

type ExportOption = {
  value: ResultExportFormat
  title: string
  description: string
  icon: React.ReactNode
}

export interface ExportDialogProps {
  taskId: string
  open: boolean
  onOpenChange: (open: boolean) => void
}

const EXPORT_OPTIONS: ExportOption[] = [
  {
    value: 'markdown',
    title: 'Markdown',
    description: '适合文档分享与 Markdown 编辑器',
    icon: <FileText className="h-4 w-4" />,
  },
  {
    value: 'json',
    title: 'JSON',
    description: '结构化数据，便于二次处理',
    icon: <Braces className="h-4 w-4" />,
  },
  {
    value: 'xml',
    title: 'XML',
    description: '传统系统兼容格式',
    icon: <FileCode2 className="h-4 w-4" />,
  },
]

function triggerDownload(blob: Blob, filename: string) {
  const url = window.URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  window.URL.revokeObjectURL(url)
}

export function ExportDialog({ taskId, open, onOpenChange }: ExportDialogProps) {
  const [selectedFormat, setSelectedFormat] = useState<ResultExportFormat>('markdown')
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const { mutateAsync, isPending } = useExportResult()

  const options = useMemo(() => EXPORT_OPTIONS, [])

  const handleExport = async () => {
    setErrorMessage(null)
    try {
      const { blob, filename } = await mutateAsync({ taskId, format: selectedFormat })
      triggerDownload(blob, filename)
      onOpenChange(false)
    } catch (error) {
      const message = error instanceof Error ? error.message : '导出失败，请稍后重试'
      setErrorMessage(message)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[520px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="h-5 w-5 text-primary" />
            导出优化结果
          </DialogTitle>
          <DialogDescription>
            选择导出格式后生成文件，内容包含最佳 Prompt、通过率与迭代摘要。
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-3 py-2">
          {options.map((option) => {
            const isSelected = selectedFormat === option.value
            return (
              <button
                key={option.value}
                type="button"
                onClick={() => setSelectedFormat(option.value)}
                className={`flex items-start gap-3 rounded-lg border px-3 py-3 text-left transition ${
                  isSelected
                    ? 'border-primary bg-primary/5'
                    : 'border-border hover:border-primary/50'
                }`}
                aria-pressed={isSelected}
              >
                <div className="mt-0.5 text-primary">{option.icon}</div>
                <div>
                  <div className="text-sm font-medium">{option.title}</div>
                  <div className="text-xs text-muted-foreground">{option.description}</div>
                </div>
              </button>
            )
          })}
        </div>

        {errorMessage ? (
          <div className="text-sm text-destructive">{errorMessage}</div>
        ) : null}

        <DialogFooter className="gap-2 sm:gap-0">
          <Button variant="outline" type="button" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button type="button" onClick={handleExport} disabled={isPending}>
            {isPending ? '导出中...' : '确认导出'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default ExportDialog
