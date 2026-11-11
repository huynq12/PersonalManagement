import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { errorMessageKey } from '../i18n/errorMessage'
import { getHealth } from '../lib/api'

const STATUS_OK_CLASSES = 'text-success'
const STATUS_ERROR_CLASSES = 'text-destructive'

/** The Today screen. Its cards arrive with the checklist, meal plan, and Pomodoro features. */
export default function HomePage() {
  const { t, i18n } = useTranslation()
  const health = useQuery({ queryKey: ['health'], queryFn: getHealth })
  // Read the clock once per visit; the date picker (SPEC.md, Today) will replace this.
  const [now] = useState(() => new Date())

  const today = new Intl.DateTimeFormat(i18n.resolvedLanguage, { dateStyle: 'full' }).format(now)

  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <header className="flex flex-col gap-1">
        <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t('home.title')}</h1>
        <p className="text-sm text-muted-foreground">{today}</p>
      </header>
      <p className="text-sm">{t('home.intro')}</p>

      <section className="flex flex-col gap-2 rounded-xl border border-border p-5">
        <h2 className="text-base font-semibold">{t('home.backendStatus')}</h2>
        {health.isPending ? (
          <p className="text-sm text-muted-foreground">{t('home.checking')}</p>
        ) : health.isError ? (
          <p className={`text-sm ${STATUS_ERROR_CLASSES}`} role="alert">
            {t('home.connectionFailed', { reason: t(errorMessageKey(health.error)) })}
          </p>
        ) : health.data.status === 'ok' && health.data.database === 'ok' ? (
          <p className={`text-sm ${STATUS_OK_CLASSES}`}>{t('home.healthy')}</p>
        ) : (
          <p className={`text-sm ${STATUS_ERROR_CLASSES}`}>{t('home.degraded')}</p>
        )}
      </section>
    </main>
  )
}
