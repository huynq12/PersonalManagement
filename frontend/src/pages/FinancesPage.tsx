import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import BudgetBar from '../components/finance/BudgetBar'
import DeleteTransactionDialog from '../components/finance/DeleteTransactionDialog'
import ExpenseBreakdown from '../components/finance/ExpenseBreakdown'
import TransactionFormCard from '../components/finance/TransactionForm'
import TransactionList from '../components/finance/TransactionList'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import MonthPicker from '../components/work/MonthPicker'
import { useFinanceCategories, useFinanceSummary, useTransactions } from '../hooks/useFinance'
import { useSession } from '../hooks/useSession'
import { errorMessageKey } from '../i18n/errorMessage'
import { isIsoMonth, monthOf, todayIn, type IsoMonth } from '../lib/dates'
import type { Transaction } from '../lib/finance'
import { formatMoney } from '../lib/money'

/**
 * The Finances screen: income, expenses, and the budget for one month at a time.
 *
 * The month comes from the URL (`/finances?month=YYYY-MM`); without it, or with an invalid
 * value, the screen shows the current month. Transactions may be dated in the future, so any
 * month can be shown.
 */
export default function FinancesPage() {
  const { t, i18n } = useTranslation()
  const [searchParams, setSearchParams] = useSearchParams()
  const session = useSession()
  const [editing, setEditing] = useState<Transaction | null>(null)
  const [deleting, setDeleting] = useState<Transaction | null>(null)

  // "Today" is the date in the user's time zone, not the browser's. Read the clock once per
  // visit so the page does not jump to another day while it is open.
  const timeZone = session.data?.timezone ?? Intl.DateTimeFormat().resolvedOptions().timeZone
  const [today] = useState(() => todayIn(timeZone))
  const currentMonth = monthOf(today)

  const requested = searchParams.get('month')
  const month: IsoMonth = requested && isIsoMonth(requested) ? requested : currentMonth
  const currency = session.data?.currency

  const categories = useFinanceCategories()
  const transactions = useTransactions(month)
  const summary = useFinanceSummary(month)

  const changeMonth = (next: IsoMonth) => {
    setEditing(null)
    setSearchParams(next === currentMonth ? {} : { month: next })
  }

  const failed = categories.isError ? categories : transactions.isError ? transactions : summary
  const hasError = categories.isError || transactions.isError || summary.isError
  const loading = !currency || categories.isPending || transactions.isPending || summary.isPending

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('finances.title')}</h1>
        <p className="text-sm text-muted-foreground">{t('finances.description')}</p>
      </header>

      <MonthPicker month={month} onChange={changeMonth} />

      {hasError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(failed.error))}</Alert>
          <Button
            variant="secondary"
            onClick={() => {
              void categories.refetch()
              void transactions.refetch()
              void summary.refetch()
            }}
          >
            {t('common.retry')}
          </Button>
        </div>
      ) : loading ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : (
        <>
          <dl className="grid gap-3 sm:grid-cols-3">
            <Figure
              label={t('finances.summary.income')}
              value={formatMoney(summary.data.income_total, currency, i18n.language)}
            />
            <Figure
              label={t('finances.summary.expenses')}
              value={formatMoney(summary.data.expense_total, currency, i18n.language)}
            />
            <Figure
              label={t('finances.summary.balance')}
              value={formatMoney(summary.data.balance, currency, i18n.language)}
              negative={summary.data.balance < 0}
            />
          </dl>

          <BudgetBar budget={summary.data.budget} currency={currency} />

          <TransactionFormCard
            // A different transaction (or none) starts the form over with its own values.
            key={editing?.id ?? 'new'}
            today={today}
            currency={currency}
            categories={categories.data}
            editing={editing}
            onCancelEdit={() => setEditing(null)}
            onSaved={() => setEditing(null)}
          />

          <ExpenseBreakdown summary={summary.data} currency={currency} />

          <TransactionList
            transactions={transactions.data}
            currency={currency}
            onEdit={setEditing}
            onDelete={setDeleting}
          />
        </>
      )}

      {deleting ? (
        <DeleteTransactionDialog transaction={deleting} onClose={() => setDeleting(null)} />
      ) : null}
    </main>
  )
}

function Figure({
  label,
  value,
  negative = false,
}: {
  label: string
  value: string
  negative?: boolean
}) {
  return (
    <div className="flex flex-col gap-1 rounded-xl border border-border p-4">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className={`text-xl font-semibold tabular-nums ${negative ? 'text-destructive' : ''}`}>
        {value}
      </dd>
    </div>
  )
}
