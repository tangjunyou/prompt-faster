/**
 * 登录/注册页面
 * 根据系统状态显示登录或注册表单
 */

import { useState, useEffect } from 'react'
import { useLocation, useNavigate } from 'react-router'
import { useAuthStore } from '@/stores/useAuthStore'
import { getSystemStatus } from '../services/authService'
import { isApiError } from '@/lib/api'

export function LoginPage() {
  const navigate = useNavigate()
  const location = useLocation()
  const {
    authStatus,
    login,
    register,
    setRequiresRegistration,
    requiresRegistration,
  } = useAuthStore()

  const [isRegisterMode, setIsRegisterMode] = useState(false)
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [checkingStatus, setCheckingStatus] = useState(true)

  const redirectTo =
    (typeof (location.state as { from?: unknown } | null)?.from === 'string'
      ? ((location.state as { from: string }).from ?? '')
      : '') || '/'

  // 检查系统状态
  useEffect(() => {
    async function checkStatus() {
      try {
        const response = await getSystemStatus()
        if (!isApiError(response)) {
          setRequiresRegistration(response.data.requires_registration)
          setIsRegisterMode(response.data.requires_registration)
        }
      } catch (err) {
        console.error('检查系统状态失败:', err)
      } finally {
        setCheckingStatus(false)
      }
    }
    checkStatus()
  }, [setRequiresRegistration])

  // 已登录则跳转首页
  useEffect(() => {
    if (authStatus === 'authenticated') {
      navigate(redirectTo, { replace: true })
    }
  }, [authStatus, navigate, redirectTo])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    // 验证输入
    if (!username.trim()) {
      setError('请输入用户名')
      return
    }
    if (!password) {
      setError('请输入密码')
      return
    }
    if (isRegisterMode && password !== confirmPassword) {
      setError('两次输入的密码不一致')
      return
    }
    if (isRegisterMode && password.length < 6) {
      setError('密码长度至少 6 个字符')
      return
    }

    setLoading(true)

    try {
      const response = isRegisterMode
        ? await register({ username: username.trim(), password })
        : await login({ username: username.trim(), password })

      if (isApiError(response)) {
        setError(response.error.message)
      } else {
        navigate(redirectTo, { replace: true })
      }
    } catch {
      setError('操作失败，请稍后重试')
    } finally {
      setLoading(false)
    }
  }

  if (checkingStatus) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-gray-500">加载中...</div>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-md w-full space-y-8">
        <div>
          <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Prompt Faster
          </h2>
          <p className="mt-2 text-center text-sm text-gray-600">
            {isRegisterMode ? '创建您的账户' : '登录您的账户'}
          </p>
        </div>

        <form className="mt-8 space-y-6" onSubmit={handleSubmit}>
          {error && (
            <div className="rounded-md bg-red-50 p-4">
              <div className="text-sm text-red-700">{error}</div>
            </div>
          )}

          <div className="rounded-md shadow-sm -space-y-px">
            <div>
              <label htmlFor="username" className="sr-only">
                用户名
              </label>
              <input
                id="username"
                name="username"
                type="text"
                required
                data-testid="username-input"
                className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-t-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                placeholder="用户名"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={loading}
              />
            </div>
            <div>
              <label htmlFor="password" className="sr-only">
                密码
              </label>
              <input
                id="password"
                name="password"
                type="password"
                required
                data-testid="password-input"
                className={`appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 ${
                  isRegisterMode ? '' : 'rounded-b-md'
                } focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm`}
                placeholder="密码"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={loading}
              />
            </div>
            {isRegisterMode && (
              <div>
                <label htmlFor="confirmPassword" className="sr-only">
                  确认密码
                </label>
                <input
                  id="confirmPassword"
                  name="confirmPassword"
                  type="password"
                  required
                  data-testid="confirm-password-input"
                  className="appearance-none rounded-none relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 rounded-b-md focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10 sm:text-sm"
                  placeholder="确认密码"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  disabled={loading}
                />
              </div>
            )}
          </div>

          <div>
            <button
              type="submit"
              disabled={loading}
              data-testid="login-button"
              className="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? '处理中...' : isRegisterMode ? '注册' : '登录'}
            </button>
          </div>

          {!requiresRegistration && (
            <div className="text-center">
              <button
                type="button"
                onClick={() => {
                  setIsRegisterMode(!isRegisterMode)
                  setError(null)
                }}
                data-testid="toggle-auth-mode"
                className="text-sm text-blue-600 hover:text-blue-500"
              >
                {isRegisterMode ? '已有账户？点击登录' : '没有账户？点击注册'}
              </button>
            </div>
          )}
        </form>
      </div>
    </div>
  )
}
