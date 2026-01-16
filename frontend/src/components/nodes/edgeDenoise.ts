import { iterationGraphEdgeIds, type IterationGraphEdgeFlowStates, type IterationGraphEdgeId } from './types'

export type EdgeDenoiseLevel = 'off' | 'strong' | 'weak'

function edgeIdSortKey(edgeId: IterationGraphEdgeId): number {
  const idx = iterationGraphEdgeIds.indexOf(edgeId)
  return idx === -1 ? Number.POSITIVE_INFINITY : idx
}

export function computeIterationGraphEdgeDenoiseLevels(
  edgeFlowStates: IterationGraphEdgeFlowStates,
): Record<IterationGraphEdgeId, EdgeDenoiseLevel> {
  const activeEdges = iterationGraphEdgeIds
    .filter((edgeId) => edgeFlowStates[edgeId].state !== 'idle')
    .sort((a, b) => {
      const seqDiff = edgeFlowStates[b].lastActivatedSeq - edgeFlowStates[a].lastActivatedSeq
      if (seqDiff !== 0) return seqDiff
      return edgeIdSortKey(a) - edgeIdSortKey(b)
    })

  const strongEdgeIds = new Set(activeEdges.slice(0, 2))

  const result = {} as Record<IterationGraphEdgeId, EdgeDenoiseLevel>
  for (const edgeId of iterationGraphEdgeIds) {
    const flowState = edgeFlowStates[edgeId].state
    if (flowState === 'idle') {
      result[edgeId] = 'off'
    } else if (strongEdgeIds.has(edgeId)) {
      result[edgeId] = 'strong'
    } else {
      result[edgeId] = 'weak'
    }
  }

  return result
}
