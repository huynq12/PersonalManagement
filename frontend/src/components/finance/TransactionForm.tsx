import { zodResolver } from '@hookform/resolvers/zod'
import { useId, useMemo } from 'react'
import { useForm, useWatch } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { useCreateTransaction, useUpdateTransaction } from '../../hooks/useFinance'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { IsoDate } from '../../lib/dates'
import { KINDS, type Category, type Kind, type Transaction } from '../../lib/finance'
import { transactionFormSchema, type TransactionForm } from '../../lib/financeSchemas'
import { toInputValue } from '../../lib/money'
import { parseOptionalAmount } from '../../lib/settingsSchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'
import Select from '../ui/Select'
import TextField from '../ui/TextField'

interface TransactionFormCardProps {
  today: IsoDate
  currency: string
  /** Every category of the user, archived ones included. */
  categories: Category[]
  /** The transaction being edited, or `null` to record a new one. */
  editing: Transaction | null
  onCancelEdit: () => void
  onSaved: () => void
}

/** Records an income or an expense, or edits a transaction loaded from the list. */
export default function TransactionFormCard({
  today,
  currency,
  categories,
  editing,
  onCancelEdit,
  onSaved,
}: TransactionFormCardProps) {
  const { t } = useTranslation()
  const selectId = useId()
  const create = useCreateTransaction()
  const update = useUpdateTransaction()
  const save = editing ? update : create
  const schema = useMemo(() => transactionFormSchema(currency), [currency])
  const emptyValues: TransactionForm = {
    kind: 'expense',
    amount: '',
    date: today,
    categoryId: '',
    note: '',
  }
  const {
    register,
    handleSubmit,
    reset,
    setValue,
    control,
    formState: { errors },
  } = useForm<TransactionForm>({
    resolver: zodResolver(schema),
    defaultValues: editing
      ? {
          kind: editing.kind,
          amount: toInputValue(editing.amount, currency),
          date: editing.date,
          categoryId: editing.category.id,
          note: editing.note ?? '',
        }
      : emptyValues,
  })

  const kind = useWatch({ control, name: 'kind' })
  // Only active categories of the chosen kind can be picked. When editing, the transaction's
  // own category stays selectable even if it has been archived since.
  const options = categories.filter(
    (category) =>
      category.kind === kind &&
      (category.archived_at === null || category.id === editing?.category.id),
  )

  const fieldError = (message: string | undefined) =>
    message ? t(message as ValidationMessageKey) : undefined

  const onSubmit = handleSubmit((values) => {
    // The schema already accepted the amount, so the conversion yields a number.
    const amount = parseOptionalAmount(values.amount, currency)
    if (typeof amount !== 'number') {
      return
    }
    const note = values.note.trim()
    const input = {
      category_id: values.categoryId,
      amount,
      date: values.date,
      note: note === '' ? null : note,
    }
    const onSuccess = () => {
      reset({ ...emptyValues, kind: values.kind })
      onSaved()
    }
    if (editing) {
      update.mutate({ id: editing.id, input }, { onSuccess })
    } else {
      create.mutate(input, { onSuccess })
    }
  })

  const kindLabel = (value: Kind) =>
    t(value === 'expense' ? 'finances.form.expense' : 'finances.form.income')

  return (
    <Card title={editing ? t('finances.form.editTitle') : t('finances.form.title')}>
      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <fieldset className="flex gap-4" disabled={editing !== null}>
          <legend className="mb-2 text-sm font-medium">{t('finances.form.kind')}</legend>
          {KINDS.map((value) => (
            <label key={value} className="flex items-center gap-2 text-sm">
              <input
                type="radio"
                value={value}
                className="size-4 accent-primary"
                {...register('kind', {
                  // A category belongs to one kind, so the choice starts over.
                  onChange: () => setValue('categoryId', ''),
                })}
              />
              {kindLabel(value)}
            </label>
          ))}
        </fieldset>

        <div className="grid gap-4 sm:grid-cols-2">
          <TextField
            label={t('finances.form.amount', { currency })}
            inputMode="decimal"
            autoComplete="off"
            error={fieldError(errors.amount?.message)}
            {...register('amount')}
          />
          <TextField
            label={t('finances.form.date')}
            type="date"
            error={fieldError(errors.date?.message)}
            {...register('date')}
          />
        </div>

        <div className="flex flex-col gap-2">
          <label htmlFor={selectId} className="text-sm font-medium">
            {t('finances.form.category')}
          </label>
          <Select
            id={selectId}
            className="h-10"
            aria-invalid={errors.categoryId ? true : undefined}
            {...register('categoryId')}
          >
            <option value="">{t('finances.form.categoryPlaceholder')}</option>
            {options.map((category) => (
              <option key={category.id} value={category.id}>
                {category.name}
              </option>
            ))}
          </Select>
          {errors.categoryId ? (
            <p className="text-sm text-destructive">{fieldError(errors.categoryId.message)}</p>
          ) : options.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t('finances.form.noCategories')}{' '}
              <Link to="/finance-categories" className="font-medium underline underline-offset-4">
                {t('finances.form.manageCategories')}
              </Link>
            </p>
          ) : null}
        </div>

        <TextField
          label={t('finances.form.note')}
          autoComplete="off"
          error={fieldError(errors.note?.message)}
          {...register('note')}
        />

        {save.isError ? <Alert variant="error">{t(errorMessageKey(save.error))}</Alert> : null}

        <div className="flex flex-wrap gap-2">
          <Button type="submit" disabled={save.isPending}>
            {t('finances.form.save')}
          </Button>
          {editing ? (
            <Button variant="ghost" onClick={onCancelEdit}>
              {t('finances.form.cancelEdit')}
            </Button>
          ) : null}
        </div>
      </form>
    </Card>
  )
}
