import { useRef, useState } from 'react'

type Result = {
  frames: number
  durationMs: number
  fps: number
}

function buildBoxes(count: number): number[] {
  return Array.from({ length: count }, (_, i) => i)
}

export function PerfNfr3View() {
  const [result, setResult] = useState<Result | null>(null)
  const [isRunning, setIsRunning] = useState(false)
  const rafId = useRef<number | null>(null)

  const boxes = buildBoxes(800)

  const run = async () => {
    if (isRunning) return
    setIsRunning(true)
    setResult(null)

    const start = performance.now()
    const durationMs = 1000
    let frames = 0

    await new Promise<void>((resolve) => {
      const tick = () => {
        frames += 1
        const now = performance.now()
        if (now - start >= durationMs) {
          resolve()
          return
        }
        rafId.current = requestAnimationFrame(tick)
      }
      rafId.current = requestAnimationFrame(tick)
    })

    const end = performance.now()
    const d = end - start
    setResult({
      frames,
      durationMs: d,
      fps: frames / (d / 1000),
    })
    setIsRunning(false)
  }

  const stop = () => {
    if (rafId.current != null) {
      cancelAnimationFrame(rafId.current)
      rafId.current = null
    }
    setIsRunning(false)
  }

  return (
    <div className="mx-auto max-w-3xl p-6">
      <h1 className="text-xl font-semibold">NFR3：节点图渲染性能（口径预置）</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        该页面用于预置 NFR3 的 FPS 测量口径与回归入口（纯本地、确定性、不出网）。Epic 5 接入节点图后，
        将此测量替换为“真实节点图渲染 + 动画”场景。
      </p>

      <div className="mt-6 flex items-center gap-3">
        <button
          className="rounded border px-3 py-2 text-sm"
          onClick={run}
          disabled={isRunning}
          data-testid="nfr3-run"
        >
          {isRunning ? '测量中...' : '运行测量'}
        </button>
        <button
          className="rounded border px-3 py-2 text-sm"
          onClick={stop}
          disabled={!isRunning}
          data-testid="nfr3-stop"
        >
          停止
        </button>
      </div>

      <div className="mt-6 rounded border p-4 text-sm" data-testid="nfr3-result">
        <div>fps: {result ? result.fps.toFixed(2) : '-'}</div>
        <div>frames: {result ? result.frames : '-'}</div>
        <div>durationMs: {result ? result.durationMs.toFixed(2) : '-'}</div>
      </div>

      <div className="mt-6 grid grid-cols-20 gap-1 rounded border p-3">
        {boxes.map((i) => (
          <div key={i} className="h-3 w-3 rounded bg-muted" />
        ))}
      </div>
    </div>
  )
}

