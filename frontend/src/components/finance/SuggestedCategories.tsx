import { useTranslation } from 'react-i18next'
import { useCreateSuggestedCategories } from '../../hooks/useFinance'
import { errorMessageKey } from '../../i18n/errorMessage'
import { CATEGORY_COLORS, SUGGESTED_CATEGORIES } from '../../lib/finance'
import Alert from '../ui/Alert'
import Button from '../ui/Button'

/** Offers to create the starter set of categories, named in the active language. */
export default function SuggestedCategories() {
  const { t } = useTranslation()
  const create = useCreateSuggestedCategories()

  const onCreate = () => {
    const color = (index: number) => CATEGORY_COLORS[index % CATEGORY_COLORS.length]
    // Written out per kind so the translation keys stay checked against the locale files.
    const categories = [
      ...SUGGESTED_CATEGORIES.expense.map((id, index) => ({
        kind: 'expense' as const,
        name: t(`finances.suggestedCategories.expense.${id}`),
        color: color(index),
      })),
      ...SUGGESTED_CATEGORIES.income.map((id, index) => ({
        kind: 'income' as const,
        name: t(`finances.suggestedCategories.income.${id}`),
        color: color(index),
      })),
    ]
    create.mutate(categories)
  }

  return (
    <div className="flex flex-col items-start gap-3 rounded-xl border border-dashed border-border p-5">
      <h2 className="text-base font-semibold">{t('financeCategories.suggested.title')}</h2>
      <p className="text-sm text-muted-foreground">
        {t('financeCategories.suggested.description')}
      </p>
      {create.isError ? <Alert variant="error">{t(errorMessageKey(create.error))}</Alert> : null}
      <Button disabled={create.isPending} onClick={onCreate}>
        {t('financeCategories.suggested.button')}
      </Button>
    </div>
  )
}
