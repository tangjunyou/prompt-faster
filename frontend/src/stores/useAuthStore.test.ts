import { describe, it, expect, beforeEach } from 'vitest'
import { useAuthStore } from './useAuthStore'

describe('useAuthStore', () => {
  beforeEach(() => {
    useAuthStore.setState({
      authStatus: 'unauthenticated',
      sessionToken: null,
      currentUser: null,
      requiresRegistration: null,
    })
  })

  it('loginSuccess 应设置 authenticated + token + currentUser', () => {
    useAuthStore.getState().loginSuccess('test-token', { id: 'u1', username: 'user1' })

    const state = useAuthStore.getState()
    expect(state.authStatus).toBe('authenticated')
    expect(state.sessionToken).toBe('test-token')
    expect(state.currentUser).toEqual({ id: 'u1', username: 'user1' })
  })

  it('logout 应清空 token 与 currentUser，并设置 unauthenticated', () => {
    useAuthStore.setState({
      authStatus: 'authenticated',
      sessionToken: 'test-token',
      currentUser: { id: 'u1', username: 'user1' },
    })

    useAuthStore.getState().logout()

    const state = useAuthStore.getState()
    expect(state.authStatus).toBe('unauthenticated')
    expect(state.sessionToken).toBeNull()
    expect(state.currentUser).toBeNull()
  })

  it('setRequiresRegistration 应更新 requiresRegistration', () => {
    useAuthStore.getState().setRequiresRegistration(true)
    expect(useAuthStore.getState().requiresRegistration).toBe(true)

    useAuthStore.getState().setRequiresRegistration(false)
    expect(useAuthStore.getState().requiresRegistration).toBe(false)
  })
})
