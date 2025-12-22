/**
 * API 类型定义
 * 由 ts-rs 从后端自动生成（占位）
 */

export interface HealthResponse {
  status: string
  version: string
  timestampMs: number
}

export interface User {
  id: string
  username: string
  created_at: number
  updated_at: number
}

export interface Workspace {
  id: string
  user_id: string
  name: string
  description?: string
  created_at: number
  updated_at: number
}
