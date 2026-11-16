import { useTranslation } from 'react-i18next'
import { COMPARISON_LABEL_KEYS, STATUS_CLASSES } from './dayStatus'

const ITEMS = [
  { status: 'none', labelKey: 'workLog.legend.none' },
  { status: 'below', labelKey: COMPARISON_LABEL_KEYS.below },
  { status: 'equal', labelKey: COMPARISON_LABEL_KEYS.equal },
  { status: 'above', labelKey: COMPARISON_LABEL_KEYS.above },
] as const

/** Explains the colors of the calendar days. */
export default function WorkLegend() {
  const { t } = useTranslation()

  return (
    <ul aria-label={t('workLog.legend.title')} className="flex flex-wrap gap-x-4 gap-y-1.5">
      {ITEMS.map((item) => (
        <li key={item.status} className="flex items-center gap-1.5 text-xs text-muted-foreground">
          <span
            aria-hidden="true"
            className={`size-3 rounded-sm border ${STATUS_CLASSES[item.status]}`}
          />
          {t(item.labelKey)}
        </li>
      ))}
    </ul>
  )
}
