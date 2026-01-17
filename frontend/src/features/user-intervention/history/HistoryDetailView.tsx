/**
 * 历史迭代详情视图组件
 * 分 tab 展示：规律假设 / 候选 Prompt / 评估结果 / 反思总结
 */

import { useState } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Lightbulb, FileText, BarChart3, MessageSquare } from 'lucide-react'
import { ArtifactEditor } from '../ArtifactEditor'
import type { IterationHistoryDetail } from '@/types/generated/models/IterationHistoryDetail'

export interface HistoryDetailViewProps {
  /** 迭代详情数据 */
  detail: IterationHistoryDetail
}

/**
 * 历史迭代详情视图
 * 以只读模式展示历史产物
 */
export function HistoryDetailView({ detail }: HistoryDetailViewProps) {
  const [activeTab, setActiveTab] = useState<string>('artifacts')

  return (
    <div className="space-y-4">
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="artifacts" className="flex items-center gap-1">
            <Lightbulb className="h-4 w-4" />
            <span className="hidden sm:inline">产物</span>
          </TabsTrigger>
          <TabsTrigger value="evaluation" className="flex items-center gap-1">
            <BarChart3 className="h-4 w-4" />
            <span className="hidden sm:inline">评估</span>
          </TabsTrigger>
          <TabsTrigger value="reflection" className="flex items-center gap-1">
            <MessageSquare className="h-4 w-4" />
            <span className="hidden sm:inline">反思</span>
          </TabsTrigger>
          <TabsTrigger value="raw" className="flex items-center gap-1">
            <FileText className="h-4 w-4" />
            <span className="hidden sm:inline">原始</span>
          </TabsTrigger>
        </TabsList>

        {/* 产物 Tab - 使用 ArtifactEditor 只读模式 */}
        <TabsContent value="artifacts" className="mt-4">
          <ArtifactEditor
            taskId={detail.id}
            artifacts={detail.artifacts}
            readOnly
          />
        </TabsContent>

        {/* 评估结果 Tab */}
        <TabsContent value="evaluation" className="mt-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">通过率</span>
              <span className="font-medium">
                {(detail.passRate * 100).toFixed(1)}% ({detail.passedCases}/
                {detail.totalCases})
              </span>
            </div>

            {detail.evaluationResults.length > 0 ? (
              <div className="border rounded-lg divide-y max-h-[300px] overflow-y-auto">
                {detail.evaluationResults.map((result: { testCaseId: string; passed: boolean; score?: number | null; failureReason?: string | null }, index: number) => (
                  <div
                    key={result.testCaseId || index}
                    className="p-3 flex items-start gap-3"
                  >
                    <div
                      className={`shrink-0 w-2 h-2 mt-1.5 rounded-full ${
                        result.passed ? 'bg-green-500' : 'bg-red-500'
                      }`}
                    />
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium truncate">
                        {result.testCaseId}
                      </div>
                      {result.score !== null && result.score !== undefined && (
                        <div className="text-xs text-muted-foreground">
                          分数: {result.score.toFixed(2)}
                        </div>
                      )}
                      {result.failureReason && (
                        <div className="text-xs text-red-600 mt-1">
                          {result.failureReason}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground text-center py-4">
                暂无评估结果
              </p>
            )}
          </div>
        </TabsContent>

        {/* 反思总结 Tab */}
        <TabsContent value="reflection" className="mt-4">
          {detail.reflectionSummary ? (
            <div className="border rounded-lg p-4 bg-muted/20">
              <pre className="text-sm whitespace-pre-wrap font-sans">
                {detail.reflectionSummary}
              </pre>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              暂无反思总结
            </p>
          )}
        </TabsContent>

        {/* 原始数据 Tab */}
        <TabsContent value="raw" className="mt-4">
          <div className="border rounded-lg p-4 bg-muted/20 max-h-[300px] overflow-y-auto">
            <pre className="text-xs font-mono whitespace-pre-wrap">
              {JSON.stringify(detail, null, 2)}
            </pre>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}

export default HistoryDetailView
