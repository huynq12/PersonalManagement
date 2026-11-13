import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useUpdateSettings } from '../../hooks/useSettings'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { Settings } from '../../lib/settings'
import { pomodoroFormSchema, type PomodoroForm } from '../../lib/settingsSchemas'
import Button from '../ui/Button'
import Card from '../ui/Card'
import TextField from '../ui/TextField'
import SaveStatus from './SaveStatus'

interface PomodoroSectionProps {
  settings: Settings
}

/** Lengths of the Pomodoro focus sessions and breaks. */
export default function PomodoroSection({ settings }: PomodoroSectionProps) {
  const { t } = useTranslation()
  const save = useUpdateSettings()
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<PomodoroForm>({
    resolver: zodResolver(pomodoroFormSchema),
    defaultValues: {
      focus_minutes: settings.focus_minutes,
      short_break_minutes: settings.short_break_minutes,
      long_break_minutes: settings.long_break_minutes,
      long_break_every: settings.long_break_every,
    },
  })

  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined
  const numberField = (name: keyof PomodoroForm, label: string) => (
    <TextField
      label={label}
      type="number"
      inputMode="numeric"
      error={fieldError(errors[name]?.message)}
      {...register(name, { valueAsNumber: true })}
    />
  )

  return (
    <Card title={t('settings.pomodoro.title')}>
      <form
        className="flex flex-col gap-4"
        noValidate
        onSubmit={(event) => void handleSubmit((values) => save.mutate(values))(event)}
      >
        <div className="grid gap-4 sm:grid-cols-2">
          {numberField('focus_minutes', t('settings.pomodoro.focus'))}
          {numberField('short_break_minutes', t('settings.pomodoro.shortBreak'))}
          {numberField('long_break_minutes', t('settings.pomodoro.longBreak'))}
          {numberField('long_break_every', t('settings.pomodoro.longBreakEvery'))}
        </div>

        <SaveStatus isError={save.isError} isSuccess={save.isSuccess} error={save.error} />

        <div>
          <Button type="submit" disabled={save.isPending}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Card>
  )
}
