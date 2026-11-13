import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { BUTTON_CLASSES } from '../ui/buttonClasses'

/** Shown instead of the entry form while the hourly wage or base hours are not set. */
export default function WorkSetupNotice() {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col items-start gap-3 rounded-xl border border-dashed border-border p-5">
      <h2 className="text-base font-semibold">{t('workLog.setup.title')}</h2>
      <p className="text-sm text-muted-foreground">{t('workLog.setup.description')}</p>
      <Link to="/settings" className={BUTTON_CLASSES.primary}>
        {t('workLog.setup.link')}
      </Link>
    </div>
  )
}
