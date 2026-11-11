import { useTranslation } from 'react-i18next'

interface BrandMarkProps {
  /** Also show the app name next to the logo. */
  withName?: boolean
}

/** The app logo: a monogram square, optionally followed by the app name. */
export default function BrandMark({ withName = false }: BrandMarkProps) {
  const { t } = useTranslation()

  return (
    <span className="flex items-center gap-2.5">
      <span
        aria-hidden="true"
        className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary text-sm font-semibold text-primary-foreground"
      >
        PM
      </span>
      {withName ? (
        <span className="text-sm font-semibold">{t('app.title')}</span>
      ) : (
        <span className="sr-only">{t('app.title')}</span>
      )}
    </span>
  )
}
