import { Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { formatShortDate } from '../../lib/dates'
import { formatMoney } from '../../lib/money'
import { formatHours, type WorkEntry, type WorkMonth } from '../../lib/workEntries'
import IconButton from '../ui/IconButton'
import ComparisonBadge from './ComparisonBadge'

interface WorkEntryListProps {
  month: WorkMonth
  currency: string
  /** Loads an entry into the form; `undefined` hides the edit buttons. */
  onEdit?: (entry: WorkEntry) => void
  onDelete: (entry: WorkEntry) => void
}

const CELL = 'px-3 py-2 align-top'
const HEADER = 'px-3 py-2 text-left text-sm font-medium text-muted-foreground'

/** The month's entries as a table, ending with the month's totals. */
export default function WorkEntryList({ month, currency, onEdit, onDelete }: WorkEntryListProps) {
  const { t, i18n } = useTranslation()
  const language = i18n.language

  if (month.entries.length === 0) {
    return <p className="text-sm text-muted-foreground">{t('workLog.list.empty')}</p>
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse text-sm">
        <caption className="sr-only">{t('workLog.list.title')}</caption>
        <thead>
          <tr className="border-b border-border">
            <th scope="col" className={HEADER}>
              {t('workLog.list.date')}
            </th>
            <th scope="col" className={`${HEADER} text-right`}>
              {t('workLog.list.hours')}
            </th>
            <th scope="col" className={`${HEADER} text-right`}>
              {t('workLog.list.pay')}
            </th>
            <th scope="col" className={HEADER}>
              {t('workLog.list.comparison')}
            </th>
            <td className={HEADER} />
          </tr>
        </thead>
        <tbody>
          {month.entries.map((entry) => {
            const date = formatShortDate(entry.date, language)
            return (
              <tr key={entry.date} className="border-b border-border">
                <th scope="row" className={`${CELL} text-left font-medium`}>
                  <span className="block">{date}</span>
                  {entry.note ? (
                    <span className="block font-normal text-muted-foreground">{entry.note}</span>
                  ) : null}
                </th>
                <td className={`${CELL} text-right tabular-nums`}>
                  {formatHours(entry.hours, language)}
                </td>
                <td className={`${CELL} text-right tabular-nums`}>
                  {formatMoney(entry.pay, currency, language)}
                </td>
                <td className={CELL}>
                  <ComparisonBadge comparison={entry.comparison} />
                </td>
                <td className={`${CELL} text-right whitespace-nowrap`}>
                  {onEdit ? (
                    <IconButton
                      icon={Pencil}
                      label={t('workLog.list.edit', { date })}
                      onClick={() => onEdit(entry)}
                    />
                  ) : null}
                  <IconButton
                    icon={Trash2}
                    label={t('workLog.list.delete', { date })}
                    onClick={() => onDelete(entry)}
                  />
                </td>
              </tr>
            )
          })}
        </tbody>
        <tfoot>
          <tr className="font-semibold">
            <th scope="row" className={`${CELL} text-left`}>
              {t('workLog.list.total')}
            </th>
            <td className={`${CELL} text-right tabular-nums`}>
              {formatHours(month.summary.total_hours, language)}
            </td>
            <td className={`${CELL} text-right tabular-nums`}>
              {formatMoney(month.summary.total_pay, currency, language)}
            </td>
            <td className={CELL} colSpan={2}>
              {t('workLog.list.daysWorked', { count: month.summary.days_worked })}
            </td>
          </tr>
        </tfoot>
      </table>
    </div>
  )
}
