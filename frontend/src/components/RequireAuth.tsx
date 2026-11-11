import { useTranslation } from 'react-i18next'
import { Navigate, Outlet } from 'react-router'
import { useSession } from '../hooks/useSession'
import { errorMessageKey } from '../i18n/errorMessage'
import FullPageMessage from './FullPageMessage'
import Alert from './ui/Alert'
import Button from './ui/Button'

/** Renders the nested routes only for a signed-in user; everyone else goes to the login page. */
export default function RequireAuth() {
  const { t } = useTranslation()
  const session = useSession()

  if (session.isPending) {
    return (
      <FullPageMessage>
        <p>{t('auth.checkingSession')}</p>
      </FullPageMessage>
    )
  }

  // Not knowing is different from being signed out: keep the user here and let them retry.
  if (session.isError) {
    return (
      <FullPageMessage>
        <Alert variant="error">{t(errorMessageKey(session.error))}</Alert>
        <Button variant="secondary" onClick={() => void session.refetch()}>
          {t('auth.retry')}
        </Button>
      </FullPageMessage>
    )
  }

  return session.data ? <Outlet /> : <Navigate to="/login" replace />
}
