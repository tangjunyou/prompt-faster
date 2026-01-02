/**
 * Health 连接测试 Hook
 */

import { useQuery } from '@tanstack/react-query'
import { fetchHealth } from '../services/healthService'

const HEALTH_QUERY_KEY = ['health'] as const

export function useHealth() {
  return useQuery({
    queryKey: HEALTH_QUERY_KEY,
    queryFn: fetchHealth,
  })
}
