/**
 * 产物编辑器组件
 * 支持编辑规律假设和候选 Prompt
 */

import { useState, useCallback, useMemo, lazy, Suspense } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { Pencil, Save, X, Lightbulb, FileText, Trash2 } from 'lucide-react'
import type { IterationArtifacts } from '@/types/generated/models/IterationArtifacts'
import type { PatternHypothesis } from '@/types/generated/models/PatternHypothesis'
import type { CandidatePrompt } from '@/types/generated/models/CandidatePrompt'

const MonacoEditor = lazy(async () => import('@monaco-editor/react'))

export interface ArtifactEditorProps {
  /** 任务 ID */
  taskId: string
  /** 产物数据 */
  artifacts: IterationArtifacts | undefined
  /** 保存回调 */
  onSave?: (artifacts: IterationArtifacts, correlationId: string) => void
  /** 是否禁用（非 Paused 状态） */
  disabled?: boolean
  /** 是否只读模式（历史产物查看） */
  readOnly?: boolean
  /** 是否正在保存 */
  isSaving?: boolean
  /** 保存失败错误信息 */
  saveError?: string | null
  /** 保存成功提示是否显示 */
  showSuccess?: boolean
}

/**
 * 产物编辑器
 * 支持查看和编辑规律假设、候选 Prompt
 */
