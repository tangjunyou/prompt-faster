/**
 * 网络连接状态 Hook
 */

import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { getConnectivity } from '../services/recoveryService'
import type { ConnectivityResponse } from '@/types/generated/models/ConnectivityResponse'

const CONNECTIVITY_QUERY_KEY = ['connectivity'] as const

export function useConnectivity() {
  const [isBrowserOffline, setIsBrowserOffline] = useState(
    typeof navigator !== 'undefined' ? !navigator.onLine : false,
  )

  useEffect(() => {
    const handleOnline = () => setIsBrowserOffline(false)
    const handleOffline = () => setIsBrowserOffline(true)
    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)
    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [])

  const query = useQuery<ConnectivityResponse, Error>({
    queryKey: CONNECTIVITY_QUERY_KEY,
    queryFn: getConnectivity,
    enabled: !isBrowserOffline,
    staleTime: 30 * 1000,
    refetchInterval: 30 * 1000,
    retry: 1,
  })

  const status = isBrowserOffline
    ? 'offline'
    : query.data?.status ?? 'online'
  const isOffline = status === 'offline'
  const isLimited = status === 'limited'
  const message = isBrowserOffline ? '浏览器检测为离线' : query.data?.message

  return {
    status,
    isOffline,
    isLimited,
    message,
    availableFeatures: query.data?.availableFeatures ?? [],
    restrictedFeatures: query.data?.restrictedFeatures ?? [],
    lastCheckedAt: query.data?.lastCheckedAt,
    isLoading: query.isLoading,
    isError: query.isError,
    isFetching: query.isFetching,
    refetch: query.refetch,
  }
}
