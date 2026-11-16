import { useTranslation } from 'react-i18next'
import BudgetSection from '../components/settings/BudgetSection'
import CurrencySection from '../components/settings/CurrencySection'
import GoogleSection from '../components/settings/GoogleSection'
import LanguageSection from '../components/settings/LanguageSection'
import PomodoroSection from '../components/settings/PomodoroSection'
import ProfileSection from '../components/settings/ProfileSection'
import SignOutSection from '../components/settings/SignOutSection'
import TimezoneSection from '../components/settings/TimezoneSection'
import Alert from '../components/ui/Alert'
import Button from '../components/ui/Button'
import { useSession } from '../hooks/useSession'
import { useSettings } from '../hooks/useSettings'
import { errorMessageKey } from '../i18n/errorMessage'

/** The Settings screen: profile, time zone, language, currency, budget, Pomodoro, Google linking, and sign out.
 * The work settings live at the bottom of the Work log. */
export default function SettingsPage() {
  const { t } = useTranslation()
  const settings = useSettings()
  const session = useSession()
  const user = session.data

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">
          {t('settings.title')}
        </h1>
        <p className="text-sm text-muted-foreground">{t('settings.description')}</p>
      </header>

      {settings.isPending || !user ? (
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
          <ProfileSection key={`profile-${user.display_name}`} user={user} />
          <TimezoneSection key={`timezone-${user.timezone}`} timezone={user.timezone} />
          <LanguageSection />
          <CurrencySection currency={user.currency} />
          <BudgetSection key={`budget-${user.currency}`} settings={settings.data} currency={user.currency} />
          <PomodoroSection settings={settings.data} />
          <GoogleSection />
          <SignOutSection />
        </>
      )}
    </main>
  )
}
