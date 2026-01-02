import { useHealth } from '@/features/health/hooks/useHealth'

export function HealthCheck() {
  const { data: health, isLoading, error } = useHealth()
  const errorMessage = error instanceof Error ? error.message : '网络错误'

  if (isLoading) {
    return <div className="text-gray-500">检查后端连接...</div>
  }

  if (error) {
    return <div className="text-red-500">❌ 后端连接失败: {errorMessage}</div>
  }

  return (
    <div className="text-green-500">
      ✅ 后端连接成功 | 版本: {health?.version} | 状态: {health?.status}
    </div>
  )
}
