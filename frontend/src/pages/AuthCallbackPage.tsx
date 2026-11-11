import { useTranslation } from 'react-i18next'
import { Navigate } from 'react-router'
import FullPageMessage from '../components/FullPageMessage'
import { useSession } from '../hooks/useSession'

/**
 * Where the backend sends the browser after a successful Google sign-in.
 *
 * By then the refresh cookie is set, and no token ever appears in the URL. Restoring the
 * session, which the app does on every load, exchanges that cookie for an access token.
 */
export default function AuthCallbackPage() {
  const { t } = useTranslation()
  const session = useSession()

  if (session.isPending) {
    return (
      <FullPageMessage>
        <p>{t('auth.signingIn')}</p>
      </FullPageMessage>
    )
  }

  return session.data ? (
    <Navigate to="/" replace />
  ) : (
    <Navigate to="/login?error=google_sign_in_failed" replace />
  )
}
