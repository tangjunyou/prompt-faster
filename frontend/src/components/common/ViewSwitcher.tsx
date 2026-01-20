import { useEffect } from 'react'
import { NavLink, useLocation, useNavigate } from 'react-router'
import { Button } from '@/components/ui/button'

const VIEW_DEFINITIONS = [
  { key: 'run', label: 'Run', path: '/run', shortcut: '1' },
  { key: 'focus', label: 'Focus', path: '/focus', shortcut: '2' },
  { key: 'workspace', label: 'Workspace', path: '/workspace', shortcut: '3' },
  { key: 'meta', label: 'Meta', path: '/meta-optimization', shortcut: '4' },
] as const

const isEditableTarget = (target: EventTarget | null): boolean => {
  if (!(target instanceof HTMLElement)) return false
  const tagName = target.tagName
  return (
    tagName === 'INPUT' ||
    tagName === 'TEXTAREA' ||
    target.isContentEditable ||
    target.getAttribute('role') === 'textbox'
  )
}

export function ViewSwitcher() {
  const location = useLocation()
  const navigate = useNavigate()

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey)) return
      if (event.altKey) return
      if (isEditableTarget(event.target)) return

      const matchedView = VIEW_DEFINITIONS.find((view) => {
        return event.code === `Digit${view.shortcut}` || event.code === `Numpad${view.shortcut}`
      })

      if (!matchedView) return
      event.preventDefault()
      navigate(matchedView.path)
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [navigate])

  return (
    <div className="flex items-center gap-2" data-testid="view-switcher">
      {VIEW_DEFINITIONS.map((view) => {
        const isActive =
          location.pathname === view.path || location.pathname.startsWith(`${view.path}/`)
        return (
          <Button
            key={view.key}
            size="sm"
            variant={isActive ? 'default' : 'outline'}
            asChild
          >
            <NavLink
              to={view.path}
              data-testid={`view-switcher-${view.key}`}
              data-active={isActive}
              aria-current={isActive ? 'page' : undefined}
            >
              {view.label}
            </NavLink>
          </Button>
        )
      })}
    </div>
  )
}
