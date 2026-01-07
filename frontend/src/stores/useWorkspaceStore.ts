import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'

interface WorkspaceState {
  lastWorkspaceIdByUser: Record<string, string>
}

interface WorkspaceActions {
  setLastWorkspaceId: (userId: string, workspaceId: string) => void
  clearLastWorkspaceId: (userId: string) => void
  reset: () => void
}

const initialState: WorkspaceState = {
  lastWorkspaceIdByUser: {},
}

export const useWorkspaceStore = create<WorkspaceState & WorkspaceActions>()(
  persist(
    (set) => ({
      ...initialState,

      setLastWorkspaceId: (userId, workspaceId) =>
        set((state) => ({
          lastWorkspaceIdByUser: {
            ...state.lastWorkspaceIdByUser,
            [userId]: workspaceId,
          },
        })),

      clearLastWorkspaceId: (userId) =>
        set((state) => {
          const next = { ...state.lastWorkspaceIdByUser }
          delete next[userId]
          return { lastWorkspaceIdByUser: next }
        }),

      reset: () => set(initialState),
    }),
    {
      name: 'workspace-store',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ lastWorkspaceIdByUser: state.lastWorkspaceIdByUser }),
    }
  )
)

