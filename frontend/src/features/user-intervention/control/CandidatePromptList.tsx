/**
 * 候选 Prompt 列表
 *
 * 用于终止任务时展示可选的候选 Prompt。
 */

import { useState } from 'react'
import { Check, Copy, ChevronDown, ChevronUp } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { CandidatePromptSummary } from '@/types/generated/models/CandidatePromptSummary'

export interface CandidatePromptListProps {
  candidates: CandidatePromptSummary[]
  selectedId: string | null
  copiedId: string | null
  onSelect: (id: string) => void
  onCopy: (prompt: string, id: string) => void
}

export function CandidatePromptList({
  candidates,
  selectedId,
  copiedId,
  onSelect,
  onCopy,
}: CandidatePromptListProps) {
  const formatPassRate = (rate: number) => `${(rate * 100).toFixed(1)}%`
  const [expandedId, setExpandedId] = useState<string | null>(null)

  return (
    <div className="space-y-3">
      {candidates.map((candidate) => {
        const isExpanded = expandedId === candidate.iterationId
        return (
          <div
            key={candidate.iterationId}
            role="button"
            tabIndex={0}
            aria-pressed={selectedId === candidate.iterationId}
            className={`p-4 rounded-lg border cursor-pointer transition-colors ${
              selectedId === candidate.iterationId
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-primary/50'
            }`}
            onClick={() => onSelect(candidate.iterationId)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault()
                onSelect(candidate.iterationId)
              }
            }}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <Badge variant={selectedId === candidate.iterationId ? 'default' : 'secondary'}>
                  第 {candidate.round} 轮
                </Badge>
                <span className="text-sm font-medium">
                  通过率: {formatPassRate(candidate.passRate)}
                </span>
                <span className="text-sm text-muted-foreground">
                  ({candidate.passedCases}/{candidate.totalCases})
                </span>
              </div>
              <div className="flex items-center gap-2">
                {selectedId === candidate.iterationId && (
                  <Check className="h-4 w-4 text-primary" />
                )}
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8"
                  onClick={(e) => {
                    e.stopPropagation()
                    onCopy(candidate.prompt, candidate.iterationId)
                  }}
                  title="复制 Prompt"
                >
                  {copiedId === candidate.iterationId ? (
                    <Check className="h-4 w-4 text-green-500" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>
            <div className="text-sm text-muted-foreground">
              {isExpanded ? (
                <div className="whitespace-pre-wrap break-words">{candidate.prompt}</div>
              ) : (
                <div className="line-clamp-2">{candidate.promptPreview}</div>
              )}
            </div>
            <div className="mt-2 flex justify-end">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="text-muted-foreground"
                aria-expanded={isExpanded}
                onClick={(e) => {
                  e.stopPropagation()
                  setExpandedId((prev) =>
                    prev === candidate.iterationId ? null : candidate.iterationId
                  )
                }}
              >
                {isExpanded ? (
                  <>
                    收起
                    <ChevronUp className="ml-1 h-4 w-4" />
                  </>
                ) : (
                  <>
                    展开全文
                    <ChevronDown className="ml-1 h-4 w-4" />
                  </>
                )}
              </Button>
            </div>
          </div>
        )
      })}
    </div>
  )
}

export default CandidatePromptList
