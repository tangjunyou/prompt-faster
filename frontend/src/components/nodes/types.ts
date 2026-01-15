export const iterationGraphNodeIds = [
  'pattern_extractor',
  'prompt_engineer',
  'quality_assessor',
  'reflection_agent',
] as const

export type IterationGraphNodeId = (typeof iterationGraphNodeIds)[number]

export type NodeStatus = 'idle' | 'running' | 'success' | 'error' | 'paused'

export type IterationGraphNodeStates = Record<IterationGraphNodeId, NodeStatus>

