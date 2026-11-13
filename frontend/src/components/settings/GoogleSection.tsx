import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import { useLinkGoogle } from '../../hooks/useSession'
import { errorCodeMessageKey, errorMessageKey } from '../../i18n/errorMessage'
import GoogleIcon from '../GoogleIcon'
import Alert from '../ui/Alert'
import Button from '../ui/Button'
import Card from '../ui/Card'

/**
 * Links a Google account to the signed-in account. Google sends the browser back to
 * `/settings` with `?google=linked` on success or `?error=<code>` on failure.
 */
export default function GoogleSection() {
  const { t } = useTranslation()
  const [searchParams] = useSearchParams()
  const link = useLinkGoogle()

  const redirectErrorCode = searchParams.get('error')
  const errorKey = link.isError
    ? errorMessageKey(link.error)
    : redirectErrorCode
      ? errorCodeMessageKey(redirectErrorCode)
      : null

  return (
    <Card title={t('settings.google.title')}>
      <p className="text-sm text-muted-foreground">{t('settings.google.description')}</p>

      {errorKey ? (
        <Alert variant="error">{t(errorKey)}</Alert>
      ) : searchParams.get('google') === 'linked' ? (
        <Alert variant="success">{t('settings.google.linked')}</Alert>
      ) : null}

      <div>
        <Button variant="secondary" disabled={link.isPending} onClick={() => link.mutate()}>
          <GoogleIcon />
          {t('settings.google.link')}
        </Button>
      </div>
    </Card>
  )
}
