import { zodResolver } from '@hookform/resolvers/zod'
import { Archive, ArchiveRestore, Pencil } from 'lucide-react'
import { useState } from 'react'
import { Controller, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useCreateCategory, useUpdateCategory } from '../../hooks/useFinance'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import { CATEGORY_COLORS, type Category, type Kind } from '../../lib/finance'
import { categoryFormSchema, type CategoryForm } from '../../lib/financeSchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'
import IconButton from '../ui/IconButton'
import TextField from '../ui/TextField'
import CategoryColorPicker from './CategoryColorPicker'
import CategoryFormDialog from './CategoryFormDialog'

interface CategorySectionProps {
  kind: Kind
  /** Every category of this kind, archived ones included. */
  categories: Category[]
  showArchived: boolean
}

/** One kind's categories: the list with edit and archive buttons, and a form to add one. */
export default function CategorySection({ kind, categories, showArchived }: CategorySectionProps) {
  const { t } = useTranslation()
  const create = useCreateCategory()
  const update = useUpdateCategory()
  const [editing, setEditing] = useState<Category | null>(null)
  const {
    register,
    control,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CategoryForm>({
    resolver: zodResolver(categoryFormSchema),
    defaultValues: { name: '', color: CATEGORY_COLORS[5] },
  })

  const visible = categories.filter((category) => showArchived || category.archived_at === null)
  const title = t(
    kind === 'expense' ? 'financeCategories.expenseTitle' : 'financeCategories.incomeTitle',
  )

  const onSubmit = handleSubmit((values) =>
    create.mutate(
      { kind, name: values.name.trim(), color: values.color },
      { onSuccess: () => reset({ name: '', color: values.color }) },
    ),
  )

  return (
    <Card title={title}>
      {visible.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t('financeCategories.empty')}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-border">
          {visible.map((category) => {
            const archived = category.archived_at !== null
            return (
              <li key={category.id} className="flex items-center gap-3 py-2">
                <span
                  aria-hidden="true"
                  className="size-4 shrink-0 rounded-full"
                  style={{ backgroundColor: category.color }}
                />
                <span className={`flex-1 text-sm ${archived ? 'text-muted-foreground' : ''}`}>
                  {category.name}
                  {archived ? (
                    <span className="ml-2 text-xs">({t('financeCategories.archivedBadge')})</span>
                  ) : null}
                </span>
                <IconButton
                  icon={Pencil}
                  label={t('financeCategories.edit', { name: category.name })}
                  onClick={() => setEditing(category)}
                />
                <IconButton
                  icon={archived ? ArchiveRestore : Archive}
                  label={t(archived ? 'financeCategories.restore' : 'financeCategories.archive', {
                    name: category.name,
                  })}
                  disabled={update.isPending}
                  onClick={() => update.mutate({ id: category.id, archived: !archived })}
                />
              </li>
            )
          })}
        </ul>
      )}

      {update.isError ? <Alert variant="error">{t(errorMessageKey(update.error))}</Alert> : null}

      <form
        className="flex flex-col gap-3 border-t border-border pt-4"
        noValidate
        aria-label={t('financeCategories.addForm', { kind: title })}
        onSubmit={(event) => void onSubmit(event)}
      >
        <TextField
          label={t('financeCategories.name')}
          autoComplete="off"
          error={errors.name?.message ? t(errors.name.message as ValidationMessageKey) : undefined}
          {...register('name')}
        />
        <Controller
          control={control}
          name="color"
          render={({ field }) => (
            <CategoryColorPicker value={field.value} onChange={field.onChange} />
          )}
        />

        {create.isError ? <Alert variant="error">{t(errorMessageKey(create.error))}</Alert> : null}

        <div>
          <Button type="submit" disabled={create.isPending}>
            {t('financeCategories.add')}
          </Button>
        </div>
      </form>

      {editing ? <CategoryFormDialog category={editing} onClose={() => setEditing(null)} /> : null}
    </Card>
  )
}
