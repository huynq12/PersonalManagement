import { Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { formatShortDate } from '../../lib/dates'
import type { Transaction } from '../../lib/finance'
import { formatMoney } from '../../lib/money'
import Card from '../ui/Card'
import IconButton from '../ui/IconButton'

interface TransactionListProps {
  transactions: Transaction[]
  currency: string
  onEdit: (transaction: Transaction) => void
  onDelete: (transaction: Transaction) => void
}

/** The month's transactions, newest first. Income is marked with a plus sign. */
export default function TransactionList({
  transactions,
  currency,
  onEdit,
  onDelete,
}: TransactionListProps) {
  const { t, i18n } = useTranslation()
  const language = i18n.language

  return (
    <Card title={t('finances.list.title')}>
      {transactions.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t('finances.list.empty')}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-border">
          {transactions.map((transaction) => {
            const date = formatShortDate(transaction.date, language)
            const income = transaction.kind === 'income'
            return (
              <li key={transaction.id} className="flex items-center gap-3 py-2">
                <span
                  aria-hidden="true"
                  className="size-3 shrink-0 rounded-full"
                  style={{ backgroundColor: transaction.category.color }}
                />
                <div className="flex min-w-0 flex-1 flex-col">
                  <span className="text-sm font-medium">{transaction.category.name}</span>
                  <span className="truncate text-xs text-muted-foreground">
                    {date}
                    {transaction.note ? ` · ${transaction.note}` : ''}
                  </span>
                </div>
                <span
                  className={`text-sm font-medium tabular-nums ${income ? 'text-success' : ''}`}
                >
                  {income ? '+' : '-'}
                  {formatMoney(transaction.amount, currency, language)}
                </span>
                <IconButton
                  icon={Pencil}
                  label={t('finances.list.edit', {
                    category: transaction.category.name,
                    date,
                  })}
                  onClick={() => onEdit(transaction)}
                />
                <IconButton
                  icon={Trash2}
                  label={t('finances.list.delete', {
                    category: transaction.category.name,
                    date,
                  })}
                  onClick={() => onDelete(transaction)}
                />
              </li>
            )
          })}
        </ul>
      )}
    </Card>
  )
}
