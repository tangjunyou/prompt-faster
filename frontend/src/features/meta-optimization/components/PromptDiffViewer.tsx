import { lazy, Suspense } from 'react'

const MonacoDiffEditor = lazy(async () =>
  import('@monaco-editor/react').then((module) => ({ default: module.DiffEditor }))
)

export interface PromptDiffViewerProps {
  versionA: { version: number; content: string }
  versionB: { version: number; content: string }
}

export function PromptDiffViewer({ versionA, versionB }: PromptDiffViewerProps) {
  return (
    <div className="overflow-hidden rounded-lg border">
      <div className="flex items-center justify-between bg-muted px-4 py-2 text-sm">
        <span>版本 {versionA.version}（基准）</span>
        <span>版本 {versionB.version}（对比）</span>
      </div>
      <Suspense
        fallback={
          <div className="flex h-[300px] items-center justify-center text-sm text-muted-foreground">
            加载 Diff 编辑器中...
          </div>
        }
      >
        <MonacoDiffEditor
          height="400px"
          language="markdown"
          theme="vs-light"
          original={versionA.content}
          modified={versionB.content}
          options={{
            readOnly: true,
            renderSideBySide: true,
            minimap: { enabled: false },
            wordWrap: 'on',
          }}
        />
      </Suspense>
    </div>
  )
}
