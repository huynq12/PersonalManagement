import { useTranslation } from 'react-i18next'
import { BUTTON_CLASSES } from '../ui/buttonClasses'

/** Shown while the hourly wage or base hours are not set; the card to set them is further down. */
export default function WorkSetupNotice() {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col items-start gap-3 rounded-xl border border-dashed border-border p-5">
      <h2 className="text-base font-semibold">{t('workLog.setup.title')}</h2>
      <p className="text-sm text-muted-foreground">{t('workLog.setup.description')}</p>
      <a href="#work-settings" className={BUTTON_CLASSES.primary}>
        {t('workLog.setup.link')}
      </a>
    </div>
  )
}
