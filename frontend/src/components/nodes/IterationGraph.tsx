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
import { computeIterationGraphEdgeDenoiseLevels } from './edgeDenoise'
import type {
  IterationGraphEdgeFlowStates,
  IterationGraphEdgeId,
  IterationGraphNodeId,
  IterationGraphNodeStates,
  NodeStatus,
} from './types'
import { iterationGraphEdgeIds } from './types'

import './IterationGraphEdges.css'

export type IterationGraphProps = {
  nodeStates: Partial<IterationGraphNodeStates>
  edgeFlowStates?: IterationGraphEdgeFlowStates
  prefersReducedMotion?: boolean
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
    id: iterationGraphEdgeIds[0],
    source: 'pattern_extractor',
    target: 'prompt_engineer',
    type: 'smoothstep',
    markerEnd: { type: MarkerType.ArrowClosed },
  },
  {
    id: iterationGraphEdgeIds[1],
    source: 'prompt_engineer',
    target: 'quality_assessor',
    type: 'smoothstep',
    markerEnd: { type: MarkerType.ArrowClosed },
  },
  {
    id: iterationGraphEdgeIds[2],
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
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>(graphEdges)

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

  useEffect(() => {
    if (!props.edgeFlowStates) return

    const prefersReducedMotion = props.prefersReducedMotion ?? false
    const denoiseLevels = computeIterationGraphEdgeDenoiseLevels(props.edgeFlowStates)

    setEdges((prev) => {
      let changed = false
      const next = prev.map((edge) => {
        const edgeId = edge.id as IterationGraphEdgeId
        const flow = props.edgeFlowStates![edgeId]
        const denoise = denoiseLevels[edgeId]

        const isFlowing = flow.state === 'flowing'
        const isCooldown = flow.state === 'cooldown'
        const isActive = isFlowing || isCooldown

        const animated = !prefersReducedMotion && isFlowing && denoise === 'strong'
        const className = [
          'iteration-graph__edge',
          denoise === 'strong' ? 'iteration-graph__edge--strong' : null,
          denoise === 'weak' ? 'iteration-graph__edge--weak' : null,
          isFlowing ? 'iteration-graph__edge--flowing' : null,
          isCooldown ? 'iteration-graph__edge--cooldown' : null,
          prefersReducedMotion && isActive ? 'iteration-graph__edge--pulse' : null,
        ]
          .filter(Boolean)
          .join(' ')

        const sameAnimated = edge.animated === animated
        const sameClass = edge.className === className
        if (sameAnimated && sameClass) return edge

        changed = true
        return {
          ...edge,
          animated,
          className,
        }
      })
      return changed ? next : prev
    })
  }, [props.edgeFlowStates, props.prefersReducedMotion, setEdges])

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
