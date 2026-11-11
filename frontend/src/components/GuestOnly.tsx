import { useTranslation } from 'react-i18next'
import { Navigate, Outlet } from 'react-router'
import { useSession } from '../hooks/useSession'
import FullPageMessage from './FullPageMessage'

/** Renders the nested routes (login, register) only while nobody is signed in. */
export default function GuestOnly() {
  const { t } = useTranslation()
  const session = useSession()

  if (session.isPending) {
    return (
      <FullPageMessage>
        <p>{t('auth.checkingSession')}</p>
      </FullPageMessage>
    )
  }

  // If the session check failed, still show the form: signing in may work.
  return session.data ? <Navigate to="/" replace /> : <Outlet />
}
