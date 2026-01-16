// 注意：此处为 demo 临时映射，真实语义以服务端 iteration_stage.rs / /api/v1/meta/iteration-stages 为权威。
export type StageType = 'pattern' | 'prompt' | 'quality' | 'reflection'

export type StageHistoryItem = {
  stage: StageType
  summary: string
  text: string
  startSeq: number
  endSeq: number
}

export const STAGE_LABELS: Record<StageType, string> = {
  pattern: '规律抽取中',
  prompt: '候选生成中',
  quality: '质量评估中',
  reflection: '反思迭代中',
}

export const STAGE_COLORS: Record<StageType, { badge: string; dot: string }> = {
  pattern: { badge: 'bg-blue-100 text-blue-800 border-blue-200', dot: 'bg-blue-500' },
  prompt: { badge: 'bg-indigo-100 text-indigo-800 border-indigo-200', dot: 'bg-indigo-500' },
  quality: { badge: 'bg-emerald-100 text-emerald-800 border-emerald-200', dot: 'bg-emerald-500' },
  reflection: { badge: 'bg-amber-100 text-amber-900 border-amber-200', dot: 'bg-amber-500' },
}
