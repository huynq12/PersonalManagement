import { useTranslation } from 'react-i18next'
import type { Comparison } from '../../lib/workEntries'

const LABEL_KEYS = {
  above: 'workLog.comparison.above',
  equal: 'workLog.comparison.equal',
  below: 'workLog.comparison.below',
} as const

const CLASSES: Record<Comparison, string> = {
  above: 'border-success-border bg-success-muted text-success',
  equal: 'border-border bg-muted text-foreground',
  below: 'border-border bg-background text-muted-foreground',
}

interface ComparisonBadgeProps {
  comparison: Comparison
}

/** Says whether a day's hours were above, equal to, or below the base hours of its entry. */
export default function ComparisonBadge({ comparison }: ComparisonBadgeProps) {
  const { t } = useTranslation()

  return (
    <span
      className={`inline-block rounded-full border px-2 py-0.5 text-xs font-medium whitespace-nowrap ${CLASSES[comparison]}`}
    >
      {t(LABEL_KEYS[comparison])}
    </span>
  )
}
