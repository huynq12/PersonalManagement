import { useTranslation } from 'react-i18next'
import type { FinanceSummary } from '../../lib/finance'
import { formatMoney } from '../../lib/money'
import Card from '../ui/Card'

interface ExpenseBreakdownProps {
  summary: FinanceSummary
  currency: string
}

/** The month's expenses per category as a list with proportional bars. */
export default function ExpenseBreakdown({ summary, currency }: ExpenseBreakdownProps) {
  const { t, i18n } = useTranslation()
  const rows = summary.by_category.filter((row) => row.kind === 'expense')

  return (
    <Card title={t('finances.byCategory.title')}>
      {rows.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t('finances.byCategory.empty')}</p>
      ) : (
        <ul className="flex flex-col gap-3">
          {rows.map((row) => (
            <li key={row.category.id} className="flex flex-col gap-1">
              <div className="flex justify-between gap-3 text-sm">
                <span>{row.category.name}</span>
                <span className="tabular-nums">
                  {formatMoney(row.total, currency, i18n.language)}
                </span>
              </div>
              <div aria-hidden="true" className="h-2 overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full rounded-full"
                  style={{
                    width: `${(row.total / summary.expense_total) * 100}%`,
                    backgroundColor: row.category.color,
                  }}
                />
              </div>
            </li>
          ))}
        </ul>
      )}
    </Card>
  )
}
