import { useEffect, useMemo } from 'react'
import {
  Background,
  Controls,
  MarkerType,
  ReactFlow,
  type Edge,
  type Node,
  useEdgesState,
  useNodesState,
} from '@xyflow/react'

import { nodeStatusToClassName } from './nodeStyles'
import type { IterationGraphNodeId, IterationGraphNodeStates, NodeStatus } from './types'

export type IterationGraphProps = {
  nodeStates: Partial<IterationGraphNodeStates>
  className?: string
}

type GraphNodeData = { label: string; status: NodeStatus }

const graphNodeDescriptors: Array<{
  id: IterationGraphNodeId
  label: string
  position: { x: number; y: number }
}> = [
  { id: 'pattern_extractor', label: 'Pattern Extractor', position: { x: 0, y: 0 } },
  { id: 'prompt_engineer', label: 'Prompt Engineer', position: { x: 0, y: 140 } },
  { id: 'quality_assessor', label: 'Quality Assessor', position: { x: 0, y: 280 } },
  { id: 'reflection_agent', label: 'Reflection Agent', position: { x: 0, y: 420 } },
]

const graphEdges: Edge[] = [
  {
    id: 'pattern->prompt',
    source: 'pattern_extractor',
    target: 'prompt_engineer',
    type: 'smoothstep',
    markerEnd: { type: MarkerType.ArrowClosed },
  },
  {
    id: 'prompt->quality',
    source: 'prompt_engineer',
    target: 'quality_assessor',
    type: 'smoothstep',
    markerEnd: { type: MarkerType.ArrowClosed },
  },
  {
    id: 'quality->reflection',
    source: 'quality_assessor',
    target: 'reflection_agent',
    type: 'smoothstep',
    markerEnd: { type: MarkerType.ArrowClosed },
  },
]

function resolveStatus(nodeStates: Partial<IterationGraphNodeStates>, nodeId: IterationGraphNodeId): NodeStatus {
  return nodeStates[nodeId] ?? 'idle'
}

export function IterationGraph(props: IterationGraphProps) {
  const initialNodes = useMemo((): Node<GraphNodeData>[] => {
    return graphNodeDescriptors.map((descriptor) => ({
      id: descriptor.id,
      position: descriptor.position,
      data: { label: descriptor.label, status: 'idle' },
      className: nodeStatusToClassName('idle'),
      draggable: true,
      selectable: false,
    }))
  }, [])

  const [nodes, setNodes, onNodesChange] = useNodesState<Node<GraphNodeData>>(initialNodes)
  const [edges, , onEdgesChange] = useEdgesState<Edge>(graphEdges)

  useEffect(() => {
    setNodes((prev) =>
      prev.map((node) => {
        const nodeId = node.id as IterationGraphNodeId
        const status = resolveStatus(props.nodeStates, nodeId)
        return {
          ...node,
          data: { ...node.data, status },
          className: nodeStatusToClassName(status),
        }
      }),
    )
  }, [props.nodeStates, setNodes])

  return (
    <div className={props.className} data-testid="iteration-graph">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        minZoom={0.2}
        nodesDraggable
        panOnDrag
        zoomOnScroll
        zoomOnPinch
        proOptions={{ hideAttribution: true }}
      >
        <Background />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  )
}
