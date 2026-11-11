import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import LanguageSwitcher from '../components/LanguageSwitcher'
import { errorMessageKey } from '../i18n/errorMessage'
import { getHealth } from '../lib/api'

export default function HomePage() {
  const { t } = useTranslation()
  const health = useQuery({ queryKey: ['health'], queryFn: getHealth })

  return (
    <main className="page">
      <header className="page-header">
        <h1>{t('app.title')}</h1>
        <LanguageSwitcher />
      </header>
      <p>{t('home.intro')}</p>

      <section className="card">
        <h2>{t('home.backendStatus')}</h2>
        {health.isPending ? (
          <p>{t('home.checking')}</p>
        ) : health.isError ? (
          <p className="status-error" role="alert">
            {t('home.connectionFailed', { reason: t(errorMessageKey(health.error)) })}
          </p>
        ) : health.data.status === 'ok' && health.data.database === 'ok' ? (
          <p className="status-ok">{t('home.healthy')}</p>
        ) : (
          <p className="status-error">{t('home.degraded')}</p>
        )}
      </section>
    </main>
  )
}
