/**
 * 多样性指标雷达图
 */

import {
  RadarChart,
  Radar,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  ResponsiveContainer,
  Tooltip,
} from 'recharts'
import type { DiversityMetrics } from '@/types/generated/models/DiversityMetrics'

interface ChartDatum {
  label: string
  score: number
}

function clampScore(value: number) {
  if (Number.isNaN(value)) return 0
  return Math.max(0, Math.min(1, value))
}

export interface DiversityMetricsChartProps {
  metrics: DiversityMetrics
}

export function DiversityMetricsChart({ metrics }: DiversityMetricsChartProps) {
  const data: ChartDatum[] = [
    { label: '词汇多样性', score: clampScore(metrics.lexicalDiversity) },
    { label: '结构多样性', score: clampScore(metrics.structuralDiversity) },
    { label: '语义多样性', score: clampScore(metrics.semanticDiversity) },
  ]

  return (
    <div className="h-56 w-full">
      <ResponsiveContainer width="100%" height="100%">
        <RadarChart data={data} margin={{ top: 8, right: 16, bottom: 8, left: 16 }}>
          <PolarGrid stroke="hsl(var(--border))" />
          <PolarAngleAxis dataKey="label" tick={{ fontSize: 12 }} />
          <PolarRadiusAxis domain={[0, 1]} tick={{ fontSize: 10 }} />
          <Tooltip
            formatter={(value: number) => [`${(value * 100).toFixed(1)}%`, '分值']}
            contentStyle={{ fontSize: 12 }}
          />
          <Radar
            dataKey="score"
            stroke="hsl(var(--primary))"
            fill="hsl(var(--primary) / 0.2)"
            fillOpacity={0.6}
          />
        </RadarChart>
      </ResponsiveContainer>
    </div>
  )
}

export default DiversityMetricsChart
