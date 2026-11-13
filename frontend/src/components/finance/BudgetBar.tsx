import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { formatMoney } from '../../lib/money'
import Card from '../ui/Card'

interface BudgetBarProps {
  /** `null` when the user has not set a budget. */
  budget: { amount: number; spent: number } | null
  currency: string
}

/** How much of the monthly budget the month has spent; a link to Settings when none is set. */
export default function BudgetBar({ budget, currency }: BudgetBarProps) {
  const { t, i18n } = useTranslation()
  const language = i18n.language

  if (budget === null) {
    return (
      <Card title={t('finances.budget.title')}>
        <p className="text-sm text-muted-foreground">{t('finances.budget.none')}</p>
        <Link to="/settings" className="text-sm font-medium underline underline-offset-4">
          {t('finances.budget.setLink')}
        </Link>
      </Card>
    )
  }

  const over = budget.spent > budget.amount
  const percent = Math.min(100, Math.round((budget.spent / budget.amount) * 100))
  const difference = formatMoney(Math.abs(budget.amount - budget.spent), currency, language)

  return (
    <Card
      title={t('finances.budget.title')}
      aside={t('finances.budget.used', {
        spent: formatMoney(budget.spent, currency, language),
        budget: formatMoney(budget.amount, currency, language),
      })}
    >
      <div
        role="progressbar"
        aria-label={t('finances.budget.title')}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={percent}
        data-state={over ? 'over' : 'within'}
        className="h-3 overflow-hidden rounded-full bg-muted"
      >
        <div
          className={`h-full rounded-full ${over ? 'bg-destructive' : 'bg-primary'}`}
          style={{ width: `${percent}%` }}
        />
      </div>
      <p className={`text-sm ${over ? 'font-medium text-destructive' : 'text-muted-foreground'}`}>
        {over
          ? t('finances.budget.over', { amount: difference })
          : t('finances.budget.left', { amount: difference })}
      </p>
    </Card>
  )
}
