import { zodResolver } from '@hookform/resolvers/zod'
import { useMemo } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useUpdateSettings } from '../../hooks/useSettings'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import { toInputValue } from '../../lib/money'
import type { Settings } from '../../lib/settings'
import { budgetFormSchema, parseOptionalAmount, type BudgetForm } from '../../lib/settingsSchemas'
import Button from '../ui/Button'
import Card from '../ui/Card'
import TextField from '../ui/TextField'
import SaveStatus from './SaveStatus'

interface BudgetSectionProps {
  settings: Settings
  currency: string
}

/** The monthly expense budget. Leaving the field empty removes the budget. */
export default function BudgetSection({ settings, currency }: BudgetSectionProps) {
  const { t } = useTranslation()
  const save = useUpdateSettings()
  const schema = useMemo(() => budgetFormSchema(currency), [currency])
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<BudgetForm>({
    resolver: zodResolver(schema),
    defaultValues: {
      monthly_budget:
        settings.monthly_budget === null ? '' : toInputValue(settings.monthly_budget, currency),
    },
  })

  const message = errors.monthly_budget?.message
  const onSubmit = handleSubmit((values) => {
    // The schema already accepted the text, so the conversion cannot be undefined here.
    save.mutate({ monthly_budget: parseOptionalAmount(values.monthly_budget, currency) ?? null })
  })

  return (
    <Card title={t('settings.budget.title')}>
      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <div className="max-w-sm">
          <TextField
            label={t('settings.budget.label', { currency })}
            hint={t('settings.budget.hint')}
            inputMode="decimal"
            autoComplete="off"
            error={message ? t(message as ValidationMessageKey) : undefined}
            {...register('monthly_budget')}
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
