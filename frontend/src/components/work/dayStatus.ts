/** What a calendar day shows: its entry's comparison, no entry on a past day, or a day to come. */
export type DayStatus = 'above' | 'equal' | 'below' | 'none' | 'future'

/** Tailwind classes per status. The legend swatches use the same ones as the days. */
export const STATUS_CLASSES: Record<DayStatus, string> = {
  above: 'border-success-border bg-success-muted text-success',
  equal: 'border-border bg-muted text-foreground',
  below: 'border-warning-border bg-warning-muted text-warning',
  none: 'border-missing-border bg-missing-muted text-missing',
  future: 'border-transparent text-muted-foreground',
}

export const COMPARISON_LABEL_KEYS = {
  above: 'workLog.comparison.above',
  equal: 'workLog.comparison.equal',
  below: 'workLog.comparison.below',
} as const
