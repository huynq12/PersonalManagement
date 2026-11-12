import { zodResolver } from '@hookform/resolvers/zod'
import { useForm, useWatch } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useCreateHabit, useUpdateHabit } from '../../hooks/useHabits'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { Habit } from '../../lib/checklist'
import { EVERY_WEEKDAY, habitFormSchema, type HabitForm } from '../../lib/checklistSchemas'
import { weekdayNames } from '../../lib/dates'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Dialog from '../ui/Dialog'
import TextField from '../ui/TextField'

interface HabitFormDialogProps {
  /** The habit to edit; leave out to add a new one. */
  habit?: Habit
  onClose: () => void
}

/** Dialog with the habit form: a name and the weekdays the habit repeats on. */
export default function HabitFormDialog({ habit, onClose }: HabitFormDialogProps) {
  const { t, i18n } = useTranslation()
  const create = useCreateHabit()
  const update = useUpdateHabit()
  const save = habit ? update : create

  const {
    register,
    handleSubmit,
    setValue,
    control,
    formState: { errors, isSubmitted },
  } = useForm<HabitForm>({
    resolver: zodResolver(habitFormSchema),
    defaultValues: {
      name: habit?.name ?? '',
      weekdays: habit?.weekdays ?? EVERY_WEEKDAY,
    },
  })
  const selected = useWatch({ control, name: 'weekdays' })

  const toggleWeekday = (weekday: number) => {
    const next = selected.includes(weekday)
      ? selected.filter((value) => value !== weekday)
      : [...selected, weekday].sort((a, b) => a - b)
    // Before the first submit, do not nag about a rule the user has not had a chance to meet.
    setValue('weekdays', next, { shouldValidate: isSubmitted })
  }

  const onSubmit = handleSubmit((values) => {
    if (habit) {
      update.mutate({ id: habit.id, changes: values }, { onSuccess: onClose })
    } else {
      create.mutate(values, { onSuccess: onClose })
    }
  })

  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined
  const weekdaysError = fieldError(errors.weekdays?.message)

  return (
    <Dialog title={t(habit ? 'habits.form.editTitle' : 'habits.form.addTitle')} onClose={onClose}>
      {save.isError ? <Alert variant="error">{t(errorMessageKey(save.error))}</Alert> : null}

      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <TextField
          label={t('habits.form.name')}
          autoComplete="off"
          // The dialog was opened to fill in this form, so starting in its first field is expected.
          autoFocus
          error={fieldError(errors.name?.message)}
          {...register('name')}
        />

        <div className="flex flex-col gap-2">
          <div
            role="group"
            aria-label={t('habits.form.weekdays')}
            className="flex flex-col gap-2"
          >
            <span className="text-sm font-medium" aria-hidden="true">
              {t('habits.form.weekdays')}
            </span>
            <div className="flex flex-wrap gap-1.5">
              {weekdayNames(i18n.resolvedLanguage).map((name, index) => {
                const weekday = index + 1
                const isSelected = selected.includes(weekday)
                return (
                  <button
                    key={weekday}
                    type="button"
                    aria-pressed={isSelected}
                    onClick={() => toggleWeekday(weekday)}
                    className={`h-10 min-w-11 rounded-md border px-2 text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring ${
                      isSelected
                        ? 'border-primary bg-primary text-primary-foreground'
                        : 'border-input bg-background text-muted-foreground hover:bg-accent'
                    }`}
                  >
                    {name}
                  </button>
                )
              })}
            </div>
          </div>
          {weekdaysError ? <p className="text-sm text-destructive">{weekdaysError}</p> : null}
        </div>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button type="submit" disabled={save.isPending}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Dialog>
  )
}
