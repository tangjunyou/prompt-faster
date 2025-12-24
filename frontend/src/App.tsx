import { useEffect, useState } from 'react'
import { Link, Routes, Route, useLocation, useNavigate } from 'react-router'
import './App.css'
import { HomePage, ApiConfigPage } from './pages'
import { LoginPage } from './features/auth/components/LoginPage'
import { ProtectedRoute } from './features/auth/components/ProtectedRoute'
import { registerUnauthorizedHandler } from './lib/api'
import { useAuthStore } from './stores/useAuthStore'
import { logout as logoutRequest } from './features/auth/services/authService'
import { Button } from './components/ui/button'

function App() {
  const navigate = useNavigate()
  const location = useLocation()
  const logout = useAuthStore((state) => state.logout)
  const sessionToken = useAuthStore((state) => state.sessionToken)
  const currentUser = useAuthStore((state) => state.currentUser)
  const authStatus = useAuthStore((state) => state.authStatus)
  const [isLoggingOut, setIsLoggingOut] = useState(false)

  useEffect(() => {
    registerUnauthorizedHandler(() => {
      logout()
      navigate('/login', { replace: true })
    })
  }, [logout, navigate])

  const showHeader = location.pathname !== '/login'

  const handleLogout = async () => {
    if (!sessionToken) {
      logout()
      navigate('/login', { replace: true })
      return
    }

    setIsLoggingOut(true)
    try {
      await logoutRequest(sessionToken)
    } finally {
      setIsLoggingOut(false)
      logout()
      navigate('/login', { replace: true })
    }
  }

  return (
    <>
      {showHeader && (
        <header className="border-b bg-background">
          <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
            <Link to="/" className="text-sm font-semibold">
              Prompt Faster
            </Link>
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
        <Route path="/" element={<HomePage />} />
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
