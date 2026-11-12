import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { addDays, formatLongDate, type IsoDate } from '../../lib/dates'
import { BUTTON_CLASSES } from '../ui/buttonClasses'

interface DayNavigationProps {
  /** The day being viewed. */
  date: IsoDate
  /** Today in the user's time zone. */
  today: IsoDate
}

/** Path of the Today screen for a date. Today itself has the clean URL `/`. */
function dayPath(date: IsoDate, today: IsoDate): string {
  return date === today ? '/' : `/?date=${date}`
}

const ARROW_CLASSES =
  'flex size-9 shrink-0 items-center justify-center rounded-md border border-input text-foreground transition-colors hover:bg-accent focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring'

/**
 * Moves the Today screen between days. The day lives in the URL, so these are plain links:
 * the browser's back and forward buttons walk through the days that were viewed.
 */
export default function DayNavigation({ date, today }: DayNavigationProps) {
  const { t, i18n } = useTranslation()

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Link
        to={dayPath(addDays(date, -1), today)}
        aria-label={t('home.previousDay')}
        title={t('home.previousDay')}
        className={ARROW_CLASSES}
      >
        <ChevronLeft className="size-4" aria-hidden="true" />
      </Link>
      <p className="min-w-0 px-1 text-sm font-medium">
        {formatLongDate(date, i18n.resolvedLanguage)}
      </p>
      <Link
        to={dayPath(addDays(date, 1), today)}
        aria-label={t('home.nextDay')}
        title={t('home.nextDay')}
        className={ARROW_CLASSES}
      >
        <ChevronRight className="size-4" aria-hidden="true" />
      </Link>
      {date === today ? null : (
        <Link to="/" className={`${BUTTON_CLASSES.ghost} h-9 px-3`}>
          {t('home.backToToday')}
        </Link>
      )}
    </div>
  )
}
