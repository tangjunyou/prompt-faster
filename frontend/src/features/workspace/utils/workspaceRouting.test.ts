import { describe, expect, it } from 'vitest'
import { getWorkspaceIdFromPathname, getWorkspaceSwitchTargetPath } from './workspaceRouting'

describe('workspaceRouting', () => {
  describe('getWorkspaceIdFromPathname', () => {
    it('extracts workspace id from /workspaces/:id', () => {
      expect(getWorkspaceIdFromPathname('/workspaces/ws-1')).toBe('ws-1')
      expect(getWorkspaceIdFromPathname('/workspaces/ws-1/')).toBe('ws-1')
      expect(getWorkspaceIdFromPathname('/workspaces/ws-1/tasks')).toBe('ws-1')
    })

    it('returns null for non workspace-scoped routes', () => {
      expect(getWorkspaceIdFromPathname('/workspace')).toBeNull()
      expect(getWorkspaceIdFromPathname('/run')).toBeNull()
      expect(getWorkspaceIdFromPathname('/')).toBeNull()
    })
  })

  describe('getWorkspaceSwitchTargetPath', () => {
    it('keeps section: test-sets', () => {
      expect(getWorkspaceSwitchTargetPath('/workspaces/ws-1/test-sets', 'ws-2')).toBe('/workspaces/ws-2/test-sets')
      expect(getWorkspaceSwitchTargetPath('/workspaces/ws-1/test-sets/', 'ws-2')).toBe('/workspaces/ws-2/test-sets')
    })

    it('keeps section: tasks', () => {
      expect(getWorkspaceSwitchTargetPath('/workspaces/ws-1/tasks', 'ws-2')).toBe('/workspaces/ws-2/tasks')
      expect(getWorkspaceSwitchTargetPath('/workspaces/ws-1/tasks/', 'ws-2')).toBe('/workspaces/ws-2/tasks')
    })

    it('keeps tail for tasks/:taskId', () => {
      expect(getWorkspaceSwitchTargetPath('/workspaces/ws-1/tasks/task-1', 'ws-2')).toBe('/workspaces/ws-2/tasks/task-1')
    })

    it('falls back to /tasks for unknown sections', () => {
      expect(getWorkspaceSwitchTargetPath('/workspaces/ws-1/unknown', 'ws-2')).toBe('/workspaces/ws-2/tasks')
      expect(getWorkspaceSwitchTargetPath('/workspace', 'ws-2')).toBe('/workspaces/ws-2/tasks')
    })
  })
})

