import type { NodeStatus } from './types'

const baseNodeClassName =
  'rounded-md border px-3 py-2 text-sm font-medium shadow-sm transition-colors select-none'

export function nodeStatusToClassName(status: NodeStatus): string {
  switch (status) {
    case 'idle':
      return `${baseNodeClassName} bg-slate-100 border-slate-300 text-slate-700`
    case 'running':
      return `${baseNodeClassName} bg-blue-50 border-blue-400 text-blue-800 animate-pulse`
    case 'success':
      return `${baseNodeClassName} bg-emerald-50 border-emerald-400 text-emerald-800`
    case 'error':
      return `${baseNodeClassName} bg-red-50 border-red-400 text-red-800`
    case 'paused':
      return `${baseNodeClassName} bg-yellow-50 border-yellow-400 text-yellow-900`
  }
}

