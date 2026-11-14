import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { useSession } from '../../hooks/useSession'
import { formatMoney } from '../../lib/money'
import { formatHours } from '../../lib/workEntries'
import Card from '../ui/Card'

interface MoneyCardProps {
  /** The hours and the pay of the day, or `null` when nothing was logged. */
  work: { hours: number; pay: number } | null
  /** Total expenses of the day in minor units. */
  expenseTotal: number
}

const LINK_CLASSES = 'font-medium text-foreground underline underline-offset-4'

/** A read-only summary of the day's work and spending, with links to the screens that edit it. */
export default function MoneyCard({ work, expenseTotal }: MoneyCardProps) {
  const { t, i18n } = useTranslation()
  const session = useSession()
  const currency = session.data?.currency ?? 'VND'
  const money = (amount: number) => formatMoney(amount, currency, i18n.language)

  return (
    <Card title={t('home.money.title')}>
      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
        {work ? (
          <>
            <dt className="text-muted-foreground">{t('home.money.hoursWorked')}</dt>
            <dd className="text-right font-medium">{formatHours(work.hours, i18n.language)}</dd>
            <dt className="text-muted-foreground">{t('home.money.pay')}</dt>
            <dd className="text-right font-medium">{money(work.pay)}</dd>
          </>
        ) : (
          <dd className="col-span-2 text-muted-foreground">{t('home.money.noWork')}</dd>
        )}
        <dt className="text-muted-foreground">{t('home.money.spent')}</dt>
        <dd className="text-right font-medium">{money(expenseTotal)}</dd>
      </dl>
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm">
        <Link to="/work-log" className={LINK_CLASSES}>
          {t('home.money.workLog')}
        </Link>
        <Link to="/finances" className={LINK_CLASSES}>
          {t('home.money.finances')}
        </Link>
      </div>
    </Card>
  )
}
