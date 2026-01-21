import { lazy, Suspense } from 'react'

const MonacoEditor = lazy(async () => import('@monaco-editor/react'))

export type MonacoEditorHandle = {
  getAction: (actionId: string) => { run?: () => void } | null
}

export interface PromptEditorProps {
  value: string
  onChange?: (value: string) => void
  readOnly?: boolean
  height?: string
  onMount?: (editor: MonacoEditorHandle) => void
}

export function PromptEditor({
  value,
  onChange,
  readOnly = false,
  height = '400px',
  onMount,
}: PromptEditorProps) {
  return (
    <Suspense
      fallback={<div className="h-[300px] text-sm text-muted-foreground">加载编辑器中...</div>}
    >
      <MonacoEditor
        height={height}
        defaultLanguage="markdown"
        theme="vs-light"
        value={value}
        onChange={(val) => onChange?.(val ?? '')}
        onMount={(editor) => onMount?.(editor as MonacoEditorHandle)}
        options={{
          readOnly,
          domReadOnly: readOnly,
          minimap: { enabled: false },
          wordWrap: 'on',
          lineNumbers: readOnly ? 'off' : 'on',
          fontSize: 14,
          scrollBeyondLastLine: false,
          formatOnPaste: true,
          formatOnType: true,
        }}
      />
    </Suspense>
  )
}
