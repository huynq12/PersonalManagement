import { useTranslation } from 'react-i18next'
import type { NavItem } from '../components/layout/navigation'

interface ComingSoonPageProps {
  item: NavItem
}

/** Placeholder for a screen that has a menu entry but is not built yet. */
export default function ComingSoonPage({ item }: ComingSoonPageProps) {
  const { t } = useTranslation()
  const Icon = item.icon

  return (
    <main className="mx-auto flex w-full max-w-5xl flex-col gap-6 px-4 py-6 md:px-8 md:py-8">
      <h1 className="text-2xl font-semibold tracking-tight md:text-3xl">{t(item.labelKey)}</h1>
      <div className="flex flex-col items-center gap-3 rounded-xl border border-dashed border-border px-6 py-16 text-center">
        <Icon className="size-8 text-muted-foreground" aria-hidden="true" />
        <p className="text-sm text-muted-foreground">{t('comingSoon.description')}</p>
      </div>
    </main>
  )
}
