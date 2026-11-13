import { useTranslation } from 'react-i18next'
import BudgetSection from '../components/settings/BudgetSection'
import CurrencySection from '../components/settings/CurrencySection'
import GoogleSection from '../components/settings/GoogleSection'
import PomodoroSection from '../components/settings/PomodoroSection'
import WorkSection from '../components/settings/WorkSection'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import { useSession } from '../hooks/useSession'
import { useSettings } from '../hooks/useSettings'
import { errorMessageKey } from '../i18n/errorMessage'

/** The Settings screen: currency, Pomodoro lengths, work settings, the monthly budget, and Google linking. */
export default function SettingsPage() {
  const { t } = useTranslation()
  const settings = useSettings()
  const session = useSession()
  const currency = session.data?.currency

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
          {t('settings.title')}
        </h1>
        <p className="text-sm text-muted-foreground">{t('settings.description')}</p>
      </header>

      {settings.isPending || !currency ? (
        <p className="text-sm text-muted-foreground">{t('common.loading')}</p>
      ) : settings.isError ? (
        <div className="flex flex-col items-start gap-3">
          <Alert variant="error">{t(errorMessageKey(settings.error))}</Alert>
          <Button variant="secondary" onClick={() => void settings.refetch()}>
            {t('common.retry')}
          </Button>
        </div>
      ) : (
        <>
          <CurrencySection currency={currency} />
          <WorkSection
            // A new currency changes how the saved wage is written, so start the form over.
            key={`work-${currency}`}
            settings={settings.data}
            currency={currency}
          />
          <BudgetSection key={`budget-${currency}`} settings={settings.data} currency={currency} />
          <PomodoroSection settings={settings.data} />
          <GoogleSection />
        </>
      )}
    </main>
  )
}