export function ArtifactEditor({
  taskId: _taskId,
  artifacts,
  onSave,
  disabled = false,
  readOnly = false,
  isSaving = false,
  saveError = null,
  showSuccess = false,
}: ArtifactEditorProps) {
  // taskId 保留用于未来扩展（如日志记录）
  void _taskId
  // 编辑模式状态
  const [isEditing, setIsEditing] = useState(false)
  // 当前编辑的规律假设
  const [editingPatterns, setEditingPatterns] = useState<PatternHypothesis[]>([])
  // 当前编辑的候选 Prompt
  const [editingPrompts, setEditingPrompts] = useState<CandidatePrompt[]>([])
  // 当前选中的 tab
  const [activeTab, setActiveTab] = useState<'patterns' | 'prompts'>('patterns')
  // 当前编辑的项目索引
  const [selectedIndex, setSelectedIndex] = useState<number>(0)
  const editorFallback = (
    <div className="p-4 h-[300px] text-sm text-muted-foreground">
      正在加载编辑器...
    </div>
  )

  const renderReadOnlyViewer = (value: string) => (
    <Suspense fallback={editorFallback}>
      <MonacoEditor
        height="300px"
        language="markdown"
        value={value}
        options={{
          minimap: { enabled: false },
          lineNumbers: 'off',
          wordWrap: 'on',
          fontSize: 14,
          padding: { top: 12 },
          readOnly: true,
          domReadOnly: true,
        }}
        theme="vs-light"
      />
    </Suspense>
  )

  // 进入编辑模式
  const handleStartEdit = useCallback(() => {
    if (!artifacts) return
    setEditingPatterns([...artifacts.patterns])
    setEditingPrompts([...artifacts.candidatePrompts])
    setIsEditing(true)
    setSelectedIndex(0)
  }, [artifacts])

  // 取消编辑
  const handleCancel = useCallback(() => {
    setIsEditing(false)
    setEditingPatterns([])
    setEditingPrompts([])
    setSelectedIndex(0)
  }, [])

  // 保存编辑
  const handleSave = useCallback(() => {
    if (!onSave) return
    const correlationId = `cid-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
    const updatedArtifacts: IterationArtifacts = {
      patterns: editingPatterns,
      candidatePrompts: editingPrompts,
      userGuidance: artifacts?.userGuidance ?? null,
      updatedAt: new Date().toISOString(),
    }
    onSave(updatedArtifacts, correlationId)
  }, [editingPatterns, editingPrompts, onSave, artifacts?.userGuidance])

  // 更新规律假设内容
  const handlePatternChange = useCallback((value: string | undefined) => {
    if (value === undefined) return
    setEditingPatterns((prev) => {
      const updated = [...prev]
      if (updated[selectedIndex]) {
        updated[selectedIndex] = { ...updated[selectedIndex], pattern: value }
      }
      return updated
    })
  }, [selectedIndex])

  // 更新候选 Prompt 内容
  const handlePromptChange = useCallback((value: string | undefined) => {
    if (value === undefined) return
    setEditingPrompts((prev) => {
      const updated = [...prev]
      if (updated[selectedIndex]) {
        updated[selectedIndex] = { ...updated[selectedIndex], content: value }
      }
      return updated
    })
  }, [selectedIndex])

  // 删除规律假设
  const handleDeletePattern = useCallback((index: number) => {
    setEditingPatterns((prev) => prev.filter((_, i) => i !== index))
    if (selectedIndex >= editingPatterns.length - 1 && selectedIndex > 0) {
      setSelectedIndex(selectedIndex - 1)
    }
  }, [selectedIndex, editingPatterns.length])

  // 删除候选 Prompt
  const handleDeletePrompt = useCallback((index: number) => {
    setEditingPrompts((prev) => prev.filter((_, i) => i !== index))
    if (selectedIndex >= editingPrompts.length - 1 && selectedIndex > 0) {
      setSelectedIndex(selectedIndex - 1)
    }
  }, [selectedIndex, editingPrompts.length])

  // 当前显示的列表
  const currentList = useMemo(() => {
    if (isEditing) {
      return activeTab === 'patterns' ? editingPatterns : editingPrompts
    }
    if (!artifacts) return []
    return activeTab === 'patterns' ? artifacts.patterns : artifacts.candidatePrompts
  }, [isEditing, activeTab, editingPatterns, editingPrompts, artifacts])

  // 当前编辑的内容
  const currentContent = useMemo(() => {
    if (!isEditing) return ''
    if (activeTab === 'patterns') {
      return editingPatterns[selectedIndex]?.pattern ?? ''
    }
    return editingPrompts[selectedIndex]?.content ?? ''
  }, [isEditing, activeTab, editingPatterns, editingPrompts, selectedIndex])

  // 空状态
  if (!artifacts || (artifacts.patterns.length === 0 && artifacts.candidatePrompts.length === 0)) {
    return (
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <FileText className="h-5 w-5" />
            {readOnly ? '历史产物' : '中间产物'}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground text-sm">
            {readOnly ? '暂无历史产物' : '暂无可编辑的产物'}
          </p>
          {!readOnly && disabled ? (
            <>
              <div className="mt-3">
                <Button
                  variant="outline"
                  size="sm"
                  disabled
                  className="min-w-[44px] min-h-[44px]"
                >
                  编辑
                </Button>
              </div>
              <p className="text-sm text-amber-600 mt-2">
                ⚠️ 请先暂停任务再编辑
              </p>
            </>
          ) : null}
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="w-full">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg flex items-center gap-2">
            <FileText className="h-5 w-5" />
            {readOnly ? '历史产物' : '中间产物'}
          </CardTitle>
          {!readOnly && (
            <div className="flex items-center gap-2">
              {isEditing ? (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleCancel}
                    disabled={isSaving}
                    className="min-w-[44px] min-h-[44px]"
                  >
                    <X className="h-4 w-4 mr-1" />
                    取消
                  </Button>
                  <Button
                    size="sm"
                    onClick={handleSave}
                    disabled={isSaving}
                    className="min-w-[44px] min-h-[44px]"
                  >
                    <Save className="h-4 w-4 mr-1" />
                    {isSaving ? '保存中...' : '保存'}
                  </Button>
                </>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleStartEdit}
                  disabled={disabled}
                  className="min-w-[44px] min-h-[44px]"
                  title={disabled ? '请先暂停任务再编辑' : '编辑产物'}
                >
                  <Pencil className="h-4 w-4 mr-1" />
                  编辑
                </Button>
              )}
            </div>
          )}
        </div>
        {readOnly && (
          <p className="text-sm text-muted-foreground mt-2">
            📜 历史记录仅供查看
          </p>
        )}
        {!readOnly && isEditing && (
          <p className="text-sm text-muted-foreground mt-2">
            💡 修改后的内容将用于后续迭代
          </p>
        )}
        {!readOnly && disabled && !isEditing && (
          <p className="text-sm text-amber-600 mt-2">
            ⚠️ 请先暂停任务再编辑
          </p>
        )}
        {showSuccess ? (
          <p className="text-sm text-emerald-600 mt-2">✅ 保存成功</p>
        ) : null}
        {saveError ? (
          <p className="text-sm text-destructive mt-2">⚠️ {saveError}</p>
        ) : null}
      </CardHeader>
      <CardContent>
        <Tabs value={activeTab} onValueChange={(v: string) => {
          setActiveTab(v as 'patterns' | 'prompts')
          setSelectedIndex(0)
        }}>
          <TabsList className="grid w-full grid-cols-2 mb-4">
            <TabsTrigger value="patterns" className="flex items-center gap-1">
              <Lightbulb className="h-4 w-4" />
              规律假设 ({isEditing ? editingPatterns.length : artifacts.patterns.length})
            </TabsTrigger>
            <TabsTrigger value="prompts" className="flex items-center gap-1">
              <FileText className="h-4 w-4" />
              候选 Prompt ({isEditing ? editingPrompts.length : artifacts.candidatePrompts.length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value="patterns" className="mt-0">
            <div className="flex gap-4">
              {/* 列表 */}
              <div className="w-1/3 space-y-2 max-h-[400px] overflow-y-auto">
                {(isEditing ? editingPatterns : artifacts.patterns).map((pattern, index) => (
                  <div
                    key={pattern.id}
                    className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                      selectedIndex === index
                        ? 'border-primary bg-primary/5'
                        : 'border-border hover:border-primary/50'
                    }`}
                    onClick={() => setSelectedIndex(index)}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">
                          {pattern.pattern.length > 50
                            ? `${pattern.pattern.slice(0, 50)}...`
                            : pattern.pattern}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <Badge variant={pattern.source === 'user_edited' ? 'default' : 'secondary'} className="text-xs">
                            {pattern.source === 'user_edited' ? '已编辑' : '系统'}
                          </Badge>
                          {pattern.confidence != null && Number.isFinite(pattern.confidence) && (
                            <span className="text-xs text-muted-foreground">
                              置信度: {(pattern.confidence * 100).toFixed(0)}%
                            </span>
                          )}
                        </div>
                      </div>
                      {isEditing && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 shrink-0"
                          onClick={(e) => {
                            e.stopPropagation()
                            handleDeletePattern(index)
                          }}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
                {currentList.length === 0 && (
                  <p className="text-sm text-muted-foreground p-3">暂无规律假设</p>
                )}
              </div>

              {/* 编辑器 */}
              <div className="flex-1 border rounded-lg overflow-hidden">
                {isEditing && editingPatterns.length > 0 ? (
                  <Suspense fallback={editorFallback}>
                    <MonacoEditor
                      height="300px"
                      language="markdown"
                      value={currentContent}
                      onChange={handlePatternChange}
                      options={{
                        minimap: { enabled: false },
                        lineNumbers: 'off',
                        wordWrap: 'on',
                        fontSize: 14,
                        padding: { top: 12 },
                      }}
                      theme="vs-light"
                    />
                  </Suspense>
                ) : readOnly ? (
                  renderReadOnlyViewer(artifacts.patterns[selectedIndex]?.pattern ?? '')
                ) : (
                  <div className="p-4 h-[300px] overflow-y-auto">
                    {artifacts.patterns[selectedIndex] ? (
                      <pre className="text-sm whitespace-pre-wrap font-sans">
                        {artifacts.patterns[selectedIndex].pattern}
                      </pre>
                    ) : (
                      <p className="text-sm text-muted-foreground">选择一个规律假设查看详情</p>
                    )}
                  </div>
                )}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="prompts" className="mt-0">
            <div className="flex gap-4">
              {/* 列表 */}
              <div className="w-1/3 space-y-2 max-h-[400px] overflow-y-auto">
                {(isEditing ? editingPrompts : artifacts.candidatePrompts).map((prompt, index) => (
                  <div
                    key={prompt.id}
                    className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                      selectedIndex === index
                        ? 'border-primary bg-primary/5'
                        : 'border-border hover:border-primary/50'
                    }`}
                    onClick={() => setSelectedIndex(index)}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">
                          {prompt.content.length > 50
                            ? `${prompt.content.slice(0, 50)}...`
                            : prompt.content}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <Badge variant={prompt.source === 'user_edited' ? 'default' : 'secondary'} className="text-xs">
                            {prompt.source === 'user_edited' ? '已编辑' : '系统'}
                          </Badge>
                          {prompt.isBest && (
                            <Badge variant="outline" className="text-xs text-green-600 border-green-600">
                              最佳
                            </Badge>
                          )}
                          {prompt.score != null && Number.isFinite(prompt.score) && (
                            <span className="text-xs text-muted-foreground">
                              分数: {prompt.score.toFixed(2)}
                            </span>
                          )}
                        </div>
                      </div>
                      {isEditing && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 shrink-0"
                          onClick={(e) => {
                            e.stopPropagation()
                            handleDeletePrompt(index)
                          }}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      )}
                    </div>
                  </div>
                ))}
                {currentList.length === 0 && (
                  <p className="text-sm text-muted-foreground p-3">暂无候选 Prompt</p>
                )}
              </div>

              {/* 编辑器 */}
              <div className="flex-1 border rounded-lg overflow-hidden">
                {isEditing && editingPrompts.length > 0 ? (
                  <Suspense fallback={editorFallback}>
                    <MonacoEditor
                      height="300px"
                      language="markdown"
                      value={currentContent}
                      onChange={handlePromptChange}
                      options={{
                        minimap: { enabled: false },
                        lineNumbers: 'off',
                        wordWrap: 'on',
                        fontSize: 14,
                        padding: { top: 12 },
                      }}
                      theme="vs-light"
                    />
                  </Suspense>
                ) : readOnly ? (
                  renderReadOnlyViewer(artifacts.candidatePrompts[selectedIndex]?.content ?? '')
                ) : (
                  <div className="p-4 h-[300px] overflow-y-auto">
                    {artifacts.candidatePrompts[selectedIndex] ? (
                      <pre className="text-sm whitespace-pre-wrap font-sans">
                        {artifacts.candidatePrompts[selectedIndex].content}
                      </pre>
                    ) : (
                      <p className="text-sm text-muted-foreground">选择一个候选 Prompt 查看详情</p>
                    )}
                  </div>
                )}
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}

export default ArtifactEditor
