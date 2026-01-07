export function getWorkspaceIdFromPathname(pathname: string): string | null {
  const match = pathname.match(/^\/workspaces\/([^/]+)(?:\/|$)/)
  return match?.[1] ?? null
}

export function getWorkspaceSwitchTargetPath(pathname: string, workspaceId: string): string {
  const match = pathname.match(/^\/workspaces\/[^/]+\/(test-sets|tasks)(?:\/([^/]+))?\/?$/)
  if (match) {
    const section = match[1]
    const tail = match[2]
    if (section === 'test-sets') return `/workspaces/${workspaceId}/test-sets`
    if (section === 'tasks' && tail) return `/workspaces/${workspaceId}/tasks/${tail}`
    if (section === 'tasks') return `/workspaces/${workspaceId}/tasks`
  }
  return `/workspaces/${workspaceId}/tasks`
}
