import { useState, useEffect } from 'react'
import { get, isApiSuccess } from '../lib/api'

interface HealthResponse {
  status: string
  version: string
  timestampMs: number
}

export function HealthCheck() {
  const [health, setHealth] = useState<HealthResponse | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function fetchHealth() {
      try {
        const response = await get<HealthResponse>('/health')
        if (isApiSuccess(response)) {
          setHealth(response.data)
          setError(null)
        } else {
          setError(response.error.message)
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : '网络错误')
      } finally {
        setLoading(false)
      }
    }
    fetchHealth()
  }, [])

  if (loading) {
    return <div className="text-gray-500">检查后端连接...</div>
  }

  if (error) {
    return <div className="text-red-500">❌ 后端连接失败: {error}</div>
  }

  return (
    <div className="text-green-500">
      ✅ 后端连接成功 | 版本: {health?.version} | 状态: {health?.status}
    </div>
  )
}
