import { useTranslation } from 'react-i18next'
import { useDeleteWorkEntry } from '../../hooks/useWorkEntries'
import { errorMessageKey } from '../../i18n/errorMessage'
import { formatShortDate } from '../../lib/dates'
import type { WorkEntry } from '../../lib/workEntries'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Dialog from '../ui/Dialog'

interface DeleteWorkEntryDialogProps {
  entry: WorkEntry
  onClose: () => void
}

/** Asks before deleting a day's entry; deleting cannot be undone. */
export default function DeleteWorkEntryDialog({ entry, onClose }: DeleteWorkEntryDialogProps) {
  const { t, i18n } = useTranslation()
  const remove = useDeleteWorkEntry()

  return (
    <Dialog title={t('workLog.deleteDialog.title')} onClose={onClose}>
      {remove.isError ? <Alert variant="error">{t(errorMessageKey(remove.error))}</Alert> : null}

      <p className="text-sm text-muted-foreground">
        {t('workLog.deleteDialog.description', {
          date: formatShortDate(entry.date, i18n.language),
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
          onClick={() => remove.mutate(entry.date, { onSuccess: onClose })}
        >
          {t('workLog.deleteDialog.confirm')}
        </Button>
      </div>
    </Dialog>
  )
}
