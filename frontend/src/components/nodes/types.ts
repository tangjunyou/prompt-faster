export const iterationGraphNodeIds = [
  'pattern_extractor',
  'prompt_engineer',
  'quality_assessor',
  'reflection_agent',
] as const

export type IterationGraphNodeId = (typeof iterationGraphNodeIds)[number]

export const iterationGraphEdgeIds = [
  'pattern->prompt',
  'prompt->quality',
  'quality->reflection',
] as const

export type IterationGraphEdgeId = (typeof iterationGraphEdgeIds)[number]

export type NodeStatus = 'idle' | 'running' | 'success' | 'error' | 'paused'

export type IterationGraphNodeStates = Record<IterationGraphNodeId, NodeStatus>

export type EdgeFlowState = 'idle' | 'flowing' | 'cooldown'

export type IterationGraphEdgeFlowState = {
  state: EdgeFlowState
  lastActivatedSeq: number
}

export type IterationGraphEdgeFlowStates = Record<IterationGraphEdgeId, IterationGraphEdgeFlowState>
