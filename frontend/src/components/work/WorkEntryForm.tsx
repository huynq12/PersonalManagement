import { zodResolver } from '@hookform/resolvers/zod'
import type { UseMutationResult } from '@tanstack/react-query'
import { useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { IsoDate } from '../../lib/dates'
import { parseOptionalHours } from '../../lib/settingsSchemas'
import type { WorkEntry } from '../../lib/workEntries'
import { workEntryFormSchema, type WorkEntryForm } from '../../lib/workEntrySchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'
import TextField from '../ui/TextField'

type PutMutation = UseMutationResult<
  WorkEntry,
  Error,
  { date: IsoDate; hours: number; note: string | null }
>

interface WorkEntryFormCardProps {
  today: IsoDate
  /** The entry being edited, or `null` to log a new day. */
  editing: WorkEntry | null
  save: PutMutation
  onCancelEdit: () => void
  onSaved: () => void
}

/** Logs the hours of a day, or edits an entry loaded from the list. */
export default function WorkEntryFormCard({
  today,
  editing,
  save,
  onCancelEdit,
  onSaved,
}: WorkEntryFormCardProps) {
  const { t } = useTranslation()
  const schema = useMemo(() => workEntryFormSchema(today), [today])
  const emptyValues: WorkEntryForm = { date: today, hours: '', note: '' }
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<WorkEntryForm>({
    resolver: zodResolver(schema),
    defaultValues: editing
      ? { date: editing.date, hours: String(editing.hours), note: editing.note ?? '' }
      : emptyValues,
  })

  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined

  const onSubmit = handleSubmit((values) => {
    // The schema already accepted the hours, so the conversion cannot be undefined or null.
    const hours = parseOptionalHours(values.hours)
    if (hours === null || hours === undefined) {
      return
    }
    const note = values.note.trim()
    save.mutate(
      { date: values.date, hours, note: note === '' ? null : note },
      {
        onSuccess: () => {
          reset(emptyValues)
          onSaved()
        },
      },
    )
  })

  return (
    <Card title={editing ? t('workLog.form.editTitle') : t('workLog.form.title')}>
      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <div className="grid gap-4 sm:grid-cols-2">
          <TextField
            label={t('workLog.form.date')}
            type="date"
            max={today}
            error={fieldError(errors.date?.message)}
            {...register('date')}
          />
          <TextField
            label={t('workLog.form.hours')}
            hint={t('workLog.form.hoursHint')}
            inputMode="decimal"
            autoComplete="off"
            autoFocus={editing !== null}
            error={fieldError(errors.hours?.message)}
            {...register('hours')}
          />
        </div>
        <TextField
          label={t('workLog.form.note')}
          autoComplete="off"
          error={fieldError(errors.note?.message)}
          {...register('note')}
        />

        {save.isError ? <Alert variant="error">{t(errorMessageKey(save.error))}</Alert> : null}

        <div className="flex flex-wrap gap-2">
          <Button type="submit" disabled={save.isPending}>
            {t('workLog.form.save')}
          </Button>
          {editing ? (
            <Button variant="ghost" onClick={onCancelEdit}>
              {t('workLog.form.cancelEdit')}
            </Button>
          ) : null}
        </div>
      </form>
    </Card>
  )
}
