import { useTranslation } from 'react-i18next'
import GoogleCard from '../components/settings/GoogleCard'
import PomodoroSettingsCard from '../components/settings/PomodoroForm'
import ProfileForm from '../components/settings/ProfileForm'
import Button from '../components/ui/Button'
import Card from '../components/ui/Card'
import { useLogout, useSession } from '../hooks/useSession'

/** The Settings screen: profile, language, time zone, Pomodoro settings, Google, sign out. */
export default function SettingsPage() {
  const { t } = useTranslation()
  const session = useSession()
  const logout = useLogout()

  return (
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('settings.title')}</h1>
        <p className="text-sm text-muted-foreground">{t('settings.description')}</p>
      </header>

      {session.data ? <ProfileForm user={session.data} /> : null}
      <PomodoroSettingsCard />
      <GoogleCard />

      <Card title={t('settings.account.title')}>
        <div>
          <Button variant="secondary" disabled={logout.isPending} onClick={() => logout.mutate()}>
            {t('settings.account.logout')}
          </Button>
        </div>
      </Card>
    </main>
  )
}
