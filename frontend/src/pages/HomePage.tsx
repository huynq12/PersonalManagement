import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import LanguageSwitcher from '../components/LanguageSwitcher'
import { errorMessageKey } from '../i18n/errorMessage'
import { getHealth } from '../lib/api'

const STATUS_OK_CLASSES = 'text-green-700 dark:text-green-400'
const STATUS_ERROR_CLASSES = 'text-red-700 dark:text-red-400'

export default function HomePage() {
  const { t } = useTranslation()
  const health = useQuery({ queryKey: ['health'], queryFn: getHealth })

  return (
    <main className="mx-auto flex max-w-2xl flex-col gap-4 px-4 py-8">
      <header className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <h1 className="text-2xl font-semibold">{t('app.title')}</h1>
        <LanguageSwitcher />
      </header>
      <p>{t('home.intro')}</p>

      <section className="rounded-xl border border-slate-200 bg-white px-5 py-4 dark:border-slate-800 dark:bg-slate-900">
        <h2 className="mb-2 text-lg font-medium">{t('home.backendStatus')}</h2>
        {health.isPending ? (
          <p>{t('home.checking')}</p>
        ) : health.isError ? (
          <p className={STATUS_ERROR_CLASSES} role="alert">
            {t('home.connectionFailed', { reason: t(errorMessageKey(health.error)) })}
          </p>
        ) : health.data.status === 'ok' && health.data.database === 'ok' ? (
          <p className={STATUS_OK_CLASSES}>{t('home.healthy')}</p>
        ) : (
          <p className={STATUS_ERROR_CLASSES}>{t('home.degraded')}</p>
        )}
      </section>
    </main>
  )
}
