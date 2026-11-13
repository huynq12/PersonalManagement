import { useTranslation } from 'react-i18next'
import { useDeleteTransaction } from '../../hooks/useFinance'
import { errorMessageKey } from '../../i18n/errorMessage'
import { formatShortDate } from '../../lib/dates'
import type { Transaction } from '../../lib/finance'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Dialog from '../ui/Dialog'

interface DeleteTransactionDialogProps {
  transaction: Transaction
  onClose: () => void
}

/** Asks before deleting a transaction; deleting cannot be undone. */
export default function DeleteTransactionDialog({
  transaction,
  onClose,
}: DeleteTransactionDialogProps) {
  const { t, i18n } = useTranslation()
  const remove = useDeleteTransaction()

  return (
    <Dialog title={t('finances.deleteDialog.title')} onClose={onClose}>
      {remove.isError ? <Alert variant="error">{t(errorMessageKey(remove.error))}</Alert> : null}

      <p className="text-sm text-muted-foreground">
        {t('finances.deleteDialog.description', {
          category: transaction.category.name,
          date: formatShortDate(transaction.date, i18n.language),
        })}
      </p>

      <div className="flex justify-end gap-2">
        {/* Cancel comes first and takes focus, so Enter does not delete by accident. */}
        <Button variant="ghost" autoFocus onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button
          className="bg-destructive text-background hover:bg-destructive/90"
          disabled={remove.isPending}
          onClick={() => remove.mutate(transaction.id, { onSuccess: onClose })}
        >
          {t('finances.deleteDialog.confirm')}
        </Button>
      </div>
    </Dialog>
  )
}
