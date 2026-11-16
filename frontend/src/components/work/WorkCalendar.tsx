import { useTranslation } from 'react-i18next'
import { formatShortDate, monthGrid, weekdayNames, type IsoDate, type IsoMonth } from '../../lib/dates'
import { formatMoney } from '../../lib/money'
import { formatHours, type WorkEntry, type WorkMonth } from '../../lib/workEntries'
import { COMPARISON_LABEL_KEYS, STATUS_CLASSES, type DayStatus } from './dayStatus'

interface WorkCalendarProps {
  month: WorkMonth
  /** The month the grid shows; `month` holds that month's entries. */
  monthKey: IsoMonth
  today: IsoDate
  currency: string
  /** Called with the clicked day and its entry, or `null` when nothing is logged that day. */
  onSelectDay: (date: IsoDate, entry: WorkEntry | null) => void
}

/**
 * A month as a grid of days, Monday first. Each day is colored by how its hours compare with
 * the base hours and shows the hours and pay. Days after today cannot be clicked, because the
 * backend does not accept entries for the future.
 */
export default function WorkCalendar({
  month,
  monthKey,
  today,
  currency,
  onSelectDay,
}: WorkCalendarProps) {
  const { t, i18n } = useTranslation()
  const language = i18n.language
  const entryByDate = new Map(month.entries.map((entry) => [entry.date, entry]))

  return (
    <div role="grid" aria-label={t('workLog.calendar.label')} className="flex flex-col gap-1">
      <div role="row" className="grid grid-cols-7 gap-1">
        {weekdayNames(language).map((name) => (
          <div
            key={name}
            role="columnheader"
            className="py-1 text-center text-xs font-medium text-muted-foreground"
          >
            {name}
          </div>
        ))}
      </div>
      {monthGrid(monthKey).map((week) => (
        <div key={week.find((date) => date !== null) ?? 'empty'} role="row" className="grid grid-cols-7 gap-1">
          {week.map((date, column) => (
            <div key={date ?? `blank-${column}`} role="gridcell" className="min-w-0">
              {date === null ? null : (
                <DayCell
                  date={date}
                  entry={entryByDate.get(date) ?? null}
                  today={today}
                  currency={currency}
                  onSelect={onSelectDay}
                />
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  )
}

interface DayCellProps {
  date: IsoDate
  entry: WorkEntry | null
  today: IsoDate
  currency: string
  onSelect: (date: IsoDate, entry: WorkEntry | null) => void
}

function DayCell({ date, entry, today, currency, onSelect }: DayCellProps) {
  const { t, i18n } = useTranslation()
  const language = i18n.language
  const status: DayStatus = entry ? entry.comparison : date > today ? 'future' : 'none'
  const isToday = date === today
  const dateLabel = formatShortDate(date, language)

  const label = entry
    ? t('workLog.calendar.entry', {
        date: dateLabel,
        count: entry.hours,
        hours: formatHours(entry.hours, language),
        pay: formatMoney(entry.pay, currency, language),
        status: t(COMPARISON_LABEL_KEYS[entry.comparison]),
      })
    : status === 'future'
      ? t('workLog.calendar.future', { date: dateLabel })
      : t('workLog.calendar.empty', { date: dateLabel })

  return (
    <button
      type="button"
      disabled={status === 'future'}
      data-status={status}
      aria-label={label}
      aria-current={isToday ? 'date' : undefined}
      onClick={() => onSelect(date, entry)}
      className={`flex h-full min-h-16 w-full min-w-0 flex-col items-start gap-0.5 rounded-md border p-1 text-left text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring enabled:cursor-pointer enabled:hover:brightness-95 sm:min-h-24 sm:rounded-lg sm:p-2 dark:enabled:hover:brightness-125 ${STATUS_CLASSES[status]}`}
    >
      <span
        className={`inline-flex size-5 items-center justify-center rounded-full font-medium tabular-nums sm:size-6 sm:text-sm ${isToday ? 'bg-primary text-primary-foreground' : ''}`}
      >
        {Number(date.slice(8))}
      </span>
      {entry ? (
        <>
          <span className="font-semibold tabular-nums sm:text-sm">
            {t('workLog.calendar.hoursShort', { hours: formatHours(entry.hours, language) })}
          </span>
          <span className="hidden tabular-nums sm:block">
            {formatMoney(entry.pay, currency, language)}
          </span>
          {entry.note ? (
            <span className="hidden max-w-full line-clamp-1 opacity-80 sm:block">{entry.note}</span>
          ) : null}
        </>
      ) : status === 'none' ? (
        <>
          <span className="font-semibold tabular-nums sm:hidden">0</span>
          <span className="hidden font-semibold tabular-nums sm:block">
            {formatMoney(0, currency, language)}
          </span>
        </>
      ) : null}
    </button>
  )
}
