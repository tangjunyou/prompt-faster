import { useEffect, useState } from 'react'
import { Link, Routes, Route, Navigate, useLocation, useNavigate } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import './App.css'
import { ApiConfigPage, FocusView, OptimizationTaskConfigView, OptimizationTasksView, PerfNfr2View, PerfNfr3View, RunView, TestSetsView, WorkspaceView } from './pages'
import { LoginPage } from './features/auth/components/LoginPage'
import { ProtectedRoute } from './features/auth/components/ProtectedRoute'
import { registerUnauthorizedHandler } from './lib/api'
import { useAuthStore } from './stores/useAuthStore'
import { logout as logoutRequest } from './features/auth/services/authService'
import { Button } from './components/ui/button'
import { ViewSwitcher } from './components/common/ViewSwitcher'
import { WorkspaceSelector } from './components/common/WorkspaceSelector'
import { useWorkspaceStore } from './stores/useWorkspaceStore'

function App() {
  const navigate = useNavigate()
  const location = useLocation()
  const queryClient = useQueryClient()
  const logout = useAuthStore((state) => state.logout)
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const currentUser = useAuthStore((state) => state.currentUser)
  const authStatus = useAuthStore((state) => state.authStatus)
  const [isLoggingOut, setIsLoggingOut] = useState(false)

  useEffect(() => {
    registerUnauthorizedHandler(() => {
      queryClient.clear()
      useWorkspaceStore.getState().reset()
      logout()
      navigate('/login', { replace: true })
    })
  }, [logout, navigate, queryClient])

  const showHeader = location.pathname !== '/login'

  const handleLogout = async () => {
    if (!sessionToken) {
      queryClient.clear()
      logout()
      navigate('/login', { replace: true })
      return
    }

    setIsLoggingOut(true)
    try {
      await logoutRequest(sessionToken)
    } finally {
      setIsLoggingOut(false)
      queryClient.clear()
      logout()
      navigate('/login', { replace: true })
    }
  }

  return (
    <>
      {showHeader && (
        <header className="border-b bg-background">
          <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
            <div className="flex items-center gap-4">
              <Link to="/run" className="text-sm font-semibold">
              Prompt Faster
              </Link>
              <ViewSwitcher />
              {authStatus === 'authenticated' && <WorkspaceSelector />}
            </div>
            <div className="flex items-center gap-3" data-testid="user-menu">
              {authStatus === 'authenticated' && currentUser ? (
                <div className="text-sm text-muted-foreground">
                  已登录：{currentUser.username}
                </div>
              ) : (
                <div className="text-sm text-muted-foreground">未登录</div>
              )}

              {authStatus === 'authenticated' && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLogout}
                  disabled={isLoggingOut}
                  data-testid="logout-button"
                >
                  {isLoggingOut ? '退出中...' : '退出登录'}
                </Button>
              )}
            </div>
          </div>
        </header>
      )}

      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/" element={<Navigate to="/run" replace />} />
        <Route path="/run" element={<RunView />} />
        <Route path="/focus" element={<FocusView />} />
        <Route path="/workspace" element={<WorkspaceView />} />
        <Route path="/__perf__/nfr2" element={<PerfNfr2View />} />
        <Route path="/__perf__/nfr3" element={<PerfNfr3View />} />
        <Route
          path="/workspaces/:id/test-sets"
          element={
            <ProtectedRoute>
              <TestSetsView />
            </ProtectedRoute>
          }
        />
        <Route
          path="/workspaces/:id/tasks"
          element={
            <ProtectedRoute>
              <OptimizationTasksView />
            </ProtectedRoute>
          }
        />
        <Route
          path="/workspaces/:id/tasks/:taskId"
          element={
            <ProtectedRoute>
              <OptimizationTaskConfigView />
            </ProtectedRoute>
          }
        />
        <Route
          path="/settings/api"
          element={
            <ProtectedRoute>
              <ApiConfigPage />
            </ProtectedRoute>
          }
        />
      </Routes>
    </>
  )
}

export default App
