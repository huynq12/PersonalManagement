import { zodResolver } from '@hookform/resolvers/zod'
import { Controller, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useUpdateCategory } from '../../hooks/useFinance'
import { errorMessageKey } from '../../i18n/errorMessage'
import type { ValidationMessageKey } from '../../lib/authSchemas'
import type { Category } from '../../lib/finance'
import { categoryFormSchema, type CategoryForm } from '../../lib/financeSchemas'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Dialog from '../ui/Dialog'
import TextField from '../ui/TextField'
import CategoryColorPicker from './CategoryColorPicker'

interface CategoryFormDialogProps {
  category: Category
  onClose: () => void
}

/** Renames and recolors a category. The kind never changes. */
export default function CategoryFormDialog({ category, onClose }: CategoryFormDialogProps) {
  const { t } = useTranslation()
  const update = useUpdateCategory()
  const {
    register,
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<CategoryForm>({
    resolver: zodResolver(categoryFormSchema),
    defaultValues: { name: category.name, color: category.color },
  })

  const onSubmit = handleSubmit((values) =>
    update.mutate(
      { id: category.id, name: values.name.trim(), color: values.color },
      { onSuccess: onClose },
    ),
  )

  return (
    <Dialog title={t('financeCategories.editTitle')} onClose={onClose}>
      <form className="flex flex-col gap-4" noValidate onSubmit={(event) => void onSubmit(event)}>
        <TextField
          label={t('financeCategories.name')}
          autoComplete="off"
          autoFocus
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

        {update.isError ? <Alert variant="error">{t(errorMessageKey(update.error))}</Alert> : null}

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button type="submit" disabled={update.isPending}>
            {t('common.save')}
          </Button>
        </div>
      </form>
    </Dialog>
  )
}
