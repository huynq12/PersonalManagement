import { useTranslation } from 'react-i18next'
import { useLogout } from '../../hooks/useSession'
import Button from '../ui/Button'
import Card from '../ui/Card'

/** Ends the session. Signing out clears the cached user, which sends the app to the login page. */
export default function SignOutSection() {
  const { t } = useTranslation()
  const logout = useLogout()

  return (
    <Card title={t('settings.signOut.title')}>
      <p className="text-sm text-muted-foreground">{t('settings.signOut.description')}</p>
      <div>
        <Button variant="secondary" disabled={logout.isPending} onClick={() => logout.mutate()}>
          {t('auth.logout')}
        </Button>
      </div>
    </Card>
  )
}
