import { zodResolver } from '@hookform/resolvers/zod'
import { useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useUpdateSettings } from '../../hooks/useSettings'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import { toInputValue } from '../../lib/money'
import type { Settings } from '../../lib/settings'
import {
  parseOptionalAmount,
  parseOptionalHours,
  workFormSchema,
  type WorkForm,
} from '../../lib/settingsSchemas'
import Button from '../ui/Button'
import Card from '../ui/Card'
import TextField from '../ui/TextField'
import SaveStatus from './SaveStatus'

interface WorkSectionProps {
  settings: Settings
  currency: string
}

/** Hourly wage and usual hours per day, which the work log copies into each entry. */
export default function WorkSection({ settings, currency }: WorkSectionProps) {
  const { t } = useTranslation()
  const save = useUpdateSettings()
  const schema = useMemo(() => workFormSchema(currency), [currency])
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<WorkForm>({
    resolver: zodResolver(schema),
    defaultValues: {
      hourly_wage:
        settings.hourly_wage === null ? '' : toInputValue(settings.hourly_wage, currency),
      base_work_hours: settings.base_work_hours === null ? '' : String(settings.base_work_hours),
    },
  })

  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined

  const onSubmit = handleSubmit((values) => {
    // The schema already accepted both texts, so neither conversion can be undefined here.
    save.mutate({
      hourly_wage: parseOptionalAmount(values.hourly_wage, currency) ?? null,
      base_work_hours: parseOptionalHours(values.base_work_hours) ?? null,
    })
  })

  return (
    <Card title={t('settings.work.title')}>
      <p className="text-sm text-muted-foreground">{t('settings.work.description')}</p>
      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <div className="grid gap-4 sm:grid-cols-2">
          <TextField
            label={t('settings.work.hourlyWage', { currency })}
            inputMode="decimal"
            autoComplete="off"
            error={fieldError(errors.hourly_wage?.message)}
            {...register('hourly_wage')}
          />
          <TextField
            label={t('settings.work.baseHours')}
            hint={t('settings.work.baseHoursHint')}
            inputMode="decimal"
            autoComplete="off"
            error={fieldError(errors.base_work_hours?.message)}
            {...register('base_work_hours')}
          />
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
