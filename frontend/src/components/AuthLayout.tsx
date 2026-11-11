import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import LanguageSwitcher from './LanguageSwitcher'

interface AuthLayoutProps {
  title: string
  children: ReactNode
}

/** Centered card used by the sign-in and registration screens. */
export default function AuthLayout({ title, children }: AuthLayoutProps) {
  const { t } = useTranslation()

  return (
    <main className="grid min-h-dvh place-items-center p-4">
      <div className="w-full max-w-sm">
        <header className="mb-4 flex flex-wrap items-center justify-between gap-2">
          <p className="font-semibold">{t('app.title')}</p>
          <LanguageSwitcher />
        </header>
        <section className="flex flex-col gap-4 rounded-xl border border-slate-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900">
          <h1 className="text-xl font-semibold">{title}</h1>
          {children}
        </section>
      </div>
    </main>
  )
}
