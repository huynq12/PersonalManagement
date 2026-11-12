import { useTranslation } from 'react-i18next'
import { useDeleteHabit } from '../../hooks/useHabits'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { Habit } from '../../lib/checklist'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Dialog from '../ui/Dialog'

interface DeleteHabitDialogProps {
  habit: Habit
  onClose: () => void
}

/**
 * Asks before deleting a habit. Deleting removes the whole history and cannot be undone, so
 * the dialog also points to archiving, which keeps it.
 */
export default function DeleteHabitDialog({ habit, onClose }: DeleteHabitDialogProps) {
  const { t } = useTranslation()
  const remove = useDeleteHabit()

  return (
    <Dialog title={t('habits.deleteDialog.title')} onClose={onClose}>
      {remove.isError ? <Alert variant="error">{t(errorMessageKey(remove.error))}</Alert> : null}

      <p className="text-sm text-muted-foreground">
        {t('habits.deleteDialog.description', { name: habit.name })}
      </p>

      <div className="flex justify-end gap-2">
        {/* Cancel comes first and takes focus, so Enter does not delete by accident. */}
        <Button variant="ghost" autoFocus onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button
          className="bg-destructive text-background hover:bg-destructive/90"
          disabled={remove.isPending}
          onClick={() => remove.mutate(habit.id, { onSuccess: onClose })}
        >
          {t('habits.deleteDialog.confirm')}
        </Button>
      </div>
    </Dialog>
  )
}
