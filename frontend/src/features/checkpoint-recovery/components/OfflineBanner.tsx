/**
 * 离线提示横幅
 */

import { AlertTriangle, WifiOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useConnectivity } from '../hooks/useConnectivity'

export function OfflineBanner() {
  const { status, message, restrictedFeatures, refetch, isFetching } = useConnectivity()

  if (status === 'online') {
    return null
  }

  const isOffline = status === 'offline'
  const Icon = isOffline ? WifiOff : AlertTriangle
  const title = isOffline ? '当前离线' : '网络不稳定'
  const detail = message ?? (isOffline ? '部分功能不可用' : '部分功能可能受限')

  return (
    <div className="border-b bg-yellow-50 text-yellow-900">
      <div className="mx-auto flex max-w-5xl flex-col gap-2 px-4 py-2 text-sm sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-2">
          <Icon className="h-4 w-4" />
          <span className="font-medium">{title}</span>
          <span className="text-yellow-800">{detail}</span>
        </div>
        <div className="flex flex-col gap-2 text-xs text-yellow-800 sm:flex-row sm:items-center sm:gap-3">
          {restrictedFeatures.length > 0 && (
            <span>受限功能：{restrictedFeatures.join('、')}</span>
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            disabled={isFetching}
          >
            {isFetching ? '检测中…' : '重试检测'}
          </Button>
        </div>
      </div>
    </div>
  )
}

export default OfflineBanner
