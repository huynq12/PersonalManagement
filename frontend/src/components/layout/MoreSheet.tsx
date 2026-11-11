import { X } from 'lucide-react'
import { useEffect, useId, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import LanguageSwitcher from '../LanguageSwitcher'
import ThemeSwitcher from '../ThemeSwitcher'
import NavItemLink from './NavItemLink'
import { MORE_SHEET_GROUPS, SETTINGS_ITEM } from './navigation'
import UserMenu from './UserMenu'

interface MoreSheetProps {
  onClose: () => void
}

/**
 * Bottom sheet on phones with the features that do not fit the bottom bar.
 *
 * Rendered only while open. It is a custom dialog rather than `<dialog>` because the test
 * environment (jsdom) does not implement `showModal`.
 */
export default function MoreSheet({ onClose }: MoreSheetProps) {
  const { t } = useTranslation()
  const titleId = useId()
  const closeButton = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    closeButton.current?.focus()

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
      }
    }
    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  return (
    <div className="fixed inset-0 z-50 md:hidden">
      {/* The backdrop is a pointer shortcut; keyboard users close with Escape or the button. */}
      <div className="absolute inset-0 bg-overlay" aria-hidden="true" onClick={onClose} />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="absolute inset-x-0 bottom-0 flex max-h-[85dvh] flex-col gap-1 overflow-y-auto rounded-t-2xl bg-background px-3 pt-2 pb-6"
      >
        <span aria-hidden="true" className="mx-auto mb-2 h-1 w-10 rounded-full bg-border" />
        <div className="flex items-center justify-between px-2.5">
          <h2 id={titleId} className="text-base font-semibold">
            {t('nav.moreTitle')}
          </h2>
          <button
            ref={closeButton}
            type="button"
            aria-label={t('nav.close')}
            onClick={onClose}
            className="flex size-11 items-center justify-center rounded-md hover:bg-accent"
          >
            <X className="size-5" aria-hidden="true" />
          </button>
        </div>

        <nav aria-label={t('nav.moreTitle')} className="flex flex-col gap-0.5">
          {MORE_SHEET_GROUPS.map((group) => (
            <div key={group.labelKey} className="flex flex-col gap-0.5">
              <p className="px-2.5 pt-3 pb-1 text-xs font-medium text-muted-foreground">
                {t(group.labelKey)}
              </p>
              {group.items.map((item) => (
                <NavItemLink key={item.path} item={item} onNavigate={onClose} />
              ))}
            </div>
          ))}
          <div className="my-2 h-px bg-border" />
          <NavItemLink item={SETTINGS_ITEM} onNavigate={onClose} />
        </nav>

        <div className="flex flex-col gap-3 px-2.5 py-3">
          <ThemeSwitcher />
          <LanguageSwitcher />
        </div>
        <UserMenu />
      </div>
    </div>
  )
}
