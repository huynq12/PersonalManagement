import { X } from 'lucide-react'
import { useEffect, useId, useRef, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import IconButton from './IconButton'

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

interface DialogProps {
  title: string
  /** Called when the user dismisses the dialog: Escape, the close button, or the backdrop. */
  onClose: () => void
  children: ReactNode
}

/**
 * A modal dialog centered over the page. Render it only while it is open.
 *
 * It is a custom dialog rather than `<dialog>` because the test environment (jsdom) does not
 * implement `showModal`. It does what a modal must: Escape closes it, Tab stays inside it, and
 * focus returns to where it was when it closes.
 */
export default function Dialog({ title, onClose, children }: DialogProps) {
  const { t } = useTranslation()
  const titleId = useId()
  const panel = useRef<HTMLDivElement>(null)

  // Focus management runs once when the dialog opens, and is undone when it closes.
  useEffect(() => {
    const previouslyFocused = document.activeElement
    // A field inside may already have taken focus (autoFocus); otherwise focus the dialog.
    if (!panel.current?.contains(document.activeElement)) {
      panel.current?.focus()
    }
    return () => {
      if (previouslyFocused instanceof HTMLElement) {
        previouslyFocused.focus()
      }
    }
  }, [])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
        return
      }
      if (event.key !== 'Tab' || !panel.current) {
        return
      }

      // Keep Tab inside the dialog: wrap from the last element to the first and back.
      const focusable = panel.current.querySelectorAll<HTMLElement>(FOCUSABLE)
      const first = focusable[0]
      const last = focusable[focusable.length - 1]
      if (!first || !last) {
        return
      }
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault()
        last.focus()
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault()
        first.focus()
      }
    }

    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center p-0 sm:items-center sm:p-4">
      {/* The backdrop is a pointer shortcut; keyboard users close with Escape or the button. */}
      <div className="absolute inset-0 bg-overlay" aria-hidden="true" onClick={onClose} />
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        className="relative flex max-h-[90dvh] w-full flex-col gap-4 overflow-y-auto rounded-t-2xl bg-background p-6 outline-none sm:max-w-md sm:rounded-xl sm:border sm:border-border"
      >
        <div className="flex items-center justify-between gap-3">
          <h2 id={titleId} className="text-lg font-semibold">
            {title}
          </h2>
          <IconButton icon={X} label={t('common.close')} onClick={onClose} />
        </div>
        {children}
      </div>
    </div>
  )
}
