import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter, Routes, Route } from 'react-router'
import { ProtectedRoute } from './ProtectedRoute'
import { useAuthStore } from '@/stores/useAuthStore'

function TestPage() {
  return <div>受保护内容</div>
}

describe('ProtectedRoute', () => {
  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'unauthenticated',
      sessionToken: null,
      currentUser: null,
      requiresRegistration: null,
    })
  })

  it('未登录时应重定向到 /login', () => {
    render(
      <MemoryRouter initialEntries={['/settings/api']}>
        <Routes>
          <Route
            path="/settings/api"
            element={
              <ProtectedRoute>
                <TestPage />
              </ProtectedRoute>
            }
          />
          <Route path="/login" element={<div>登录页</div>} />
        </Routes>
      </MemoryRouter>
    )

    expect(screen.getByText('登录页')).toBeInTheDocument()
  })

  it('已登录时应渲染子组件', () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
    })

    render(
      <MemoryRouter initialEntries={['/settings/api']}>
        <Routes>
          <Route
            path="/settings/api"
            element={
              <ProtectedRoute>
                <TestPage />
              </ProtectedRoute>
            }
          />
          <Route path="/login" element={<div>登录页</div>} />
        </Routes>
      </MemoryRouter>
    )

    expect(screen.getByText('受保护内容')).toBeInTheDocument()
  })
})
