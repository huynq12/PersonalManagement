import { useTranslation } from 'react-i18next'
import { errorMessageKey } from '../../i18n/errorMessage'
import Alert from '../ui/Alert'

interface SaveStatusProps {
  isError: boolean
  isSuccess: boolean
  error: unknown
}

/** The outcome of saving a settings section: an error message, a confirmation, or nothing. */
export default function SaveStatus({ isError, isSuccess, error }: SaveStatusProps) {
  const { t } = useTranslation()

  if (isError) {
    return <Alert variant="error">{t(errorMessageKey(error))}</Alert>
  }
  return isSuccess ? <Alert variant="success">{t('settings.saved')}</Alert> : null
}
