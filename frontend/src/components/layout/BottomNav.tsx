import { Menu } from 'lucide-react'
import { useCallback, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { NavLink } from 'react-router'
import MoreSheet from './MoreSheet'
import { BOTTOM_BAR_ITEMS } from './navigation'

const TAB_CLASSES =
  'flex min-h-14 flex-1 flex-col items-center justify-center gap-1 text-[11px] transition-colors'

/** The tab bar at the bottom of the screen on phones, with a "More" sheet for the rest. */
export default function BottomNav() {
  const { t } = useTranslation()
  const [moreOpen, setMoreOpen] = useState(false)
  const moreButton = useRef<HTMLButtonElement>(null)

  // Stable, so the sheet's effect does not re-run (and steal focus) on every render.
  const closeMore = useCallback(() => {
    setMoreOpen(false)
    moreButton.current?.focus()
  }, [])

  return (
    <>
      <nav
        aria-label={t('nav.label')}
        className="fixed inset-x-0 bottom-0 z-40 flex border-t border-border bg-background px-1 pb-[max(env(safe-area-inset-bottom),0.5rem)] md:hidden"
      >
        {BOTTOM_BAR_ITEMS.map((item) => {
          const Icon = item.icon
          return (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === '/'}
              className={({ isActive }) =>
                `${TAB_CLASSES} ${isActive ? 'font-semibold text-foreground' : 'text-muted-foreground'}`
              }
            >
              <Icon className="size-5" aria-hidden="true" />
              {t(item.shortLabelKey ?? item.labelKey)}
            </NavLink>
          )
        })}
        <button
          ref={moreButton}
          type="button"
          aria-haspopup="dialog"
          aria-expanded={moreOpen}
          onClick={() => setMoreOpen(true)}
          className={`${TAB_CLASSES} text-muted-foreground`}
        >
          <Menu className="size-5" aria-hidden="true" />
          {t('nav.more')}
        </button>
      </nav>

      {moreOpen ? <MoreSheet onClose={closeMore} /> : null}
    </>
  )
}
