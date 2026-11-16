import { useTranslation } from 'react-i18next'
import { formatLongDate, type IsoDate } from '../../lib/dates'
import { formatMoney } from '../../lib/money'
import { formatHours, type WorkEntry } from '../../lib/workEntries'
import Dialog from '../ui/Dialog'
import ComparisonBadge from './ComparisonBadge'
import WorkEntryForm, { type PutMutation } from './WorkEntryForm'

interface WorkEntryDialogProps {
  date: IsoDate
  today: IsoDate
  /** The day's entry to edit, or `null` to log the day for the first time. */
  entry: WorkEntry | null
  currency: string
  save: PutMutation
  onClose: () => void
  /** Asks to delete the entry; the page confirms it. Only used when `entry` is set. */
  onDelete: (entry: WorkEntry) => void
}

/** The form for one day of the calendar, shown in a dialog. */
export default function WorkEntryDialog({
  date,
  today,
  entry,
  currency,
  save,
  onClose,
  onDelete,
}: WorkEntryDialogProps) {
  const { t, i18n } = useTranslation()
  const language = i18n.language

  return (
    <Dialog title={formatLongDate(date, language)} onClose={onClose}>
      {entry ? (
        <div className="flex flex-col gap-2 rounded-lg bg-muted p-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <p className="text-lg font-semibold tabular-nums">
              {formatMoney(entry.pay, currency, language)}
            </p>
            <ComparisonBadge comparison={entry.comparison} />
          </div>
          <p className="text-sm text-muted-foreground">
            {t('workLog.dialog.snapshot', {
              wage: formatMoney(entry.hourly_wage, currency, language),
              hours: formatHours(entry.base_hours, language),
            })}
          </p>
        </div>
      ) : null}

      <WorkEntryForm
        date={date}
        today={today}
        entry={entry}
        save={save}
        onSaved={onClose}
        onCancel={onClose}
        onDelete={entry ? () => onDelete(entry) : undefined}
      />
    </Dialog>
  )
}
