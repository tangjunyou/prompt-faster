/**
 * 多样性分析 Hook
 */

import { useQuery } from '@tanstack/react-query'
import type { DiversityAnalysisResult } from '@/types/generated/models/DiversityAnalysisResult'
import { getDiversityAnalysis } from '../services/diversityService'

const DIVERSITY_KEY = 'diversityAnalysis'

export function useDiversityAnalysis(
  taskId: string,
  options?: {
    enabled?: boolean
    staleTime?: number
  }
) {
  const { enabled = true, staleTime = 10 * 1000 } = options ?? {}

  return useQuery<DiversityAnalysisResult | null, Error>({
    queryKey: [DIVERSITY_KEY, taskId],
    queryFn: () => getDiversityAnalysis(taskId),
    enabled: enabled && !!taskId,
    staleTime,
  })
}
