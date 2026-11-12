import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useReplaceSettings, useSettings } from '../../hooks/useSettings'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { PomodoroSettings } from '../../lib/settings'
import {
  POMODORO_LIMITS,
  pomodoroFormSchema,
  type PomodoroForm as PomodoroFormValues,
} from '../../lib/settingsSchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'
import TextField from '../ui/TextField'

const FIELDS = [
  { name: 'focus_minutes', labelKey: 'settings.pomodoro.focusMinutes' },
  {
    name: 'short_break_minutes',
    labelKey: 'settings.pomodoro.shortBreakMinutes',
  },
  {
    name: 'long_break_minutes',
    labelKey: 'settings.pomodoro.longBreakMinutes',
  },
  { name: 'long_break_every', labelKey: 'settings.pomodoro.longBreakEvery' },
] as const

/** The Pomodoro card: loads the settings, then shows the form once they are known. */
export default function PomodoroSettingsCard() {
  const { t } = useTranslation()
  const settings = useSettings()

  return settings.isPending ? (
    <Card title={t('settings.pomodoro.title')}>
      <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
    </Card>
  ) : settings.isError ? (
    <Card title={t('settings.pomodoro.title')}>
      <Alert variant="error">{t(errorMessageKey(settings.error))}</Alert>
      <div>
        <Button variant="secondary" onClick={() => void settings.refetch()}>
          {t('common.retry')}
        </Button>
      </div>
    </Card>
  ) : (
    <PomodoroForm settings={settings.data} />
  )
}

function PomodoroForm({ settings }: { settings: PomodoroSettings }) {
  const { t } = useTranslation()
  const replace = useReplaceSettings()

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<PomodoroFormValues>({
    resolver: zodResolver(pomodoroFormSchema),
    defaultValues: settings,
  })

  const onSubmit = handleSubmit((values) => replace.mutate(values))

  return (
    <Card title={t('settings.pomodoro.title')}>
      {replace.isError ? <Alert variant="error">{t(errorMessageKey(replace.error))}</Alert> : null}
      {replace.isSuccess ? <Alert variant="success">{t('settings.pomodoro.saved')}</Alert> : null}

      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        {FIELDS.map(({ name, labelKey }) => {
          const message = errors[name]?.message
          return (
            <TextField
              key={name}
              type="number"
              inputMode="numeric"
              min={POMODORO_LIMITS[name].min}
              max={POMODORO_LIMITS[name].max}
              label={t(labelKey)}
              error={message ? t(message as ValidationMessageKey) : undefined}
              {...register(name, { valueAsNumber: true })}
            />
          )
        })}

        <div className="flex justify-end">
          <Button type="submit" disabled={replace.isPending}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Card>
  )
}
