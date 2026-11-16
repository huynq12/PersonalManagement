import { useTranslation } from 'react-i18next'
import { formatMoney } from '../../lib/money'
import { formatHours, type WorkMonth } from '../../lib/workEntries'

interface WorkSummaryProps {
  month: WorkMonth
  currency: string
}

/** Total hours, total pay, and days worked for the month, as three small cards. */
export default function WorkSummary({ month, currency }: WorkSummaryProps) {
  const { t, i18n } = useTranslation()
  const { total_hours, total_pay, days_worked } = month.summary

  const items = [
    { label: t('workLog.summary.hours'), value: formatHours(total_hours, i18n.language) },
    { label: t('workLog.summary.pay'), value: formatMoney(total_pay, currency, i18n.language) },
    { label: t('workLog.summary.days'), value: t('workLog.summary.daysWorked', { count: days_worked }) },
  ]

  return (
    <section aria-label={t('workLog.summary.title')} className="grid grid-cols-3 gap-2 sm:gap-4">
      {items.map((item) => (
        <dl
          key={item.label}
          className="flex min-w-0 flex-col gap-1 rounded-xl border border-border p-2.5 sm:p-4"
        >
          <dt className="text-xs text-muted-foreground sm:text-sm">{item.label}</dt>
          <dd className="text-base font-semibold tabular-nums tracking-tight sm:text-2xl">
            {item.value}
          </dd>
        </dl>
      ))}
    </section>
  )
}
