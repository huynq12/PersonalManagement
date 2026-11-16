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
import { workEntryFormSchema, type WorkEntryForm as WorkEntryFormValues } from '../../lib/workEntrySchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import TextField from '../ui/TextField'

export type PutMutation = UseMutationResult<
  WorkEntry,
  Error,
  { date: IsoDate; hours: number; note: string | null }
>

interface WorkEntryFormProps {
  /** The day being logged. It is fixed by the clicked calendar cell, so it has no input. */
  date: IsoDate
  today: IsoDate
  /** The day's entry to edit, or `null` to log the day for the first time. */
  entry: WorkEntry | null
  save: PutMutation
  onSaved: () => void
  onCancel: () => void
  /** Offers a Delete button when set; only an existing entry can be deleted. */
  onDelete?: () => void
}

/** The hours and note of one day, with Save, Cancel and, for an existing entry, Delete. */
export default function WorkEntryForm({
  date,
  today,
  entry,
  save,
  onSaved,
  onCancel,
  onDelete,
}: WorkEntryFormProps) {
  const { t } = useTranslation()
  const schema = useMemo(() => workEntryFormSchema(today), [today])
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<WorkEntryFormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      date,
      hours: entry ? String(entry.hours) : '',
      note: entry?.note ?? '',
    },
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
    save.mutate({ date, hours, note: note === '' ? null : note }, { onSuccess: onSaved })
  })

  return (
    <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
      <TextField
        label={t('workLog.form.hours')}
        hint={t('workLog.form.hoursHint')}
        inputMode="decimal"
        autoComplete="off"
        autoFocus
        error={fieldError(errors.hours?.message)}
        {...register('hours')}
      />
      <TextField
        label={t('workLog.form.note')}
        autoComplete="off"
        error={fieldError(errors.note?.message)}
        {...register('note')}
      />

      {save.isError ? <Alert variant="error">{t(errorMessageKey(save.error))}</Alert> : null}

      <div className="flex flex-wrap items-center gap-2">
        {onDelete ? (
          <Button
            variant="ghost"
            className="mr-auto text-destructive"
            disabled={save.isPending}
            onClick={onDelete}
          >
            {t('workLog.dialog.delete')}
          </Button>
        ) : null}
        <Button variant="ghost" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button type="submit" disabled={save.isPending}>
          {t('common.save')}
        </Button>
      </div>
    </form>
  )
}
