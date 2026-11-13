import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import CategorySection from '../components/finance/CategorySection'
import SuggestedCategories from '../components/finance/SuggestedCategories'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import Checkbox from '../components/ui/Checkbox'
import { useFinanceCategories } from '../hooks/useFinance'
import { errorMessageKey } from '../i18n/errorMessage'
import { KINDS } from '../lib/finance'

/** The Finance categories screen: expense and income categories the user can edit. */
export default function FinanceCategoriesPage() {
  const { t } = useTranslation()
  const categories = useFinanceCategories()
  const [showArchived, setShowArchived] = useState(false)

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
          {t('financeCategories.title')}
        </h1>
        <p className="text-sm text-muted-foreground">{t('financeCategories.description')}</p>
      </header>

      {categories.isPending ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : categories.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(categories.error))}</Alert>
          <Button variant="secondary" onClick={() => void categories.refetch()}>
            {t('common.retry')}
          </Button>
        </div>
      ) : (
        <>
          {categories.data.length === 0 ? <SuggestedCategories /> : null}

          <label className="flex items-center gap-2 text-sm">
            <Checkbox
              checked={showArchived}
              onChange={(event) => setShowArchived(event.target.checked)}
            />
            {t('financeCategories.showArchived')}
          </label>

          {KINDS.map((kind) => (
            <CategorySection
              key={kind}
              kind={kind}
              categories={categories.data.filter((category) => category.kind === kind)}
              showArchived={showArchived}
            />
          ))}
        </>
      )}
    </main>
  )
}
