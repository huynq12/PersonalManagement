import type { ReactNode } from 'react'
import BrandMark from './BrandMark'
import LanguageSwitcher from './LanguageSwitcher'
import ThemeSwitcher from './ThemeSwitcher'

interface AuthLayoutProps {
  title: string
  description: string
  /** The card's content: alerts, the form, and alternative sign-in options. */
  children: ReactNode
  /** Shown under the card, such as the link to the other auth screen. */
  footer: ReactNode
}

/** Frame for the sign-in and registration screens: a top bar and a centered card. */
export default function AuthLayout({ title, description, children, footer }: AuthLayoutProps) {
  return (
    <div className="flex min-h-dvh flex-col">
      <header className="flex flex-wrap items-center justify-between gap-3 px-6 py-3">
        <BrandMark withName />
        <div className="flex flex-wrap items-center gap-4">
          <ThemeSwitcher />
          <LanguageSwitcher />
        </div>
      </header>

      <main className="flex flex-1 items-center justify-center px-4 pt-8 pb-16">
        <div className="flex w-full max-w-100 flex-col gap-6">
          <div className="flex flex-col gap-1.5 text-center">
            <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
            <p className="text-sm text-muted-foreground">{description}</p>
          </div>
          <section className="flex flex-col gap-4 rounded-xl border border-border p-6">
            {children}
          </section>
          <p className="text-center text-sm text-muted-foreground">{footer}</p>
        </div>
      </main>
    </div>
  )
}
